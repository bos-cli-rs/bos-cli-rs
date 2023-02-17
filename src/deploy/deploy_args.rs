use std::collections::HashMap;
use std::str::FromStr;

use color_eyre::eyre::WrapErr;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::DeployToAccountContext)]
#[interactive_clap(output_context = near_cli_rs::commands::ActionContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct Signer {
    /// What is the signer account ID?
    signer_account_id: near_cli_rs::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_for_transaction: near_cli_rs::network_for_transaction::NetworkForTransactionArgs,
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
        let deploy_to_account_id = item.deploy_to_account_id.clone();
        let deploy_to_account_id_copy = item.deploy_to_account_id.clone();
        let signer_account_id = item.signer_account_id.clone();

        let on_after_getting_network_callback: near_cli_rs::commands::OnAfterGettingNetworkCallback = {
            std::sync::Arc::new(
                move |prepopulated_unsigned_transaction, network_config| {
                    let near_social_account_id = match &network_config.near_social_account_id {
                        Some(account_id) => account_id.clone(),
                        None => {
                            match crate::consts::NEAR_SOCIAL_ACCOUNT_ID
                                .get(&network_config.network_name.as_str())
                            {
                                Some(account_id) => account_id.clone(),
                                None => {
                                    return Err(color_eyre::Report::msg(format!(
                                        "The <{}> network does not have a near-social contract.",
                                        network_config.network_name
                                    )))
                                }
                            }
                        }
                    };
                    prepopulated_unsigned_transaction.receiver_id = near_social_account_id.clone();
                    let widgets = crate::common::get_widgets()?;

                    if widgets.is_empty() {
                        println!("There are no widgets in the current ./src folder. Goodbye.");
                        return Ok(());
                    }

                    let input_args = serde_json::to_string(&crate::socialdb_types::SocialDbQuery {
                        keys: widgets
                            .keys()
                            .map(|name| format!("{deploy_to_account_id}/widget/{name}/**"))
                            .collect(),
                    })
                    .wrap_err("Internal error: could not serialize SocialDB input args")?;

                    let query_view_method_response = tokio::runtime::Runtime::new()
                        .unwrap()
                        .block_on(network_config.json_rpc_client().call(
                            near_jsonrpc_client::methods::query::RpcQueryRequest {
                                block_reference: near_primitives::types::Finality::Final.into(),
                                request: near_primitives::views::QueryRequest::CallFunction {
                                    account_id: near_social_account_id,
                                    method_name: "get".to_string(),
                                    args: near_primitives::types::FunctionArgs::from(
                                        input_args.into_bytes(),
                                    ),
                                },
                            },
                        ))
                        .wrap_err("Failed to fetch the widgets state from SocialDB")?;

                    let call_result =
                            if let near_jsonrpc_primitives::types::query::QueryResponseKind::CallResult(result) =
                                query_view_method_response.kind
                            {
                                result
                            } else {
                                return Err(color_eyre::Report::msg(
                                    "Received unexpected query kind on fetching widgets state from SocialDB",
                                ));
                            };

                    let old_social_db: crate::socialdb_types::SocialDb =
                        serde_json::from_slice(&call_result.result)
                            .wrap_err("Failed to parse the widgets state from SocialDB")?;

                    let old_social_account_metadata: &crate::socialdb_types::SocialDbAccountMetadata =
                            if let Some(account_metadata) =
                                old_social_db
                                    .accounts
                                    .get(&near_primitives::types::AccountId::from(
                                        deploy_to_account_id.clone(),
                                    ))
                            {
                                account_metadata
                            } else {
                                println!("\nThere are currently no widgets in the account <{deploy_to_account_id}>. Therefore, all widgets will be deployed as new");
                                let args = get_function_args(deploy_to_account_id.clone(), widgets)?.into_bytes();
                                prepopulated_unsigned_transaction.actions = vec![near_primitives::transaction::Action::FunctionCall(
                                    near_primitives::transaction::FunctionCallAction {
                                        method_name: "set".to_string(),
                                        args,
                                        gas: near_cli_rs::common::NearGas::from_str("100 TeraGas")
                                            .unwrap()
                                            .inner,
                                        deposit: near_cli_rs::common::NearBalance::from_str("1 NEAR").unwrap().to_yoctonear(), // XXX   1 NEAR: need calculation!!!!!!!! for new account
                                    },
                                )];
                                        return Ok(());
                            };

                    let output_widgets = widgets
                        .into_iter()
                        .filter(|(widget_name, new_widget)| {
                            if let Some(old_widget) =
                                old_social_account_metadata.widgets.get(widget_name)
                            {
                                let has_code_changed =
                                    crate::common::diff_code(&old_widget.code, &new_widget.code)
                                        .is_err();
                                let has_metadata_changed = old_widget.metadata
                                    != new_widget.metadata
                                    && new_widget.metadata.is_some();
                                if has_code_changed {
                                    println!("Code for widget <{widget_name}> changed");
                                } else {
                                    println!("Code for widget <{widget_name}> has not changed");
                                }
                                if has_metadata_changed {
                                    println!(
                                        "{:?}\n{:?}",
                                        old_widget.metadata, new_widget.metadata
                                    );
                                    println!("Metadata for widget <{widget_name}> changed");
                                } else {
                                    println!("Metadata for widget <{widget_name}> has not changed");
                                }
                                has_code_changed || has_metadata_changed
                            } else {
                                println!("Found new widget <{widget_name}> to deploy");
                                true
                            }
                        })
                        .collect::<HashMap<String, crate::socialdb_types::SocialDbWidget>>();

                    if output_widgets.is_empty() {
                        println!("There are no new or modified widgets in the current ./src folder. Goodbye.");
                        return Ok(());
                    }

                    let args = get_function_args(deploy_to_account_id.clone(), output_widgets)?
                        .into_bytes();

                    prepopulated_unsigned_transaction.actions =
                        vec![near_primitives::transaction::Action::FunctionCall(
                            near_primitives::transaction::FunctionCallAction {
                                method_name: "set".to_string(),
                                args,
                                gas: near_cli_rs::common::NearGas::from_str("100 TeraGas")
                                    .unwrap()
                                    .inner,
                                deposit: 0,
                            },
                        )];

                    Ok(())
                },
            )
        };

        let on_before_signing_callback: near_cli_rs::commands::OnBeforeSigningCallback = {
            std::sync::Arc::new(move |prepopulated_unsigned_transaction, network_config| {
                if let near_primitives::transaction::Action::FunctionCall(action) =
                    &mut prepopulated_unsigned_transaction.actions[0]
                {
                    action.deposit = tokio::runtime::Runtime::new()
                        .unwrap()
                        .block_on(get_deposit(
                            network_config,
                            item.signer_account_id.clone().into(),
                            prepopulated_unsigned_transaction.public_key.clone(),
                            item.deploy_to_account_id.clone(),
                            prepopulated_unsigned_transaction.receiver_id.clone(),
                            near_cli_rs::common::NearBalance::from_yoctonear(action.deposit),
                        ))?
                        .to_yoctonear();
                } else {
                    return Err(color_eyre::Report::msg(
                        "Unexpected action to change widgets",
                    ));
                }
                Ok(())
            })
        };

        let on_after_sending_transaction_callback: near_cli_rs::transaction_signature_options::OnAfterSendingTransactionCallback = {
            std::sync::Arc::new(
                move |transaction_info, network_config| match transaction_info.status {
                    near_primitives::views::FinalExecutionStatus::SuccessValue(_) => {
                        let args = if let near_primitives::views::ActionView::FunctionCall {
                            args,
                            ..
                        } = &transaction_info.transaction.actions[0]
                        {
                            args
                        } else {
                            return Err(color_eyre::Report::msg(
                                "Internal error: Unexpected function call arguments",
                            ));
                        };

                        let transaction_function_args: super::TransactionFunctionArgs =
                            serde_json::from_slice(args).map_err(|err| {
                                color_eyre::Report::msg(format!("Error reading data: {}", err))
                            })?;

                        let social_account_metadata = if let Some(account_metadata) =
                            transaction_function_args.data.accounts.get(
                                &near_primitives::types::AccountId::from(
                                    deploy_to_account_id_copy.clone(),
                                ),
                            ) {
                            account_metadata
                        } else {
                            return Err(color_eyre::Report::msg(format!(
                                "Internal error: Unexpected metadata for account <{}>",
                                &deploy_to_account_id_copy
                            )));
                        };
                        let widgets = &social_account_metadata.widgets;

                        println!("\n<{}> widgets were successfully deployed:", widgets.len());
                        for widget in widgets.keys() {
                            println!(" * {widget}")
                        }
                        println!();
                        Ok(())
                    }
                    _ => {
                        near_cli_rs::common::print_transaction_status(
                            transaction_info.clone(),
                            network_config.clone(),
                        )?;
                        color_eyre::eyre::bail!("Widgets deployment failed!");
                    }
                },
            )

        };

        Self {
            config: item.config,
            signer_account_id: signer_account_id.into(),
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
        let deploy_args_context =
            SignerContext::from_previous_context(context, &new_context_scope)?;
        let new_context = near_cli_rs::commands::ActionContext::from(deploy_args_context);

        let optional_network_for_transaction =
            near_cli_rs::network_for_transaction::NetworkForTransactionArgs::from_cli(
                optional_clap_variant.and_then(|clap_variant| {
                    clap_variant.network_for_transaction.map(
                        |ClapNamedArgNetworkForTransactionArgsForSigner::NetworkForTransaction(
                            cli_arg,
                        )| cli_arg,
                    )
                }),
                new_context,
            )?;
        let network_for_transaction = if let Some(network) = optional_network_for_transaction {
            network
        } else {
            return Ok(None);
        };

        Ok(Some(Self {
            signer_account_id,
            network_for_transaction,
        }))
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

fn get_function_args(
    deploy_to_account_id: near_cli_rs::types::account_id::AccountId,
    widgets: HashMap<String, crate::socialdb_types::SocialDbWidget>,
) -> color_eyre::eyre::Result<String> {
    let mut accounts = HashMap::new();
    accounts.insert(
        near_primitives::types::AccountId::from(deploy_to_account_id),
        crate::socialdb_types::SocialDbAccountMetadata { widgets },
    );

    let function_args = serde_json::to_string(&super::TransactionFunctionArgs {
        data: crate::socialdb_types::SocialDb { accounts },
    })?;
    Ok(function_args)
}
