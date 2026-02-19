//! # Server Module
//!
//! This module provides the entry point for the server, including configuration
//! loading, environment variable overrides, and the web server bootstrap process.
//!
//! ## Overview
//!
//! The server module is responsible for:
//!
//! - Initializing the logger
//! - Loading the configuration from YAML files and environment variables
//! - Bootstrapping the web server
//!
//! ## Configuration
//!
//! The server module loads its configuration from the following sources, in order of
//! precedence:
//!
//! 1. Environment variables
//! 2. CLI parameters
//! 3. YAML configuration file
//!
//! The configuration is represented by the `Settings` struct, which is
//! serialized and deserialized using `serde`.
//!
//! ## Web Server
//!
//! The web server is bootstrapped using `actix-web` and is responsible for:
//!
//! - Registering routers and microservice endpoints
//! - Managing middlewares and shared application state
//! - Running the asynchronous runtime using `tokio::main` or a custom runtime

//! ## Environment Variables
//!
//! The server module loads environment variables from the system environment and overrides
//! the configuration accordingly.
//!
//! ## CLI Parameters
//!
//! The server module loads CLI parameters from the command line and overrides the configuration
//! accordingly.

use crate::settings::{OAuth2Configuration, Security, Settings};
use crate::{cmd::root::Cli, data::bigquery};
use crate::{data, security};
use actix_web::dev::ServiceRequest;
use actix_web::http::header;
use actix_web::web::ServiceConfig;
use clap::Parser;
use colored::Colorize;
use env_logger::{Builder, Env};
use jsonwebtoken::jwk::JwkSet;
use log::{info, warn};
use reqwest_middleware::ClientBuilder;
use sea_orm::DatabaseConnection;
use std::any::Any;
use std::io::Write;
use std::sync::OnceLock;
use thiserror::Error;
use tracing::{debug, error};

#[cfg(feature = "memory-database")]
use sea_orm::MockDatabase;

#[cfg(feature = "memory-database")]
use std::sync::Arc;

/// Global static instance of the [`Server`].
static SERVER: OnceLock<Box<dyn GlobalServer + Send + Sync>> = OnceLock::new();

/// Represents the high-level server controller responsible for
/// loading configuration, applying custom setup, and running the
/// application.
///
/// This structure encapsulates CLI arguments, server settings,
/// and an optional configuration callback for the Actix-Web service.
#[derive(Clone)]
pub struct Server {
    running: bool,
    args: Option<Cli>,
    settings: Option<Settings>,
    fnconfig: Option<fn(&mut ServiceConfig)>,
    database: Option<data::ServerDatabase>,
}

