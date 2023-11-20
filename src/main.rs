#![allow(
    clippy::enum_variant_names,
    clippy::large_enum_variant,
    clippy::arc_with_non_send_sync
)]
use interactive_clap::ToCliArgs;
pub use near_cli_rs::CliResult;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

pub mod common;
mod components;
pub mod consts;
mod project;
mod social_db;
pub mod socialdb_types;

/// near-cli is a toolbox for interacting with NEAR protocol

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
struct Cmd {
    #[interactive_clap(subcommand)]
    command: self::Command,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = near_cli_rs::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(disable_back)]
/// What are you up to? (select one of the options with the up-down arrows on your keyboard and press Enter)
pub enum Command {
    #[strum_discriminants(strum(message = "project      -   Project management"))]
    /// Project management
    Project(self::project::Project),
    #[strum_discriminants(strum(
        message = "components   -   Working with components (Download, Deploy, etc.)"
    ))]
    /// Working with components (Download, Deploy, etc.)
    Components(self::components::Components),
    #[strum_discriminants(strum(message = "socialdb     -   SocialDb management"))]
    /// Storage management: deposit, withdrawal, balance review
    SocialDb(self::social_db::SocialDb),
}

fn main() -> CliResult {
    let config = near_cli_rs::common::get_config_toml()?;

    color_eyre::install()?;

    let cli = match Cmd::try_parse() {
        Ok(cli) => cli,
        Err(error) => error.exit(),
    };

    let global_context = near_cli_rs::GlobalContext {
        config,
        offline: false,
    };

    loop {
        match <Cmd as interactive_clap::FromCli>::from_cli(
            Some(cli.clone()),
            global_context.clone(),
        ) {
            interactive_clap::ResultFromCli::Ok(cli_cmd)
            | interactive_clap::ResultFromCli::Cancel(Some(cli_cmd)) => {
                println!(
                    "Your console command:\n{} {}",
                    std::env::args().next().as_deref().unwrap_or("./bos"),
                    shell_words::join(cli_cmd.to_cli_args())
                );
                return Ok(());
            }
            interactive_clap::ResultFromCli::Cancel(None) => {
                println!("Goodbye!");
                return Ok(());
            }
            interactive_clap::ResultFromCli::Back => {}
            interactive_clap::ResultFromCli::Err(optional_cli_cmd, err) => {
                if let Some(cli_cmd) = optional_cli_cmd {
                    println!(
                        "Your console command:\n{} {}",
                        std::env::args().next().as_deref().unwrap_or("./bos"),
                        shell_words::join(cli_cmd.to_cli_args())
                    );
                }
                return Err(err);
            }
        }
    }
}
