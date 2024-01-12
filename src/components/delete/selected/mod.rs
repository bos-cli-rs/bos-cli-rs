use inquire::CustomType;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::DeleteCmdContext)]
#[interactive_clap(output_context = ComponentContext)]
pub struct Selected {
    #[interactive_clap(skip_default_input_arg)]
    components: near_cli_rs::types::vec_string::VecString,
    #[interactive_clap(named_arg)]
    /// Specify signer account ID
    sign_as: super::sign_as::Signer,
}

#[derive(Clone)]
pub struct ComponentContext {
    pub global_context: near_cli_rs::GlobalContext,
    pub social_db_prefix: String,
    pub account_id: near_cli_rs::types::account_id::AccountId,
    pub components: Vec<String>,
}

impl ComponentContext {
    pub fn from_previous_context(
        previous_context: super::DeleteCmdContext,
        scope: &<Selected as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.0.global_context,
            social_db_prefix: previous_context.0.social_db_prefix,
            account_id: previous_context.0.account_id,
            components: scope.components.clone().into(),
        })
    }
}

impl Selected {
    pub fn input_components(
        _context: &super::DeleteCmdContext,
    ) -> color_eyre::eyre::Result<Option<near_cli_rs::types::vec_string::VecString>> {
        loop {
            let input_components: near_cli_rs::types::vec_string::VecString =
                CustomType::new("Enter a comma-separated list of components to remove:")
                    .prompt()?;
            if input_components.0.is_empty() {
                continue;
            } else {
                return Ok(Some(input_components));
            }
        }
    }
}
