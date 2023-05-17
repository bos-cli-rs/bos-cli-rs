use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod component;
mod sign_as;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = DeleleteComponentsFromAccountContext)]
pub struct DeleleteComponentsFromAccount {
    /// Which account do you want to delete the components to?
    account_id: near_cli_rs::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    delete_command: DeleteCommand,
}

#[derive(Clone)]
pub struct DeleleteComponentsFromAccountContext(self::component::ComponentContext);

impl DeleleteComponentsFromAccountContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<DeleleteComponentsFromAccount as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(self::component::ComponentContext {
            config: previous_context.0,
            account_id: scope.account_id.clone(),
            components: vec![],
        }))
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = DeleleteComponentsFromAccountContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Which components do you want to remove?
pub enum DeleteCommand {
    #[strum_discriminants(strum(
        message = "selected  - Delete selected components from your account"
    ))]
    Selected(self::component::Component),
    #[strum_discriminants(strum(message = "all       - Delete all components from your account"))]
    All(self::sign_as::Signer),
}

impl From<DeleleteComponentsFromAccountContext> for self::component::ComponentContext {
    fn from(item: DeleleteComponentsFromAccountContext) -> Self {
        item.0
    }
}
