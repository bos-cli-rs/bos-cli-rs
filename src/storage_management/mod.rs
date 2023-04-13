use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod view_storage_balance;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct StorageManagement {
    #[interactive_clap(subcommand)]
    storage_actions: StorageActions,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// What do you want to do with the storage?
pub enum StorageActions {
    #[strum_discriminants(strum(
        message = "view-storage-balance    - View storage balance for an account"
    ))]
    /// View storage balance for an account
    ViewStorageBalance(self::view_storage_balance::AccountId),
    #[strum_discriminants(strum(
        message = "storage-deposit         - Make a storage deposit for the account"
    ))]
    /// Make a storage deposit for the account
    StorageDeposit,
    #[strum_discriminants(strum(
        message = "storage-withdraw        - Withdraw storage for the account"
    ))]
    /// Withdraw storage for the account
    StorageWithdraw,
}
