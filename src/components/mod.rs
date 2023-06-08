use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod delete;
mod deploy;
mod download;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
pub struct Components {
    #[interactive_clap(subcommand)]
    command: self::ComponentsCommand,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// What are you up to?
pub enum ComponentsCommand {
    #[strum_discriminants(strum(message = "download   -   Download components from account"))]
    /// Download components from account
    Download(self::download::AccountId),
    #[strum_discriminants(strum(
        message = "deploy     -   Deploy components if code has changed"
    ))]
    /// Deploy сomponents if code has changed
    Deploy(self::deploy::DeployToAccount),
    #[strum_discriminants(strum(message = "delete     -   Delete components from account"))]
    /// Delete components from account
    Delete(self::delete::DeleteComponentsFromAccount),
}
