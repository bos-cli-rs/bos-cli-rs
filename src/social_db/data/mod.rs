use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod view;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct Data {
    #[interactive_clap(subcommand)]
    data_command: DataCommand,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Select the data command
pub enum DataCommand {
    #[strum_discriminants(strum(message = "view     -   Viewing information by a given key"))]
    /// Viewing information by a given key
    View(self::view::View),
    #[strum_discriminants(strum(
        message = "set      -   Adding or updating information by a given key"
    ))]
    /// Adding or updating information by a given key
    Set,
    #[strum_discriminants(strum(message = "delete   -   Deleting information by a given key"))]
    /// Deleting information by a given key
    Delete,
}
