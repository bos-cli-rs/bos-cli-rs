use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod new;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
pub struct Project {
    #[interactive_clap(subcommand)]
    command: ProjectCommand,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// What are you up to?
pub enum ProjectCommand {
    #[strum_discriminants(strum(message = "new  -  Initializes a new project"))]
    /// Initializes a new project
    New(self::new::New),
}
