use interactive_clap::{FromCli, ToCliArgs};
pub use near_cli_rs::{common, config, types, CliResult};
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod sign_as;

/// near-cli is a toolbox for interacting with NEAR protocol
pub type GlobalContext = (crate::config::Config,);

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
struct Cmd {
    #[interactive_clap(subcommand)]
    command: self::Command,
}

impl Cmd {
    async fn process(&self, config: crate::config::Config) -> CliResult {
        self.command.process(config).await
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(disable_back)]
/// Deploy command
pub enum Command {
    #[strum_discriminants(strum(message = "Deploy -   Deploy widget if code has changed"))]
    /// Deploy widget if code has changed
    Deploy(self::sign_as::SignAs),
}

impl Command {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        match self {
            Self::Deploy(sign_as) => sign_as.process(config).await,
        }
    }
}

fn main() -> CliResult {
    let config = crate::common::get_config_toml()?;

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
