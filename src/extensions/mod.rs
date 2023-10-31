use strum::{EnumDiscriminants, EnumIter, EnumMessage};

pub mod self_update;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
pub struct ExtensionsCommands {
    #[interactive_clap(subcommand)]
    pub extensions_actions: ExtensionsActions,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[non_exhaustive]
/// What do you want to do with a bos CLI?
pub enum ExtensionsActions {
    #[strum_discriminants(strum(message = "self-update   -  Self update bos CLI"))]
    /// Self update bos CLI
    SelfUpdate(self::self_update::SelfUpdateCommand),
}
