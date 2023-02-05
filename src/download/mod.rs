use color_eyre::eyre::{ContextCompat, WrapErr};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
pub struct AccountId {
    /// Which account do you want to download widgets from?
    account_id: near_cli_rs::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: near_cli_rs::network::Network,
}

impl AccountId {
    pub async fn process(&self, config: near_cli_rs::config::Config) -> crate::CliResult {
        let network_config = self.network_config.get_network_config(config.clone());
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

        let input_args = serde_json::to_string(&crate::socialdb_types::SocialDbQuery {
            keys: vec![format!("{}/widget/**", self.account_id)],
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

        let downloaded_social_db: crate::socialdb_types::SocialDb =
            serde_json::from_slice(&call_result.result)
                .wrap_err("Failed to parse the widgets state from SocialDB")?;

        let downloaded_social_account_metadata: &crate::socialdb_types::SocialDbAccountMetadata =
            if let Some(account_metadata) =
                downloaded_social_db
                    .accounts
                    .get(&near_primitives::types::AccountId::from(
                        self.account_id.clone(),
                    ))
            {
                account_metadata
            } else {
                println!(
                    "\nThere are currently no widgets in the account <{}>.",
                    self.account_id
                );
                return Ok(());
            };

        let widgets = &downloaded_social_account_metadata.widgets;
        let widgets_src_folder = std::path::PathBuf::from("./src");
        for (widget_name, widget) in widgets.iter() {
            let mut widget_path = widgets_src_folder.clone();
            widget_path.extend(widget_name.split('.'));
            std::fs::create_dir_all(widget_path.parent().wrap_err_with(|| {
                format!(
                    "Failed to get the parent path for {widget_name} where the path is {}",
                    widget_path.display()
                )
            })?)?;
            let widget_code_path = widget_path.with_extension("jsx");
            std::fs::write(&widget_code_path, widget.code.as_bytes()).wrap_err_with(|| {
                format!(
                    "Failed to save widget code into {}",
                    widget_code_path.display()
                )
            })?;
            if let Some(metadata) = &widget.metadata {
                let metadata = serde_json::to_string_pretty(metadata).wrap_err_with(|| {
                    format!("Failed to serialize widget metadata for {widget_name}")
                })?;
                let widget_metadata_path = widget_path.with_extension("metadata.json");
                std::fs::write(&widget_metadata_path, metadata.as_bytes()).wrap_err_with(|| {
                    format!(
                        "Failed to save widget metadata into {}",
                        widget_metadata_path.display()
                    )
                })?;
            }
        }

        println!(
            "Widgets for account <{}> were downloaded into <{}> successfully",
            self.account_id,
            widgets_src_folder.display()
        );
        Ok(())
    }
}
