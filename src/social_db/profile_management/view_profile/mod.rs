use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod as_json;
mod as_text;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = near_cli_rs::GlobalContext)]
#[interactive_clap(output_context = AccountContext)]
pub struct Account {
    #[interactive_clap(skip_default_input_arg)]
    /// Which account's profile do you want to view?
    account_id: near_cli_rs::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    output_format: OutputFormat,
}

#[derive(Clone)]
pub struct AccountContext {
    global_context: near_cli_rs::GlobalContext,
    account_id: near_primitives::types::AccountId,
}

impl AccountContext {
    pub fn from_previous_context(
        previous_context: near_cli_rs::GlobalContext,
        scope: &<Account as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context,
            account_id: scope.account_id.clone().into(),
        })
    }
}

impl Account {
    pub fn input_account_id(
        context: &near_cli_rs::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<near_cli_rs::types::account_id::AccountId>> {
        near_cli_rs::common::input_non_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "Which account's profile do you want to view?",
        )
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = AccountContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Choose a format to view contract storage state:
pub enum OutputFormat {
    #[strum_discriminants(strum(message = "as-json    -  View account profile in JSON format"))]
    /// View account profile in JSON format
    AsJson(self::as_json::AsJson),
    #[strum_discriminants(strum(message = "as-text    -  View account profile in the text"))]
    /// View account profile in the text
    AsText(self::as_text::AsText),
}
