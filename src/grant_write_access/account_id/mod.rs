#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::WidgetContext)]
#[interactive_clap(output_context = AccessToAccountContext)]
pub struct AccessToAccount {
    /// Enter the account ID you will grant permission to.
    account_id: near_cli_rs::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Specify extra storage deposit.
    with_extra_storage_deposit: super::storage_deposit::ExtraStorageDeposit,
}

#[derive(Clone)]
pub struct AccessToAccountContext {
    pub config: near_cli_rs::config::Config,
    pub widgets: Vec<String>,
    pub account_id: near_primitives::types::AccountId,
}

impl AccessToAccountContext {
    pub fn from_previous_context(
        previous_context: super::WidgetContext,
        scope: &<AccessToAccount as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.config,
            widgets: previous_context.widgets,
            account_id: scope.account_id.clone().into(),
        })
    }
}
