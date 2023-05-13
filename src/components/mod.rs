use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod delete;
mod deploy;
mod download;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct Components {
    #[interactive_clap(subcommand)]
    command: self::ComponentsCommand,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// What are you up to?
pub enum ComponentsCommand {
    #[strum_discriminants(strum(message = "download   -   Download widgets from account"))]
    /// Download widgets from account
    Download(self::download::AccountId),
    #[strum_discriminants(strum(message = "deploy     -   Deploy widget if code has changed"))]
    /// Deploy widget if code has changed
    Deploy(self::deploy::DeployToAccount),
    #[strum_discriminants(strum(message = "delete     -   Delete widgets from account"))]
    /// Delete widgets from account
    Delete(self::delete::DeleleteWidgetsFromAccount),
}
