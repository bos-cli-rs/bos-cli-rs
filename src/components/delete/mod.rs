mod sign_as;
mod widget;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = DeleleteWidgetsFromAccountContext)]
pub struct DeleleteWidgetsFromAccount {
    /// Which account do you want to delete the widgets to?
    account_id: near_cli_rs::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Specify signer account ID
    widgets: self::widget::Widget,
}

#[derive(Clone)]
pub struct DeleleteWidgetsFromAccountContext {
    pub config: near_cli_rs::config::Config,
    pub account_id: near_cli_rs::types::account_id::AccountId,
}

impl DeleleteWidgetsFromAccountContext {
    pub fn from_previous_context(
        previous_context: near_cli_rs::GlobalContext,
        scope: &<DeleleteWidgetsFromAccount as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.0,
            account_id: scope.account_id.clone(),
        })
    }
}
