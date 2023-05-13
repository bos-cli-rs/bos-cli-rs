use inquire::Text;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod account_id;
mod function_call_access_key;
mod sign_as;
mod storage_deposit;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = SocialDbKeyContext)]
pub struct SocialDbKey {
    #[interactive_clap(skip_default_input_arg)]
    /// Enter the prefix of the social_db key that you will grant permission to (default value: 'widget')
    social_db_key: String,
    #[interactive_clap(subcommand)]
    access: Access,
}

#[derive(Clone)]
pub struct SocialDbKeyContext {
    pub config: near_cli_rs::config::Config,
    pub social_db_key: String,
}

impl SocialDbKeyContext {
    pub fn from_previous_context(
        previous_context: near_cli_rs::GlobalContext,
        scope: &<SocialDbKey as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.0,
            social_db_key: scope.social_db_key.clone(),
        })
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = SocialDbKeyContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Select grant access permissions
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

impl SocialDbKey {
    fn input_social_db_key(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        Ok(Some(
            Text::new(" Enter the prefix of the social_db key that you will grant permission to (default value: 'widget')")
                .with_default("widget")
                .prompt()?,
        ))
    }
}
