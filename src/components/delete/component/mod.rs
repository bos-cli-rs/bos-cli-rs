use inquire::CustomType;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = super::DeleleteComponentsFromAccountContext)]
pub struct AllComponents {
    #[interactive_clap(named_arg)]
    /// Specify signer account ID
    sign_as: super::sign_as::Signer,
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::DeleleteComponentsFromAccountContext)]
#[interactive_clap(output_context = ComponentContext)]
pub struct SelectedComponents {
    #[interactive_clap(skip_default_input_arg)]
    components: near_cli_rs::types::vec_string::VecString,
    #[interactive_clap(named_arg)]
    /// Specify signer account ID
    sign_as: super::sign_as::Signer,
}

#[derive(Clone)]
pub struct ComponentContext {
    pub config: near_cli_rs::config::Config,
    pub account_id: near_cli_rs::types::account_id::AccountId,
    pub components: Vec<String>,
}

impl ComponentContext {
    pub fn from_previous_context(
        previous_context: super::DeleleteComponentsFromAccountContext,
        scope: &<SelectedComponents as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.0.config,
            account_id: previous_context.0.account_id,
            components: scope.components.clone().into(),
        })
    }
}

impl SelectedComponents {
    pub fn input_components(
        _context: &super::DeleleteComponentsFromAccountContext,
    ) -> color_eyre::eyre::Result<Option<near_cli_rs::types::vec_string::VecString>> {
        loop {
            let input_components: near_cli_rs::types::vec_string::VecString =
                CustomType::new("Enter a comma-separated list of components to remove: ")
                    .prompt()?;
            if input_components.0.is_empty() {
                continue;
            } else {
                return Ok(Some(input_components));
            }
        }
    }
}
