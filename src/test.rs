//! # 🔬 Test Environment Infrastructure
//!
//! This module provides utilities for bootstrapping and managing an isolated
//! integration test environment powered by Docker containers and an async
//! server runtime.
//!
//! It is designed to:
//!
//! - Provision and manage test containers (e.g., Postgres, Keycloak)
//! - Coordinate initialization and shutdown across threads
//! - Provide global access to running containers
//! - Ensure deterministic teardown after tests complete
//! - Offer structured logging with colored output
//!
//! The module is intended for integration and end-to-end testing scenarios
//! where external dependencies must be provisioned dynamically.
//!
//! ---
//!
//! ## Architecture Overview
//!
//! The test environment follows a controlled lifecycle with three main phases:
//!
//! 1. **Setup Phase**
//!    - Initializes logging
//!    - Starts required containers
//!    - Executes optional post-initialization tasks
//!    - Signals readiness to the test runtime
//!
//! 2. **Execution Phase**
//!    - Tests run against the live server and provisioned services
//!    - Containers remain active and globally accessible
//!
//! 3. **Teardown Phase**
//!    - Receives shutdown signal
//!    - Stops and removes all registered containers
//!    - Releases global resources
//!
//! Synchronization between phases is handled using global channels and locks.
//!
//! ---
//!
//! ## Global Resource Management
//!
//! The module maintains global state using `OnceLock` to guarantee safe,
//! single initialization across threads:
//!
//! - **Docker Client**
//!   A shared connection to the Docker daemon used for container inspection
//!   and removal.
//!
//! - **Container Registry**
//!   A global, thread-safe map storing container names and their IDs. This
//!   enables coordinated teardown after tests complete.
//!
//! - **Lifecycle Channels**
//!   Internal channels synchronize initialization completion and shutdown
//!   signals between threads.
//!
//! ---
//!
//! ## Logging
//!
//! Logging is automatically configured during setup using `env_logger`.
//!
//! Features:
//!
//! - Colored log levels
//! - Timestamped output
//! - Module-aware formatting
//! - Environment-driven log filtering (`RUST_LOG`)
//! - Suppression of noisy framework logs by default
//!
//! This improves readability during test execution and debugging.
//!
//! ---
//!
//! ## Public API
//!
//! ### `setup`
//!
//! Bootstraps the full test environment.
//!
//! Responsibilities:
//!
//! - Initializes logging
//! - Executes user-provided initialization logic
//! - Starts the application server
//! - Runs optional post-initialization tasks
//! - Blocks until environment is ready
//!
//! The initialization closure must return:
//!
//! - A collection of container handles (to prevent premature drop)
//! - Application `Settings` used to start the server
//!
//! ### `teardown`
//!
//! Gracefully shuts down the environment by:
//!
//! - Sending a stop signal to all containers
//! - Removing containers from Docker
//! - Waiting for shutdown confirmation
//!
//! This function should be called once after all tests complete.
//!
//! ### `get_container`
//!
//! Retrieves metadata for a registered container by name using the Docker API.
//!
//! ### `add_container`
//!
//! Registers a container in the global container map so it can be stopped
//! automatically during teardown.
//!
//! ---
//!
//! ## Blocking Execution Helper
//!
//! The module provides an internal utility for executing async code from
//! synchronous contexts by creating a dedicated Tokio runtime. This is used
//! primarily during container shutdown and cleanup.
//!
//! ---
//!
//! ## Container Utilities
//!
//! The `containers` submodule provides helpers for starting commonly used
//! infrastructure services.
//!
//! ### Supported Services
//!
//! - **Postgres**
//!   Starts a database container with optional initialization scripts,
//!   network configuration, and credentials.
//!
//! - **Keycloak**
//!   Starts an identity provider container with realm import support and
//!   readiness checks.
//!
//! Each container:
//!
//! - Waits for readiness before returning
//! - Registers itself for automatic teardown
//! - Returns a connection URI for test usage
//!
//! ---
//!
//! ## Error Handling
//!
//! All operations use the module-specific `TestError` type, which captures:
//!
//! - Container creation failures
//! - Filesystem path resolution errors
//! - Custom runtime errors
//!
//! ---
//!
//! ## Typical Usage Pattern
//!
//! ```rust
//! #[ctor::ctor]
//! pub fn setup() {
//!     rust_microservice::test::setup(
//!         async || {
//!             let mut settings = load_test_settings();
//!
//!             // This vector serves as a workaround for Testcontainers’ automatic cleanup,
//!             // ensuring that containers remain available until all tests have completed.
//!             let mut containers: Vec<Box<dyn Any + Send>> = vec![];
//!
//!             let postgres = start_postgres_container(&mut settings).await;
//!             if let Ok(postgres) = postgres {
//!                 containers.push(Box::new(postgres.0));
//!             }
//!
//!             let keycloak = start_keycloak_container(&mut settings).await;
//!             if let Ok(keycloak) = keycloak {
//!                 containers.push(Box::new(keycloak.0));
//!             }
//!
//!             (containers, settings)
//!         },
//!         || async {
//!             info!("Getting authorization token ...");
//!             let oauth2_token = get_auth_token().await.unwrap_or("".to_string());
//!             TOKEN.set(oauth2_token);
//!             info!("Authorization token: {}...", token()[..50].bright_blue());
//!         },
//!     );
//! }
//! ```
//!
//! Teardown is handled automatically by the `dtor` attribute.
//! ```rust
//! #[ctor::dtor]
//! pub fn teardown() {
//!     rust_microservice::test::teardown();
//! }
//! ```
//!
//! ---
//!
//! ## Concurrency Model
//!
//! The environment runs inside a dedicated multi-thread Tokio runtime
//! spawned in a background thread. This allows synchronous test code to
//! coordinate with async infrastructure without requiring async test
//! functions.
//!
//! Communication is performed via channels that coordinate:
//!
//! - Initialization completion
//! - Container stop commands
//! - Shutdown confirmation
//!
//! ---
//!
//! ## Intended Use Cases
//!
//! This module is suitable for:
//!
//! - Integration testing
//! - End-to-end testing
//! - CI environments requiring ephemeral infrastructure
//! - Local development with disposable dependencies
//!
//! It is not intended for production runtime container management.
//!
//! ---
//!
//! ## Safety Guarantees
//!
//! - Containers remain alive for the full test lifecycle
//! - Teardown is deterministic and blocking
//! - Global state is initialized exactly once
//! - Async resources are properly awaited before shutdown
//!
//! ---
//!
//! ## Notes
//!
//! The environment assumes Docker is available and reachable using default
//! configuration. Failure to connect to the Docker daemon will cause setup
//! to abort.
//!
//! All containers are forcefully removed during teardown to ensure a clean
//! test environment for subsequent runs.
//!
use colored::Colorize;
use env_logger::{Builder, Env};
use std::any::Any;
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};
use std::{io::Write, sync::OnceLock, thread};
use testcontainers::bollard::Docker;
use testcontainers::bollard::query_parameters::{
    InspectContainerOptionsBuilder, RemoveContainerOptionsBuilder,
};
use testcontainers::bollard::secret::ContainerInspectResponse;
use thiserror::Error;
use tokio::sync::Mutex;
use tokio::task;
use tracing::info;

