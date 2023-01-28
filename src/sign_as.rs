use glob::glob;
use similar::{ChangeTag, TextDiff};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct SignerAccountId {
    /// What is the signer account ID?
    signer_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: near_cli_rs::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SocialDbQuery {
    keys: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransactionFunctionArgs {
    data: SocialDb,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SocialDb {
    #[serde(flatten)]
    accounts: HashMap<near_primitives::types::AccountId, SocialDbAccountMetadata>,
}

pub type WidgetName = String;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SocialDbAccountMetadata {
    #[serde(rename = "widget")]
    widgets: HashMap<WidgetName, SocialDbWidget>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SocialDbWidget {
    #[serde(rename = "")]
    code: String,
    metadata: Option<SocialDbWidgetMetadata>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
pub struct SocialDbWidgetMetadata {
    description: Option<String>,
    image: Option<SocialDbWidgetMetadataImage>,
    name: Option<String>,
    tags: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
pub struct SocialDbWidgetMetadataImage {
    url: Option<String>,
}

impl SignerAccountId {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        let network_config = self.network_config.get_network_config(config.clone());
        let near_social_account_id = match &network_config.near_social_account_id {
            Some(account_id) => account_id.clone(),
            None => {
                return Err(color_eyre::Report::msg(format!(
                    "The <{}> network does not have a near-social contract.",
                    network_config.network_name
                )))
            }
        };
        let mut widget_names: HashSet<String> = HashSet::new();
        let mut widgets = HashMap::new();

        for entry in glob("./src/**/*.jsx")?.filter_map(Result::ok) {
            let widget_name: WidgetName = entry
                .strip_prefix("src")?
                .to_str()
                .expect("Impossible to convert to_str()")
                .split(".jsx")
                .into_iter()
                .next()
                .and_then(|s| Some(s.to_string()))
                .expect("Impossible to convert to_string()")
                .replace('/', ".");
            widget_names.insert(widget_name.clone());
            let social_widget = SocialDbWidget {
                code: std::fs::read_to_string(entry)?.trim().to_string(),
                metadata: None,
            };
            widgets.insert(widget_name, social_widget);
        }

        for entry in glob("./src/**/*.json")?.filter_map(Result::ok) {
            let widget_name: WidgetName = entry
                .strip_prefix("src")?
                .to_str()
                .expect("Impossible to convert to_str()")
                .split(".metadata.json")
                .into_iter()
                .next()
                .and_then(|s| Some(s.to_string()))
                .expect("Impossible to convert to_string()")
                .replace('/', ".");

            let metadata: SocialDbWidgetMetadata =
                serde_json::from_str(&std::fs::read_to_string(entry.clone())?).map_err(|err| {
                    color_eyre::Report::msg(format!("Error reading data: {}", err))
                })?;
            if widget_names.contains(&widget_name) {
                let social_widget = SocialDbWidget {
                    metadata: Some(metadata.clone()),
                    ..widgets[&widget_name].clone()
                };
                widgets.insert(widget_name.clone(), social_widget);
            } else {
                let social_widget = SocialDbWidget {
                    code: "".to_string(),
                    metadata: Some(metadata),
                };
                widgets.insert(widget_name, social_widget);
            };
        }
        let input_args_keys: Vec<String> = widget_names
            .clone()
            .into_iter()
            .map(|name| format!("{}/widget/{}/**", self.signer_account_id, name))
            .collect();
        let input_args = serde_json::to_string(&SocialDbQuery {
            keys: input_args_keys,
        })
        .map_err(|err| {
            color_eyre::Report::msg(format!("Data not in JSON format! Error: {}", err))
        })?;
        let query_view_method_response = network_config
            .json_rpc_client()
            .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
                block_reference: near_primitives::types::Finality::Final.into(),
                request: near_primitives::views::QueryRequest::CallFunction {
                    account_id: near_social_account_id.clone(),
                    method_name: "get".to_string(),
                    args: near_primitives::types::FunctionArgs::from(input_args.into_bytes()),
                },
            })
            .await
            .map_err(|err| {
                color_eyre::Report::msg(format!("Failed to fetch query for view method: {:?}", err))
            })?;
        let call_result =
            if let near_jsonrpc_primitives::types::query::QueryResponseKind::CallResult(result) =
                query_view_method_response.kind
            {
                if result.result.is_empty() {
                    return Err(color_eyre::Report::msg("Error call result".to_string()));
                }
                result
            } else {
                return Err(color_eyre::Report::msg("Error call result".to_string()));
            };
        let old_social_db: SocialDb = {
            serde_json::from_slice(&call_result.result)
                .map_err(|err| color_eyre::Report::msg(format!("serde json: {:?}", err)))?
        };
        let old_social_account_metadata = match old_social_db
            .accounts
            .get(&near_primitives::types::AccountId::from(
                self.signer_account_id.clone(),
            ))
            .clone()
        {
            Some(account_metadata) => account_metadata,
            None => {
                println!("\nThere are currently no widgets in the account <{}>. Therefore, all selected widgets will be deployed", self.signer_account_id);
                let deposit = crate::common::NearBalance::from_str("1 NEAR") // need calculation!!!!!!!! for new account
                    .unwrap()
                    .to_yoctonear();
                return self
                    .deploy_widget_code(
                        config,
                        network_config,
                        near_social_account_id,
                        widgets,
                        deposit,
                    )
                    .await;
            }
        };

        let output_widgets = widgets
            .clone()
            .into_iter()
            .filter(|(widget_name, _)| {
                if old_social_account_metadata
                    .widgets
                    .get(widget_name)
                    .is_none()
                {
                    println!("Found new widget <{}> to deploy", widget_name);
                    true
                } else {
                    let old_code = &old_social_account_metadata.widgets[widget_name].code;
                    let new_code = &widgets[widget_name].code;
                    let old_metadata = old_social_account_metadata.widgets[widget_name]
                        .metadata
                        .clone();
                    let new_metadata = widgets[widget_name].metadata.clone();
                    if old_metadata != new_metadata {
                        println!("Metadata for widget <{}> changed", widget_name);
                        true
                    } else {
                        need_code_deploy(old_code, new_code, widget_name)
                    }
                }
            })
            .collect::<HashMap<String, SocialDbWidget>>();

        if output_widgets.is_empty() {
            return Ok(());
        }

        let deposit = crate::common::NearBalance::from_str("0.01 NEAR") // need calculation!!!!!!!! for an existing account
            .unwrap()
            .to_yoctonear();
        self.deploy_widget_code(
            config,
            network_config,
            near_social_account_id,
            output_widgets,
            deposit,
        )
        .await
    }

    async fn deploy_widget_code(
        &self,
        config: crate::config::Config,
        network_config: crate::config::NetworkConfig,
        near_social_account_id: near_primitives::types::AccountId,
        widgets: HashMap<String, SocialDbWidget>,
        deposit: u128,
    ) -> crate::CliResult {
        let social_account_metadata = SocialDbAccountMetadata { widgets };
        let mut accounts = HashMap::new();
        accounts.insert(
            near_primitives::types::AccountId::from(self.signer_account_id.clone()),
            social_account_metadata,
        );
        let new_social_db = SocialDb { accounts };

        let transaction_function_args = TransactionFunctionArgs {
            data: new_social_db,
        };
        let function_args = serde_json::to_string(&transaction_function_args)?;

        let prepopulated_unsigned_transaction = near_primitives::transaction::Transaction {
            signer_id: self.signer_account_id.clone().into(),
            public_key: near_crypto::PublicKey::empty(near_crypto::KeyType::ED25519),
            nonce: 0,
            receiver_id: near_social_account_id,
            block_hash: Default::default(),
            actions: vec![near_primitives::transaction::Action::FunctionCall(
                near_primitives::transaction::FunctionCallAction {
                    method_name: "set".to_string(),
                    args: function_args.into_bytes(),
                    gas: crate::common::NearGas::from_str("100 TeraGas")
                        .unwrap()
                        .inner,
                    deposit,
                },
            )],
        };
        match near_cli_rs::transaction_signature_options::sign_with(
            self.network_config.clone(),
            prepopulated_unsigned_transaction,
            config.clone(),
        )
        .await?
        {
            Some(transaction_info) => {
                crate::common::print_transaction_status(transaction_info, network_config)
            }
            None => Ok(()),
        }
    }
}

fn need_code_deploy(old_code: &str, new_code: &str, widget_name: &str) -> bool {
    println!();
    let diff = TextDiff::from_lines(old_code, new_code);

    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
            ChangeTag::Equal => " ",
        };
        print!("{}{}", sign, change);
    }

    if old_code == new_code {
        println!("Code for widget <{}> has not changed\n", widget_name);
        return false;
    }
    println!("Code for widget <{}> changed\n", widget_name);
    true
}
