use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod sign_as;
mod widget;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = DeleleteWidgetsFromAccountContext)]
pub struct DeleleteWidgetsFromAccount {
    /// Which account do you want to delete the widgets to?
    account_id: near_cli_rs::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    delete_command: DeleteCommand,
}

#[derive(Clone)]
pub struct DeleleteWidgetsFromAccountContext(self::widget::WidgetContext);

impl DeleleteWidgetsFromAccountContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<DeleleteWidgetsFromAccount as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(self::widget::WidgetContext {
            config: previous_context.0,
            account_id: scope.account_id.clone(),
            widgets: vec![],
        }))
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = DeleleteWidgetsFromAccountContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum DeleteCommand {
    #[strum_discriminants(strum(
        message = "selected-widgets  - Delete selected widgets from your account"
    ))]
    SelectedWidgets(self::widget::Widget),
    #[strum_discriminants(strum(
        message = "all-widgets       - Delete all widgets from your account"
    ))]
    AllWidgets(self::sign_as::Signer),
}

impl From<DeleleteWidgetsFromAccountContext> for self::widget::WidgetContext {
    fn from(item: DeleleteWidgetsFromAccountContext) -> Self {
        item.0
    }
}
