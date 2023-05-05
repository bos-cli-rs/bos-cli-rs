use std::str::FromStr;

use strum::{EnumDiscriminants, EnumIter, EnumMessage};
use inquire::{CustomType, Text, Select};

mod storage_deposit;
mod sign_as;
mod account_id;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = WidgetContext)]
pub struct Widget {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    widgets: near_cli_rs::types::vec_string::VecString,
    #[interactive_clap(subcommand)]
    access: Access,
}

#[derive(Clone)]
pub struct WidgetContext {
    pub config: near_cli_rs::config::Config,
    pub widgets: Vec<String>,
}

impl WidgetContext {
    pub fn from_previous_context(
        previous_context: near_cli_rs::GlobalContext,
        scope: &<Widget as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.0,
            widgets: scope.widgets.clone().into(),
        })
    }
}

impl Widget {
    pub fn input_widgets(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<near_cli_rs::types::vec_string::VecString>> {
        #[derive(strum_macros::Display)]
        enum ConfirmOptions {
            #[strum(to_string = "Yes, I want to enter a list of widgets that can be granted permission.")]
            Yes,
            #[strum(
                to_string = "No, I want to grant permission to all widgets."
            )]
            No,
        }

        eprintln!();
        let select_choose_input = Select::new(
            "Do you want to enter a list of widgets that you want to grant permission to?",
            vec![ConfirmOptions::Yes, ConfirmOptions::No],
        )
        .prompt()?;
        if let ConfirmOptions::Yes = select_choose_input {
            let mut input_widget = Text::new("Enter a comma-separated list of allowed widgets.")
                    .prompt()?;
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

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = WidgetContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// What are you up to?
pub enum Access {
    #[strum_discriminants(strum(message = "download -   Granting access to a function-call-only access key"))]
    /// Granting access to a function-call-only access key
    ToFunctionCallAccessKey,
    #[strum_discriminants(strum(
        message = "deploy               -   Granting access to a different account"
    ))]
    /// Granting access to a different account
    ToAccount(self::account_id::AccessToAccount),
}

