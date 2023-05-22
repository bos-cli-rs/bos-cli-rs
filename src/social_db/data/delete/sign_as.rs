use std::str::FromStr;

use color_eyre::eyre::{ContextCompat, WrapErr};
use inquire::{CustomType, Select};
use near_cli_rs::common::{CallResultExt, JsonRpcClientExt};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::DeleteContext)]
#[interactive_clap(output_context = SignerContext)]
pub struct Signer {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the signer account ID?
    signer_account_id: near_cli_rs::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: near_cli_rs::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Clone)]
pub struct SignerContext {
    config: near_cli_rs::config::Config,
    key: String,
    account_id: near_cli_rs::types::account_id::AccountId,
    signer_account_id: near_primitives::types::AccountId,
}

impl SignerContext {
    pub fn from_previous_context(
        previous_context: super::DeleteContext,
        scope: &<Signer as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.config,
            key: previous_context.key,
            account_id: previous_context.account_id,
            signer_account_id: scope.signer_account_id.clone().into(),
        })
    }
}

impl From<SignerContext> for near_cli_rs::commands::ActionContext {
    fn from(item: SignerContext) -> Self {
        let on_after_getting_network_callback: near_cli_rs::commands::OnAfterGettingNetworkCallback = std::sync::Arc::new({
            let key = item.key.clone();
            let signer_id = item.signer_account_id.clone();

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
                        signer_id: signer_id.clone(),
                        receiver_id: near_social_account_id.clone(),
                        actions: vec![],
                    });
                }
                crate::common::mark_leaf_values_as_null(&mut social_db_data_to_remove);
                Ok(near_cli_rs::commands::PrepopulatedTransaction {
                    signer_id: signer_id.clone(),
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
            let account_id = item.account_id.clone();

            move |transaction_info, _network_config| {
                if let near_primitives::views::FinalExecutionStatus::SuccessValue(_) = transaction_info.status {
                    println!("Keys successfully removed from <{}>", &account_id);
                } else {
                    color_eyre::eyre::bail!("Keys were not successfully removed from <{}>", &account_id);
                };
                Ok(())
            }
        });

        Self {
            config: item.config,
            on_after_getting_network_callback,
            on_before_signing_callback: std::sync::Arc::new(
                |_prepolulated_unsinged_transaction, _network_config| Ok(()),
            ),
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config, _message| Ok(()),
            ),
            on_after_sending_transaction_callback,
        }
    }
}

impl Signer {
    fn input_signer_account_id(
        context: &super::DeleteContext,
    ) -> color_eyre::eyre::Result<Option<near_cli_rs::types::account_id::AccountId>> {
        loop {
            let signer_account_id: near_cli_rs::types::account_id::AccountId =
                CustomType::new(" What is the signer account ID?")
                    .with_default(context.account_id.clone())
                    .prompt()?;
            if !near_cli_rs::common::is_account_exist(
                &context.config.network_connection,
                signer_account_id.clone().into(),
            ) {
                println!("\nThe account <{signer_account_id}> does not yet exist.");
                #[derive(strum_macros::Display)]
                enum ConfirmOptions {
                    #[strum(to_string = "Yes, I want to enter a new account name.")]
                    Yes,
                    #[strum(to_string = "No, I want to use this account name.")]
                    No,
                }
                let select_choose_input = Select::new(
                    "Do you want to enter another signer account id?",
                    vec![ConfirmOptions::Yes, ConfirmOptions::No],
                )
                .prompt()?;
                if let ConfirmOptions::No = select_choose_input {
                    return Ok(Some(signer_account_id));
                }
            } else {
                return Ok(Some(signer_account_id));
            }
        }
    }
}
