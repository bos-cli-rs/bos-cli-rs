use std::str::FromStr;

mod sign_as;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = DeleteContext)]
pub struct Delete {
    /// For which key do you want to delete information?
    key: String,
    #[interactive_clap(named_arg)]
    /// Select network
    sign_as: self::sign_as::Signer,
}

#[derive(Clone)]
pub struct DeleteContext {
    pub config: near_cli_rs::config::Config,
    pub key: String,
    pub account_id: near_cli_rs::types::account_id::AccountId,
}

impl DeleteContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<Delete as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let account_id = near_cli_rs::types::account_id::AccountId::from_str(
            scope.key.split('/').map(|s| s.trim()).collect::<Vec<_>>()[0],
        )?;
        Ok(Self {
            config: previous_context.0,
            key: scope.key.clone(),
            account_id,
        })
    }
}
