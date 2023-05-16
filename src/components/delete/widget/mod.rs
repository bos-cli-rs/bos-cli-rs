use std::str::FromStr;

use inquire::Text;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::DeleleteWidgetsFromAccountContext)]
#[interactive_clap(output_context = WidgetContext)]
pub struct Widget {
    #[interactive_clap(skip_default_input_arg)]
    widgets: near_cli_rs::types::vec_string::VecString,
    #[interactive_clap(named_arg)]
    /// Specify signer account ID
    sign_as: super::sign_as::Signer,
}

#[derive(Clone)]
pub struct WidgetContext {
    pub config: near_cli_rs::config::Config,
    pub account_id: near_cli_rs::types::account_id::AccountId,
    pub widgets: Vec<String>,
}

impl WidgetContext {
    pub fn from_previous_context(
        previous_context: super::DeleleteWidgetsFromAccountContext,
        scope: &<Widget as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.0.config,
            account_id: previous_context.0.account_id,
            widgets: scope.widgets.clone().into(),
        })
    }
}

impl Widget {
    pub fn input_widgets(
        _context: &super::DeleleteWidgetsFromAccountContext,
    ) -> color_eyre::eyre::Result<Option<near_cli_rs::types::vec_string::VecString>> {
        loop {
            let mut input_widget =
                    Text::new("Enter a comma-separated list of widgets to be removed.\nNote! You can delete no more than 12 widgets at a time.").prompt()?;
            if input_widget.contains('\"') {
                input_widget.clear()
            };
            if input_widget.is_empty() {
                continue;
            } else {
                let widgets = near_cli_rs::types::vec_string::VecString::from_str(&input_widget)?;
                if widgets.0.len() > 12 {
                    println!("You have specified more than 12 widgets at once.")
                } else {
                    return Ok(Some(widgets));
                }
            }
        }
    }
}
