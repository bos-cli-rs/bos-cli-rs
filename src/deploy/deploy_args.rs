use std::collections::HashMap;
use std::str::FromStr;

use color_eyre::eyre::WrapErr;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::DeployToAccountContext)]
#[interactive_clap(output_context = near_cli_rs::commands::ActionContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct DeployArgs {
    /// What is the signer account ID?
    signer_account_id: near_cli_rs::types::account_id::AccountId,
    #[interactive_clap(skip)]
    near_social_account_id: near_cli_rs::types::account_id::AccountId,
    #[interactive_clap(skip)]
    function_args: String,
    #[interactive_clap(skip)]
    deposit: near_cli_rs::common::NearBalance,
    #[interactive_clap(named_arg)]
    /// Select network
    network_for_transaction: near_cli_rs::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Clone)]
pub struct DeployArgsContext {
    config: near_cli_rs::config::Config,
    deploy_to_account_id: near_cli_rs::types::account_id::AccountId,
    signer_account_id: near_cli_rs::types::account_id::AccountId,
    near_social_account_id: near_cli_rs::types::account_id::AccountId,
    function_args: String,
    deposit: near_cli_rs::common::NearBalance,
}

impl DeployArgsContext {
    pub fn from_previous_context(
        previous_context: super::DeployToAccountContext,
        scope: &<DeployArgs as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let mut deposit = near_cli_rs::common::NearBalance::from_str("0 NeEAR");

        Ok(Self {
            config: previous_context.config,
            deploy_to_account_id: previous_context.deploy_to_account_id,
            signer_account_id: scope.signer_account_id.clone(),
            near_social_account_id: scope.near_social_account_id.clone(),
            function_args: scope.function_args.clone(),
            deposit: scope.deposit.clone(),
        })
    }
}

impl From<DeployArgsContext> for near_cli_rs::commands::ActionContext {
    fn from(item: DeployArgsContext) -> Self {
        let deploy_to_account_id = item.deploy_to_account_id.clone();
        let args = item.function_args.clone().into_bytes();
        Self {
            config: item.config,
            signer_account_id: item.signer_account_id.clone().into(),
            receiver_account_id: item.near_social_account_id.into(),
            actions: vec![near_primitives::transaction::Action::FunctionCall(
                near_primitives::transaction::FunctionCallAction {
                    method_name: "set".to_string(),
                    args: item.function_args.into_bytes(),
                    gas: near_cli_rs::common::NearGas::from_str("100 TeraGas")
                        .unwrap()
                        .inner,
                    deposit: item.deposit.to_yoctonear(),
                },
            )],
            on_before_signing_callback: std::sync::Arc::new(
                move |prepolulated_unsinged_transaction, network_config| {
                    let near_social_account_id = match &network_config.near_social_account_id {
                        Some(account_id) => account_id.clone(),
                        None => {
                            return Err(color_eyre::Report::msg(format!(
                                "The <{}> network does not have a near-social contract.",
                                network_config.network_name
                            )))
                        }
                    };
                    prepolulated_unsinged_transaction.receiver_id = near_social_account_id.clone();
                    let deposit = tokio::runtime::Runtime::new()
                        .unwrap()
                        .block_on(get_deposit(
                            network_config,
                            prepolulated_unsinged_transaction.signer_id.clone(),
                            prepolulated_unsinged_transaction.public_key.clone(),
                            deploy_to_account_id.clone(),
                            near_social_account_id,
                            near_cli_rs::common::NearBalance::from_str("1 NEAR").unwrap(), // XXX   1 NEAR: need calculation!!!!!!!! for new account
                        ))?;
                    if let near_primitives::transaction::Action::FunctionCall(action) =
                        &mut prepolulated_unsinged_transaction.actions[0]
                    {
                        action.deposit = deposit.to_yoctonear();
                        action.args = args.clone();
                    } else {
                        return Err(color_eyre::Report::msg(
                            "Unexpected action to change widgets",
                        ));
                    }
                    Ok(())
                },
            ),
            on_after_signing_callback: std::sync::Arc::new(|singed_transaction| {
                if let near_primitives::transaction::SignedTransaction { transaction, .. } =
                    singed_transaction
                {
                    println!(
                        "= = = = = = = = = = signed transaction: {}",
                        transaction.public_key
                    )
                }

                Ok(())
            }),
        }
    }
}

impl interactive_clap::FromCli for DeployArgs {
    type FromCliContext = super::DeployToAccountContext;
    type FromCliError = color_eyre::eyre::Error;

