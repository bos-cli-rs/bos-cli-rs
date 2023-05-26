use std::str::FromStr;

use color_eyre::eyre::Context;
use inquire::Text;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::SetContext)]
#[interactive_clap(output_context = FunctionArgsContext)]
pub struct FunctionArgs {
    #[interactive_clap(skip_default_input_arg)]
    /// Enter the arguments to this function:
    args: String,
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
        let value =
            serde_json::Value::from_str(&scope.args).wrap_err("Data not in JSON format!")?;
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

impl FunctionArgs {
    fn input_args(_context: &super::super::SetContext) -> color_eyre::eyre::Result<Option<String>> {
        loop {
            let args = Text::new("Enter the arguments to this function:").prompt()?;
            if serde_json::Value::from_str(&args).is_err() {
                println!("Data not in JSON format!");
            } else {
                return Ok(Some(args));
            }
        }
    }
}
