use std::collections::HashMap;
use std::sync::Arc;

use color_eyre::eyre::{ContextCompat, WrapErr};
use inquire::{CustomType, Select};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::DeployCmdContext)]
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
    global_context: near_cli_rs::GlobalContext,
    social_db_prefix: String,
    deploy_to_account_id: near_primitives::types::AccountId,
    signer_account_id: near_primitives::types::AccountId,
}

impl SignerContext {
    pub fn from_previous_context(
        previous_context: super::DeployCmdContext,
        scope: &<Signer as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.global_context,
            social_db_prefix: previous_context.social_db_prefix,
            deploy_to_account_id: previous_context.deploy_to_account_id.into(),
            signer_account_id: scope.signer_account_id.clone().into(),
        })
    }
}

impl From<SignerContext> for near_cli_rs::commands::ActionContext {
    fn from(item: SignerContext) -> Self {
        let deploy_to_account_id = item.deploy_to_account_id.clone();
        let signer_id = item.signer_account_id.clone();
        let social_db_prefix = item.social_db_prefix.clone();

        let on_after_getting_network_callback: near_cli_rs::commands::OnAfterGettingNetworkCallback = Arc::new({
            move |network_config| {
                let near_social_account_id = crate::consts::NEAR_SOCIAL_ACCOUNT_ID.get(network_config.network_name.as_str())
                    .wrap_err_with(|| format!("The <{}> network does not have a near-social contract.", network_config.network_name))?;
                let mut prepopulated_transaction = near_cli_rs::commands::PrepopulatedTransaction {
                    signer_id: signer_id.clone(),
                    receiver_id: near_social_account_id.clone(),
                    actions: vec![],
                };
                let local_components = crate::common::get_local_components()?;
                if local_components.is_empty() {
                    println!("There are no components in the current ./src folder. Goodbye.");
                    return Ok(prepopulated_transaction);
                }
                let local_component_name_list = local_components.keys().collect::<Vec<_>>();
                let remote_components = crate::common::get_remote_components(
                    network_config,
                    local_component_name_list,
                    near_social_account_id,
                    &deploy_to_account_id,
                    &item.social_db_prefix
                )?;

                let components_to_deploy =
                    if !remote_components.is_empty() {
                        let updated_components = crate::common::get_updated_components(local_components, &remote_components);
                        if updated_components.is_empty() {
                            println!("There are no new or modified components in the current ./src folder. Goodbye.");
                            return Ok(prepopulated_transaction);
                        }
                        updated_components
                    } else {
                        println!("\nAll local components will be deployed to <{deploy_to_account_id}> as new.");
                        local_components
                    };

                let new_social_db_state = crate::socialdb_types::SocialDb {
                    accounts: HashMap::from([(
                        deploy_to_account_id.clone(),
                        crate::socialdb_types::SocialDbComponentKey {
                            key: HashMap::from([(
                                item.social_db_prefix.clone(),
                                crate::socialdb_types::SocialDbAccountMetadata {
                                    components: components_to_deploy
                                }
                            )])
                        }
                    )])
                };
                let new_social_db_state_json = serde_json::json!(&new_social_db_state);
                let remote_social_db_state_json = serde_json::json!(&crate::socialdb_types::SocialDb {
                    accounts: HashMap::from([(
                        deploy_to_account_id.clone(),
                        crate::socialdb_types::SocialDbComponentKey {
                            key: HashMap::from([(
                                item.social_db_prefix.clone(),
                                crate::socialdb_types::SocialDbAccountMetadata {
                                    components: remote_components
                                }
                            )])
                        }
                    )])
                });

                let args = serde_json::to_string(&super::TransactionFunctionArgs {
                    data: new_social_db_state,
                })?
                .into_bytes();

                let json_rpc_client = network_config.json_rpc_client();

                let deposit = tokio::runtime::Runtime::new().unwrap().block_on(
                    near_socialdb_client::required_deposit(
                        &json_rpc_client,
                        near_social_account_id,
                        &deploy_to_account_id,
                        &new_social_db_state_json,
                        Some(&remote_social_db_state_json),
                    )
                )?;

                prepopulated_transaction.actions = vec![
                    near_primitives::transaction::Action::FunctionCall(
                        near_primitives::transaction::FunctionCallAction {
                            method_name: "set".to_string(),
                            args,
                            gas: near_cli_rs::common::NearGas::from_tgas(300).as_gas(),
                            deposit: deposit.as_yoctonear(),
                        },
                    )
                ];

                Ok(prepopulated_transaction)
            }
        });

        let db_prefix = social_db_prefix.clone();
        let on_before_signing_callback: near_cli_rs::commands::OnBeforeSigningCallback =
            Arc::new({
                let signer_account_id = item.signer_account_id.clone();
                let deploy_to_account_id = item.deploy_to_account_id.clone();
                move |prepopulated_unsigned_transaction, network_config| {
                    let json_rpc_client = network_config.json_rpc_client();
                    if let near_primitives::transaction::Action::FunctionCall(action) =
                        &mut prepopulated_unsigned_transaction.actions[0]
                    {
                        action.deposit = tokio::runtime::Runtime::new()
                            .unwrap()
                            .block_on(near_socialdb_client::get_deposit(
                                &json_rpc_client,
                                &signer_account_id,
                                &prepopulated_unsigned_transaction.public_key,
                                &deploy_to_account_id,
                                &social_db_prefix,
                                &prepopulated_unsigned_transaction.receiver_id,
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

        let on_after_sending_transaction_callback: near_cli_rs::transaction_signature_options::OnAfterSendingTransactionCallback = Arc::new({
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
                    color_eyre::eyre::bail!("Components deployment failed!");
                };

                let transaction_function_args: super::TransactionFunctionArgs =
                    serde_json::from_slice(args).wrap_err("Internal error: Could not parse SocialDB request that we just created.")?;

                let social_account_metadata = transaction_function_args.data.accounts.get(item.deploy_to_account_id.as_ref())
                    .wrap_err("Internal error: Could not get the key for the component from SocialDB request that we just created.")?
                    .key
                    .get(&db_prefix)
                    .wrap_err("Internal error: Could not get metadata from SocialDB request that we just created.")?;
                let updated_components = &social_account_metadata.components;

                println!("\n<{}> components were successfully deployed to <{}>/{db_prefix}/:", updated_components.len(), item.deploy_to_account_id);
                for component in updated_components.keys() {
                    println!(" * {component}")
                }
                println!();
                Ok(())
            }
        });

        Self {
            global_context: item.global_context,
            interacting_with_account_ids: vec![item.signer_account_id],
            on_after_getting_network_callback,
            on_before_signing_callback,
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config, _message| Ok(()),
            ),
            on_after_sending_transaction_callback,
        }
    }
}

impl Signer {
    fn input_signer_account_id(
        context: &super::DeployCmdContext,
    ) -> color_eyre::eyre::Result<Option<near_cli_rs::types::account_id::AccountId>> {
        loop {
            let signer_account_id: near_cli_rs::types::account_id::AccountId =
                CustomType::new("What is the signer account ID?")
                    .with_default(context.deploy_to_account_id.clone())
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
