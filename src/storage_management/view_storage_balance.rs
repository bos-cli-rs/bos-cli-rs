use color_eyre::eyre::WrapErr;
use near_cli_rs::common::{CallResultExt, JsonRpcClientExt};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = AccountIdContext)]
pub struct AccountId {
    /// What account balance do you want to view?
    account_id: near_cli_rs::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: near_cli_rs::network::Network,
}

#[derive(Clone)]
pub struct AccountIdContext(near_cli_rs::network::NetworkContext);

impl AccountIdContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<AccountId as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_network_callback: near_cli_rs::network::OnAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let account_id: near_primitives::types::AccountId = scope.account_id.clone().into();

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

                    let storage_balance = network_config
                        .json_rpc_client()
                        .blocking_call_view_function(
                            near_social_account_id,
                            "storage_balance_of",
                            serde_json::json!({
                                "account_id": account_id,
                            })
                            .to_string()
                            .into_bytes(),
                            near_primitives::types::Finality::Final.into(),
                        )
                        .wrap_err_with(|| {
                            "Failed to fetch query for view method: 'storage_balance_of'"
                        })?
                        .parse_result_from_json::<crate::common::StorageBalance>()
                        .wrap_err_with(|| {
                            "Failed to parse return value of view function call for StorageBalance."
                        })?;

                    println!("storage balance for <{account_id}>:");
                    println!(" {:<13} {:>33}", "available:", &storage_balance.available);
                    println!(" {:<13} {:>33}", "total:", &storage_balance.total);

                    Ok(())
                }
            });

        Ok(Self(near_cli_rs::network::NetworkContext {
            config: previous_context.0,
            on_after_getting_network_callback,
        }))
    }
}

impl From<AccountIdContext> for near_cli_rs::network::NetworkContext {
    fn from(item: AccountIdContext) -> Self {
        item.0
    }
}
