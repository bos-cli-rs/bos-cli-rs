use color_eyre::eyre::{ContextCompat, WrapErr};
use near_cli_rs::common::{CallResultExt, JsonRpcClientExt};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::AccountContext)]
#[interactive_clap(output_context = AsTextContext)]
pub struct AsText {
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: near_cli_rs::network_view_at_block::NetworkViewAtBlockArgs,
}

#[derive(Clone)]
pub struct AsTextContext(near_cli_rs::network_view_at_block::ArgsForViewContext);

impl AsTextContext {
    pub fn from_previous_context(
        previous_context: super::AccountContext,
        _scope: &<AsText as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let account_id = previous_context.account_id;
        let on_after_getting_block_reference_callback: near_cli_rs::network_view_at_block::OnAfterGettingBlockReferenceCallback =
            std::sync::Arc::new({
                let account_id = account_id.clone();
                move |network_config, block_reference| {
                    let near_social_account_id = crate::consts::NEAR_SOCIAL_ACCOUNT_ID.get(network_config.network_name.as_str())
                        .wrap_err_with(|| format!("The <{}> network does not have a near-social contract.", network_config.network_name))?;

                    let rpc_query_response = network_config
                        .json_rpc_client()
                        .blocking_call_view_account(&account_id, block_reference.clone())
                        .wrap_err_with(|| {
                            format!(
                                "Failed to fetch query ViewAccount for <{}>",
                                &account_id
                            )
                        })?;

                    let social_db = network_config
                        .json_rpc_client()
                        .blocking_call_view_function(
                            near_social_account_id,
                            "get",
                            serde_json::json!({
                                "keys": vec![format!("{account_id}/profile/**")],
                            })
                            .to_string()
                            .into_bytes(),
                            block_reference.clone(),
                        )
                        .wrap_err_with(|| {format!("Failed to fetch query for view method: 'get {account_id}/profile/**'")})?
                        .parse_result_from_json::<near_socialdb_client::types::socialdb_types::SocialDb>()
                        .wrap_err_with(|| {
                            format!("Failed to parse view function call return value for {account_id}/profile.")
                        })?;

                    near_cli_rs::common::display_account_profile(
                        &rpc_query_response.block_hash,
                        &rpc_query_response.block_height,
                        &account_id,
                        social_db.accounts.get(&account_id)
                    );

                    Ok(())
                }
            });

        Ok(Self(
            near_cli_rs::network_view_at_block::ArgsForViewContext {
                config: previous_context.global_context.config,
                interacting_with_account_ids: vec![account_id],
                on_after_getting_block_reference_callback,
            },
        ))
    }
}

impl From<AsTextContext> for near_cli_rs::network_view_at_block::ArgsForViewContext {
    fn from(item: AsTextContext) -> Self {
        item.0
    }
}
