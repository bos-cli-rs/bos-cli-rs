use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod data;
mod permissions;
mod prepaid_storage;
mod profile_management;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
pub struct SocialDb {
    #[interactive_clap(subcommand)]
    social_db_command: SocialDbCommand,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// What are you up to? (select one of the options with the up-down arrows on your keyboard and press Enter)
pub enum SocialDbCommand {
    #[strum_discriminants(strum(
        message = "data              -   Data management: viewing, adding, updating, deleting information by a given key"
    ))]
    /// Data management: viewing, adding, updating, deleting information by a given key
    Data(self::data::Data),
    #[strum_discriminants(strum(
        message = "manage-profile    -   Profile management: view, update"
    ))]
    /// Profile management: view, update
    ManageProfile(self::profile_management::ManageProfile),
    #[strum_discriminants(strum(
        message = "prepaid-storage   -   Storage management: deposit, withdrawal, balance review"
    ))]
    /// Storage management: deposit, withdrawal, balance review
    PrepaidStorage(self::prepaid_storage::PrepaidStorage),
    #[strum_discriminants(strum(
        message = "permissions       -   Granting access permissions to a different account"
    ))]
    /// Granting access permissions to a different account
    Permissions(self::permissions::Permissions),
}
