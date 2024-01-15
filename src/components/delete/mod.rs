use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod selected;
mod sign_as;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::ComponentsContext)]
#[interactive_clap(output_context = DeleteCmdContext)]
pub struct DeleteCmd {
    #[interactive_clap(skip_default_input_arg)]
    /// Which account do you want to delete the components from?
    account_id: near_cli_rs::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    delete_command: DeleteCommand,
}

#[derive(Clone)]
pub struct DeleteCmdContext(self::selected::ComponentContext);

impl DeleteCmdContext {
    pub fn from_previous_context(
        previous_context: super::ComponentsContext,
        scope: &<DeleteCmd as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(self::selected::ComponentContext {
            global_context: previous_context.global_context,
            social_db_folder: previous_context.social_db_folder,
            account_id: scope.account_id.clone(),
            components: vec![],
        }))
    }
}

impl DeleteCmd {
    pub fn input_account_id(
        context: &super::ComponentsContext,
    ) -> color_eyre::eyre::Result<Option<near_cli_rs::types::account_id::AccountId>> {
        near_cli_rs::common::input_signer_account_id_from_used_account_list(
            &context.global_context.config.credentials_home_dir,
            "Which account do you want to delete the components from?",
        )
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = DeleteCmdContext)]
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

impl From<DeleteCmdContext> for self::selected::ComponentContext {
    fn from(item: DeleteCmdContext) -> Self {
        item.0
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = DeleteCmdContext)]
pub struct All {
    #[interactive_clap(named_arg)]
    /// Specify signer account ID
    sign_as: self::sign_as::Signer,
}