    fn from_cli(
        optional_clap_variant: Option<<DeployArgs as interactive_clap::ToCli>::CliVariant>,
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
        // XXX
        let new_context_scope = InteractiveClapContextScopeForDeployArgs {
            signer_account_id: signer_account_id.clone(),
            near_social_account_id: "v1.social08.testnet".parse().unwrap(),
            function_args: "".to_string(),
            deposit: near_cli_rs::common::NearBalance::from_str("0 NEAR").unwrap(),
        };
        let deploy_args_context =
            DeployArgsContext::from_previous_context(context.clone(), &new_context_scope)?;
        let new_context = near_cli_rs::commands::ActionContext::from(deploy_args_context);

        let optional_network_for_transaction =
            near_cli_rs::network_for_transaction::NetworkForTransactionArgs::from_cli(
                optional_clap_variant.and_then(|clap_variant| {
                    match clap_variant.network_for_transaction {
                        Some(
                            ClapNamedArgNetworkForTransactionArgsForDeployArgs::NetworkForTransaction(cli_arg)
                        ) => Some(cli_arg),
                        None => None,
                    }
                }),
                new_context.clone().into(),
            )?;
        let network_for_transaction = if let Some(network) = optional_network_for_transaction {
            network
        } else {
            return Ok(None);
        };


        

        let network_config = &network_for_transaction.get_network_config(context.config);

        let signer_public_key = &network_for_transaction.get_signer_public_key();
        println!("************ signer_public_key: {:?}", signer_public_key);

        let near_social_account_id = match network_config.near_social_account_id.clone() {
            Some(account_id) => account_id,
            None => {
                return Err(color_eyre::Report::msg(format!(
                    "The <{}> network does not have a near-social contract.",
                    network_config.network_name
                )))
            }
        };




        let widgets = crate::common::get_widgets()?;

        if widgets.is_empty() {
            // XXX ??????????????????
            println!("There are no widgets in the current ./src folder. Goodbye.");
        }

        let input_args = serde_json::to_string(&crate::socialdb_types::SocialDbQuery {
            keys: widgets
                .keys()
                .map(|name| format!("{}/widget/{name}/**", context.deploy_to_account_id))
                .collect(),
        })
        .wrap_err("Internal error: could not serialize SocialDB input args")?;

        let query_view_method_response = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(network_config.json_rpc_client().call(
                near_jsonrpc_client::methods::query::RpcQueryRequest {
                    block_reference: near_primitives::types::Finality::Final.into(),
                    request: near_primitives::views::QueryRequest::CallFunction {
                        account_id: near_social_account_id.clone(),
                        method_name: "get".to_string(),
                        args: near_primitives::types::FunctionArgs::from(input_args.into_bytes()),
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
                        context.deploy_to_account_id.clone(),
                    ))
            {
                account_metadata
            } else {
                println!("\nThere are currently no widgets in the account <{}>. Therefore, all widgets will be deployed as new", context.deploy_to_account_id);
                let deposit = tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(get_deposit(
                        network_config,
                        signer_account_id.clone().into(),
                        signer_public_key.clone(),
                        context.deploy_to_account_id.clone(),
                        near_social_account_id.clone(),
                        near_cli_rs::common::NearBalance::from_str("1 NEAR").unwrap(), // XXX   1 NEAR: need calculation!!!!!!!! for new account
                    ))?;
                return Ok(Some(Self {
                    signer_account_id: signer_account_id.clone(),
                    near_social_account_id: near_social_account_id.into(),
                    function_args: get_function_args(context.deploy_to_account_id, widgets)?,
                    deposit,
                    network_for_transaction,
                }));
            };

        let output_widgets = widgets
            .into_iter()
            .filter(|(widget_name, new_widget)| {
                if let Some(old_widget) = old_social_account_metadata.widgets.get(widget_name) {
                    let has_code_changed =
                        crate::common::diff_code(&old_widget.code, &new_widget.code).is_err();
                    let has_metadata_changed =
                        old_widget.metadata != new_widget.metadata && new_widget.metadata.is_some();
                    if has_code_changed {
                        println!("Code for widget <{widget_name}> changed");
                    } else {
                        println!("Code for widget <{widget_name}> has not changed");
                    }
                    if has_metadata_changed {
                        println!("{:?}\n{:?}", old_widget.metadata, new_widget.metadata);
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
            // XXX ??????????????????
            println!("There are no new or modified widgets in the current ./src folder. Goodbye.");
        }

        let deposit = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(get_deposit(
                network_config,
                signer_account_id.clone().into(),
                signer_public_key.clone(),
                context.deploy_to_account_id.clone(),
                near_social_account_id.clone(),
                near_cli_rs::common::NearBalance::from_str("0 NEAR").unwrap(), // XXX: need calculation!!!!!!!! for an existing account
            ))?;

        Ok(Some(Self {
            signer_account_id: signer_account_id.clone(),
            near_social_account_id: near_social_account_id.into(),
            function_args: get_function_args(context.deploy_to_account_id, output_widgets)?,
            deposit,
            network_for_transaction,
        }))
    }
}

impl DeployArgs {
    pub fn get_signer_account_id(&self) -> near_cli_rs::types::account_id::AccountId {
        self.signer_account_id.clone()
    }

    pub fn get_network_config_for_transaction(
        &self,
    ) -> near_cli_rs::network_for_transaction::NetworkForTransactionArgs {
        self.network_for_transaction.clone()
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
            near_social_account_id.clone(),
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
        near_primitives::types::AccountId::from(deploy_to_account_id.clone()),
        crate::socialdb_types::SocialDbAccountMetadata {
            widgets: widgets.clone(),
        },
    );

    let function_args = serde_json::to_string(&super::TransactionFunctionArgs {
        data: crate::socialdb_types::SocialDb { accounts },
    })?;
    Ok(function_args)
}
