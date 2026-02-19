//! Run subcommand starts the server with a configuration based on
//! parameters, environment variables, or default config values.
//!
//! ## Run Subcommand
//!
//! Starts the server and loads all necessary configurations before
//! accepting incoming requests. This command initializes the application
//! runtime, sets up the API routes, and begins listening on the configured
//! host and port. It is used in both development and production
//! environments to bring the service online.
//!
//! ### Usage
//!
//! ```text
//! <binary> run
//! ```

use crate::{http::web::bootstrap_server, settings::Settings};
use actix_web::web::ServiceConfig;
use clap::Args;
use colored::Colorize;
use compose_rs::ComposeCommand;
use thiserror::Error;

/// Command-line arguments for starting the application.
///
/// This struct is derived from `clap::Args` and represents
/// the basic CLI parameters accepted by the `run` command.
#[derive(Args, Debug, Clone)]
pub struct RunArgs {}

/// Processes a server shutdown command using the provided settings.
///
/// This function triggers the server bootstrap routine and, if a Docker
/// Compose is detected in the configuration, starts the docker compose.
/// At the server shutdown, attempts to gracefully stop its containers.
/// After the shutdown process, a success or failure message is printed.
///
/// # Parameters
/// - `settings`: Reference to the server configuration.
/// - `fnconfig`: Optional callback used to customize the Actix-Web
///   `ServiceConfig` during initialization.
///
/// # Behavior
/// - Bootstrap the server using the settings configuration;
/// - At the server shutdown process:
///   - Stops Docker Compose services when available.
///   - Prints a confirmation message when the server is shut down.
/// - If an error occurs:
///   - Prints a failure message.
pub async fn process_command(settings: &Settings, fnconfig: Option<fn(&mut ServiceConfig)>) {
    if let Err(error) = try_process_command(settings, fnconfig).await {
        tracing::error!(
            "{} {}",
            "An unexpected error occurred on the server.".bright_red(),
            format!("Detail: {error:?}").red()
        );
    }
}

/// Attempts to process a shutdown command for the server.
///
/// This function initializes the server bootstrap using the provided
/// `settings` and optional service configuration function. If a Docker
/// Compose environment is detected, it gracefully stops all running
/// containers associated with the server.
///
/// Any errors encountered while stopping Docker Compose services are
/// logged as warnings and do not prevent the shutdown process from
/// completing.
///
/// # Parameters
/// - `settings`: Reference to the application [`Settings`] used during bootstrap.
/// - `fnconfig`: Optional function used to customize the [`ServiceConfig`].
///
/// # Returns
/// - `Ok(())` if the shutdown process completes successfully.
/// - An [`std::io::Error`] if the server bootstrap fails.
async fn try_process_command(
    settings: &Settings,
    fnconfig: Option<fn(&mut ServiceConfig)>,
) -> Result<()> {
    if let Some(compose) = bootstrap_server(settings, fnconfig)
        .await
        .map_err(|e| RunError::RunError(e.to_string()))?
    {
        tracing::info!(
            "{}",
            "Stopping Docker Compose Containers. Please wait...".bright_green()
        );

        if let Err(error) = compose.down().exec() {
            tracing::warn!(
                "It was not possible to stop the Docker Compose services: {}",
                error
            );
        }
    }

    tracing::info!("{}", "Server successfully shut down.".bright_green().bold());

    Ok(())
}

/// A type alias for a `Result` with the `RunError` error type.
pub type Result<T, E = RunError> = std::result::Result<T, E>;

/// Represents an error that occurred during the server bootstrap process.
///
/// This enum groups all errors that may occur when starting the server
/// into a single type, making error handling more consistent across
/// the application. Each variant represents a specific failure scenario
/// and provides a human-readable error message.
///
/// # Variants
/// - `RunError`: An error that occurred during the server bootstrap process.
/// - `Custom`: A custom error message that can be used to wrap other types of errors.
#[derive(Debug, Error)]
pub enum RunError {
    #[error("Error initializing the HTTP server. Detail: {0}")]
    RunError(String),

    #[error("{0}")]
    _Custom(String),
}
