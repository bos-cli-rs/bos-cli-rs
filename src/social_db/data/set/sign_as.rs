use color_eyre::eyre::{ContextCompat, WrapErr};
use inquire::{CustomType, Select};
use near_cli_rs::common::{CallResultExt, JsonRpcClientExt};
use std::sync::Arc;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::data::DataContext)]
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
pub struct SignerContext(near_cli_rs::commands::ActionContext);

impl SignerContext {
    pub fn from_previous_context(
        previous_context: super::data::DataContext,
        scope: &<Signer as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let set_to_account_id: near_primitives::types::AccountId =
            previous_context.set_to_account_id.clone().into();
        let signer_id: near_primitives::types::AccountId = scope.signer_account_id.clone().into();
        let key = previous_context.key.clone();

        let get_prepopulated_transaction_after_getting_network_callback: near_cli_rs::commands::GetPrepopulatedTransactionAfterGettingNetworkCallback = Arc::new({
            let signer_id = signer_id.clone();
            let set_to_account_id = set_to_account_id.clone();

            move |network_config| {
                let near_social_account_id = crate::consts::NEAR_SOCIAL_ACCOUNT_ID.get(network_config.network_name.as_str())
                    .wrap_err_with(|| format!("The <{}> network does not have a near-social contract.", network_config.network_name))?;
                let key = previous_context.key.clone();
                let input_args = serde_json::to_string(&crate::socialdb_types::SocialDbQuery {
                    keys: vec![format!("{key}")],
                })
                .wrap_err("Internal error: could not serialize SocialDB input args")?;

                let remote_social_db_data_for_key: serde_json::Value = network_config
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

                let optional_remote_social_db_data_for_key =
                    if remote_social_db_data_for_key.as_object().map(|result| result.is_empty()).unwrap_or(true) {
                        None
                    } else {
                        Some(&remote_social_db_data_for_key)
                    };

                let mut social_db_data_to_set = previous_context.value.clone();

                crate::common::social_db_data_from_key(&key, &mut social_db_data_to_set);

                let json_rpc_client = network_config.json_rpc_client();

                let deposit = tokio::runtime::Runtime::new().unwrap().block_on(
                    near_socialdb_client::required_deposit(
                        &json_rpc_client,
                        near_social_account_id,
                        &set_to_account_id,
                        &social_db_data_to_set,
                        optional_remote_social_db_data_for_key,
                    ),
                )?;

                Ok(near_cli_rs::commands::PrepopulatedTransaction {
                    signer_id: signer_id.clone(),
                    receiver_id: near_social_account_id.clone(),
                    actions: vec![
                    near_primitives::transaction::Action::FunctionCall(
                        Box::new(near_primitives::transaction::FunctionCallAction {
                            method_name: "set".to_string(),
                            args: serde_json::json!({
                                "data": social_db_data_to_set
                            }).to_string().into_bytes(),
                            gas: near_cli_rs::common::NearGas::from_tgas(300).as_gas(),
                            deposit: deposit.as_yoctonear(),
                        }),
                    )
                ]})
            }
        });

        let on_before_signing_callback: near_cli_rs::commands::OnBeforeSigningCallback =
            Arc::new({
                let set_to_account_id = set_to_account_id.clone();

                move |prepopulated_unsigned_transaction, network_config| {
                    let json_rpc_client = network_config.json_rpc_client();
                    let public_key = prepopulated_unsigned_transaction.public_key.clone();
                    let receiver_id = prepopulated_unsigned_transaction.receiver_id.clone();
                    if let Some(near_primitives::transaction::Action::FunctionCall(action)) =
                        prepopulated_unsigned_transaction.actions.get_mut(0)
                    {
                        action.deposit = tokio::runtime::Runtime::new()
                            .unwrap()
                            .block_on(near_socialdb_client::get_deposit(
                                &json_rpc_client,
                                &signer_id,
                                &public_key,
                                &set_to_account_id,
                                &key,
                                &receiver_id,
                                near_cli_rs::types::near_token::NearToken::from_yoctonear(
                                    action.deposit,
                                )
                                .into(),
                            ))?
                            .as_yoctonear();
                        Ok(())
                    } else {
                        color_eyre::eyre::bail!("Unexpected action to change components",);
                    }
                }
            });

        let on_after_sending_transaction_callback: near_cli_rs::transaction_signature_options::OnAfterSendingTransactionCallback = std::sync::Arc::new({
                move |transaction_info, _network_config| {
                    if let near_primitives::views::FinalExecutionStatus::SuccessValue(_) = transaction_info.status {
                        println!("Keys successfully installed on <{set_to_account_id}>");
                    } else {
                        color_eyre::eyre::bail!("Keys were not successfully installed on <{set_to_account_id}>");
                    };
                    Ok(())
                }
            });

        Ok(Self(near_cli_rs::commands::ActionContext {
            global_context: previous_context.global_context,
            interacting_with_account_ids: vec![previous_context.set_to_account_id.into()],
            get_prepopulated_transaction_after_getting_network_callback,
            on_before_signing_callback,
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction: &near_cli_rs::transaction_signature_options::SignedTransactionOrSignedDelegateAction, _network_config| Ok(String::new())
            ),
            on_after_sending_transaction_callback,
        }))
    }
}

impl From<SignerContext> for near_cli_rs::commands::ActionContext {
    fn from(item: SignerContext) -> Self {
        item.0
    }
}

impl Signer {
    fn input_signer_account_id(
        context: &super::data::DataContext,
    ) -> color_eyre::eyre::Result<Option<near_cli_rs::types::account_id::AccountId>> {
        loop {
            let signer_account_id: near_cli_rs::types::account_id::AccountId =
                CustomType::new("What is the signer account ID?")
                    .with_default(context.set_to_account_id.clone())
                    .prompt()?;
            if !near_cli_rs::common::is_account_exist(
                &context.global_context.config.network_connection,
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
