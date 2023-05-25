use std::collections::HashMap;
use std::str::FromStr;

use color_eyre::eyre::{ContextCompat, WrapErr};
use futures::StreamExt;
use inquire::{CustomType, Select};
use near_cli_rs::common::{CallResultExt, JsonRpcClientExt};
use std::sync::Arc;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::SetContext)]
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
        previous_context: super::SetContext,
        scope: &<Signer as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let set_to_account_id: near_primitives::types::AccountId =
            previous_context.set_to_account_id.clone().into();
        let signer_id: near_primitives::types::AccountId = scope.signer_account_id.clone().into();
        let data_to_set = serde_json::Value::String(previous_context.function_args.clone());


        let on_after_getting_network_callback: near_cli_rs::commands::OnAfterGettingNetworkCallback = Arc::new({
            let key = previous_context.key.clone();
            let signer_id = signer_id.clone();
            let function_args = previous_context.function_args.clone();
            let function_args_type = previous_context.function_args_type.clone();

            move |network_config| {
                let near_social_account_id = crate::consts::NEAR_SOCIAL_ACCOUNT_ID.get(network_config.network_name.as_str())
                    .wrap_err_with(|| format!("The <{}> network does not have a near-social contract.", network_config.network_name))?;
                
                let mut prepopulated_transaction = near_cli_rs::commands::PrepopulatedTransaction {
                    signer_id: signer_id.clone(),
                    receiver_id: near_social_account_id.clone(),
                    actions: vec![],
                };

                let input_args = serde_json::to_string(&crate::socialdb_types::SocialDbQuery {
                    keys: vec![format!("{key}")],
                })
                .wrap_err("Internal error: could not serialize SocialDB input args")?;

                let remote_social_db_data_to_set: serde_json::Value = network_config
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
                if remote_social_db_data_to_set.as_object().map(|result| result.is_empty()).unwrap_or(true) { //XXX
                    println!("No keys to remove. Goodbye.");
                    return Ok(near_cli_rs::commands::PrepopulatedTransaction {
                        signer_id: signer_id.clone().into(),
                        receiver_id: near_social_account_id.clone(),
                        actions: vec![],
                    });
                }




                // let data_to_set = near_cli_rs::commands::contract::call_function::call_function_args_type::function_args(
                //     function_args.clone(),
                //     function_args_type.clone(),
                // )?;

                let mut data = serde_json::Map::new();
                crate::common::social_db_data_from_key(&mut data, key.clone(), data_to_set.clone());

                let social_db_data_to_set = serde_json::Value::Object(data);



                
                // let data_to_set_value: serde_json::Value = serde_json::from_slice(&data_to_set).wrap_err_with(|| {
                //     format!(
                //         "Failed to parse view-function call return value: {}",
                //         String::from_utf8_lossy(&data_to_set)
                //     )
                // })?;
                // let mut social_db_data_to_set = serde_json::Map::new();

                

                // let components_to_deploy =
                // if !remote_components.is_empty() {
                //         let updated_components: HashMap<String, crate::socialdb_types::SocialDbComponent> = local_components
                //             .into_iter()
                //             .filter(|(component_name, new_component)| {
                //                 if let Some(old_component) = remote_components.get(component_name) {
                //                     let has_code_changed = crate::common::diff_code(old_component.code(), new_component.code()).is_err();
                //                     let has_metadata_changed = old_component.metadata() != new_component.metadata() && new_component.metadata().is_some();
                //                     if !has_code_changed {
                //                         println!("Code for component <{component_name}> has not changed");
                //                     }
                //                     if has_metadata_changed {
                //                         println!(
                //                             "Metadata for component <{component_name}> changed:\n - old metadata: {:?}\n - new metadata: {:?}",
                //                             old_component.metadata(), new_component.metadata()
                //                         );
                //                     } else {
                //                         println!("Metadata for component <{component_name}> has not changed");
                //                     }
                //                     has_code_changed || has_metadata_changed
                //                 } else {
                //                     println!("Found new component <{component_name}> to deploy");
                //                     true
                //                 }
                //             })
                //             .collect();

                //         if updated_components.is_empty() {
                //             println!("There are no new or modified components in the current ./src folder. Goodbye.");
                //             return Ok(prepopulated_transaction);
                //         }
                //         updated_components
                //     } else {
                //         println!("\nAll local components will be deployed to <{deploy_to_account_id}> as new.");
                //         local_components
                //     };

                // let new_social_db_state = crate::socialdb_types::SocialDb {
                //     accounts: HashMap::from([(
                //         set_to_account_id.clone(),
                //         crate::socialdb_types::SocialDbAccountMetadata {
                //             components: components_to_deploy
                //         },
                //     )])
                // };
                // let new_social_db_state_json = serde_json::json!(&new_social_db_state);
                // let remote_social_db_state_json = serde_json::json!(&crate::socialdb_types::SocialDb {
                //     accounts: HashMap::from([(
                //         set_to_account_id.clone(),
                //         crate::socialdb_types::SocialDbAccountMetadata {
                //             components: remote_components
                //         }
                //     )])
                // });

                // let args = serde_json::to_string(&super::TransactionFunctionArgs {
                //     data: new_social_db_state,
                // })?
                // .into_bytes();

                // let deposit = crate::common::required_deposit(
                //     network_config,
                //     near_social_account_id,
                //     &set_to_account_id,
                //     &new_social_db_state_json,
                //     Some(&remote_social_db_state_json),
                // )?;

                prepopulated_transaction.actions = vec![
                    near_primitives::transaction::Action::FunctionCall(
                        near_primitives::transaction::FunctionCallAction {
                            method_name: "set".to_string(),
                            args: vec![],
                            gas: near_cli_rs::common::NearGas::from_str("300 TeraGas")
                                .unwrap()
                                .inner,
                            // deposit: deposit.to_yoctonear(),
                            deposit: near_cli_rs::common::NearBalance::from_yoctonear(0).to_yoctonear()
                        },
                    )
                ];

                Ok(prepopulated_transaction)
            }
        });

        let on_before_signing_callback: near_cli_rs::commands::OnBeforeSigningCallback =
            Arc::new({
                let signer_account_id = signer_id.clone();
                let deploy_to_account_id = set_to_account_id.clone();
                move |prepopulated_unsigned_transaction, network_config| {
                    if let near_primitives::transaction::Action::FunctionCall(action) =
                        &mut prepopulated_unsigned_transaction.actions[0]
                    {
                        action.deposit = get_deposit(
                            network_config,
                            &signer_account_id,
                            &prepopulated_unsigned_transaction.public_key,
                            &deploy_to_account_id,
                            &prepopulated_unsigned_transaction.receiver_id,
                            near_cli_rs::common::NearBalance::from_yoctonear(action.deposit),
                        )?
                        .to_yoctonear();
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

                // let transaction_function_args: super::TransactionFunctionArgs =
                //     serde_json::from_slice(args).wrap_err("Internal error: Could not parse SocialDB request that we just created.")?;

                // let social_account_metadata = transaction_function_args.data.accounts.get(item.deploy_to_account_id.as_ref())
                //     .wrap_err("Internal error: Could not get metadata from SocialDB request that we just created.")?;
                // let updated_components = &social_account_metadata.components;

                // println!("\n<{}> components were successfully deployed:", updated_components.len());
                // for component in updated_components.keys() {
                //     println!(" * {component}")
                // }
                println!();
                Ok(())
            }
        });

        Ok(Self(near_cli_rs::commands::ActionContext {
            config: previous_context.config,
            on_after_getting_network_callback,
            on_before_signing_callback,
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config, _message| Ok(()),
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
        context: &super::SetContext,
    ) -> color_eyre::eyre::Result<Option<near_cli_rs::types::account_id::AccountId>> {
        loop {
            let signer_account_id: near_cli_rs::types::account_id::AccountId =
                CustomType::new("What is the signer account ID?")
                    .with_default(context.set_to_account_id.clone())
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

fn get_deposit(
    network_config: &near_cli_rs::config::NetworkConfig,
    signer_account_id: &near_primitives::types::AccountId,
    signer_public_key: &near_crypto::PublicKey,
    deploy_to_account_id: &near_primitives::types::AccountId,
    near_social_account_id: &near_primitives::types::AccountId,
    required_deposit: near_cli_rs::common::NearBalance,
) -> color_eyre::eyre::Result<near_cli_rs::common::NearBalance> {
    let signer_access_key_permission = crate::common::get_access_key_permission(
        network_config,
        signer_account_id,
        signer_public_key,
    )?;

    let is_signer_access_key_full_access = matches!(
        signer_access_key_permission,
        near_primitives::views::AccessKeyPermissionView::FullAccess
    );

    let is_write_permission_granted_to_public_key = crate::common::is_write_permission_granted(
        network_config,
        near_social_account_id,
        signer_public_key.clone(),
        format!("{deploy_to_account_id}/widget"),
    )?;

    let is_write_permission_granted_to_signer = crate::common::is_write_permission_granted(
        network_config,
        near_social_account_id,
        signer_account_id.clone(),
        format!("{deploy_to_account_id}/widget"),
    )?;

    let deposit = if is_signer_access_key_full_access
        || crate::common::is_signer_access_key_function_call_access_can_call_set_on_social_db_account(
            near_social_account_id,
            &signer_access_key_permission
        )?
    {
        if is_write_permission_granted_to_public_key || is_write_permission_granted_to_signer {
            if required_deposit.is_zero()
            {
                near_cli_rs::common::NearBalance::from_str("0 NEAR").unwrap()
            } else if is_signer_access_key_full_access {
                required_deposit
            } else {
                color_eyre::eyre::bail!("ERROR: Social DB requires more storage deposit, but we cannot cover it when signing transaction with a Function Call only access key")
            }
        } else if signer_account_id == deploy_to_account_id {
            if is_signer_access_key_full_access {
                if required_deposit.is_zero()
                {
                    near_cli_rs::common::NearBalance::from_str("1 yoctoNEAR").unwrap()
                } else {
                    required_deposit
                }
            } else {
                color_eyre::eyre::bail!("ERROR: Social DB requires more storage deposit, but we cannot cover it when signing transaction with a Function Call only access key")
            }
        } else {
            color_eyre::eyre::bail!(
                "ERROR: signer is not allowed to modify deploy_to_account_id components."
            )
        }
    } else {
        color_eyre::eyre::bail!("ERROR: signer access key cannot be used to sign a transaction to update components in Social DB.")
    };
    Ok(deposit)
}

async fn get_components(
    network_config: &near_cli_rs::config::NetworkConfig,
    near_social_account_id: &near_primitives::types::AccountId,
    deploy_to_account_id: &near_primitives::types::AccountId,
    local_components_names_batch: &[&crate::socialdb_types::ComponentName],
) -> color_eyre::Result<
    HashMap<crate::socialdb_types::ComponentName, crate::socialdb_types::SocialDbComponent>,
> {
    let args = serde_json::to_string(&crate::socialdb_types::SocialDbQuery {
        keys: local_components_names_batch
            .iter()
            .map(|name| format!("{deploy_to_account_id}/widget/{name}/**"))
            .collect(),
    })
    .wrap_err("Internal error: could not serialize SocialDB input args")?
    .into_bytes();

    match network_config
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
        .wrap_err("Failed to query batch of components from Social DB")?
        .kind
    {
        near_jsonrpc_primitives::types::query::QueryResponseKind::CallResult(call_result) => {
            Ok(call_result
                .parse_result_from_json::<crate::socialdb_types::SocialDb>()
                .wrap_err("ERROR: failed to parse Social DB response")?
                .accounts
                .remove(deploy_to_account_id)
                .map(|crate::socialdb_types::SocialDbAccountMetadata { components }| components)
                .unwrap_or_default())
        }
        _ => unreachable!("ERROR: unexpected response type from JSON RPC client"),
    }
}
