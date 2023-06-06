use color_eyre::eyre::WrapErr;
use near_cli_rs::common::{CallResultExt, JsonRpcClientExt};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = near_cli_rs::GlobalContext)]
#[interactive_clap(output_context = ViewContext)]
pub struct View {
    /// Enter SocialDB key path to view (e.g. root.near/profile/**):
    key: String,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: near_cli_rs::network::Network,
}

#[derive(Clone)]
pub struct ViewContext(near_cli_rs::network::NetworkContext);

impl ViewContext {
    pub fn from_previous_context(
        previous_context: near_cli_rs::GlobalContext,
        scope: &<View as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_network_callback: near_cli_rs::network::OnAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let key = scope.key.clone();

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
                        keys: vec![format!("{key}")],
                    })
                    .wrap_err("Internal error: could not serialize SocialDB input args")?;

                    let call_result = network_config
                        .json_rpc_client()
                        .blocking_call_view_function(
                            near_social_account_id,
                            "get",
                            input_args.into_bytes(),
                            near_primitives::types::Finality::Final.into(),
                        )
                        .wrap_err("Failed to fetch the widgets state from SocialDB")?;
                    if call_result.result.is_empty() {
                        eprintln!("There is no information for this request");
                    } else if let Ok(json_result) =
                        call_result.parse_result_from_json::<serde_json::Value>()
                    {
                        println!("{}", serde_json::to_string_pretty(&json_result)?);
                    } else if let Ok(string_result) = String::from_utf8(call_result.result) {
                        println!("{string_result}");
                    } else {
                        eprintln!("The returned value is not printable (binary data)");
                    }
                    Ok(())
                }
            });
        Ok(Self(near_cli_rs::network::NetworkContext {
            config: previous_context.config,
            on_after_getting_network_callback,
        }))
    }
}

impl From<ViewContext> for near_cli_rs::network::NetworkContext {
    fn from(item: ViewContext) -> Self {
        item.0
    }
}
