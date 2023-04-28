use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use color_eyre::eyre::{ContextCompat, WrapErr};
use futures::StreamExt;
use inquire::{CustomType, Select};
use near_cli_rs::common::CallResultExt;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::DeployToAccountContext)]
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
    deploy_to_account_id: near_primitives::types::AccountId,
    signer_account_id: near_primitives::types::AccountId,
}

impl SignerContext {
    pub fn from_previous_context(
        previous_context: super::DeployToAccountContext,
        scope: &<Signer as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.config,
            deploy_to_account_id: previous_context.deploy_to_account_id.into(),
            signer_account_id: scope.signer_account_id.clone().into(),
        })
    }
}

impl From<SignerContext> for near_cli_rs::commands::ActionContext {
    fn from(item: SignerContext) -> Self {
        let deploy_to_account_id = item.deploy_to_account_id.clone();
        let signer_id = item.signer_account_id.clone();

        let on_after_getting_network_callback: near_cli_rs::commands::OnAfterGettingNetworkCallback = Arc::new({
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
                    actions: vec![],
                };
                let local_widgets = crate::common::get_local_widgets()?;
                if local_widgets.is_empty() {
                    println!("There are no widgets in the current ./src folder. Goodbye.");
                    return Ok(prepopulated_transaction);
                }

                let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;
                let chunk_size = 15;
                let concurrency = 10;
                let deploy_account_id = deploy_to_account_id.as_ref();

                let remote_widgets = runtime.block_on(
                    futures::stream::iter(&mut local_widgets.keys().cloned().collect::<Vec<_>>().chunks(chunk_size))
                        .map(|local_widgets_name_batch| async move {
                            Ok(get_widgets(
                                network_config.clone(),
                                near_social_account_id.clone(),
                                deploy_account_id,
                                local_widgets_name_batch.to_vec()
                            ).await?)
                        })
                        .buffer_unordered(concurrency)
                        .collect::<Vec<_>>()
                )
                    .into_iter()
                    .filter_map(|result: color_eyre::Result<HashMap<crate::socialdb_types::WidgetName, crate::socialdb_types::SocialDbWidget>>| result.ok())
                    .reduce(|mut acc, next| {
                        acc.extend(next);
                        acc
                    });

                let widgets_to_deploy =
                    if let Some(widgets) = remote_widgets.clone() {
                        let updated_widgets: HashMap<String, crate::socialdb_types::SocialDbWidget> = local_widgets
                            .into_iter()
                            .filter(|(widget_name, new_widget)| {
                                if let Some(old_widget) = widgets.get(widget_name) {
                                    let has_code_changed = crate::common::diff_code(old_widget.code(), new_widget.code()).is_err();
                                    let has_metadata_changed = old_widget.metadata() != new_widget.metadata() && new_widget.metadata().is_some();
                                    if !has_code_changed {
                                        println!("Code for widget <{widget_name}> has not changed");
                                    }
                                    if has_metadata_changed {
                                        println!(
                                            "Metadata for widget <{widget_name}> changed:\n - old metadata: {:?}\n - new metadata: {:?}",
                                            old_widget.metadata(), new_widget.metadata()
                                        );
                                    } else {
                                        println!("Metadata for widget <{widget_name}> has not changed");
                                    }
                                    has_code_changed || has_metadata_changed
                                } else {
                                    println!("Found new widget <{widget_name}> to deploy");
                                    true
                                }
                            })
                            .collect();

                        if updated_widgets.is_empty() {
                            println!("There are no new or modified widgets in the current ./src folder. Goodbye.");
                            return Ok(prepopulated_transaction);
                        }
                        updated_widgets
                    } else {
                        println!("\nAll local widgets will be deployed to <{deploy_to_account_id}> as new.");
                        local_widgets
                    };

                let mut accounts = HashMap::new();
                accounts.insert(
                    deploy_to_account_id.clone(),
                    crate::socialdb_types::SocialDbAccountMetadata {
                        widgets: widgets_to_deploy
                    },
                );

                let new_social_db_state = crate::socialdb_types::SocialDb { accounts: accounts.clone() };
                let new_social_db_state_json = serde_json::json!(&new_social_db_state);
                let remote_social_db_state_json = serde_json::json!(&crate::socialdb_types::SocialDb {
                    accounts: HashMap::from([(
                        deploy_to_account_id.clone(),
                        crate::socialdb_types::SocialDbAccountMetadata {
                            widgets: remote_widgets.unwrap_or_default()
                        }
                    )])
                });

                let args = serde_json::to_string(&super::TransactionFunctionArgs {
                    data: new_social_db_state,
                })?
                .into_bytes();

                let deposit = crate::common::required_deposit(
                    network_config,
                    near_social_account_id,
                    &deploy_to_account_id,
                    &new_social_db_state_json,
                    Some(&remote_social_db_state_json),
                )?;

                prepopulated_transaction.actions = vec![
                    near_primitives::transaction::Action::FunctionCall(
                        near_primitives::transaction::FunctionCallAction {
                            method_name: "set".to_string(),
                            args,
                            gas: near_cli_rs::common::NearGas::from_str("300 TeraGas")
                                .unwrap()
                                .inner,
                            deposit: deposit.to_yoctonear(),
                        },
                    )
                ];

                Ok(prepopulated_transaction)
            }
        });

        let on_before_signing_callback: near_cli_rs::commands::OnBeforeSigningCallback =
            Arc::new({
                let signer_account_id = item.signer_account_id.clone();
                let deploy_to_account_id = item.deploy_to_account_id.clone();
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
                        color_eyre::eyre::bail!("Unexpected action to change widgets",);
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
                    color_eyre::eyre::bail!("Widgets deployment failed!");
                };

                let transaction_function_args: super::TransactionFunctionArgs =
                    serde_json::from_slice(args).wrap_err("Internal error: Could not parse SocialDB request that we just created.")?;

                let social_account_metadata = transaction_function_args.data.accounts.get(item.deploy_to_account_id.as_ref())
                    .wrap_err("Internal error: Could not get metadata from SocialDB request that we just created.")?;
                let updated_widgets = &social_account_metadata.widgets;

                println!("\n<{}> widgets were successfully deployed:", updated_widgets.len());
                for widget in updated_widgets.keys() {
                    println!(" * {widget}")
                }
                println!();
                Ok(())
            }
        });

        Self {
            config: item.config,
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
        context: &super::DeployToAccountContext,
    ) -> color_eyre::eyre::Result<Option<near_cli_rs::types::account_id::AccountId>> {
        loop {
            let signer_account_id: near_cli_rs::types::account_id::AccountId =
                CustomType::new(" What is the signer account ID?")
                    .with_default(context.deploy_to_account_id.clone())
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
                "ERROR: signer is not allowed to modify deploy_to_account_id widgets."
            )
        }
    } else {
        color_eyre::eyre::bail!("ERROR: signer access key cannot be used to sign a transaction to update widgets in Social DB.")
    };
    Ok(deposit)
}

