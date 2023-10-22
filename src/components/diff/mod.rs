use color_eyre::eyre::ContextCompat;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = near_cli_rs::GlobalContext)]
#[interactive_clap(output_context = DiffCmdContext)]
pub struct DiffCmd {
    #[interactive_clap(skip_default_input_arg)]
    /// On which account do you want to compare local components?
    account_id: near_cli_rs::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: near_cli_rs::network::Network,
}

#[derive(Clone)]
pub struct DiffCmdContext(near_cli_rs::network::NetworkContext);

impl DiffCmdContext {
    pub fn from_previous_context(
        previous_context: near_cli_rs::GlobalContext,
        scope: &<DiffCmd as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let account_id: near_primitives::types::AccountId = scope.account_id.clone().into();
        let on_after_getting_network_callback: near_cli_rs::network::OnAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let account_id = account_id.clone();
                move |network_config| {
                    let near_social_account_id = crate::consts::NEAR_SOCIAL_ACCOUNT_ID
                        .get(network_config.network_name.as_str())
                        .wrap_err_with(|| {
                            format!(
                                "The <{}> network does not have a near-social contract.",
                                network_config.network_name
                            )
                        })?;

                    let local_components = crate::common::get_local_components()?;
                    if local_components.is_empty() {
                        println!("There are no components in the current ./src folder. Goodbye.");
                        return Ok(());
                    }
                    let local_component_name_list = local_components.keys().collect::<Vec<_>>();

                    let remote_components = crate::common::get_remote_components(
                        network_config,
                        local_component_name_list,
                        near_social_account_id,
                        &account_id,
                    )?;

                    if !remote_components.is_empty() {
                        let updated_components = crate::common::get_updated_components(
                            local_components,
                            &remote_components,
                        );
                        if updated_components.is_empty() {
                            println!("There are no new or modified components in the current ./src folder. Goodbye.");
                            return Ok(());
                        }
                    } else {
                        println!("\nAll local components are new to <{account_id}>.");
                    };
                    Ok(())
                }
            });
        Ok(Self(near_cli_rs::network::NetworkContext {
            config: previous_context.config,
            interacting_with_account_ids: vec![account_id],
            on_after_getting_network_callback,
        }))
    }
}

impl From<DiffCmdContext> for near_cli_rs::network::NetworkContext {
    fn from(item: DiffCmdContext) -> Self {
        item.0
    }
}

impl DiffCmd {
    pub fn input_account_id(
        context: &near_cli_rs::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<near_cli_rs::types::account_id::AccountId>> {
        near_cli_rs::common::input_non_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "On which account do you want to compare local components?",
        )
    }
}
