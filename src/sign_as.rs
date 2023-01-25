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

#[derive(Debug, serde::Serialize)]
pub struct SocialDbQuery {
    keys: Vec<String>
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SocialDb {
    #[serde(flatten)]
    accounts: std::collections::HashMap<near_primitives::types::AccountId, SocialDbAccountMetadata>,
}

pub type WidgetName = String;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SocialDbAccountMetadata {
    #[serde(rename = "widget")]
    widgets: std::collections::HashMap<WidgetName, SocialDbWidget>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SocialDbWidget {
    #[serde(rename = "")]
    code: String,
    metadata: Option<SocialDbWidgetMetadata>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
pub struct SocialDbWidgetMetadata {
    description: Option<String>,
    image: Option<SocialDbWidgetMetadataImage>,
    name: Option<String>,
    tags: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
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
        let args = 
            // serde_json::Value::from_str("{\"keys\": [\"frol14.testnet/widget/HelloWorld/**\"]}")
            // serde_json::json!({"keys": ["frol14.testnet/widget/HelloWorld/**"]})
            serde_json::to_string(&SocialDbQuery { keys: vec!["frol14.testnet/widget/HelloWorld/**".to_string()] })
                .map_err(|err| {
                    color_eyre::Report::msg(format!("Data not in JSON format! Error: {}", err))
                })?
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
                    args: near_primitives::types::FunctionArgs::from(args),
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
                result
            } else {
                return Err(color_eyre::Report::msg("Error call result".to_string()));
            };
        let serde_call_result: Option<SocialDb> = if call_result.result.is_empty() {
            None
        } else {
            std::fs::File::create("./src/input.json")
                .map_err(|err| {
                    color_eyre::Report::msg(format!("Failed to create file: {:?}", err))
                })?
                .write(&call_result.result)
                .map_err(|err| {
                    color_eyre::Report::msg(format!("Failed to write to file: {:?}", err))
                })?;
            Some(serde_json::from_slice(&call_result.result)
                .map_err(|err| color_eyre::Report::msg(format!("serde json: {:?}", err)))?)
        };
        println!("serde_call_result: {:#?}", serde_call_result);
        let old_code = if let Some(code) = serde_call_result
            .accounts
            .get("frol14.testnet")
            .and_then(|value| value.widgets.get("HelloWorld"))
            .map(|value| value.code)
        {
            code.as_str().expect("Unable to get widget code!").trim()
        } else {
            return Err(color_eyre::Report::msg(
                "This widget has no code".to_string(),
            ));
        };
        println!("***Old Code: {:#?}", &old_code);
        let new_code = std::fs::read_to_string("./src/Test.jsx")?;
        let new_code = new_code.trim();
        println!("***New Code: {:#?}", &new_code);
        let output_function_args = serde_json::json!({
            //"data": serde_json::from_str::<serde_json::Value>(&std::fs::read_to_string("./src/output.json")?)?
            // "data": {
            //     "frol14.testnet": {
            //         "widget": {
            //             "": "code"
            //         }
            //     }
            // }
            "data": SocialDb { accounts: ... }
            "data": {
                account_id: SocialDbAccountMetadata { widgets: ... },
            }
        })
        .to_string();

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

        let new_metadata = std::fs::read_to_string("./src/Test.metadata.json")?;
        println!("\n***New metadata: {:#?}", &new_metadata);

        let old_metadata = if let Some(code) = serde_call_result
            .get("frol14.testnet")
            .and_then(|value| value.get("widget"))
            .and_then(|value| value.get("HelloWorld"))
            .and_then(|value| value.get("metadata"))
        {
            code.as_str().expect("Unable to get widget metadata!")
        } else {
            return Err(color_eyre::Report::msg(
                "This widget has no metadata".to_string(),
            ));
        };
        println!("***metadata: {:#?}", &old_metadata);

        if need_code_deploy(old_metadata, &new_metadata)? {
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