/// Implementation of the [`GlobalServer`] trait for the [`Server`] struct.
impl Server {
    /// Returns a reference to the globally initialized [`Server`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// let server = Server::global();
    /// ```
    pub fn global() -> Result<&'static (dyn GlobalServer + Send + Sync)> {
        SERVER
            .get()
            .map(|server| server.as_ref())
            .ok_or(ServerError::NotInitialized)
    }

    /// Returns a reference to the globally initialized `Server`, if it exists.
    ///
    /// This function provides access to the global server instance managed
    /// through the `GlobalServer` trait.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let server = Server::global_server();
    /// ```
    pub fn global_server() -> Option<&'static Server> {
        SERVER
            .get()
            .map(|s| s.as_any().downcast_ref::<Server>())
            .unwrap_or_default()
    }

    /// Sets the global [`Server`] instance.
    ///
    /// This function should be called **exactly once**, typically during
    /// application startup.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let server = Server::new();
    /// Server::set_global(server);
    /// ```
    pub fn set_global(server: Server) {
        if SERVER.set(Box::new(server)).is_err() {
            debug!(
                "{}",
                "Server is already initialized. The new instance will be ignored.".yellow()
            );
        }
    }

    /// Checks whether the server has already been initialized.
    /// This function should be called before any server initialization
    /// to prevent multiple initializations.
    ///
    /// # Panics
    ///
    /// Panics if the server is already initialized.  
    /// This method logs an error message before aborting execution.
    ///
    /// # Behavior
    ///
    /// - Verifies if the global `SERVER` instance is set.
    /// - If initialized, logs an error and terminates the program.
    fn check_initialized() {
        if SERVER.get().is_some() {
            error!("{}", "Server is already initialized.".red());
            panic!()
        }
    }

    /// Performs pre-flight initialization tasks such as configuring the server logger
    /// and printing out the ASCII art banner. This function is called once during
    /// server startup, before the server starts accepting requests.
    fn preflight(app_version: String, banner: Option<String>) {
        // Configure server logger
        Server::configure_log().expect("Log is already initialized!");

        //println!("Starting server...");
        let _standard_ascii_art = r#"
         ____               _     ____                                
        |  _ \  _   _  ___ | |_  / ___|   ___  _ __ __   __ ___  _ __ 
        | |_) || | | |/ __|| __| \___ \  / _ \| '__|\ \ / // _ \| '__|
        |  _ < | |_| |\__ \| |_   ___) ||  __/| |    \ V /|  __/| |   
        |_| \_\ \__,_||___/ \__| |____/  \___||_|     \_/  \___||_|   
        "#;

        let ascii_art = r#"
            ___             __     ____                         
           / _ \ __ __ ___ / /_   / __/___  ____ _  __ ___  ____
          / , _// // /(_-</ __/  _\ \ / -_)/ __/| |/ // -_)/ __/
         /_/|_| \_,_//___/\__/  /___/ \__//_/   |___/ \__//_/   
        "#;

        if let Some(banner) = banner
            && !banner.is_empty()
        {
            println!("{}", banner);
        } else {
            println!("{}", ascii_art);
        }

        //println!("{}", _standard_ascii_art);
        println!(
            "\t{} {}\n\t{} {}\n\t{} {}\n",
            "License:".green(),
            env!("CARGO_PKG_LICENSE").bright_blue(),
            "Server Version:".green(),
            env!("CARGO_PKG_VERSION").bright_blue(),
            "Application Version:".green(),
            app_version.bright_blue(),
        );
    }

    /// Creates a new empty `Server` instance with no configuration loaded.
    ///
    /// Useful as the starting point for building and initializing
    /// the server lifecycle.
    pub fn new(app_version: String, banner: Option<String>) -> Self {
        Server::check_initialized();

        Server::preflight(app_version, banner);

        Server {
            running: false,
            args: None,
            settings: None,
            fnconfig: None,
            database: None,
        }
    }

    /// Creates a new `Server` instance with a mock database.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the mock database.
    /// * `database` - The mock database connection.
    ///
    /// # Returns
    ///
    /// A new `Server` instance with the mock database loaded.
    #[cfg(feature = "memory-database")]
    pub fn new_with_mock_database(name: String, database: MockDatabase) -> Self {
        Server::preflight("".into(), None);

        let databases = data::ServerDatabase::new_with_mock_database(name, database);

        Server {
            running: false,
            args: None,
            settings: None,
            fnconfig: None,
            database: Some(databases),
        }
    }

    /// Creates a new `Server` instance with a memory database.
    ///
    /// This method creates a new `Server` instance with a memory database
    /// connection. It is useful for testing purposes only and should not be used
    /// in production code.
    ///
    /// # Parameters
    ///
    /// - `name`: The name of the memory database.
    ///
    /// # Returns
    ///
    /// A new `Server` instance with the memory database loaded.
    ///
    /// # Erros
    ///
    /// An error if the memory database connection cannot be established.
    #[cfg(feature = "memory-database")]
    pub async fn new_with_memory_database(name: String) -> Result<Self> {
        Server::preflight("".into(), None);

        let databases = data::ServerDatabase::new_with_memory_database(name)
            .await
            .map_err(|e| ServerError::Database(e.to_string()))?;

        Ok(Server {
            running: false,
            args: None,
            settings: None,
            fnconfig: None,
            database: Some(databases),
        })
    }

    pub async fn new_with_settings(settings: Settings) -> Result<Self> {
        Server::preflight("".into(), None);

        let result = Server::discover_oauth_security_settings(&settings).await;

        let settings = match result {
            Ok(s) => s,
            Err(e) => {
                info!("Failed to discover OAuth2 security settings: {}", e);
                settings
            }
        };

        let server = Server {
            running: false,
            args: None,
            settings: Some(settings),
            fnconfig: None,
            database: None,
        };

        Ok(server)
    }

    /// Loads CLI arguments and resolves the application configuration.
    ///
    /// This method parses command-line arguments, attempts to load the
    /// server settings, and stores both inside the `Server` instance.
    /// Panics if configuration loading fails.
    ///
    /// # Returns
    /// The updated `Server` instance.
    pub async fn init(mut self) -> Result<Self> {
        Server::check_initialized();

        let args = Cli::parse();
        let settings =
            Cli::load_config(&args).map_err(|e| ServerError::Configuration(e.to_string()))?;

        let result = Server::discover_oauth_security_settings(&settings).await;
        let settings = match result {
            Ok(s) => s,
            Err(e) => {
                info!("Failed to discover OAuth2 security settings: {}", e);
                settings
            }
        };

        self.settings = Some(settings);
        self.args = Some(args);

        Ok(self)
    }

    /// Discovers OAuth2 security settings from the configuration or an
    /// optional discovery URL.
    ///
    /// This function looks for the OAuth2 security settings in the
    /// provided configuration. If the settings are not found, it panics.
    /// If the discovery URL is provided, it attempts to fetch the settings
    /// from the URL. If the fetch fails, it panics. If the fetch succeeds,
    /// it updates the provided configuration with the discovered settings.
    ///
    /// # Parameters
    ///
    /// - `settings`: The configuration to search for OAuth2 security settings.
    ///
    /// # Returns
    ///
    /// A `Result` containing the updated configuration or an error if
    /// configuration loading or discovery fails.
    async fn discover_oauth_security_settings(settings: &Settings) -> Result<Settings> {
        let oauth2 = settings
            .security
            .as_ref()
            .and_then(|s| s.oauth2.as_ref())
            .ok_or(ServerError::Configuration(
                "Oauth2 security settings not found.".to_string(),
            ))?;

        if oauth2.discovery_enabled.unwrap_or(false)
            && let Some(discovery_url) = &oauth2.discovery_url
        {
            let client = ClientBuilder::new(reqwest::Client::new()).build();

            info!(
                "Discovering OAuth2 security settings from {}",
                discovery_url.bright_blue()
            );

            let mut discovery = client
                .get(discovery_url)
                .send()
                .await
                .map_err(|e| {
                    info!("Failed to fetch OAuth2 discovery settings: {}", e);
                    ServerError::Configuration(e.to_string())
                })?
                .json::<OAuth2Configuration>()
                .await
                .map_err(|e| {
                    info!("Failed to parse OAuth2 discovery settings: {}", e);
                    ServerError::Configuration(e.to_string())
                })?;

            discovery.enabled = oauth2.enabled;
            discovery.discovery_url = Some(discovery_url.clone());
            discovery.discovery_enabled = Some(true);

            //info!("Discovered OAuth2 security settings: {:#?}", discovery);

            if let Some(jwks_uri) = discovery.jwks_uri.clone() {
                info!("Fetching JWKs Certs from {}", jwks_uri.bright_blue());

                let jwks = client
                    .get(jwks_uri)
                    .send()
                    .await
                    .map_err(|e| {
                        info!("Failed to fetch JWKs: {}", e);
                        ServerError::Configuration(e.to_string())
                    })?
                    .json::<JwkSet>()
                    .await
                    .map_err(|e| {
                        info!("Failed to parse JWKs: {}", e);
                        ServerError::Configuration(e.to_string())
                    })?;

                //info!("Discovered JWKs: {:#?}", jwks);

                discovery.jwks = Some(jwks);
            }

            let mut settings = settings.clone();
            settings.security = Some(Security {
                oauth2: Some(discovery),
            });

            //info!("Updated OAuth2 security settings: {:#?}", settings);

            return Ok(settings);
        }

        Ok(settings.clone())
    }

    /// Configures and initializes the application logger.
    ///
    /// This method sets up the logger using environment variables, applying a default
    /// log level configuration when none is provided. It defines a custom log format
    /// with colored log levels, timestamps, module paths, and messages to improve
    /// readability during development and debugging.
    ///
    /// # Behavior
    ///
    /// - Uses `RUST_LOG` environment variable when available.
    /// - Defaults to `info` level and suppresses noisy logs from `actix_web`
    ///   and `actix_web_prom`.
    /// - Applies colorized output based on the log level.
    /// - Formats log entries with timestamp, level, module path, and message.
    ///
    /// # Returns
    ///
    /// Returns `Self` to allow method chaining during application configuration.
    fn configure_log() -> Result<()> {
        // Initialize Logger ENV
        let level = Env::default().default_filter_or("info,actix_web=error,actix_web_prom=error");

        let _ = Builder::from_env(level)
            .format(|buf, record| {
                let level = match record.level() {
                    log::Level::Info => record.level().as_str().bright_green(),
                    log::Level::Debug => record.level().as_str().bright_blue(),
                    log::Level::Trace => record.level().as_str().bright_cyan(),
                    log::Level::Warn => record.level().as_str().bright_yellow(),
                    log::Level::Error => record.level().as_str().bright_red(),
                };

                let datetime = chrono::Local::now()
                    .format("%d-%m-%YT%H:%M:%S%.3f%:z")
                    .to_string()
                    .white();

                // Align timestamp, level, and module path
                writeln!(
                    buf,
                    "{:<24}  {:<5} [{:<60}] - {}",
                    datetime,                                         // Timestamp
                    level,                                            // Log level
                    record.module_path().unwrap_or("unknown").blue(), // Module path
                    record.args()                                     // Log message
                )
            })
            .try_init();

        Ok(())

        //env_logger::init_from_env(level);
    }

    /// Applies a custom Actix-Web configuration callback to the server.
    ///
    /// This allows the application to register routes or middlewares
    /// before the server is executed.
    ///
    /// # Parameters
    /// - `fnconfig`: Optional function used to configure `ServiceConfig`.
    ///
    /// # Returns
    /// The updated `Server` instance.
    pub fn configure(mut self, fnconfig: Option<fn(&mut ServiceConfig)>) -> Self {
        Server::check_initialized();
        self.fnconfig = fnconfig;
        self
    }

    /// Initializes the database connections using the previously loaded settings.
    ///
    /// This method creates and initializes all required database connections,
    /// including the BigQuery client, based on the application settings.
    /// It must be called **after** the settings have been loaded.
    ///
    /// # Behavior
    ///
    /// - Validates that the application settings are available.
    /// - Instantiates the `ServerDatabase` using the provided settings.
    /// - Stores the initialized database instance in the application state.
    /// - Retrieves and logs the list of available BigQuery tables.
    ///
    /// # Panics
    ///
    /// This method will panic if:
    /// - The settings have not been loaded before calling this method.
    ///
    /// # Returns
    ///
    /// Returns `Self` with the database field initialized.
    ///
    /// # Example
    ///
    /// ```no_run
    /// let server = Server::new()
    ///     .init()
    ///     .await
    ///     .intialize_database()
    ///     .await;
    /// ```
    pub async fn intialize_database(mut self) -> Result<Self> {
        Server::check_initialized();

        // Initialize the database connections based on loaded settings.
        let settings = self.settings.as_ref().ok_or_else(|| {
            ServerError::InvalidState("Cannot initialize database before calling init()".into())
        })?;

        let database = data::ServerDatabase::new_with_settings(settings)
            .await
            .map_err(|e| ServerError::Database(e.to_string()))?;

        self.database = Some(database);
        Ok(self)
    }

    /// Executes the server using the previously loaded settings and CLI input.
    ///
    /// If both arguments and settings are available, this method delegates
    /// execution to the CLI dispatcher, starting the server workflow.
    pub async fn run(&self) {
        if self.running {
            warn!("The server is already running and cannot be started again.");
            return;
        }

        self.clone().running = true;

        if let (Some(args), Some(settings)) = (&self.args, &self.settings) {
            Cli::init(args, settings, self.fnconfig).await;
        }

        if let Some(database) = &self.database {
            database.close();
        }
    }
}

