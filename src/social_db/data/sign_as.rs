use inquire::{CustomType, Select};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = PreparedSignerContext)]
#[interactive_clap(output_context = SignerContext)]
pub struct Signer {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the signer account ID?
    signer_account_id: near_cli_rs::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: near_cli_rs::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Clone)]
pub struct SignerContext(near_cli_rs::commands::ActionContext);

impl SignerContext {
    pub fn from_previous_context(
        previous_context: PreparedSignerContext,
        scope: &<Signer as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let signer_account_id = scope.signer_account_id.clone();

        let on_after_getting_network_callback: near_cli_rs::commands::OnAfterGettingNetworkCallback = std::sync::Arc::new(
            move |network_config| {
                let mut prepopulated_transaction = (previous_context.on_after_getting_network_callback)(network_config)?;
                prepopulated_transaction.signer_id = signer_account_id.clone().into();
                Ok(prepopulated_transaction)
            });

        Ok(Self(near_cli_rs::commands::ActionContext {
            config: previous_context.config,
            on_after_getting_network_callback,
            on_before_signing_callback: std::sync::Arc::new(|_transaction, _network_config| Ok(())),
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config, _message| Ok(()),
            ),
            on_after_sending_transaction_callback: previous_context
                .on_after_sending_transaction_callback,
        }))
    }
}

impl From<SignerContext> for near_cli_rs::commands::ActionContext {
    fn from(item: SignerContext) -> Self {
        item.0
    }
}

impl Signer {
    fn input_signer_account_id(
        context: &PreparedSignerContext,
    ) -> color_eyre::eyre::Result<Option<near_cli_rs::types::account_id::AccountId>> {
        loop {
            let signer_account_id: near_cli_rs::types::account_id::AccountId =
                CustomType::new(" What is the signer account ID?")
                    .with_default(context.account_id.clone())
                    .prompt()?;
            if !near_cli_rs::common::is_account_exist(
                &context.config.network_connection,
                signer_account_id.clone().into(),
            ) {
                println!("\nThe account <{signer_account_id}> does not yet exist.");
                #[derive(strum_macros::Display)]
                enum ConfirmOptions {
                    #[strum(to_string = "Yes, I want to enter a new account name.")]
                    Yes,
                    #[strum(to_string = "No, I want to use this account name.")]
                    No,
                }
                let select_choose_input = Select::new(
                    "Do you want to enter another signer account id?",
                    vec![ConfirmOptions::Yes, ConfirmOptions::No],
                )
                .prompt()?;
                if let ConfirmOptions::No = select_choose_input {
                    return Ok(Some(signer_account_id));
                }
            } else {
                return Ok(Some(signer_account_id));
            }
        }
    }
}

#[derive(Clone)]
pub struct PreparedSignerContext {
    pub config: near_cli_rs::config::Config,
    pub account_id: near_cli_rs::types::account_id::AccountId,
    pub on_after_getting_network_callback: near_cli_rs::commands::OnAfterGettingNetworkCallback,
    pub on_before_signing_callback: near_cli_rs::commands::OnBeforeSigningCallback,
    pub on_before_sending_transaction_callback:
        near_cli_rs::transaction_signature_options::OnBeforeSendingTransactionCallback,
    pub on_after_sending_transaction_callback:
        near_cli_rs::transaction_signature_options::OnAfterSendingTransactionCallback,
}
