use std::str::FromStr;

use color_eyre::eyre::{ContextCompat, WrapErr};
use near_cli_rs::common::{CallResultExt, JsonRpcClientExt};

mod sign_as;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = DeleteContext)]
pub struct Delete {
    /// For which key do you want to delete information?
    key: String,
    #[interactive_clap(named_arg)]
    /// Select network
    sign_as: self::sign_as::Signer,
}

#[derive(Clone)]
pub struct DeleteContext {
    pub config: near_cli_rs::config::Config,
    pub account_id: near_cli_rs::types::account_id::AccountId,
    pub on_after_getting_network_callback: near_cli_rs::commands::OnAfterGettingNetworkCallback,
    pub on_before_signing_callback: near_cli_rs::commands::OnBeforeSigningCallback,
    pub on_before_sending_transaction_callback:
        near_cli_rs::transaction_signature_options::OnBeforeSendingTransactionCallback,
    pub on_after_sending_transaction_callback:
        near_cli_rs::transaction_signature_options::OnAfterSendingTransactionCallback,
}

impl DeleteContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<Delete as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let account_id = near_cli_rs::types::account_id::AccountId::from_str(
            scope.key.split('/').map(|s| s.trim()).collect::<Vec<_>>()[0],
        )?;
        
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

                let mut social_db_data_to_remove: serde_json::Value = network_config
                    .json_rpc_client()
                    .blocking_call_view_function(
                        near_social_account_id,
                        "get",
                        input_args.into_bytes(),
                        near_primitives::types::Finality::Final.into(),
                    )
                    .wrap_err("Failed to fetch the components from SocialDB")?
                    .parse_result_from_json()
                    .wrap_err("SocialDB `get` data response cannot be parsed")?;
                if social_db_data_to_remove.as_object().map(|result| result.is_empty()).unwrap_or(true) {
                    println!("No keys to remove. Goodbye.");
                    return Ok(near_cli_rs::commands::PrepopulatedTransaction {
                        signer_id: signer_id.clone().into(),
                        receiver_id: near_social_account_id.clone(),
                        actions: vec![],
                    });
                }
                crate::common::mark_leaf_values_as_null(&mut social_db_data_to_remove);
                Ok(near_cli_rs::commands::PrepopulatedTransaction {
                    signer_id: signer_id.clone().into(),
                    receiver_id: near_social_account_id.clone(),
                    actions: vec![near_primitives::transaction::Action::FunctionCall(
                        near_primitives::transaction::FunctionCallAction {
                            method_name: "set".to_string(),
                            args: serde_json::json!({
                                "data": social_db_data_to_remove
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
            // let account_id = account_id.clone();

            move |transaction_info, _network_config| {
                if let near_primitives::views::FinalExecutionStatus::SuccessValue(_) = transaction_info.status {
                    println!("Keys successfully removed from <{}>", &account_id);
                } else {
                    color_eyre::eyre::bail!("Keys were not successfully removed from <{}>", &account_id);
                };
                Ok(())
            }
        });




        Ok(Self {
            config: previous_context.0,
            account_id,
            on_after_getting_network_callback,
            on_before_signing_callback: std::sync::Arc::new(
                |_prepolulated_unsinged_transaction, _network_config| Ok(()),
            ),
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config, _message| Ok(()),
            ),
            on_after_sending_transaction_callback,
        })
    }
}

impl From<DeleteContext> for near_cli_rs::commands::ActionContext {
    fn from(item: DeleteContext) -> Self {
        item.0
    }
}
