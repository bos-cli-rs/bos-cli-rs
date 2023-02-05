use interactive_clap::{FromCli, ToCliArgs};
pub use near_cli_rs::CliResult;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

pub mod common;
mod deploy;
mod download;
pub mod socialdb_types;

/// near-cli is a toolbox for interacting with NEAR protocol
pub type GlobalContext = (near_cli_rs::config::Config,);

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
struct Cmd {
    #[interactive_clap(subcommand)]
    command: self::Command,
}

impl Cmd {
    async fn process(&self, config: near_cli_rs::config::Config) -> CliResult {
        self.command.process(config).await
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(disable_back)]
/// What are you up to? (select one of the options with the up-down arrows on your keyboard and press Enter)
pub enum Command {
    #[strum_discriminants(strum(message = "download -   Download widgets from account"))]
    /// Download widgets from account
    Download(self::download::AccountId),
    #[strum_discriminants(strum(message = "deploy   -   Deploy widget if code has changed"))]
    /// Deploy widget if code has changed
    Deploy(self::deploy::DeployArgs),
}

impl Command {
    pub async fn process(&self, config: near_cli_rs::config::Config) -> crate::CliResult {
        match self {
            Self::Download(account_id) => account_id.process(config).await,
            Self::Deploy(sign_as) => sign_as.process(config).await,
        }
    }
}

fn main() -> CliResult {
    let config = near_cli_rs::common::get_config_toml()?;

    color_eyre::install()?;

    let cli = match Cmd::try_parse() {
        Ok(cli) => cli,
        Err(error) => error.exit(),
    };

    let cmd = loop {
        match Cmd::from_cli(Some(cli.clone()), (config.clone(),)) {
            Ok(Some(cmd)) => {
                break cmd;
            }
            Ok(None) => {}
            Err(err) => match err.downcast_ref() {
                Some(
                    inquire::InquireError::OperationCanceled
                    | inquire::InquireError::OperationInterrupted,
                ) => {
                    println!("<Operation was interrupted. Goodbye>");
                    return Ok(());
                }
                Some(_) | None => return Err(err),
            },
        }
    };

    let completed_cli = CliCmd::from(cmd.clone());

    let process_result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(cmd.process(config));

    println!(
        "Your console command:\n{} {}",
        std::env::args()
            .next()
            .as_deref()
            .unwrap_or("./near-social"),
        shell_words::join(completed_cli.to_cli_args())
    );

    match process_result {
        Ok(()) => Ok(()),
        Err(err) => match err.downcast_ref() {
            Some(
                inquire::InquireError::OperationCanceled
                | inquire::InquireError::OperationInterrupted,
            ) => {
                println!("<Operation was interrupted. Goodbye>");
                Ok(())
            }
            Some(_) | None => Err(err),
        },
    }
}
