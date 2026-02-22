//! Root command provides the command-line interface for this application.
//! It defines all supported flags, arguments, and subcommands.
//!
//! # Command-Line Parameters
//!
//! The application exposes several optional and required command-line
//! parameters. These parameters are parsed at runtime and control the
//! behavior of the tool.
//!
//! ## Global Flags
//!
//! `--config <path>`
//! Specifies the path to the configuration file. If omitted, the
//! application attempts to load a default configuration from common
//! lookup locations.
//!
//! `--log-level <level>`
//! Defines the logging verbosity. Valid levels include `"debug"`,
//! `"info"`, `"warn"`, and `"error"`. Defaults to `"info"`.
//!
//! ## Additional Subcommands
//!
//! Other subcommands may be declared in separate files under the
//! `cmd/` package. Each subcommand may define its own flags and
//! execution logic. Refer to the corresponding file for detailed
//! documentation.
//!
//! ## Notes
//!
//! - All commands support the standard `--help` flag, which displays
//!   contextual help.
//! - Flags must be placed after the subcommand unless they are global
//!   flags available to the root command.
use crate::cmd::style;
use crate::cmd::style::CURRENT;
use crate::settings::Settings;
use crate::{
    Asset,
    cmd::run::{RunArgs, process_command},
};
use actix_web::web::ServiceConfig;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use clap::Parser;
use clap::Subcommand;
use colored::Colorize;
use config::{Case, Config, ConfigError, Environment, File, FileFormat};
use std::path::PathBuf;

/// SERVER CLI Clap root command.
#[derive(Parser, Debug, Clone)]
#[command(author, version, about)]
#[command(propagate_version = true)]
#[command(styles=style::CURRENT)]
#[command(help_template = Cli::help_template())]
pub(crate) struct Cli {
    /// Full path to the config.yaml file. This file will be merged with the default configuration.
    #[clap(short, long, env)]
    pub config_file: Option<PathBuf>,

    /// Yaml config file in Base64 format. This file will be merged with the default configuration
    #[clap(short, long, env)]
    pub b64_config_file: Option<String>,

    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// Starts the server and loads all necessary configurations before
    /// accepting incoming requests.
    Run(RunArgs),
}

impl Cli {
    /// Generates the formatted help template used by the CLI.
    ///
    /// This function returns a string that defines the structure of the help
    /// output, including version, author, usage, options, and available
    /// subcommands. It embeds dynamic header and context values for consistent
    /// styling across the CLI.
    ///
    /// # Returns
    /// A formatted `String` representing the CLI help template.
    fn help_template() -> String {
        format!(
            "\
        {header}SERVER version {{version}}\n\
        {header}Author:{context} {{author}}\n\n\
        {{about}}\n\n\
        {header}Usage: {{usage}}\n\n\
        {header}Options:\n{{options}}\n\n\
        {header}Commands:\n{{subcommands}}
        ",
            header = CURRENT.get_header(),
            context = CURRENT.get_context_value()
        )
    }

    /// Initializes the application based on the parsed CLI arguments.
    ///
    /// This function evaluates the command provided by the user and triggers
    /// the appropriate action. Currently, it supports the `run` command,
    /// which starts the server workflow.
    ///
    /// # Parameters
    /// - `args`: Parsed CLI arguments.
    /// - `settings`: Loaded application settings.
    /// - `fnconfig`: Optional callback to configure the main Actix-Web service.
    pub(crate) async fn init(
        args: &Cli,
        settings: &Settings,
        fnconfig: Option<fn(&mut ServiceConfig)>,
    ) {
        match &args.commands {
            Commands::Run(_) => process_command(settings, fnconfig).await,
        }
    }

    /// Loads the application configuration from multiple possible sources.
    ///
    /// This function assembles a configuration object using the following
    /// sources, in order:
    /// - Embedded `config.yaml`
    /// - CLI-provided config file
    /// - Base64-encoded config passed via CLI
    /// - Environment variables (UPPER_SNAKE_CASE)
    ///
    /// After merging all sources, it attempts to deserialize the resulting
    /// data into a [`Settings`] structure.
    ///
    /// # Parameters
    /// - `args`: CLI arguments containing optional config inputs.
    ///
    /// # Returns
    /// - `Ok(Settings)` on successful deserialization.
    /// - `Err(ConfigError)` containing the formatted error message otherwise.
    pub(crate) fn load_config(args: &Cli) -> std::result::Result<Settings, ConfigError> {
        let mut builder = Config::builder();

        // Get the embeded Yaml config file
        if let Some(file) = Asset::get("config.yaml") {
            let contents =
                std::str::from_utf8(&file.data).map_err(|e| ConfigError::Message(e.to_string()))?;

            builder = builder.add_source(File::from_str(contents, FileFormat::Yaml));
        }

        // Get the yaml config file from CLI parameter address
        if let Some(path) = &args.config_file
            && path.exists()
        {
            builder = builder.add_source(File::from(path.as_ref()));
        }

        // Get the Base64 Yaml config file
        if let Some(data) = &args.b64_config_file {
            let decoded = BASE64_STANDARD
                .decode(data)
                .map_err(|e| ConfigError::Message(e.to_string()))?;

            let contents =
                std::str::from_utf8(&decoded).map_err(|e| ConfigError::Message(e.to_string()))?;

            builder = builder.add_source(File::from_str(contents, FileFormat::Yaml));
        }

        // Get config environment variables
        builder =
            builder.add_source(Environment::with_convert_case(Case::UpperSnake).separator("_"));

        // Build configuration and convert the server configuration
        let config = builder.build().map_err(build_error)?;
        config.try_deserialize::<Settings>().map_err(build_error)
    }
}

/// Builds a `ConfigError` message based on the given error.
///
/// This function is a utility used by the `load_config` method to format
/// configuration-related errors. It wraps the provided error in a
/// `ConfigError::Message` instance, prefixing it with a red "Failed to
/// build configuration. Root Cause:" message.
///
/// # Parameters
/// - `error`: The error to be formatted into a `ConfigError`.
///
/// # Returns
/// - `ConfigError` containing the formatted error message.
fn build_error<E: std::fmt::Display>(error: E) -> ConfigError {
    ConfigError::Message(format!(
        "{} {}",
        "Failed to build configuration. Root Cause:".red(),
        error.to_string().red()
    ))
}
