use std::str::FromStr;

mod call_function_args_type;
mod sign_as;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = SetContext)]
pub struct Set {
    /// For which key do you want to set information?
    key: String,
    #[interactive_clap(value_enum)]
    #[interactive_clap(skip_default_input_arg)]
    /// How do you want to pass the function call arguments?
    function_args_type: self::call_function_args_type::FunctionArgsType,
    /// Enter the arguments to this function or the path to the arguments file:
    function_args: String,
    #[interactive_clap(named_arg)]
    /// Specify signer account ID
    sign_as: self::sign_as::Signer,
}

#[derive(Clone)]
pub struct SetContext {
    pub config: near_cli_rs::config::Config,
    pub set_to_account_id: near_cli_rs::types::account_id::AccountId,
    pub key: String,
    pub function_args_type: self::call_function_args_type::FunctionArgsType,
    pub function_args: String,
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
            function_args_type: scope.function_args_type.clone(),
            function_args: scope.function_args.clone(),
        })
    }
}

impl Set {
    fn input_function_args_type(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<self::call_function_args_type::FunctionArgsType>> {
        self::call_function_args_type::input_function_args_type()
    }
}
