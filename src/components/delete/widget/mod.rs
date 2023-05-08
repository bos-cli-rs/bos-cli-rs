use std::str::FromStr;

use inquire::{CustomType, Select, Text};
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

// mod storage_deposit;
// mod sign_as;
// mod account_id;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::DeleleteWidgetsFromAccountContext)]
#[interactive_clap(output_context = WidgetContext)]
pub struct Widget {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    widgets: near_cli_rs::types::vec_string::VecString,
    #[interactive_clap(named_arg)]
    /// Specify signer account ID
    sign_as: super::sign_as::Signer,
}

#[derive(Clone)]
pub struct WidgetContext {
    pub config: near_cli_rs::config::Config,
    pub account_id: near_primitives::types::AccountId,
    pub widgets: Vec<String>,
}

impl WidgetContext {
    pub fn from_previous_context(
        previous_context: super::DeleleteWidgetsFromAccountContext,
        scope: &<Widget as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.config,
            account_id: previous_context.account_id,
            widgets: scope.widgets.clone().into(),
        })
    }
}

impl Widget {
    pub fn input_widgets(
        _context: &super::DeleleteWidgetsFromAccountContext,
    ) -> color_eyre::eyre::Result<Option<near_cli_rs::types::vec_string::VecString>> {
        #[derive(strum_macros::Display)]
        enum ConfirmOptions {
            #[strum(
                to_string = "Yes, I want to enter a list of widgets that can be granted permission."
            )]
            Yes,
            #[strum(to_string = "No, I want to grant permission to all widgets.")]
            No,
        }

        eprintln!();
        let select_choose_input = Select::new(
            "Do you want to enter a list of widgets that you want to grant permission to?",
            vec![ConfirmOptions::Yes, ConfirmOptions::No],
        )
        .prompt()?;
        if let ConfirmOptions::Yes = select_choose_input {
            let mut input_widget =
                Text::new("Enter a comma-separated list of allowed widgets.").prompt()?;
            if input_widget.contains('\"') {
                input_widget.clear()
            };
            if input_widget.is_empty() {
                Ok(Some(near_cli_rs::types::vec_string::VecString(vec![])))
            } else {
                Ok(Some(near_cli_rs::types::vec_string::VecString::from_str(
                    &input_widget,
                )?))
            }
        } else {
            Ok(Some(near_cli_rs::types::vec_string::VecString(vec![])))
        }
    }
}
