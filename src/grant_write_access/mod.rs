use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod account_id;
mod function_call_access_key;
mod sign_as;
mod storage_deposit;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct Widget {
    #[interactive_clap(subcommand)]
    access: Access,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// What are you up to?
pub enum Access {
    #[strum_discriminants(strum(
        message = "to-function-call-access-key  -   Granting access to a function-call-only access key"
    ))]
    /// Granting access to a function-call-only access key
    ToFunctionCallAccessKey(self::function_call_access_key::AccessToPublicKey),
    #[strum_discriminants(strum(
        message = "to-account                   -   Granting access to a different account"
    ))]
    /// Granting access to a different account
    ToAccount(self::account_id::AccessToAccount),
}
