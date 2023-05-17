use inquire::CustomType;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::DeleleteComponentsFromAccountContext)]
#[interactive_clap(output_context = ComponentContext)]
pub struct Component {
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
        scope: &<Component as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.0.config,
            account_id: previous_context.0.account_id,
            components: scope.components.clone().into(),
        })
    }
}

impl Component {
    pub fn input_components(
        _context: &super::DeleleteComponentsFromAccountContext,
    ) -> color_eyre::eyre::Result<Option<near_cli_rs::types::vec_string::VecString>> {
        loop {
            let input_components: near_cli_rs::types::vec_string::VecString =
                    CustomType::new("Enter a list of components to be removed (not more than 12 components at a time, separated by comma): ").prompt()?;
            if input_components.0.is_empty() {
                continue;
            } else if input_components.0.len() > 12 {
                println!("You have specified more than 12 components at once.")
            } else {
                return Ok(Some(input_components));
            }
        }
    }
}
