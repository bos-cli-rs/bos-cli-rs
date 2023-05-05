use std::str::FromStr;

use strum::{EnumDiscriminants, EnumIter, EnumMessage};
use inquire::{CustomType, Text, Select};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::account_id::AccessToAccountContext)]
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
    pub config: near_cli_rs::config::Config,
    pub widgets: Vec<String>,
    pub account_id: near_primitives::types::AccountId,
    pub extra_storage_deposit: near_cli_rs::common::NearBalance,
}

impl ExtraStorageDepositContext {
    pub fn from_previous_context(
        previous_context: super::account_id::AccessToAccountContext,
        scope: &<ExtraStorageDeposit as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.config,
            widgets: previous_context.widgets,
            account_id: previous_context.account_id,
            extra_storage_deposit: scope.extra_storage_deposit.clone()
        })
    }
}


impl ExtraStorageDeposit {
    fn input_extra_storage_deposit(
        _context: &super::account_id::AccessToAccountContext,
    ) -> color_eyre::eyre::Result<Option<near_cli_rs::common::NearBalance>> {
        eprintln!();
        match near_cli_rs::common::NearBalance::from_str(&Text::new("Enter the amount of the NEAR tokens you want to extra storage deposit.")
            .with_initial_value("1 NEAR")
            .prompt()?
            ) {
                Ok(deposit) => Ok(Some(deposit)),
                Err(err) => Err(color_eyre::Report::msg(
                    err,
                ))
            }
    }
}
