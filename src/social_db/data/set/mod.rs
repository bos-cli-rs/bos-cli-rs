use std::str::FromStr;

mod call_function_args_type;
mod function_args;
mod sign_as;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = SetContext)]
pub struct Set {
    /// For which key do you want to set information?
    key: String,
    #[interactive_clap(subcommand)]
    function_args_type: self::call_function_args_type::FunctionArgsType,
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
                scope.key.split('/').map(|s| s.trim()).collect::<Vec<_>>()[0],
            )?,
            key: scope.key.clone(),
        })
    }
}
