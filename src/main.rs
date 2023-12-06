#![allow(
    clippy::enum_variant_names,
    clippy::large_enum_variant,
    clippy::arc_with_non_send_sync
)]
use color_eyre::eyre::WrapErr;
use interactive_clap::ToCliArgs;
pub use near_cli_rs::CliResult;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

pub mod common;
mod components;
pub mod consts;
mod extensions;
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
    /// SocialDb management
    SocialDb(self::social_db::SocialDb),
    #[strum_discriminants(strum(message = "extension    -   Manage bos CLI and extensions"))]
    /// Manage bos CLI and extensions
    Extensions(self::extensions::Extensions),
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

    let bos_exec_path: String = std::env::args().next().unwrap_or("./bos".to_owned());

    let cli_cmd = match <Cmd as interactive_clap::FromCli>::from_cli(
        Some(cli.clone()),
        global_context.clone(),
    ) {
        interactive_clap::ResultFromCli::Ok(cli_cmd)
        | interactive_clap::ResultFromCli::Cancel(Some(cli_cmd)) => {
            eprintln!(
                "Your console command:\n{} {}",
                bos_exec_path,
                shell_words::join(cli_cmd.to_cli_args())
            );
            Ok(Some(cli_cmd))
        }
        interactive_clap::ResultFromCli::Cancel(None) => {
            eprintln!("Goodbye!");
            Ok(None)
        }
        interactive_clap::ResultFromCli::Back => {
            unreachable!("The Command does not have back option")
        }
        interactive_clap::ResultFromCli::Err(optional_cli_cmd, err) => {
            if let Some(cli_cmd) = optional_cli_cmd {
                eprintln!(
                    "Your console command:\n{} {}",
                    bos_exec_path,
                    shell_words::join(cli_cmd.to_cli_args())
                );
            }
            Err(err)
        }
    };

    let handle = std::thread::spawn(|| -> color_eyre::eyre::Result<String> {
        self::extensions::self_update::get_latest_version()
    });

    // We don't need to check the version if user has just called self-update
    if !matches!(
        cli_cmd,
        Ok(Some(CliCmd {
            command: Some(self::CliCommand::Extensions(
                self::extensions::CliExtensions {
                    extensions_actions: Some(self::extensions::CliExtensionsActions::SelfUpdate(
                        self::extensions::self_update::CliSelfUpdateCommand {},
                    )),
                },
            )),
            ..
        }))
    ) {
        if let Ok(Ok(latest_version)) = handle.join() {
            let current_version = semver::Version::parse(self_update::cargo_crate_version!())
                .wrap_err("Failed to parse current version of `bos` CLI")?;

            let latest_version = semver::Version::parse(&latest_version)
                .wrap_err("Failed to parse latest version of `bos` CLI")?;

            if current_version < latest_version {
                eprintln!();
                eprintln!(
                    "`bos` CLI has a new update available \x1b[2m{current_version}\x1b[0m →  \x1b[32m{latest_version}\x1b[0m"
                );
                let self_update_cli_cmd = CliCmd {
                    command: Some(self::CliCommand::Extensions(
                        self::extensions::CliExtensions {
                            extensions_actions: Some(
                                self::extensions::CliExtensionsActions::SelfUpdate(
                                    self::extensions::self_update::CliSelfUpdateCommand {},
                                ),
                            ),
                        },
                    )),
                };
                eprintln!(
                    "To update `bos` CLI use: {}",
                    shell_words::join(
                        std::iter::once(bos_exec_path).chain(self_update_cli_cmd.to_cli_args())
                    )
                );
            }
        }
    };

    cli_cmd.map(|_| ())
}
