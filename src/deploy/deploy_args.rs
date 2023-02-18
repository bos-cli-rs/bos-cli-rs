use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use color_eyre::eyre::{ContextCompat, WrapErr};
use inquire::{CustomType, Select};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::DeployToAccountContext)]
#[interactive_clap(output_context = near_cli_rs::commands::ActionContext)]
#[interactive_clap(skip_default_from_cli)]
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
    deploy_to_account_id: near_cli_rs::types::account_id::AccountId,
    signer_account_id: near_cli_rs::types::account_id::AccountId,
}

impl SignerContext {
    pub fn from_previous_context(
        previous_context: super::DeployToAccountContext,
        scope: &<Signer as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.config,
            deploy_to_account_id: previous_context.deploy_to_account_id,
            signer_account_id: scope.signer_account_id.clone(),
        })
    }
}

impl From<SignerContext> for near_cli_rs::commands::ActionContext {
    fn from(item: SignerContext) -> Self {
        let on_after_getting_network_callback: near_cli_rs::commands::OnAfterGettingNetworkCallback = Arc::new({
            let deploy_to_account_id = item.deploy_to_account_id.clone();

            move |prepopulated_unsigned_transaction, network_config| {
                let near_social_account_id = network_config.near_social_account_id.as_ref()
                    .or_else(|| crate::consts::NEAR_SOCIAL_ACCOUNT_ID.get(network_config.network_name.as_str()))
                    .ok_or_else(||
                        color_eyre::eyre::eyre!(
                            "The <{}> network does not have a near-social contract.",
                            network_config.network_name
                        )
                    )?;

                let local_widgets = crate::common::get_local_widgets()?;
                if local_widgets.is_empty() {
                    println!("There are no widgets in the current ./src folder. Goodbye.");
                    return Ok(());
                }

                let query_view_method_response = tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(network_config.json_rpc_client().call(
                        near_jsonrpc_client::methods::query::RpcQueryRequest {
                            block_reference: near_primitives::types::Finality::Final.into(),
                            request: near_primitives::views::QueryRequest::CallFunction {
                                account_id: near_social_account_id.clone(),
                                method_name: "get".to_string(),
                                args: near_primitives::types::FunctionArgs::from(
                                    serde_json::to_string(&crate::socialdb_types::SocialDbQuery {
                                        keys: local_widgets
                                            .keys()
                                            .map(|name| format!("{deploy_to_account_id}/widget/{name}/**"))
                                            .collect(),
                                    })
                                    .wrap_err("Internal error: could not serialize SocialDB input args")?
                                    .into_bytes()
                                ),
                            },
                        },
                    ))
                    .wrap_err("Failed to fetch the widgets state from SocialDB")?;

                let old_social_db: crate::socialdb_types::SocialDb =
                    if let near_jsonrpc_primitives::types::query::QueryResponseKind::CallResult(result) =
                        query_view_method_response.kind
                    {
                        serde_json::from_slice(&result.result)
                            .wrap_err("Failed to parse the widgets state from SocialDB")?
                    } else {
                        return Err(color_eyre::Report::msg(
                            "Received unexpected query kind on fetching widgets state from SocialDB",
                        ));
                    };

                prepopulated_unsigned_transaction.receiver_id = near_social_account_id.clone();
                let (widgets_to_deploy, deposit) =
                    if let Some(account_metadata) = old_social_db.accounts.get(deploy_to_account_id.as_ref()) {
                        let updated_widgets: HashMap<String, crate::socialdb_types::SocialDbWidget> = local_widgets
                            .into_iter()
                            .filter(|(widget_name, new_widget)| {
                                if let Some(old_widget) = account_metadata.widgets.get(widget_name) {
                                    let has_code_changed = crate::common::diff_code(&old_widget.code, &new_widget.code).is_err();
                                    let has_metadata_changed = old_widget.metadata != new_widget.metadata && new_widget.metadata.is_some();
                                    if !has_code_changed {
                                        println!("Code for widget <{widget_name}> has not changed");
                                    }
                                    if has_metadata_changed {
                                        println!(
                                            "Metadata for widget <{widget_name}> changed\n{:?}\n{:?}",
                                            old_widget.metadata, new_widget.metadata
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
                            return Ok(());
                        }

                        (
                            updated_widgets,
                            near_cli_rs::common::NearBalance::from_str("0 NEAR").unwrap(), // TODO: storage cost should be properly calculated
                        )
                    } else {
                        println!("\nAll local widgets will be deployed to <{deploy_to_account_id}> as new.");
                        (
                            local_widgets,
                            near_cli_rs::common::NearBalance::from_str("1 NEAR").unwrap(), // TODO: storage cost should be properly calculated
                        )
                    };

                let args = {
                    let mut accounts = HashMap::new();
                    accounts.insert(
                        deploy_to_account_id.clone().into(),
                        crate::socialdb_types::SocialDbAccountMetadata {
                            widgets: widgets_to_deploy
                        },
                    );

                    serde_json::to_string(&super::TransactionFunctionArgs {
                        data: crate::socialdb_types::SocialDb { accounts },
                    })?
                };

                prepopulated_unsigned_transaction.actions = vec![
                    near_primitives::transaction::Action::FunctionCall(
                        near_primitives::transaction::FunctionCallAction {
                            method_name: "set".to_string(),
                            args: args.into_bytes(),
                            gas: near_cli_rs::common::NearGas::from_str("100 TeraGas")
                                .unwrap()
                                .inner,
                            deposit: deposit.to_yoctonear(),
                        },
                    )
                ];

                Ok(())
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
                        action.deposit = tokio::runtime::Runtime::new()
                            .unwrap()
                            .block_on(get_deposit(
                                network_config,
                                signer_account_id.clone().into(),
                                prepopulated_unsigned_transaction.public_key.clone(),
                                deploy_to_account_id.clone(),
                                prepopulated_unsigned_transaction.receiver_id.clone(),
                                near_cli_rs::common::NearBalance::from_yoctonear(action.deposit),
                            ))?
                            .to_yoctonear();
                        Ok(())
                    } else {
                        color_eyre::eyre::bail!("Unexpected action to change widgets",);
                    }
                }
            });

        let on_after_sending_transaction_callback: near_cli_rs::transaction_signature_options::OnAfterSendingTransactionCallback = Arc::new({
            let deploy_to_account_id = item.deploy_to_account_id.clone();

            move |transaction_info, network_config| {
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
                    near_cli_rs::common::print_transaction_status(
                        transaction_info.clone(),
                        network_config.clone(),
                    )?;
                    color_eyre::eyre::bail!("Widgets deployment failed!");
                };

                let transaction_function_args: super::TransactionFunctionArgs =
                    serde_json::from_slice(args).wrap_err("Internal error: Could not parse SocialDB request that we just created.")?;

                let social_account_metadata = transaction_function_args.data.accounts.get(deploy_to_account_id.as_ref())
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
            signer_account_id: item.signer_account_id.into(),
            receiver_account_id: "v1.social08.testnet".parse().unwrap(),
            actions: vec![],
            on_after_getting_network_callback,
            on_before_signing_callback,
            on_after_sending_transaction_callback,
        }
    }
}

impl interactive_clap::FromCli for Signer {
    type FromCliContext = super::DeployToAccountContext;
    type FromCliError = color_eyre::eyre::Error;

    fn from_cli(
        optional_clap_variant: Option<<Signer as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> Result<Option<Self>, Self::FromCliError>
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let signer_account_id = Self::from_cli_signer_account_id(
            optional_clap_variant
                .clone()
                .and_then(|clap_variant| clap_variant.signer_account_id),
            &context,
        )?;
        let new_context_scope = InteractiveClapContextScopeForSigner {
            signer_account_id: signer_account_id.clone(),
        };
        let new_context = SignerContext::from_previous_context(context, &new_context_scope)?;

        let optional_network_for_transaction =
            near_cli_rs::network_for_transaction::NetworkForTransactionArgs::from_cli(
                optional_clap_variant.and_then(|clap_variant| {
                    clap_variant.network_config.map(
                        |ClapNamedArgNetworkForTransactionArgsForSigner::NetworkConfig(cli_arg)| {
                            cli_arg
                        },
                    )
                }),
                new_context.into(),
            )?;
        let network_for_transaction = if let Some(network) = optional_network_for_transaction {
            network
        } else {
            return Ok(None);
        };

        Ok(Some(Self {
            signer_account_id,
            network_config: network_for_transaction,
        }))
    }
}

impl Signer {
    fn input_signer_account_id(
        context: &super::DeployToAccountContext,
    ) -> color_eyre::eyre::Result<near_cli_rs::types::account_id::AccountId> {
        loop {
            let signer_account_id: near_cli_rs::types::account_id::AccountId =
                CustomType::new(" What is the signer account ID?")
                    .with_default(context.deploy_to_account_id.clone())
                    .prompt()?;
            if !crate::common::is_account_exist(
                &context.config.networks,
                signer_account_id.clone().into(),
            ) {
                println!("\nThe account <{}> does not yet exist.", signer_account_id);
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
                    return Ok(signer_account_id);
                }
            } else {
                return Ok(signer_account_id);
            }
        }
    }
}

async fn get_deposit(
    network_config: &near_cli_rs::config::NetworkConfig,
    signer_account_id: near_primitives::types::AccountId,
    signer_public_key: near_crypto::PublicKey,
    deploy_to_account_id: near_cli_rs::types::account_id::AccountId,
    near_social_account_id: near_primitives::types::AccountId,
    required_deposit: near_cli_rs::common::NearBalance,
) -> color_eyre::eyre::Result<near_cli_rs::common::NearBalance> {
    let signer_access_key_permission = crate::common::get_access_key_permission(
        network_config,
        signer_account_id.clone(),
        signer_public_key.clone(),
    )
    .await?;

    let is_signer_access_key_full_access = matches!(
        signer_access_key_permission,
        near_primitives::views::AccessKeyPermissionView::FullAccess
    );

    let is_write_permission_granted_to_public_key = crate::common::is_write_permission_granted(
        network_config,
        near_social_account_id.clone(),
        signer_public_key,
        format!("{deploy_to_account_id}/widget"),
    )
    .await?;

    let is_write_permission_granted_to_signer = crate::common::is_write_permission_granted(
        network_config,
        near_social_account_id.clone(),
        signer_account_id.clone(),
        format!("{deploy_to_account_id}/widget"),
    )
    .await?;

    let deposit = if is_signer_access_key_full_access
        || crate::common::is_signer_access_key_function_call_access_can_call_set_on_social_db_account(
            near_social_account_id,
            signer_access_key_permission
        )?
    {
        if is_write_permission_granted_to_public_key || is_write_permission_granted_to_signer {
            if required_deposit == near_cli_rs::common::NearBalance::from_str("0 NEAR").unwrap()
            {
                near_cli_rs::common::NearBalance::from_str("0 NEAR").unwrap()
            } else if is_signer_access_key_full_access {
                required_deposit
            } else {
                color_eyre::eyre::bail!("ERROR: Social DB requires more storage deposit, but we cannot cover it when signing transaction with a Function Call only access key")
            }
        } else if signer_account_id == deploy_to_account_id.into() {
            if is_signer_access_key_full_access {
                if required_deposit
                    == near_cli_rs::common::NearBalance::from_str("0 NEAR").unwrap()
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