use crate::Server;
use crate::settings::Settings;

pub type Result<T, E = TestError> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum TestError {
    #[error("Failed to get absolute path for mount source : {0}")]
    AbsolutePathConversion(String),

    #[error("Failed to create container : {0}")]
    ContainerCreation(String),

    #[error("{0}")]
    Custom(String),
}

enum ContainerCommands {
    Stop,
}

struct Channel<T> {
    tx: Sender<T>,
    rx: Mutex<Receiver<T>>,
}

/// Creates a new channel for sending and receiving messages of type `T`.
///
/// Returns a `Channel<T>` containing a sender and a receiver for the created channel.
fn channel<T>() -> Channel<T> {
    let (tx, rx) = mpsc::channel();
    Channel {
        tx,
        rx: Mutex::new(rx),
    }
}

/// A static lock used to synchronize access to the container map.
static CONTAINERS: OnceLock<Mutex<HashMap<String, String>>> = std::sync::OnceLock::new();

// Holds a channel used to notify when initialization is complete.
static CONTAINER_NOTIFIER_CHANNEL: OnceLock<Channel<ContainerCommands>> = OnceLock::new();
fn container_notifier_channel() -> &'static Channel<ContainerCommands> {
    CONTAINER_NOTIFIER_CHANNEL.get_or_init(channel)
}

// Holds a channel used to wait for shutdown notification from teardown function.
static SHUTDOWN_NOTIFIER_CHANNEL: OnceLock<Channel<()>> = OnceLock::new();
fn shutdown_notifier_channel() -> &'static Channel<()> {
    SHUTDOWN_NOTIFIER_CHANNEL.get_or_init(channel)
}

// Holds a channel used to notify when initialization is complete.
static INITIALIZE_NOTIFIER_CHANNEL: OnceLock<Channel<()>> = OnceLock::new();
fn initialize_notifier_channel() -> &'static Channel<()> {
    INITIALIZE_NOTIFIER_CHANNEL.get_or_init(channel)
}

