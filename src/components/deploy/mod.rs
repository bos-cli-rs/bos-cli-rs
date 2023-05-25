use inquire::{CustomType, Select};

mod sign_as;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransactionFunctionArgs {
    pub data: crate::socialdb_types::SocialDb,
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = DeployToAccountContext)]
pub struct DeployToAccount {
    #[interactive_clap(skip_default_input_arg)]
    /// Which account do you want to deploy the components to?
    deploy_to_account_id: near_cli_rs::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Specify signer account ID
    sign_as: self::sign_as::Signer,
}

#[derive(Clone)]
pub struct DeployToAccountContext {
    pub config: near_cli_rs::config::Config,
    pub deploy_to_account_id: near_cli_rs::types::account_id::AccountId,
}

impl DeployToAccountContext {
    pub fn from_previous_context(
        previous_context: near_cli_rs::GlobalContext,
        scope: &<DeployToAccount as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.0,
            deploy_to_account_id: scope.deploy_to_account_id.clone(),
        })
    }
}

impl DeployToAccount {
    fn input_deploy_to_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<near_cli_rs::types::account_id::AccountId>> {
        let components = crate::common::get_local_components()?;
        println!(
            "\nThere are <{}> components in the current folder ready for deployment:",
            components.len()
        );
        for component in components.keys() {
            println!(" * {component}")
        }
        loop {
            let deploy_to_account_id: near_cli_rs::types::account_id::AccountId =
                CustomType::new("Which account do you want to deploy the components to?")
                    .prompt()?;
            if !near_cli_rs::common::is_account_exist(
                &context.0.network_connection,
                deploy_to_account_id.clone().into(),
            ) {
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
                    "Do you want to enter a new component deployment account name?",
                    vec![ConfirmOptions::Yes, ConfirmOptions::No],
                )
                .prompt()?;
                if let ConfirmOptions::No = select_choose_input {
                    return Ok(Some(deploy_to_account_id));
                }
            } else {
                return Ok(Some(deploy_to_account_id));
            }
        }
    }
}
