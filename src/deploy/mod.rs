use std::collections::HashMap;
use std::str::FromStr;

use color_eyre::eyre::WrapErr;
use inquire::{CustomType, Select};

mod deploy_args;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransactionFunctionArgs {
    pub data: crate::socialdb_types::SocialDb,
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = near_cli_rs::GlobalContext)]
#[interactive_clap(output_context = DeployToAccountContext)]
pub struct DeployToAccount {
    #[interactive_clap(skip_default_input_arg)]
    /// Which account do you want to deploy the widgets to?
    deploy_to_account_id: near_cli_rs::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Specify signer account ID
    sign_as: self::deploy_args::DeployArgs,
}

#[derive(Debug, Clone)]
pub struct DeployToAccountContext {
    pub config: near_cli_rs::config::Config,
    pub deploy_to_account_id: near_cli_rs::types::account_id::AccountId,
}

impl DeployToAccountContext {
    pub fn from_previous_context(
        previous_context: near_cli_rs::GlobalContext,
        scope: &<DeployToAccount as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> Self {
        Self {
            config: previous_context.0,
            deploy_to_account_id: scope.deploy_to_account_id.clone(),
        }
    }
}

impl DeployToAccount {
    fn input_deploy_to_account_id(
        context: &near_cli_rs::GlobalContext,
    ) -> color_eyre::eyre::Result<near_cli_rs::types::account_id::AccountId> {
        let widgets = crate::common::get_widgets()?;
        println!(
            "\nThere are <{}> widgets in the current folder ready for deployment:",
            widgets.len()
        );
        for widget in widgets.keys() {
            println!(" * {widget}")
        }
        loop {
            let deploy_to_account_id: near_cli_rs::types::account_id::AccountId =
                CustomType::new("Which account do you want to deploy the widgets to?").prompt()?;
            if !crate::common::is_account_exist(context, deploy_to_account_id.clone().into()) {
                println!(
                    "\nThe account <{}> does not yet exist.",
                    &deploy_to_account_id
                );
                #[derive(strum_macros::Display)]
                enum ConfirmOptions {
                    #[strum(to_string = "Yes, I want to enter a new account name.")]
                    Yes,
                    #[strum(to_string = "No, I want to use this account name.")]
                    No,
                }
                let select_choose_input = Select::new(
                    "Do you want to enter a new widget deployment account name?",
                    vec![ConfirmOptions::Yes, ConfirmOptions::No],
                )
                .prompt()?;
                if let ConfirmOptions::No = select_choose_input {
                    return Ok(deploy_to_account_id);
                }
            } else {
                return Ok(deploy_to_account_id);
            }
        }
    }


