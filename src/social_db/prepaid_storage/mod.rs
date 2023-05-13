use color_eyre::eyre::ContextCompat;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = PrepaidStorageContext)]
pub struct PrepaidStorage {
    #[interactive_clap(subcommand)]
    storage_actions: near_cli_rs::commands::account::storage_management::StorageActions,
}

#[derive(Clone)]
pub struct PrepaidStorageContext(
    near_cli_rs::commands::account::storage_management::ContractContext,
);

impl PrepaidStorageContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        _scope: &<PrepaidStorage as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let get_contract_account_id: near_cli_rs::commands::account::storage_management::GetContractAccountId = std::sync::Arc::new(
            move |network_config|
            crate::consts::NEAR_SOCIAL_ACCOUNT_ID
                .get(&network_config.network_name.as_str())
                .cloned()
                .wrap_err_with(|| format!(
                    "The <{}> network does not have a near-social contract.",
                    network_config.network_name
                ))
        );
        Ok(Self(
            near_cli_rs::commands::account::storage_management::ContractContext {
                config: previous_context.0,
                get_contract_account_id,
            },
        ))
    }
}

impl From<PrepaidStorageContext>
    for near_cli_rs::commands::account::storage_management::ContractContext
{
    fn from(item: PrepaidStorageContext) -> Self {
        item.0
    }
}
