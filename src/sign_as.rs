#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
pub struct SignerAccountId {
    /// What is the signer account ID?
    signer_account_id: near_cli_rs::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: near_cli_rs::network_for_transaction::NetworkForTransactionArgs,
}

impl SignerAccountId {
    pub fn get_signer_account_id(&self) -> near_cli_rs::types::account_id::AccountId {
        self.signer_account_id.clone()
    }

    pub fn get_network_config_for_transaction(
        &self,
    ) -> near_cli_rs::network_for_transaction::NetworkForTransactionArgs {
        self.network_config.clone()
    }
}