// Holds a static lock used to synchronize access to the Docker client.
static DOCKER_CLIENT: OnceLock<Docker> = OnceLock::new();
pub(crate) fn docker_client() -> &'static Docker {
    DOCKER_CLIENT.get_or_init(|| {
        Docker::connect_with_defaults().expect("Failed to connect to Docker daemon.")
    })
}

/// Retrieves a container by its name from the container map.
///
/// This function returns an `Option<ContainerInspectResponse>` containing the
/// requested container, if found. Otherwise, it returns `None`.
pub async fn get_container(name: &str) -> Option<ContainerInspectResponse> {
    let container_id = CONTAINERS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .await
        .get(name)
        .cloned();

    if let Some(container_id) = container_id {
        let options = InspectContainerOptionsBuilder::default().build();
        let res = docker_client()
            .inspect_container(container_id.as_str(), Some(options))
            .await;
        if let Ok(data) = res {
            return Some(data);
        }
    }

    None
}

/// Adds a container to the container map.
///
/// This function takes a `name` and a `container_id` and inserts them into the container map.
///
/// The container map is a global, thread-safe map that stores container names as keys and
/// container IDs as values.
///
/// This function returns no value, but it will block until the insertion is complete.
pub async fn add_container(name: &str, id: String) {
    CONTAINERS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .await
        .insert(name.to_string(), id);
}

/// Sets up the test environment by initializing the log, printing an ASCII art
/// banner, and spawning a new thread that will execute the given closure.
///
/// The closure should return a `Future` that will be executed in a blocking
/// context. The `Future` should complete before the test environment is
/// considered initialized.
///
/// The test environment will wait for the setup signal to be sent before
/// proceeding with the tests. After the setup signal is received, the test
/// environment will wait for the shutdown signal before shutting down.
///
/// The shutdown signal is sent after all containers have been stopped and
/// removed.
pub fn setup<F, P, Fut, PostFut>(init: F, post_init: P)
where
    F: FnOnce() -> Fut + Send + 'static,
    P: FnOnce() -> PostFut + Send + 'static,
    Fut: Future<Output = (Vec<Box<dyn Any + Send>>, Settings)> + Send + 'static,
    PostFut: Future<Output = ()> + Send + 'static,
{
    configure_log();

    let ascii_art = r#"
       ____      __                         __   _               ______          __     
      /  _/___  / /_ ___  ___ _ ____ ___ _ / /_ (_)___   ___    /_  __/___  ___ / /_ ___
     _/ / / _ \/ __// -_)/ _ `// __// _ `// __// // _ \ / _ \    / /  / -_)(_-</ __/(_-<
    /___//_//_/\__/ \__/ \_, //_/   \_,_/ \__//_/ \___//_//_/   /_/   \__//___/\__//___/
                        /___/                                                           
    "#;
    println!("{}", ascii_art);

    info!("Initializing Test Environment ...");

    //let shutdown_tx = shutdown_notifier_channel().tx.clone();
    thread::spawn(move || {
        let body = async move {
            // This vector serves as a workaround for Testcontainers’ automatic cleanup,
            // ensuring that containers remain available until all tests have completed.
            let (mut _containers, settings) = init().await;

            info!("Starting Server ...");
            let result = Server::new_with_settings(settings).await;
            match result {
                Ok(server) => {
                    let result = server.intialize_database().await;
                    if let Ok(server) = result {
                        info!("{}", "Server started successfully!".bright_blue());
                        Server::set_global(server);
                    }
                }
                Err(e) => {
                    panic!("Failed to start server: {}", e);
                }
            }

            info!("Processing Post Initialization Tasks...");
            post_init().await;

            // Send the setup signal to indicate that initialization is complete
            // and the tests can proceed.
            initialize_notifier_channel()
                .tx
                .send(())
                .expect("Failed to send setup signal.");

            // Wait for container commands (e.g., Stop) before shutting down.
            let _ = task::spawn_blocking(move || {
                let rx = container_notifier_channel()
                    .rx
                    .blocking_lock()
                    .recv()
                    .expect("Failed to receive container command notification.");

                match rx {
                    ContainerCommands::Stop => {
                        info!("Shutting Down Test Environment. Stopping Containers...");

                        // Shutdown all containers
                        CONTAINERS
                            .get_or_init(|| Mutex::new(HashMap::new()))
                            .blocking_lock()
                            .iter()
                            .for_each(|(name, container)| {
                                execute_blocking(async || {
                                    info!(
                                        "Stopping Container with name {} and id {}",
                                        name.bright_blue(),
                                        (&container[..13]).bright_blue()
                                    );
                                    let opts = RemoveContainerOptionsBuilder::default()
                                        .force(true)
                                        .v(true)
                                        .build();
                                    let res = docker_client()
                                        .remove_container(container, Some(opts))
                                        .await;
                                    if res.is_err() {
                                        info!("Failed to remove container: {:?}", res);
                                    }
                                });
                            });

                        info!("All containers have been successfully stopped.");
                    }
                }
            })
            .await;

            // This needs to be here otherwise the container did not call the drop
            // function before the application stops.
            shutdown_notifier_channel()
                .tx
                .send(())
                .expect("Failed to send shutdown signal.");

            info!(
                "{}",
                "The test environment has been shut down successfully.".bright_green()
            );
        };

        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Cannot create Tests Tokio Runtime.")
            .block_on(body);
    });

    // Wait for the setup signal before proceeding.
    initialize_notifier_channel()
        .rx
        .blocking_lock()
        .recv()
        .expect("Failed to receive setup signal.");

    info!(
        "{} {}",
        "The test environment has been initialized successfully.",
        "Starting Tests...".bright_green()
    );
}

