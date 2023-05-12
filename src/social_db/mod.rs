use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod permissions;
mod storage_management;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct SocialDbManagement {
    #[interactive_clap(subcommand)]
    social_db_command: SocialDbCommand,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// What are you up to? (select one of the options with the up-down arrows on your keyboard and press Enter)
pub enum SocialDbCommand {
    #[strum_discriminants(strum(
        message = "perpaid-storage   -   Storage management: deposit, withdrawal, balance review"
    ))]
    /// Storage management: deposit, withdrawal, balance review
    PerpaidStorage(self::storage_management::StorageManagement),
    #[strum_discriminants(strum(
        message = "permissions       -   Granting access permissions to a different account"
    ))]
    /// Granting access permissions to a different account
    Permissions(self::permissions::PermissionsManagement),
}
