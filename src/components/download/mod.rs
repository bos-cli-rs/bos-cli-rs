use color_eyre::eyre::{ContextCompat, WrapErr};
use near_cli_rs::common::{CallResultExt, JsonRpcClientExt};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = near_cli_rs::GlobalContext)]
#[interactive_clap(output_context = AccountIdContext)]
pub struct AccountId {
    /// Which account do you want to download components from?
    account_id: near_cli_rs::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: near_cli_rs::network::Network,
}

#[derive(Clone)]
pub struct AccountIdContext(near_cli_rs::network::NetworkContext);

impl AccountIdContext {
    pub fn from_previous_context(
        previous_context: near_cli_rs::GlobalContext,
        scope: &<AccountId as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let account_id: near_primitives::types::AccountId = scope.account_id.clone().into();

        let on_after_getting_network_callback: near_cli_rs::network::OnAfterGettingNetworkCallback =
            std::sync::Arc::new({
                move |network_config| {
                    let near_social_account_id = match crate::consts::NEAR_SOCIAL_ACCOUNT_ID
                        .get(&network_config.network_name.as_str())
                    {
                        Some(account_id) => account_id,
                        None => {
                            return Err(color_eyre::Report::msg(format!(
                                "The <{}> network does not have a near-social contract.",
                                network_config.network_name
                            )))
                        }
                    };

                    let input_args = serde_json::to_string(&crate::socialdb_types::SocialDbQuery {
                        keys: vec![format!("{account_id}/widget/*")],
                    })
                    .wrap_err("Internal error: could not serialize SocialDB input args")?;

                    let call_result = network_config
                        .json_rpc_client()
                        .blocking_call_view_function(
                            near_social_account_id,
                            "keys",
                            input_args.into_bytes(),
                            near_primitives::types::Finality::Final.into(),
                        )
                        .wrap_err("Failed to fetch the components state from SocialDB")?;
                    let keys: SocialDbKeys = call_result.parse_result_from_json()?;

                    let remote_social_account_components =
                        if let Some(account_components) = keys.accounts.get(&account_id) {
                            account_components
                        } else {
                            println!(
                            "\nThere are currently no components in the account <{account_id}>.",
                        );
                            return Ok(());
                        };
                    let remote_component_name_list = remote_social_account_components
                        .components
                        .keys()
                        .collect::<Vec<_>>();
                    let remote_components = crate::common::get_remote_components(
                        network_config,
                        remote_component_name_list,
                        near_social_account_id,
                        &account_id,
                    )?;

                    let components_src_folder = std::path::PathBuf::from("./src");
                    for (component_name, component) in remote_components.iter() {
                        let mut component_path = components_src_folder.clone();
                        component_path.extend(component_name.split('.'));
                        std::fs::create_dir_all(component_path.parent().wrap_err_with(|| {
                            format!(
                                "Failed to get the parent path for {component_name} where the path is {}",
                                component_path.display()
                            )
                        })?)?;
                        let component_code_path = component_path.with_extension("jsx");
                        std::fs::write(&component_code_path, component.code().as_bytes())
                            .wrap_err_with(|| {
                                format!(
                                    "Failed to save component code into {}",
                                    component_code_path.display()
                                )
                            })?;
                        if let Some(metadata) = component.metadata() {
                            let metadata =
                                serde_json::to_string_pretty(metadata).wrap_err_with(|| {
                                    format!("Failed to serialize component metadata for {component_name}")
                                })?;
                            let component_metadata_path =
                                component_path.with_extension("metadata.json");
                            std::fs::write(&component_metadata_path, metadata.as_bytes())
                                .wrap_err_with(|| {
                                    format!(
                                        "Failed to save component metadata into {}",
                                        component_metadata_path.display()
                                    )
                                })?;
                        }
                    }

                    println!(
                        "Components for account <{}> were downloaded into <{}> successfully",
                        account_id,
                        components_src_folder.display()
                    );

                    Ok(())
                }
            });
        Ok(Self(near_cli_rs::network::NetworkContext {
            config: previous_context.config,
            interacting_with_account_ids: vec![scope.account_id.clone().into()],
            on_after_getting_network_callback,
        }))
    }
}

impl From<AccountIdContext> for near_cli_rs::network::NetworkContext {
    fn from(item: AccountIdContext) -> Self {
        item.0
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SocialDbKeys {
    #[serde(flatten)]
    pub accounts:
        std::collections::HashMap<near_primitives::types::AccountId, SocialDbAccountComponents>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SocialDbAccountComponents {
    #[serde(rename = "widget")]
    pub components: std::collections::HashMap<crate::socialdb_types::ComponentName, bool>,
}
