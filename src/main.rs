#![allow(
    clippy::enum_variant_names,
    clippy::large_enum_variant,
    clippy::arc_with_non_send_sync
)]
use color_eyre::{eyre::WrapErr, owo_colors::OwoColorize};
use interactive_clap::ToCliArgs;
use near_cli_rs::config::Config;
pub use near_cli_rs::CliResult;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

use indicatif::ProgressStyle;
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

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
    /// TEACH-ME mode
    #[interactive_clap(long)]
    teach_me: bool,
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
    let config = Config::get_config_toml()?;

    #[cfg(not(debug_assertions))]
    let display_env_section = false;
    #[cfg(debug_assertions)]
    let display_env_section = true;
    color_eyre::config::HookBuilder::default()
        .display_env_section(display_env_section)
        .install()?;

    let cli = match Cmd::try_parse() {
        Ok(cli) => cli,
        Err(error) => error.exit(),
    };

    if cli.teach_me {
        let env_filter = EnvFilter::from_default_env()
            .add_directive(tracing::Level::WARN.into())
            .add_directive("near_teach_me=info".parse()?)
            .add_directive("near_cli_rs=info".parse()?)
            .add_directive("bos=info".parse()?);
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .without_time()
                    .with_target(false),
            )
            .with(env_filter)
            .init();
    } else {
        let indicatif_layer = IndicatifLayer::new()
            .with_progress_style(
                ProgressStyle::with_template(
                    "{spinner:.blue}{span_child_prefix} {span_name} {msg} {span_fields}",
                )
                .unwrap()
                .tick_strings(&[
                    "▹▹▹▹▹",
                    "▸▹▹▹▹",
                    "▹▸▹▹▹",
                    "▹▹▸▹▹",
                    "▹▹▹▸▹",
                    "▹▹▹▹▸",
                    "▪▪▪▪▪",
                ]),
            )
            .with_span_child_prefix_symbol("↳ ");
        let env_filter = EnvFilter::from_default_env()
            .add_directive(tracing::Level::WARN.into())
            .add_directive("near_cli_rs=info".parse()?)
            .add_directive("bos=info".parse()?);
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .without_time()
                    .with_writer(indicatif_layer.get_stderr_writer()),
            )
            .with(indicatif_layer)
            .with(env_filter)
            .init();
    };

    let global_context = near_cli_rs::GlobalContext {
        config,
        offline: false,
        teach_me: false,
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
                bos_exec_path.yellow(),
                shell_words::join(cli_cmd.to_cli_args()).yellow()
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
                    bos_exec_path.yellow(),
                    shell_words::join(cli_cmd.to_cli_args()).yellow()
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
                    teach_me: false,
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
                    .yellow()
                );
            }
        }
    };

    cli_cmd.map(|_| ())
}