/// Default implementation for the `Server` structure.
impl Default for Server {
    /// Returns a default instance of the `Server` structure.
    ///
    /// This method is used to provide a default instance of the server
    /// when no configuration is provided.
    ///
    /// # Returns
    ///
    /// A default instance of the `Server` structure.
    fn default() -> Self {
        Server::new("".into(), None)
    }
}

/// Trait that defines access to global server resources and configuration.
///
/// This trait provides read-only access to shared server components,
/// such as application settings and database connections.
pub trait GlobalServer {
    // Returns a reference to the underlying `Any` object.
    fn as_any(&self) -> &dyn Any;

    // Returns a reference to the application settings.
    fn settings(&self) -> &Settings;

    // Returns a reference to the server database configuration, if available.
    #[cfg(feature = "memory-database")]
    fn database_with_name(&self, name: &str) -> Result<Arc<DatabaseConnection>>;

    // Returns a clone to the server database configuration, if available.
    #[cfg(not(feature = "memory-database"))]
    fn database_with_name(&self, name: &str) -> Result<DatabaseConnection>;

    // Returns a reference to the BigQuery client, if available.
    fn bigquery(&self) -> Option<&bigquery::BigQueryClient>;

    // Returns a boolean indicating whether the server is currently running.
    fn is_running(&self) -> bool;

