use color_eyre::eyre::{ContextCompat, WrapErr};
use near_cli_rs::common::{CallResultExt, JsonRpcClientExt};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::AccountContext)]
#[interactive_clap(output_context = AsJsonContext)]
pub struct AsJson {
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: near_cli_rs::network_view_at_block::NetworkViewAtBlockArgs,
}

#[derive(Clone)]
pub struct AsJsonContext(near_cli_rs::network_view_at_block::ArgsForViewContext);

impl AsJsonContext {
    pub fn from_previous_context(
        previous_context: super::AccountContext,
        _scope: &<AsJson as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let account_id = previous_context.account_id;
        let on_after_getting_block_reference_callback: near_cli_rs::network_view_at_block::OnAfterGettingBlockReferenceCallback =
            std::sync::Arc::new({
                let account_id = account_id.clone();
                move |network_config, block_reference| {
                    let near_social_account_id = crate::consts::NEAR_SOCIAL_ACCOUNT_ID.get(network_config.network_name.as_str())
                        .wrap_err_with(|| format!("The <{}> network does not have a near-social contract.", network_config.network_name))?;

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

                    print_profile(social_db.accounts.get(&account_id));

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

impl From<AsJsonContext> for near_cli_rs::network_view_at_block::ArgsForViewContext {
    fn from(item: AsJsonContext) -> Self {
        item.0
    }
}

fn print_profile(
    optional_account_profile: Option<&near_socialdb_client::types::socialdb_types::AccountProfile>,
) {
    if let Some(account_profile) = optional_account_profile {
        eprintln!("{{");
        if let Some(name) = &account_profile.profile.name {
            eprintln!("  \"name\": \"{name}\",");
        }
        if let Some(image) = &account_profile.profile.image {
            eprintln!("  \"image\": {{");
            if let Some(url) = &image.url {
                eprintln!("    \"url\": \"{url}\",");
            }
            if let Some(ipfs_cid) = &image.ipfs_cid {
                eprintln!("    \"ipfs_cid\": \"{ipfs_cid}\",");
            }
            eprintln!("  }},");
        }
        if let Some(background_image) = &account_profile.profile.background_image {
            eprintln!("  \"background_image\": {{");
            if let Some(url) = &background_image.url {
                eprintln!("    \"url\": \"{url}\",");
            }
            if let Some(ipfs_cid) = &background_image.ipfs_cid {
                eprintln!("    \"ipfs_cid\": \"{ipfs_cid}\",");
            }
            eprintln!("  }},");
        }
        if let Some(description) = &account_profile.profile.description {
            eprintln!(
                "  \"description\": \"{}\",",
                description.replace('\n', "\\n")
            );
        }
        if let Some(linktree) = &account_profile.profile.linktree {
            eprintln!("  \"linktree\": {{");
            for (key, optional_value) in linktree.iter() {
                if let Some(value) = &optional_value {
                    eprintln!("    \"{key}\": \"{value}\",");
                }
            }
            eprintln!("  }},")
        }
        if let Some(tags) = &account_profile.profile.tags {
            eprintln!("  \"tags\": {{");
            for (key, value) in tags.iter() {
                eprintln!("    \"{key}\": \"{value}\",");
            }
            eprintln!("  }},")
        }
        eprintln!("}}");
    } else {
        eprintln!("{{}}");
    }
}