async fn get_widgets(
    network_config: near_cli_rs::config::NetworkConfig,
    near_social_account_id: near_primitives::types::AccountId,
    deploy_to_account_id: &str,
    local_widget_names_batch: Vec<String>,
) -> color_eyre::Result<
    HashMap<crate::socialdb_types::WidgetName, crate::socialdb_types::SocialDbWidget>,
> {
    let args = serde_json::to_string(&crate::socialdb_types::SocialDbQuery {
        keys: local_widget_names_batch
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
                account_id: near_social_account_id,
                method_name: "get".to_string(),
                args: near_primitives::types::FunctionArgs::from(args),
            },
        })
        .await
        .wrap_err("Failed to query batch of widgets from Social DB")?
        .kind
    {
        near_jsonrpc_primitives::types::query::QueryResponseKind::CallResult(call_result) => {
            Ok(call_result
                .parse_result_from_json::<crate::socialdb_types::SocialDb>()
                .wrap_err("ERROR: failed to parse Social DB response")?
                .accounts
                .get(deploy_to_account_id)
                .unwrap_or(&crate::socialdb_types::SocialDbAccountMetadata {
                    widgets: HashMap::new(),
                })
                .widgets
                .clone())
        }
        _ => unreachable!("ERROR: unexpected response type from JSON RPC client"),
    }
}
