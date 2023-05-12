use std::str::FromStr;

use color_eyre::eyre::ContextCompat;
use inquire::{CustomType, Select};
use serde_json::{json, Value::Null};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::widget::WidgetContext)]
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
    account_id: near_cli_rs::types::account_id::AccountId,
    signer_account_id: near_primitives::types::AccountId,
}

impl SignerContext {
    pub fn from_previous_context(
        previous_context: super::widget::WidgetContext,
        scope: &<Signer as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.config,
            widgets: previous_context.widgets,
            account_id: previous_context.account_id,
            signer_account_id: scope.signer_account_id.clone().into(),
        })
    }
}

impl From<SignerContext> for near_cli_rs::commands::ActionContext {
    fn from(item: SignerContext) -> Self {
        let widgets = item.widgets.clone();
        let account_id: near_primitives::types::AccountId = item.account_id.clone().into();
        let signer_id = item.signer_account_id.clone();

        let on_after_getting_network_callback: near_cli_rs::commands::OnAfterGettingNetworkCallback = std::sync::Arc::new({
            move |network_config| {
                let near_social_account_id = crate::consts::NEAR_SOCIAL_ACCOUNT_ID.get(network_config.network_name.as_str())
                    .wrap_err_with(|| format!("The <{}> network does not have a near-social contract.", network_config.network_name))?;

                if let Some(remote_widgets) = crate::common::get_remote_widgets(&account_id, network_config, near_social_account_id)? {
                    let widgets = if widgets.is_empty() {
                        remote_widgets.keys().cloned().collect()
                    } else {
                        widgets.clone()
                        .into_iter()
                        .filter(|widget| remote_widgets.get(widget).is_some())
                        .collect::<Vec<_>>()
                    };
                    if widgets.is_empty() {
                        println!("No widgets to remove. Goodbye.");
                        return Ok(near_cli_rs::commands::PrepopulatedTransaction {
                            signer_id: signer_id.clone(),
                            receiver_id: near_social_account_id.clone(),
                            actions: vec![],
                        });
                    }
                    let mut actions: Vec<near_primitives::transaction::Action> = widgets.iter()
                        .map(|widget|
                            near_primitives::transaction::Action::FunctionCall(
                                near_primitives::transaction::FunctionCallAction {
                                    method_name: "set".to_string(),
                                    args: serde_json::json!({
                                        "data": serde_json::json!({
                                            account_id.to_string(): json!({
                                                "widget": json!({
                                                    widget.clone(): json!({
                                                    "metadata": json!({
                                                        "description": Null,
                                                        "image": json!({
                                                            "url": Null,
                                                        }),
                                                        "name": Null,
                                                        "tags": json!({
                                                            "app": Null,
                                                            "tag": Null,
                                                        })
                                                        })
                                                    })
                                                })
                                            })
                                        })
                                    }).to_string().into_bytes(),
                                    gas: near_cli_rs::common::NearGas::from_str("12 TeraGas")
                                        .unwrap()
                                        .inner,
                                    deposit: near_cli_rs::common::NearBalance::from_yoctonear(0).to_yoctonear(),
                                },
                            ),
                        ).collect();
                    let mut actions_new: Vec<near_primitives::transaction::Action> = widgets.iter()
                        .map(|widget|
                            near_primitives::transaction::Action::FunctionCall(
                                near_primitives::transaction::FunctionCallAction {
                                    method_name: "set".to_string(),
                                    args: json!({
                                        "data": json!({
                                            account_id.to_string(): json!({
                                                "widget": json!({
                                                    widget: Null
                                                })
                                            })
                                        })
                                    }).to_string().into_bytes(),
                                    gas: near_cli_rs::common::NearGas::from_str("12 TeraGas")
                                        .unwrap()
                                        .inner,
                                    deposit: near_cli_rs::common::NearBalance::from_yoctonear(0).to_yoctonear(),
                                },
                            ),
                        ).collect();

                    actions.append(&mut actions_new);

                    Ok(near_cli_rs::commands::PrepopulatedTransaction {
                        signer_id: signer_id.clone(),
                        receiver_id: near_social_account_id.clone(),
                        actions,
                    })
                } else {
                    println!("Goodbye.");
                    Ok(near_cli_rs::commands::PrepopulatedTransaction {
                        signer_id: signer_id.clone(),
                        receiver_id: near_social_account_id.clone(),
                        actions: vec![],
                    })
                }
            }
        });

        let on_after_sending_transaction_callback: near_cli_rs::transaction_signature_options::OnAfterSendingTransactionCallback = std::sync::Arc::new({
            let account_id = item.account_id.clone();
            move |transaction_info, _network_config| {
                if let near_primitives::views::FinalExecutionStatus::SuccessValue(_) = transaction_info.status {
                    println!("Selected widgets removed successfully for <{}>", &account_id);
                } else {
                    color_eyre::eyre::bail!("The selected widgets were not successfully removed for <{}>", &account_id);
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
        context: &super::widget::WidgetContext,
    ) -> color_eyre::eyre::Result<Option<near_cli_rs::types::account_id::AccountId>> {
        loop {
            let signer_account_id: near_cli_rs::types::account_id::AccountId =
                CustomType::new(" What is the signer account ID?")
                    .with_default(context.account_id.clone())
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
