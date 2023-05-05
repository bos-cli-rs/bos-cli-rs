use std::str::FromStr;

use strum::{EnumDiscriminants, EnumIter, EnumMessage};
use inquire::{CustomType, Text, Select};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::storage_deposit::ExtraStorageDepositContext)]
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
    widgets: Vec<String>,
    account_id: near_primitives::types::AccountId,
    extra_storage_deposit: near_cli_rs::common::NearBalance,
    signer_account_id: near_primitives::types::AccountId,
}

impl SignerContext {
    pub fn from_previous_context(
        previous_context: super::storage_deposit::ExtraStorageDepositContext,
        scope: &<Signer as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let widgets = if previous_context.widgets.is_empty() {
            vec![format!("{}/widget", scope.signer_account_id)]
        } else {
            previous_context.widgets.into_iter().map(|widget| format!("{}/widget/{}", scope.signer_account_id, widget)).collect()
        };
        Ok(Self {
            config: previous_context.config,
            widgets,
            account_id: previous_context.account_id,
            extra_storage_deposit: previous_context.extra_storage_deposit,
            signer_account_id: scope.signer_account_id.clone().into(),
        })
    }
}

impl From<SignerContext> for near_cli_rs::commands::ActionContext {
    fn from(item: SignerContext) -> Self {
        let widgets = item.widgets.clone();
        let account_id = item.account_id.clone();
        let extra_storage_deposit = item.extra_storage_deposit.clone();
        let signer_id = item.signer_account_id.clone();

        let on_after_getting_network_callback: near_cli_rs::commands::OnAfterGettingNetworkCallback = std::sync::Arc::new({
            move |network_config| {
                let near_social_account_id = crate::consts::NEAR_SOCIAL_ACCOUNT_ID.get(network_config.network_name.as_str())
                    .ok_or_else(||
                        color_eyre::eyre::eyre!(
                            "The <{}> network does not have a near-social contract.",
                            network_config.network_name
                        )
                    )?;
                let mut prepopulated_transaction = near_cli_rs::commands::PrepopulatedTransaction {
                    signer_id: signer_id.clone(),
                    receiver_id: near_social_account_id.clone(),
                    actions: vec![
                    near_primitives::transaction::Action::FunctionCall(
                        near_primitives::transaction::FunctionCallAction {
                            method_name: "grant_write_permission".to_string(),
                            args: serde_json::json!({
                                "predecessor_id": account_id.to_string(),
                                "keys": widgets
                            }).to_string().into_bytes(),
                            gas: near_cli_rs::common::NearGas::from_str("100 TeraGas")
                                .unwrap()
                                .inner,
                            deposit: extra_storage_deposit.to_yoctonear(),
                        },
                    )
                ],
                };

                Ok(prepopulated_transaction)
            }
        });

        // let on_before_signing_callback: near_cli_rs::commands::OnBeforeSigningCallback =
        //     std::sync::Arc::new({
        //         let signer_account_id = item.signer_account_id.clone();
        //         let deploy_to_account_id = item.deploy_to_account_id.clone();
        //         move |prepopulated_unsigned_transaction, network_config| {
        //             if let near_primitives::transaction::Action::FunctionCall(action) =
        //                 &mut prepopulated_unsigned_transaction.actions[0]
        //             {
        //                 action.deposit = get_deposit(
        //                     network_config,
        //                     &signer_account_id,
        //                     &prepopulated_unsigned_transaction.public_key,
        //                     &deploy_to_account_id,
        //                     &prepopulated_unsigned_transaction.receiver_id,
        //                     near_cli_rs::common::NearBalance::from_yoctonear(action.deposit),
        //                 )?
        //                 .to_yoctonear();
        //                 Ok(())
        //             } else {
        //                 color_eyre::eyre::bail!("Unexpected action to change widgets",);
        //             }
        //         }
        //     });

        let on_after_sending_transaction_callback: near_cli_rs::transaction_signature_options::OnAfterSendingTransactionCallback = std::sync::Arc::new({
            let account_id = item.account_id.clone();
            move |transaction_info, _network_config| {
                let args = if let near_primitives::views::FinalExecutionStatus::SuccessValue(_) = transaction_info.status {
                    if let near_primitives::views::ActionView::FunctionCall { args, .. } =
                        &transaction_info.transaction.actions[0]
                    {
                        args
                    } else {
                        color_eyre::eyre::bail!(
                            "Internal error: Unexpected function call arguments",
                        );
                    }
                } else {
                    color_eyre::eyre::bail!("Could not grant permission to <{}>", account_id.clone());
                };

                // let transaction_function_args: super::TransactionFunctionArgs =
                //     serde_json::from_slice(args).wrap_err("Internal error: Could not parse SocialDB request that we just created.")?;

                // let social_account_metadata = transaction_function_args.data.accounts.get(item.deploy_to_account_id.as_ref())
                //     .wrap_err("Internal error: Could not get metadata from SocialDB request that we just created.")?;
                // let updated_widgets = &social_account_metadata.widgets;

                // println!("\n<{}> widgets were successfully deployed:", updated_widgets.len());
                println!();
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
        context: &super::storage_deposit::ExtraStorageDepositContext,
    ) -> color_eyre::eyre::Result<Option<near_cli_rs::types::account_id::AccountId>> {
        loop {
            let signer_account_id: near_cli_rs::types::account_id::AccountId =
                CustomType::new(" What is the signer account ID?")
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
