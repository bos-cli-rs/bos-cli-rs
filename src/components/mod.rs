use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod delete;
mod deploy;
mod diff;
mod download;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = near_cli_rs::GlobalContext)]
#[interactive_clap(output_context = ComponentsContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct Components {
    /// Change SocialDb prefix
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    social_db_prefix: Option<String>,
    #[interactive_clap(subcommand)]
    command: self::ComponentsCommand,
}

#[derive(Clone)]
pub struct ComponentsContext {
    pub global_context: near_cli_rs::GlobalContext,
    pub social_db_prefix: String,
}

impl ComponentsContext {
    pub fn from_previous_context(
        previous_context: near_cli_rs::GlobalContext,
        scope: &<Components as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context,
            social_db_prefix: scope
                .social_db_prefix
                .clone()
                .unwrap_or("widget".to_owned()),
        })
    }
}

impl interactive_clap::FromCli for Components {
    type FromCliContext = near_cli_rs::GlobalContext;
    type FromCliError = color_eyre::eyre::Error;
    fn from_cli(
        optional_clap_variant: Option<<Self as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> interactive_clap::ResultFromCli<
        <Self as interactive_clap::ToCli>::CliVariant,
        Self::FromCliError,
    >
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let mut clap_variant = optional_clap_variant.unwrap_or_default();

        if clap_variant.social_db_prefix.is_none() {
            clap_variant.social_db_prefix = match Self::input_social_db_prefix(&context) {
                Ok(optional_social_db_prefix) => optional_social_db_prefix,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        }
        let social_db_prefix = clap_variant.social_db_prefix.clone();

        let new_context_scope = InteractiveClapContextScopeForComponents { social_db_prefix };
        let output_context =
            match ComponentsContext::from_previous_context(context, &new_context_scope) {
                Ok(new_context) => new_context,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };

        match ComponentsCommand::from_cli(clap_variant.command.take(), output_context) {
            interactive_clap::ResultFromCli::Ok(cli_components_command) => {
                clap_variant.command = Some(cli_components_command);
                interactive_clap::ResultFromCli::Ok(clap_variant)
            }
            interactive_clap::ResultFromCli::Cancel(optional_cli_components_command) => {
                clap_variant.command = optional_cli_components_command;
                interactive_clap::ResultFromCli::Cancel(Some(clap_variant))
            }
            interactive_clap::ResultFromCli::Back => interactive_clap::ResultFromCli::Back,
            interactive_clap::ResultFromCli::Err(optional_cli_components_command, err) => {
                clap_variant.command = optional_cli_components_command;
                interactive_clap::ResultFromCli::Err(Some(clap_variant), err)
            }
        }
    }
}

impl Components {
    fn input_social_db_prefix(
        _context: &near_cli_rs::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        Ok(None)
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = ComponentsContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// What are you up to?
pub enum ComponentsCommand {
    #[strum_discriminants(strum(message = "download    -   Download components from account"))]
    /// Download components from account
    Download(self::download::DownloadCmd),
    #[strum_discriminants(strum(
        message = "diff        -   Differences between component code for deployment"
    ))]
    /// Differences between component code for deployment
    Diff(self::diff::DiffCmd),
    #[strum_discriminants(strum(
        message = "deploy      -   Deploy components if code has changed"
    ))]
    /// Deploy сomponents if code has changed
    Deploy(self::deploy::DeployCmd),
    #[strum_discriminants(strum(message = "delete      -   Delete components from account"))]
    /// Delete components from account
    Delete(self::delete::DeleteCmd),
}
