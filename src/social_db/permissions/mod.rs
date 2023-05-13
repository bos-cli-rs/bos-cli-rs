use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod grant_write_access;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct Permissions {
    #[interactive_clap(subcommand)]
    permissions_command: PermissionsCommand,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Select the permissions command
pub enum PermissionsCommand {
    #[strum_discriminants(strum(
        message = "grant-write-access   -   Granting access to a function-call-only access key or a different account"
    ))]
    /// Granting access to a function-call-only access key or a different account
    GrantWriteAccess(self::grant_write_access::SocialDbKey),
}
