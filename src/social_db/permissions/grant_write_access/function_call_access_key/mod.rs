#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::SocialDbKeyContext)]
#[interactive_clap(output_context = AccessToPublicKeyContext)]
pub struct AccessToPublicKey {
    /// Enter the public access key that you will grant permission to:
    public_key: near_cli_rs::types::public_key::PublicKey,
    #[interactive_clap(named_arg)]
    /// Specify extra storage deposit
    with_extra_storage_deposit: super::storage_deposit::ExtraStorageDeposit,
}

#[derive(Clone)]
pub struct AccessToPublicKeyContext(super::storage_deposit::AccessToPermissionKeyContext);

impl AccessToPublicKeyContext {
    pub fn from_previous_context(
        previous_context: super::SocialDbKeyContext,
        scope: &<AccessToPublicKey as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(super::storage_deposit::AccessToPermissionKeyContext {
            config: previous_context.config,
            social_db_key: previous_context.social_db_key,
            permission_key: scope.public_key.0.clone().into(),
        }))
    }
}

impl From<AccessToPublicKeyContext> for super::storage_deposit::AccessToPermissionKeyContext {
    fn from(item: AccessToPublicKeyContext) -> Self {
        item.0
    }
}
