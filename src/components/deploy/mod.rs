use color_eyre::eyre::ContextCompat;
use inquire::Select;

mod sign_as;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransactionFunctionArgs {
    pub data: crate::socialdb_types::SocialDb,
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::ComponentsContext)]
#[interactive_clap(output_context = DeployCmdContext)]
pub struct DeployCmd {
    #[interactive_clap(skip_default_input_arg)]
    /// Which account do you want to deploy the components to?
    deploy_to_account_id: near_cli_rs::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Specify signer account ID
    sign_as: self::sign_as::Signer,
}

#[derive(Clone)]
pub struct DeployCmdContext {
    pub global_context: near_cli_rs::GlobalContext,
    pub social_db_prefix: String,
    pub deploy_to_account_id: near_cli_rs::types::account_id::AccountId,
}

impl DeployCmdContext {
    pub fn from_previous_context(
        previous_context: super::ComponentsContext,
        scope: &<DeployCmd as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.global_context,
            social_db_prefix: previous_context.social_db_prefix,
            deploy_to_account_id: scope.deploy_to_account_id.clone(),
        })
    }
}

impl DeployCmd {
    fn input_deploy_to_account_id(
        context: &super::ComponentsContext,
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
            let deploy_to_account_id =
                near_cli_rs::common::input_signer_account_id_from_used_account_list(
                    &context.global_context.config.credentials_home_dir,
                    "Which account do you want to deploy the components to?",
                )?
                .wrap_err("Internal error!")?;
            if !near_cli_rs::common::is_account_exist(
                &context.global_context.config.network_connection,
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
