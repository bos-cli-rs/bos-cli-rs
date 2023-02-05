use inquire::{CustomType, Select};

mod sign_as;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
pub struct DeployArgs {
    #[interactive_clap(skip_default_input_arg)]
    /// Which account do you want to deploy the widgets to?
    deploy_to_account_id: near_cli_rs::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Specify signer account ID
    sign_as: self::sign_as::SignerAccountId,
}

impl DeployArgs {
    fn input_deploy_to_account_id(
        context: &near_cli_rs::GlobalContext,
    ) -> color_eyre::eyre::Result<near_cli_rs::types::account_id::AccountId> {
        let widgets = crate::common::get_widgets()?;
        println!(
            "\nThere are <{}> widgets in the current folder ready for deployment:",
            widgets.len()
        );
        for widget in widgets.keys() {
            println!(" * {widget}")
        }
        loop {
            let deploy_to_account_id: near_cli_rs::types::account_id::AccountId =
                CustomType::new("Which account do you want to deploy the widgets to?").prompt()?;
            if !crate::common::is_account_exist(context, deploy_to_account_id.clone().into()) {
                println!(
                    "\nThe account <{}> does not yet exist.",
                    &deploy_to_account_id
                );
                #[derive(strum_macros::Display)]
                enum ConfirmOptions {
                    #[strum(to_string = "Yes, I want to enter a new account name.")]
                    Yes,
                    #[strum(to_string = "No, I want to use this account name.")]
                    No,
                }
                let select_choose_input = Select::new(
                    "Do you want to enter a new widget deployment account name?",
                    vec![ConfirmOptions::Yes, ConfirmOptions::No],
                )
                .prompt()?;
                if let ConfirmOptions::No = select_choose_input {
                    return Ok(deploy_to_account_id);
                }
            } else {
                return Ok(deploy_to_account_id);
            }
        }
    }

    pub async fn process(&self, config: near_cli_rs::config::Config) -> crate::CliResult {
        self.sign_as
            .process(config, self.deploy_to_account_id.clone())
            .await
    }
}
