use similar::{ChangeTag, TextDiff};
use std::io::Write;
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
pub struct SocialDb {
    #[serde(flatten)]
    accounts: std::collections::HashMap<near_primitives::types::AccountId, SocialDbAccountMetadata>,
}

pub type WidgetName = String;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SocialDbAccountMetadata {
    #[serde(rename = "widget")]
    widgets: std::collections::HashMap<WidgetName, SocialDbWidget>,
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
    tags: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
pub struct SocialDbWidgetMetadataImage {
    url: Option<String>,
}

impl SignerAccountId {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        // let args = super::call_function_args_type::function_args(
        //     self.function_args.clone(),
        //     self.function_args_type.clone(),
        // )?;
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
        let entries = std::fs::read_dir("./src")?
            .map(|res| res.map(|e| e.path()))
            .filter(|e| match e {
                Ok(res) => {
                    if let Some(extension) = res.extension().and_then(|s| s.to_str()) {
                        ["jsx", "json"].contains(&extension)
                    } else {
                        false
                    }
                }
                _ => false,
            })
            .collect::<Result<Vec<_>, std::io::Error>>()?;
        println!("--------------  {:#?}", &entries);
        let input_args = serde_json::to_string(&SocialDbQuery {
            keys: vec!["volodymyr.testnet/widget/HelloWorld/**".to_string(), "volodymyr.testnet/widget/Test/**".to_string()],
        })
        .map_err(|err| color_eyre::Report::msg(format!("Data not in JSON format! Error: {}", err)))?
        .into_bytes();
        // let args =
        //     serde_json::Value::from_str("{\"keys\": [\"volodymyr.testnet/widget/Test/**\"]}")
        //         .map_err(|err| {
        //             color_eyre::Report::msg(format!("Data not in JSON format! Error: {}", err))
        //         })?
        //         .to_string()
        //         .into_bytes();
        let query_view_method_response = network_config
            .json_rpc_client()
            .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
                block_reference: near_primitives::types::Finality::Final.into(),
                request: near_primitives::views::QueryRequest::CallFunction {
                    account_id: near_social_account_id.clone(),
                    method_name: "get".to_string(),
                    args: near_primitives::types::FunctionArgs::from(input_args),
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
            std::fs::File::create("./src/input.json")
                .map_err(|err| {
                    color_eyre::Report::msg(format!("Failed to create file: {:?}", err))
                })?
                .write(&call_result.result)
                .map_err(|err| {
                    color_eyre::Report::msg(format!("Failed to write to file: {:?}", err))
                })?;

            serde_json::from_slice(&call_result.result)
                .map_err(|err| color_eyre::Report::msg(format!("serde json: {:?}", err)))?
        };
        println!("serde_call_result: {:#?}", old_social_db);

        let new_code = std::fs::read_to_string("./src/Test.jsx")?;
        let new_code = new_code.trim();
        println!("***New Code: {:#?}", &new_code);

        let new_metadata: SocialDbWidgetMetadata =
            serde_json::from_str(&std::fs::read_to_string("./src/Test.metadata.json")?)
                .map_err(|err| color_eyre::Report::msg(format!("Error reading data: {}", err)))?;
        println!("\n***New metadata: {:#?}", &new_metadata);

        let new_social_widget = SocialDbWidget {
            code: new_code.to_string(),
            metadata: Some(new_metadata.clone()),
        };
        let mut widgets = std::collections::HashMap::new();
        widgets.insert("HelloWorld".to_string(), new_social_widget);
        let social_account_metadata = SocialDbAccountMetadata { widgets };
        let mut accounts = std::collections::HashMap::new();
        accounts.insert(
            near_primitives::types::AccountId::from_str("volodymyr.testnet")?,
            social_account_metadata,
        );
        let new_social_db = SocialDb { accounts };

        let output_function_args = serde_json::json!({
            "data": serde_json::from_str::<serde_json::Value>(&serde_json::to_string(&new_social_db)?)?
        })
        .to_string();
        println!("output_function_args: {}", &output_function_args);

        let old_social_widget = if let Some(widget) = old_social_db
            .accounts
            .get("volodymyr.testnet")
            .and_then(|account_metadata| account_metadata.widgets.get("HelloWorld"))
        {
            widget
        } else {
            println!(
                "Widget named <{}> does not exist. So need to deploy it.",
                "HelloWorld"
            );
            return self
                .deploy_widget_code(
                    config,
                    network_config,
                    near_social_account_id,
                    output_function_args,
                )
                .await;
        };

        let old_code = old_social_widget.code.as_str();
        println!("***Old Code: {:#?}", &old_code);

        if need_code_deploy(old_code, &new_code)? {
            return self
                .deploy_widget_code(
                    config,
                    network_config,
                    near_social_account_id,
                    output_function_args,
                )
                .await;
        }
        println!("Widget code has not changed");

        let old_metadata = &old_social_widget.metadata;
        println!("***metadata: {:#?}", &old_metadata);

        if old_metadata != &Some(new_metadata) {
            return self
                .deploy_widget_code(
                    config,
                    network_config,
                    near_social_account_id,
                    output_function_args,
                )
                .await;
        }
        println!("Widget metadata has not changed");

        Ok(())
    }

    async fn deploy_widget_code(
        &self,
        config: crate::config::Config,
        network_config: crate::config::NetworkConfig,
        near_social_account_id: near_primitives::types::AccountId,
        function_args: String,
    ) -> crate::CliResult {
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
                    deposit: crate::common::NearBalance::from_str("0.01 NEAR") // need calculation!!!!!!!!
                        .unwrap()
                        .to_yoctonear(),
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

fn need_code_deploy(old_code: &str, new_code: &str) -> color_eyre::eyre::Result<bool> {
    println!();
    let diff = TextDiff::from_lines(old_code, &new_code);

    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
            ChangeTag::Equal => " ",
        };
        print!("{}{}", sign, change);
    }

    if old_code == new_code {
        return Ok(false);
    }
    Ok(true)
}
