use std::str::FromStr;

use color_eyre::eyre::ContextCompat;

mod data;
mod sign_as;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = SetContext)]
pub struct Set {
    /// Enter SocialDB key path to set the value (e.g. root.near/profile/name):
    key: String,
    #[interactive_clap(subcommand)]
    data_type: self::data::DataType,
}

#[derive(Clone)]
pub struct SetContext {
    pub config: near_cli_rs::config::Config,
    pub set_to_account_id: near_cli_rs::types::account_id::AccountId,
    pub key: String,
}

impl SetContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<Set as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.0,
            set_to_account_id: near_cli_rs::types::account_id::AccountId::from_str(
                scope
                    .key
                    .split_once('/')
                    .wrap_err("Failed to parse account_id from this key")?
                    .0
                    .trim(),
            )?,
            key: scope.key.clone(),
        })
    }
}