/// Shuts down the test environment by blocking on the shutdown signal.
///
/// This function is used to wait for the shutdown of the test environment.
/// It blocks on the shutdown signal channel and waits for the signal to be sent.
/// Once the signal is received, it returns, indicating that the test environment
/// has been shut down.
pub fn teardown() {
    // Send the shutdown signal to containers
    let _ = container_notifier_channel()
        .tx
        .send(ContainerCommands::Stop);

    // Wait for the shutdown signal.
    // This ensures that all containers have been properly shut down before app exits.
    let guard = shutdown_notifier_channel().rx.try_lock();
    if guard.is_err() {
        panic!("Failed to receive shutdown signal.");
    }

    if let Ok(rx) = guard {
        let _ = rx.recv();
    }
}

/// Executes a given future in a blocking manner.
///
/// This function creates a new instance of the Tokio runtime and
/// blocks on the given future, waiting for its completion.
///
/// This function is useful when you need to execute a future in a
/// blocking manner, such as in tests or command-line applications.
pub(crate) fn execute_blocking<F, Fut>(future: F)
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = ()>,
{
    let rt = tokio::runtime::Runtime::new().expect("Cannot create Tokio Runtime.");
    rt.block_on(future());
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
fn configure_log() {
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
                "{:<24}  {:<5} [{:<40}] - {}",
                datetime,                                         // Timestamp
                level,                                            // Log level
                record.module_path().unwrap_or("unknown").blue(), // Module path
                record.args()                                     // Log message
            )
        })
        .try_init();
}

pub mod containers {
    use colored::Colorize;
    use std::{fs, path::Path, time::Duration};

    use testcontainers::{
        ContainerAsync, CopyDataSource, GenericImage, ImageExt,
        core::{Mount, WaitFor, ports::IntoContainerPort, wait::HttpWaitStrategy},
        runners::AsyncRunner,
    };
    use testcontainers_modules::postgres::Postgres;
    use tracing::{debug, info};

    use crate::test::Result;
    use crate::test::TestError;

    /// Returns the absolute path of a given path string.
    ///
    /// # Errors
    /// If the given path is not valid UTF-8, a `TestError::AbsolutePathConversion`
    /// error is returned.
    fn absolute_path(path: &str) -> Result<String> {
        let path = Path::new(path)
            .canonicalize()
            .map_err(|e| TestError::AbsolutePathConversion(e.to_string()))?
            .to_str()
            .ok_or_else(|| {
                TestError::AbsolutePathConversion("Path is not valid UTF-8".to_string())
            })?
            .to_string();
        Ok(path)
    }

