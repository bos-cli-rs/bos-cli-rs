use std::str::FromStr;

use color_eyre::eyre::ContextCompat;
use inquire::{CustomType, Select};

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
    permission_key: crate::common::PermissionKey,
    extra_storage_deposit: near_cli_rs::common::NearBalance,
    signer_account_id: near_primitives::types::AccountId,
}

impl SignerContext {
    pub fn from_previous_context(
        previous_context: super::storage_deposit::ExtraStorageDepositContext,
        scope: &<Signer as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.config,
            widgets: vec![format!("{}/widget", scope.signer_account_id)],
            permission_key: previous_context.permission_key,
            extra_storage_deposit: previous_context.extra_storage_deposit,
            signer_account_id: scope.signer_account_id.clone().into(),
        })
    }
}

impl From<SignerContext> for near_cli_rs::commands::ActionContext {
    fn from(item: SignerContext) -> Self {
        let widgets = item.widgets.clone();
        let permission_key = item.permission_key.clone();
        let extra_storage_deposit = item.extra_storage_deposit.clone();
        let signer_id = item.signer_account_id.clone();

        let on_after_getting_network_callback: near_cli_rs::commands::OnAfterGettingNetworkCallback = std::sync::Arc::new({
            move |network_config| {
                let near_social_account_id = crate::consts::NEAR_SOCIAL_ACCOUNT_ID.get(network_config.network_name.as_str())
                    .wrap_err_with(|| format!("The <{}> network does not have a near-social contract.", network_config.network_name))?;
                let args = match &permission_key {
                    crate::common::PermissionKey::PredecessorId(account_id) => {
                        serde_json::json!({
                            "predecessor_id": account_id.to_string(),
                            "keys": widgets
                        }).to_string().into_bytes()
                    }
                    crate::common::PermissionKey::PublicKey(public_key) => {
                        serde_json::json!({
                            "public_key": public_key.to_string(),
                            "keys": widgets
                        }).to_string().into_bytes()
                    }
                };
                let prepopulated_transaction = near_cli_rs::commands::PrepopulatedTransaction {
                    signer_id: item.signer_account_id.clone(),
                    receiver_id: near_social_account_id.clone(),
                    actions: vec![
                    near_primitives::transaction::Action::FunctionCall(
                        near_primitives::transaction::FunctionCallAction {
                            method_name: "grant_write_permission".to_string(),
                            args,
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

        let on_after_sending_transaction_callback: near_cli_rs::transaction_signature_options::OnAfterSendingTransactionCallback = std::sync::Arc::new({
            let permission_key = item.permission_key.clone();
            move |transaction_info, _network_config| {
                if let near_primitives::views::FinalExecutionStatus::SuccessValue(_) = transaction_info.status {
                    if let near_primitives::views::ActionView::FunctionCall { .. } =
                        &transaction_info.transaction.actions[0]
                    {
                        match &permission_key {
                        crate::common::PermissionKey::PredecessorId(account_id) => {
                            eprintln!("<{signer_id}> granted account <{account_id}> permission to edit its widgets");
                        }
                        crate::common::PermissionKey::PublicKey(public_key) => {
                            eprintln!("<{signer_id}> granted public key <{public_key}> permission to edit its widgets");
                        }
                    }
                    } else {
                        color_eyre::eyre::bail!(
                            "Internal error: Unexpected function call arguments",
                        );
                    }
                } else {
                    match &permission_key {
                        crate::common::PermissionKey::PredecessorId(account_id) => {
                            color_eyre::eyre::bail!("Could not grant permission to <{}>", account_id);
                        }
                        crate::common::PermissionKey::PublicKey(public_key) => {
                            color_eyre::eyre::bail!("Could not grant permission to <{}>", public_key);
                        }
                    }
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
        context: &super::storage_deposit::ExtraStorageDepositContext,
    ) -> color_eyre::eyre::Result<Option<near_cli_rs::types::account_id::AccountId>> {
        loop {
            let signer_account_id: near_cli_rs::types::account_id::AccountId =
                CustomType::new(" What is the signer account ID?").prompt()?;
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
