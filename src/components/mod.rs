use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod delete;
mod deploy;
mod diff;
mod download;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = near_cli_rs::GlobalContext)]
#[interactive_clap(output_context = ComponentsContext)]
pub struct Components {
    /// Change SocialDb prefix (default: "widget")
    #[interactive_clap(long)]
    #[interactive_clap(skip_interactive_input)]
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
