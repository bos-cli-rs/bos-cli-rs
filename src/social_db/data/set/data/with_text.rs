#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::SetContext)]
#[interactive_clap(output_context = TextDataContext)]
pub struct TextData {
    /// Enter the data to set to the key:
    args: String,
    #[interactive_clap(named_arg)]
    /// Specify signer account ID
    sign_as: super::super::sign_as::Signer,
}

#[derive(Clone)]
pub struct TextDataContext(super::DataContext);

impl TextDataContext {
    pub fn from_previous_context(
        previous_context: super::super::SetContext,
        scope: &<TextData as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let value = serde_json::Value::String(scope.args.clone());
        Ok(Self(super::DataContext {
            global_context: previous_context.global_context,
            set_to_account_id: previous_context.set_to_account_id,
            key: previous_context.key,
            value,
        }))
    }
}

impl From<TextDataContext> for super::DataContext {
    fn from(item: TextDataContext) -> Self {
        item.0
    }
}