    /// Starts a Keycloak container with a given realm data path and network.
    ///
    /// # Parameters
    ///
    /// * `realm_data_path`: Optional. The path to the realm data JSON file to be imported.
    /// * `network`: Optional. The network to use for the container. If `None`, the default
    /// * network is used.
    ///
    /// # Returns
    ///
    /// A `Result` containing the URI of the Keycloak instance if successful, or a `TestError`
    /// if an error occurred.
    pub async fn keycloak(
        realm_data_path: &str,
        network: &str,
    ) -> Result<(ContainerAsync<GenericImage>, String)> {
        let realm =
            fs::read(realm_data_path).map_err(|e| TestError::ContainerCreation(e.to_string()))?;

        let container = GenericImage::new("quay.io/keycloak/keycloak", "26.5.2")
            .with_exposed_port(8080.tcp())
            .with_exposed_port(9000.tcp())
            .with_wait_for(WaitFor::http(
                HttpWaitStrategy::new("/health/ready")
                    .with_port(9000.into())
                    .with_expected_status_code(200u16),
            ))
            .with_cmd(vec!["start-dev", "--import-realm"])
            //.with_reuse(ReuseDirective::Always)
            .with_network(network)
            .with_copy_to(
                "/opt/keycloak/data/import/realm-export.json",
                CopyDataSource::Data(realm),
            )
            .with_startup_timeout(Duration::from_secs(60))
            .with_env_var("KC_BOOTSTRAP_ADMIN_USERNAME", "admin")
            .with_env_var("KC_BOOTSTRAP_ADMIN_PASSWORD", "123456")
            .with_env_var("KC_HTTP_ENABLED", "true")
            .with_env_var("KC_HTTP_HOST", "0.0.0.0")
            .with_env_var("KC_HEALTH_ENABLED", "true")
            .with_env_var("KC_CACHE", "local")
            .with_env_var("KC_FEATURES", "scripts")
            .with_env_var("TZ", "America/Sao_Paulo")
            .start()
            .await
            .map_err(|e| {
                info!("Error starting Keycloak container: {}", e.to_string());
                TestError::ContainerCreation(e.to_string())
            })?;

        let container_ip = container
            .get_host()
            .await
            .map_err(|e| TestError::ContainerCreation(e.to_string()))?
            .to_string();

        let container_port = container
            .get_host_port_ipv4(8080)
            .await
            .map_err(|e| TestError::ContainerCreation(e.to_string()))?
            .to_string();

        // IMPORTANT: Add container to global container map for teardown
        super::add_container("keycloak", container.id().to_string()).await;

        let uri = format!("http://{}:{}", container_ip, container_port);
        debug!("Keycloak Connection URL: {}", uri.bright_blue());

        Ok((container, uri))
    }

    /// Creates a Postgres container with a default configuration.
    ///
    /// This function creates a new Postgres container with a default configuration,
    /// and returns the connection URL of the container. The container is added
    /// to the global container map for teardown.
    ///
    /// # Parameters
    ///
    /// - `path`: The path to the Postgres image Docker mount source.
    /// - `database`: The name of the Postgres database.
    /// - `network`: The name of the Docker network to connect the container to.
    ///
    /// # Errors
    ///
    /// If an error occurs while creating the container, a `TestError::ContainerCreation`
    /// error is returned. If an error occurs while getting the IP address or port of
    /// the container, a `TestError::ContainerCreation` error is returned.
    ///
    /// # Returns
    ///
    /// A `String` containing the connection URL of the Postgres container.
    pub async fn postgres(
        path: Option<String>,
        database: Option<String>,
        network: Option<String>,
        user: Option<String>,
        password: Option<String>,
    ) -> Result<(ContainerAsync<Postgres>, String)> {
        info!("Creating Default Postgres Container...");

        let mut entry_point = None;
        if let Some(path) = path {
            let mount_source_str = absolute_path(&path)?;
            entry_point = Some(Mount::bind_mount(
                mount_source_str.as_str(),
                "/docker-entrypoint-initdb.d",
            ));
        }

        let builder = Postgres::default()
            .with_db_name(database.clone().unwrap_or("postgres".into()).as_str());

        let mut builder = builder
            .with_network(network.unwrap_or("test_network".into()))
            //.with_reuse(ReuseDirective::Always)
            .with_startup_timeout(Duration::from_secs(30));

        if let Some(entry_point) = entry_point {
            builder = builder.with_mount(entry_point);
        }

        let container = builder
            .start()
            .await
            .map_err(|e| TestError::ContainerCreation(e.to_string()))?;

        let container_ip = container
            .get_host()
            .await
            .map_err(|e| TestError::ContainerCreation(e.to_string()))?;

        let container_port = container
            .get_host_port_ipv4(5432)
            .await
            .map_err(|e| TestError::ContainerCreation(e.to_string()))?;

        let uri = format!(
            "postgres://{}:{}@{}:{}/{}",
            user.unwrap_or("postgres".into()),
            password.unwrap_or("postgres".into()),
            container_ip,
            container_port,
            database.unwrap_or("postgres".into())
        );
        debug!("Default Postgres Connection URL: {}", uri.bright_blue());

        // IMPORTANT: Add container to global container map for teardown
        super::add_container("postgres", container.id().to_string()).await;

        Ok((container, uri))
    }
}
