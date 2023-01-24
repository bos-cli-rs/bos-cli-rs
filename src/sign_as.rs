use std::str::FromStr;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct SignAs {
    #[interactive_clap(named_arg)]
    ///What is the signer account ID?
    sign_as: SignerAccountId,
}

impl SignAs {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        self.sign_as.process(config).await
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct SignerAccountId {
    ///What is the signer account ID?
    signer_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    ///Select network
    network_config: near_cli_rs::network_for_transaction::NetworkForTransactionArgs,
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
            serde_json::Value::from_str("{\"keys\": [\"volodymyr.testnet/widget/Test/**\"]}")
                .map_err(|err| {
                    color_eyre::Report::msg(format!("Data not in JSON format! Error: {}", err))
                })?
                .to_string()
                .into_bytes();
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

        let serde_call_result = if call_result.result.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::from_slice(&call_result.result)
                .map_err(|err| color_eyre::Report::msg(format!("serde json: {:?}", err)))?
        };
        println!("--------------");
        if call_result.logs.is_empty() {
            println!("No logs")
        } else {
            println!("Logs:");
            println!("  {}", call_result.logs.join("\n  "));
        }
        println!("--------------");
        println!("Result:");
        println!("{}", serde_json::to_string_pretty(&serde_call_result)?);
        println!("--------------");

        // let function_args = super::call_function_args_type::function_args(
        //     self.function_args.clone(),
        //     self.function_args_type.clone(),
        // )?;
        let function_args = serde_json::Value::from_str(
            "{
                \"data\": {
                    \"volodymyr.testnet\": {
                        \"widget\": {
                            \"Test\": {
                            \"\": \"return <h1>Hello World</h1>;\"
                            }
                        }
                    }
                }
            }",
        )
        .map_err(|err| color_eyre::Report::msg(format!("Data not in JSON format! Error: {}", err)))?
        .to_string()
        .into_bytes();
        let prepopulated_unsigned_transaction = near_primitives::transaction::Transaction {
            signer_id: self.signer_account_id.clone().into(),
            public_key: near_crypto::PublicKey::empty(near_crypto::KeyType::ED25519),
            nonce: 0,
            receiver_id: near_social_account_id,
            block_hash: Default::default(),
            actions: vec![near_primitives::transaction::Action::FunctionCall(
                near_primitives::transaction::FunctionCallAction {
                    method_name: "set".to_string(),
                    args: function_args,
                    gas: 100_000_000_000_000, // 100 TeraGas
                    deposit: 1,               // 1 yNear
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

        // Ok(())
    }
}