    // Validates the JWT token in the given request and checks whether the
    // associated roles match the provided list.
    fn validate_jwt(
        &self,
        request: &ServiceRequest,
        authorize: String,
    ) -> security::oauth2::Result<()>;
}

/// Trait implementation for the `Server` structure.
impl GlobalServer for Server {
    /// Returns a reference to the underlying `Any` value.
    ///
    /// Allows access to the internal server component by
    /// enabling safe downcasting to a concrete type.
    fn as_any(&self) -> &dyn Any {
        self
    }

    /// Returns a reference to the server database configuration, if available.
    ///
    /// # Returns
    /// - `Some(&ServerDatabase)` if the database is configured.
    /// - `None` if no database configuration is present.
    #[cfg(feature = "memory-database")]
    fn database_with_name(&self, name: &str) -> Result<Arc<DatabaseConnection>> {
        let database = self
            .database
            .clone()
            .ok_or_else(|| ServerError::InvalidState("Database not initialized".into()))?;

        for database in database.databases {
            if database.name == name {
                return Ok(database.connection);
            }
        }

        Err(ServerError::Database("Database not found".into()))
    }

    /// Returns a reference to the server database configuration, if available.
    ///
    /// # Returns
    /// - `Some(&ServerDatabase)` if the database is configured.
    /// - `None` if no database configuration is present.
    #[cfg(not(feature = "memory-database"))]
    fn database_with_name(&self, name: &str) -> Result<DatabaseConnection> {
        let database = self
            .database
            .as_ref()
            .ok_or_else(|| ServerError::InvalidState("Database not initialized".into()))?;

        for database in &database.databases {
            if database.name == name {
                return Ok(database.connection.clone());
            }
        }

        Err(ServerError::Database("Database not found".into()))
    }