    pub async fn process(&self, config: near_cli_rs::config::Config) -> crate::CliResult {
        let network_config = self
            .sign_as
            .get_network_config_for_transaction()
            .get_network_config(config.clone());
        let near_social_account_id = match &network_config.near_social_account_id {
            Some(account_id) => account_id.clone(),
            None => {
                return Err(color_eyre::Report::msg(format!(
                    "The <{}> network does not have a near-social contract.",
                    network_config.network_name
                )))
            }
        };
        let widgets = crate::common::get_widgets()?;

        if widgets.is_empty() {
            println!("There are no widgets in the current ./src folder. Goodbye.");
            return Ok(());
        }

        let input_args = serde_json::to_string(&crate::socialdb_types::SocialDbQuery {
            keys: widgets
                .keys()
                .map(|name| format!("{}/widget/{name}/**", self.deploy_to_account_id))
                .collect(),
        })
        .wrap_err("Internal error: could not serialize SocialDB input args")?;

        let query_view_method_response = network_config
            .json_rpc_client()
            .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
                block_reference: near_primitives::types::Finality::Final.into(),
                request: near_primitives::views::QueryRequest::CallFunction {
                    account_id: near_social_account_id.clone(),
                    method_name: "get".to_string(),
                    args: near_primitives::types::FunctionArgs::from(input_args.into_bytes()),
                },
            })
            .await
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
                        self.deploy_to_account_id.clone(),
                    ))
            {
                account_metadata
            } else {
                println!("\nThere are currently no widgets in the account <{}>. Therefore, all widgets will be deployed as new", self.deploy_to_account_id);
                let deposit = self
                    .get_deposit(
                        &network_config,
                        self.deploy_to_account_id.clone(),
                        near_social_account_id.clone(),
                        near_cli_rs::common::NearBalance::from_str("1 NEAR").unwrap(), // XXX: need calculation!!!!!!!! for new account
                    )
                    .await?;
                return self
                    .deploy_widget_code(
                        config,
                        network_config,
                        self.deploy_to_account_id.clone(),
                        near_social_account_id,
                        widgets,
                        deposit,
                    )
                    .await;
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
            println!("There are no new or modified widgets in the current ./src folder. Goodbye.");
            return Ok(());
        }

        let deposit = self
            .get_deposit(
                &network_config,
                self.deploy_to_account_id.clone(),
                near_social_account_id.clone(),
                near_cli_rs::common::NearBalance::from_str("0 NEAR").unwrap(), // XXX: need calculation!!!!!!!! for an existing account
            )
            .await?;
        self.deploy_widget_code(
            config,
            network_config,
            self.deploy_to_account_id.clone(),
            near_social_account_id,
            output_widgets,
            deposit,
        )
        .await
    }

    async fn deploy_widget_code(
        &self,
        config: near_cli_rs::config::Config,
        network_config: near_cli_rs::config::NetworkConfig,
        deploy_to_account_id: near_cli_rs::types::account_id::AccountId,
        near_social_account_id: near_primitives::types::AccountId,
        widgets: HashMap<String, crate::socialdb_types::SocialDbWidget>,
        deposit: near_cli_rs::common::NearBalance,
    ) -> crate::CliResult {
        let mut accounts = HashMap::new();
        accounts.insert(
            near_primitives::types::AccountId::from(deploy_to_account_id.clone()),
            crate::socialdb_types::SocialDbAccountMetadata {
                widgets: widgets.clone(),
            },
        );

        let function_args = serde_json::to_string(&TransactionFunctionArgs {
            data: crate::socialdb_types::SocialDb { accounts },
        })?;

        let prepopulated_unsigned_transaction = near_primitives::transaction::Transaction {
            signer_id: self.sign_as.get_signer_account_id().into(),
            public_key: near_crypto::PublicKey::empty(near_crypto::KeyType::ED25519),
            nonce: 0,
            receiver_id: near_social_account_id,
            block_hash: Default::default(),
            actions: vec![near_primitives::transaction::Action::FunctionCall(
                near_primitives::transaction::FunctionCallAction {
                    method_name: "set".to_string(),
                    args: function_args.into_bytes(),
                    gas: near_cli_rs::common::NearGas::from_str("100 TeraGas")
                        .unwrap()
                        .inner,
                    deposit: deposit.to_yoctonear(),
                },
            )],
        };
        match near_cli_rs::transaction_signature_options::sign_with(
            self.sign_as.get_network_config_for_transaction(),
            prepopulated_unsigned_transaction,
            config,
        )
        .await?
        {
            Some(transaction_info) => match transaction_info.status {
                near_primitives::views::FinalExecutionStatus::SuccessValue(_) => {
                    println!("-------------- Logs ----------------");
                    for receipt in transaction_info.receipts_outcome.iter() {
                        if receipt.outcome.logs.is_empty() {
                            println!("Logs [{}]:   No logs", receipt.outcome.executor_id);
                        } else {
                            println!("Logs [{}]:", receipt.outcome.executor_id);
                            println!("  {}", receipt.outcome.logs.join("\n  "));
                        };
                    }
                    println!("------------------------------------");

                    println!("<{}> widgets were successfully deployed:", widgets.len());
                    for widget in widgets.keys() {
                        println!(" * {widget}")
                    }
                    println!("Transaction ID: {id}\nTo see the transaction in the transaction explorer, please open this url in your browser:\n{path}{id}\n",
                    id=transaction_info.transaction_outcome.id,
                    path=network_config.explorer_transaction_url
                    );
                    Ok(())
                }
                _ => {
                    near_cli_rs::common::print_transaction_status(
                        transaction_info,
                        network_config,
                    )?;
                    color_eyre::eyre::bail!("Widgets deployment failed!");
                }
            },
            None => Ok(()),
        }
    }

    async fn get_deposit(
        &self,
        network_config: &near_cli_rs::config::NetworkConfig,
        deploy_to_account_id: near_cli_rs::types::account_id::AccountId,
        near_social_account_id: near_primitives::types::AccountId,
        required_deposit: near_cli_rs::common::NearBalance,
    ) -> color_eyre::eyre::Result<near_cli_rs::common::NearBalance> {
        let signer_account_id: near_primitives::types::AccountId =
            self.sign_as.get_signer_account_id().into();
        let signer_public_key = self
            .sign_as
            .get_network_config_for_transaction()
            .get_signer_public_key();

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
            )
            .await?
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
}

