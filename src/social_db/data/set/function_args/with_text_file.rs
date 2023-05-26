use color_eyre::eyre::Context;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::SetContext)]
#[interactive_clap(output_context = FunctionArgsContext)]
pub struct FunctionArgs {
    /// Enter the path to the arguments file:
    path: near_cli_rs::types::path_buf::PathBuf,
    #[interactive_clap(named_arg)]
    /// Specify signer account ID
    sign_as: super::super::sign_as::Signer,
}

#[derive(Clone)]
pub struct FunctionArgsContext(super::ArgsContext);

impl FunctionArgsContext {
    pub fn from_previous_context(
        previous_context: super::super::SetContext,
        scope: &<FunctionArgs as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let data = std::fs::read_to_string(&scope.path.0)
            .wrap_err_with(|| format!("Access to data file <{:?}> not found!", scope.path))?;
        let value = serde_json::Value::String(data);
        Ok(Self(super::ArgsContext {
            config: previous_context.config,
            set_to_account_id: previous_context.set_to_account_id,
            key: previous_context.key,
            value,
        }))
    }
}

impl From<FunctionArgsContext> for super::ArgsContext {
    fn from(item: FunctionArgsContext) -> Self {
        item.0
    }
}
