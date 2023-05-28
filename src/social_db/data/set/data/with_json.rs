use inquire::CustomType;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::SetContext)]
#[interactive_clap(output_context = JsonDataContext)]
pub struct JsonData {
    #[interactive_clap(skip_default_input_arg)]
    /// Enter the data to set to the key (e.g. {\"token_id\": \"42\"}):
    args: near_cli_rs::types::json::Json,
    #[interactive_clap(named_arg)]
    /// Specify signer account ID
    sign_as: super::super::sign_as::Signer,
}

#[derive(Clone)]
pub struct JsonDataContext(super::DataContext);

impl JsonDataContext {
    pub fn from_previous_context(
        previous_context: super::super::SetContext,
        scope: &<JsonData as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(super::DataContext {
            config: previous_context.config,
            set_to_account_id: previous_context.set_to_account_id,
            key: previous_context.key,
            value: scope.args.clone().into(),
        }))
    }
}

impl From<JsonDataContext> for super::DataContext {
    fn from(item: JsonDataContext) -> Self {
        item.0
    }
}

impl JsonData {
    fn input_args(
        _context: &super::super::SetContext,
    ) -> color_eyre::eyre::Result<Option<near_cli_rs::types::json::Json>> {
        let args: near_cli_rs::types::json::Json =
            CustomType::new("Enter the data to set to the key (e.g. {\"token_id\": \"42\"}):")
                .prompt()?;
        Ok(Some(args))
    }
}
