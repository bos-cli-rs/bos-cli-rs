#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::SocialDbKeyContext)]
#[interactive_clap(output_context = AccessToAccountContext)]
pub struct AccessToAccount {
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
            config: previous_context.config,
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
