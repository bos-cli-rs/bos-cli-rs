use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod selected;
mod sign_as;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = near_cli_rs::GlobalContext)]
#[interactive_clap(output_context = DeleteComponentsFromAccountContext)]
pub struct DeleteComponentsFromAccount {
    /// Which account do you want to delete the components from?
    account_id: near_cli_rs::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    delete_command: DeleteCommand,
}

#[derive(Clone)]
pub struct DeleteComponentsFromAccountContext(self::selected::ComponentContext);

impl DeleteComponentsFromAccountContext {
    pub fn from_previous_context(
        previous_context: near_cli_rs::GlobalContext,
        scope: &<DeleteComponentsFromAccount as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(self::selected::ComponentContext {
            global_context: previous_context,
            account_id: scope.account_id.clone(),
            components: vec![],
        }))
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = DeleteComponentsFromAccountContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Which components do you want to remove?
pub enum DeleteCommand {
    #[strum_discriminants(strum(
        message = "selected  - Delete selected components from your account"
    ))]
    Selected(self::selected::Selected),
    #[strum_discriminants(strum(message = "all       - Delete all components from your account"))]
    All(All),
}

impl From<DeleteComponentsFromAccountContext> for self::selected::ComponentContext {
    fn from(item: DeleteComponentsFromAccountContext) -> Self {
        item.0
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = DeleteComponentsFromAccountContext)]
pub struct All {
    #[interactive_clap(named_arg)]
    /// Specify signer account ID
    sign_as: self::sign_as::Signer,
}
