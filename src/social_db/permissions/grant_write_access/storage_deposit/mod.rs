use std::str::FromStr;

use inquire::Text;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = AccessToPermissionKeyContext)]
#[interactive_clap(output_context = ExtraStorageDepositContext)]
pub struct ExtraStorageDeposit {
    #[interactive_clap(skip_default_input_arg)]
    extra_storage_deposit: near_cli_rs::common::NearBalance,
    #[interactive_clap(named_arg)]
    /// Specify signer account ID
    sign_as: super::sign_as::Signer,
}

#[derive(Clone)]
pub struct ExtraStorageDepositContext {
    pub global_context: near_cli_rs::GlobalContext,
    pub social_db_key: String,
    pub permission_key: near_socialdb_client::PermissionKey,
    pub extra_storage_deposit: near_cli_rs::common::NearBalance,
}

impl ExtraStorageDepositContext {
    pub fn from_previous_context(
        previous_context: AccessToPermissionKeyContext,
        scope: &<ExtraStorageDeposit as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.global_context,
            social_db_key: previous_context.social_db_key,
            permission_key: previous_context.permission_key,
            extra_storage_deposit: scope.extra_storage_deposit.clone(),
        })
    }
}

impl ExtraStorageDeposit {
    fn input_extra_storage_deposit(
        _context: &AccessToPermissionKeyContext,
    ) -> color_eyre::eyre::Result<Option<near_cli_rs::common::NearBalance>> {
        eprintln!();
        match near_cli_rs::common::NearBalance::from_str(
            &Text::new("Enter the amount of the NEAR tokens you want to extra storage deposit (each 100 kb of data requires 1 NEAR deposit):")
                .with_initial_value("0 NEAR")
                .prompt()?,
        ) {
            Ok(deposit) => Ok(Some(deposit)),
            Err(err) => Err(color_eyre::Report::msg(err)),
        }
    }
}

#[derive(Clone)]
pub struct AccessToPermissionKeyContext {
    pub global_context: near_cli_rs::GlobalContext,
    pub social_db_key: String,
    pub permission_key: near_socialdb_client::PermissionKey,
}
