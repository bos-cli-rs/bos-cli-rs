use std::str::FromStr;

use color_eyre::eyre::{ContextCompat, WrapErr};
use near_cli_rs::common::{CallResultExt, JsonRpcClientExt};

mod sign_as;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = SetContext)]
pub struct Set {
    /// For which key do you want to delete information?
    key: String,
    #[interactive_clap(value_enum)]
    #[interactive_clap(skip_default_input_arg)]
    /// How do you want to pass the function call arguments?
    function_args_type:
        near_cli_rs::commands::contract::call_function::call_function_args_type::FunctionArgsType,
    /// Enter the arguments to this function or the path to the arguments file:
    function_args: String,
    #[interactive_clap(named_arg)]
    /// Specify signer account ID
    sign_as: self::sign_as::Signer,
}

#[derive(Clone)]
pub struct SetContext {
    pub config: near_cli_rs::config::Config,
    pub set_to_account_id: near_cli_rs::types::account_id::AccountId,
    pub key: String,
    pub function_args_type:
        near_cli_rs::commands::contract::call_function::call_function_args_type::FunctionArgsType,
    pub function_args: String,
}

impl SetContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<Set as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let account_id = near_cli_rs::types::account_id::AccountId::from_str(
            scope.key.split('/').map(|s| s.trim()).collect::<Vec<_>>()[0],
        )?;
        // let function_args =
        //     near_cli_rs::commands::contract::call_function::call_function_args_type::function_args(
        //         ,
        //         scope.function_args_type.clone(),
        //     )?;
        let data_to_set = serde_json::Value::String(scope.function_args.clone());

        let on_after_getting_network_callback: near_cli_rs::commands::OnAfterGettingNetworkCallback = std::sync::Arc::new({
            let signer_id = account_id.clone();
            let key = scope.key.clone();

            move |network_config| {
                let near_social_account_id = crate::consts::NEAR_SOCIAL_ACCOUNT_ID.get(network_config.network_name.as_str())
                    .wrap_err_with(|| format!("The <{}> network does not have a near-social contract.", network_config.network_name))?;

                let input_args = serde_json::to_string(&crate::socialdb_types::SocialDbQuery {
                    keys: vec![format!("{key}")],
                })
                .wrap_err("Internal error: could not serialize SocialDB input args")?;


                // let data_to_set: serde_json::Value = serde_json::from_slice(&function_args).wrap_err_with(|| {
                //     format!(
                //         "Failed to parse view-function call return value: {}",
                //         String::from_utf8_lossy(&function_args)
                //     )
                // })?;
                

                let mut data = serde_json::Map::new();
                crate::common::social_db_data_from_key(&mut data, key.clone(), data_to_set.clone());

                let social_db_data_to_set = serde_json::Value::Object(data);

                Ok(near_cli_rs::commands::PrepopulatedTransaction {
                    signer_id: signer_id.clone().into(),
                    receiver_id: near_social_account_id.clone(),
                    actions: vec![near_primitives::transaction::Action::FunctionCall(
                        near_primitives::transaction::FunctionCallAction {
                            method_name: "set".to_string(),
                            args: serde_json::json!({
                                "data": social_db_data_to_set
                            }).to_string().into_bytes(),
                            gas: near_cli_rs::common::NearGas::from_str("300 TeraGas")
                                .unwrap()
                                .inner,
                            deposit: near_cli_rs::common::NearBalance::from_yoctonear(0).to_yoctonear(),
                        },
                    )]
                })
            }
        });

        let on_after_sending_transaction_callback: near_cli_rs::transaction_signature_options::OnAfterSendingTransactionCallback = std::sync::Arc::new({
            let account_id = account_id.clone();

            move |transaction_info, _network_config| {
                if let near_primitives::views::FinalExecutionStatus::SuccessValue(_) = transaction_info.status {
                    println!("Keys successfully updated from <{}>", &account_id);
                } else {
                    color_eyre::eyre::bail!("Keys were not successfully updated from <{}>", &account_id);
                };
                Ok(())
            }
        });

        Ok(Self {
            config: previous_context.0,
            set_to_account_id: near_cli_rs::types::account_id::AccountId::from_str(
                scope.key.split('/').map(|s| s.trim()).collect::<Vec<_>>()[0],
            )?,
            key: scope.key.clone(),
            function_args_type: scope.function_args_type.clone(),
            function_args: scope.function_args.clone(),
        })
    }
}

impl Set {
    fn input_function_args_type(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<near_cli_rs::commands::contract::call_function::call_function_args_type::FunctionArgsType>>{
        near_cli_rs::commands::contract::call_function::call_function_args_type::input_function_args_type()
    }
}
