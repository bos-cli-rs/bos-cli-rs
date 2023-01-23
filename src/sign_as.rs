#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct SignAs {
    #[interactive_clap(named_arg)]
    ///What is the signer account ID?
    sign_as: SignerAccountId,
}

// impl AddAccessKeyAction {
//     pub async fn process(
//         &self,
//         config: crate::config::Config,
//         account_properties: super::super::super::AccountProperties,
//     ) -> crate::CliResult {
//         let account_properties = super::super::super::AccountProperties {
//             public_key: self.public_key.clone().into(),
//             ..account_properties
//         };
//         let storage_properties = None;
//         self.sign_as
//             .process(config, account_properties, storage_properties)
//             .await
//     }
// }

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
        let args = super::call_function_args_type::function_args(
            self.function_args.clone(),
            self.function_args_type.clone(),
        )?;
        let query_view_method_response = self
            .network_config
            .get_network_config(config)
            .json_rpc_client()
            .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
                block_reference: near_primitives::types::Finality::Final.into(),
                request: near_primitives::views::QueryRequest::CallFunction {
                    account_id: self.signer_account_id.clone().into(),
                    method_name: self.function_name.clone(),
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
        Ok(())
    }

}
