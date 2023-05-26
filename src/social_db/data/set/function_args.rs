#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::call_function_args_type::FunctionArgsTypeContext)]
#[interactive_clap(output_context = FunctionArgsContext)]
pub struct FunctionArgs {
    /// Enter the arguments to this function or the path to the arguments file:
    function_args: String,
    #[interactive_clap(named_arg)]
    /// Specify signer account ID
    sign_as: super::sign_as::Signer,
}

#[derive(Clone)]
pub struct FunctionArgsContext {
    pub config: near_cli_rs::config::Config,
    pub set_to_account_id: near_cli_rs::types::account_id::AccountId,
    pub key: String,
    pub function_args_type: super::call_function_args_type::FunctionArgsTypeDiscriminants,
    pub function_args: String,
}

impl FunctionArgsContext {
    pub fn from_previous_context(
        previous_context: super::call_function_args_type::FunctionArgsTypeContext,
        scope: &<FunctionArgs as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.config,
            set_to_account_id: previous_context.set_to_account_id,
            key: previous_context.key,
            function_args_type: previous_context.function_args_type,
            function_args: scope.function_args.clone(),
        })
    }
}