    /// Returns a reference to the server settings, if available.
    ///
    /// # Returns
    /// - `Some(&Settings)` if the settings are configured.
    /// - `None` if no settings are present.
    fn settings(&self) -> &Settings {
        self.settings
            .as_ref()
            .expect("Settings must be initialized before calling settings()")
    }

    /// Returns a reference to the BigQuery client, if available.
    ///
    /// # Returns
    /// - `Some(&BigQueryClient)` if the BigQuery client is configured.
    /// - `None` if no BigQuery client configuration is present.
    fn bigquery(&self) -> Option<&bigquery::BigQueryClient> {
        self.database.as_ref().and_then(|db| db.bigquery.as_ref())
    }

    /// Returns a boolean indicating whether the server is currently running.
    ///
    /// # Returns
    /// - `true` if the server is running.
    /// - `false` if the server is not running.
    fn is_running(&self) -> bool {
        self.running
    }

    /// Validates the JWT token in the given request and checks whether the
    /// associated roles match the provided list.
    ///
    /// # Parameters
    /// - `request`: The request containing the JWT token to validate.
    /// - `roles`: A list of roles to check against the JWT token.
    ///
    /// # Returns
    /// - `Ok(())` if the JWT token is valid and the roles match.
    /// - `Err(securityity::oauth2::OAuth2Error)` if the JWT token is invalid or the roles do not match.
    fn validate_jwt(
        &self,
        request: &ServiceRequest,
        authorize: String,
    ) -> security::oauth2::Result<()> {
        let token: &str = request
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| {
                security::oauth2::OAuth2Error::InvalidJwt("Invalid JWT Header in request.".into())
            })?
            .trim_start_matches("Bearer ");

        // Retrieves the server settings required to proceed with the security configuration
        let settings = self.settings.as_ref().ok_or_else(|| {
            warn!("Settings not configured.");
            security::oauth2::OAuth2Error::Configuration("Settings not configured.".into())
        })?;

        // Validate JWT
        security::oauth2::validate_jwt(token, settings, authorize)?;

        Ok(())
    }
}

/// A type alias for a `Result` with the `ServerError` error type.
pub type Result<T, E = ServerError> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Invalid server state: {0}")]
    InvalidState(String),

    #[error("Invalid server configuration: {0}")]
    Configuration(String),

    #[error("Tokio runtime not found. Details: {0}")]
    RuntimeNotFound(String),

    #[error("Server database error: {0}")]
    Database(String),

    #[error("Server is not initialized.")]
    NotInitialized,

    #[error("Server is already initialized. The new instance will be ignored.")]
    AlreadyInitialized,
}
