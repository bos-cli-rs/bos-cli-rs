use color_eyre::eyre::ContextCompat;
use inquire::Select;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::SocialDbKeyContext)]
#[interactive_clap(output_context = AccessToAccountContext)]
pub struct AccessToAccount {
    #[interactive_clap(skip_default_input_arg)]
    /// Enter the account ID you will grant permission to:
    account_id: near_cli_rs::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Specify extra storage deposit
    with_extra_storage_deposit: super::storage_deposit::ExtraStorageDeposit,
}

#[derive(Clone)]
pub struct AccessToAccountContext(super::storage_deposit::AccessToPermissionKeyContext);

impl AccessToAccountContext {
    pub fn from_previous_context(
        previous_context: super::SocialDbKeyContext,
        scope: &<AccessToAccount as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(super::storage_deposit::AccessToPermissionKeyContext {
            global_context: previous_context.global_context,
            social_db_key: previous_context.social_db_key,
            permission_key: scope.account_id.0.clone().into(),
        }))
    }
}

impl From<AccessToAccountContext> for super::storage_deposit::AccessToPermissionKeyContext {
    fn from(item: AccessToAccountContext) -> Self {
        item.0
    }
}

impl AccessToAccount {
    pub fn input_account_id(
        context: &super::SocialDbKeyContext,
    ) -> color_eyre::eyre::Result<Option<near_cli_rs::types::account_id::AccountId>> {
        loop {
            let deploy_to_account_id =
                near_cli_rs::common::input_non_signer_account_id_from_used_account_list(
                    &context.global_context.config.credentials_home_dir,
                    "Enter the account ID you will grant permission to:",
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
                    "Do you want to enter a new account name?",
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
