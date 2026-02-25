//! # HTTP Web Module
//!
//! This module is responsible for initializing and starting the main API server
//! and the health-check server.
//!
//! It provides a single entry point, `initialize_servers`, which bootstraps the
//! Actix-Web infrastructure of the application. It configures the primary API
//! server, the lightweight health-check server, and optionally manages
//! a Docker Compose environment when enabled in the configuration.
//!
//! ## Main Function
//!
//! `bootstrap_server`: Initializes and starts the main API server and the
//! health-check server.
//!
//! **Arguments:**
//!
//! `settings`: Reference to the application's runtime configuration, including
//! server settings such as host, ports, worker count, and optional Docker
//! Compose usage.
//!
//! `fnconfig`: Optional callback function used to configure the main Actix-Web
//! `ServiceConfig`, where routes and middleware are attached.

use crate::http::health::{HealthApiDoc, configure_server_base};
use crate::metrics::SysInfoCollector;
use crate::settings::Settings;
use actix_cors::Cors;
use actix_web::middleware::Condition;
use actix_web::web::ServiceConfig;
use actix_web::{App, HttpServer, middleware::Logger};
use actix_web_prom::{PrometheusMetrics, PrometheusMetricsBuilder};
use colored::Colorize;
use compose_rs::{Compose, ComposeCommand};
use prometheus::Registry;
use prometheus::process_collector::ProcessCollector;
use thiserror::Error;
use utoipa::OpenApi;

use tokio::join;
use utoipa_swagger_ui::{Config, SwaggerUi};

/// Initializes and starts the main API server and the health-check server.
///
/// This function is responsible for bootstrapping the Actix-Web
/// infrastructure of the application. It configures the primary API
/// server, the lightweight health-check server, and optionally manages
/// a Docker Compose environment when enabled in the configuration.
///
/// # Parameters
/// - `settings`: Reference to the application's runtime configuration,
///   including server settings such as host, ports, worker count, and
///   optional Docker Compose usage.
/// - `fnconfig`: Optional callback function used to configure the main
///   Actix-Web [`ServiceConfig`], where routes and middleware are attached.
///
/// # Returns
/// Returns an `io::Result` containing:
/// - `Ok(Some(Compose))` if Docker Compose was enabled and successfully
///   started.
/// - `Ok(None)` if Docker Compose was not used.
/// - `Err(e)` if the server setup or binding fails.
///
/// # Behavior
/// - If `use_docker_compose` is enabled in the settings:
///   - Executes `docker-compose up` and returns a `Compose` handle for
///     later shutdown.
/// - Launches two Actix-Web servers:
///   1. **Main API Server**
///      - Uses the host and port defined in the settings.
///      - Applies CORS configuration.
///      - Invokes the user-supplied `fnconfig` callback to register routes.
///   2. **Health Check Server**
///      - Runs independently with a dedicated port.
///      - Registers only the base server configuration (e.g., `/health`).
///
/// - Applies worker thread settings if provided.
/// - Prints the listening ports to the console.
/// - Awaits both the main server and the health server concurrently.
///
/// # Notes
/// - This function blocks the current thread until both servers shut down.
/// - If Docker Compose is active, the returned `Compose` instance must be
///   used by the caller to stop the services during shutdown.
pub(crate) async fn bootstrap_server(
    settings: &Settings,
    fnconfig: Option<fn(&mut ServiceConfig)>,
) -> Result<Option<Compose>> {
    let server_config = settings
        .server
        .as_ref()
        .ok_or_else(|| HttpServerError::Configuration("Missing server configuration.".into()))?;

    // Get the thread count to configure server workers
    let num_threads = std::thread::available_parallelism().map_or_else(|_| 1, |p| p.get());

    // Verify docker compose startup
    let compose = if server_config.use_docker_compose.unwrap_or(false) {
        Some(run_docker_compose(settings).map_err(|_| HttpServerError::Compose)?)
    } else {
        None
    };

    // Check the server host
    let host = server_config
        .host
        .clone()
        .ok_or_else(|| HttpServerError::Configuration("Missing server host.".into()))?;

    // Configure Prometheus Metrics
    let (health_metrics_enabled, prometheus_health) = configure_prometheus(settings, true)?;
    let (metrics_enabled, prometheus) = configure_prometheus(settings, false)?;

    // Start the servers
    //rt::System::new().block_on(async {
    // Configure the Main Server
    //
    let server_settings = settings.clone();
    let mut main_server_builder = HttpServer::new(move || {
        // Configure Main Server Cors Config
        let cors_config = configure_cors(&server_settings);

        // Create the Prometheus condition based on settings
        let metrics_condition = Condition::new(metrics_enabled, prometheus.clone());

        App::new()
            .wrap(cors_config)
            .wrap(metrics_condition)
            .wrap(Logger::default())
            .configure(fnconfig.unwrap_or(|_| {}))
    })
    .bind((host.clone(), server_config.port))
    .map_err(|e| HttpServerError::Bootstrap(e.to_string()))?
    .workers(server_config.workers.unwrap_or(num_threads))
    .shutdown_timeout(60);

    // Configure the Health Server
    //
    let mut health_server_builder = HttpServer::new(move || {
        // Configure OpenApi Doc
        let health_openapi = HealthApiDoc::openapi();

        // Create the Prometheus condition based on settings
        let metrics_condition = Condition::new(health_metrics_enabled, prometheus_health.clone());

        // Create the Health Check and Metrics Server App
        App::new()
            .wrap(metrics_condition)
            .configure(configure_server_base)
            .service(
                SwaggerUi::new("/actuator/swagger-ui/{_:.*}")
                    .url("/actuator/api-docs/openapi.json", health_openapi)
                    .config(Config::default().validator_url("none")),
            )
    })
    .bind((host, server_config.health_check_port))
    .map_err(|e| HttpServerError::Bootstrap(e.to_string()))?
    .workers(server_config.health_check_workers.unwrap_or(num_threads))
    .shutdown_timeout(60);

    // Configure server workers if provided
    if let Some(workers) = settings.server.as_ref().and_then(|s| s.workers) {
        main_server_builder = main_server_builder.workers(workers);
        health_server_builder = health_server_builder.workers(workers);
    }
    let main_server = main_server_builder.run();
    let health_server = health_server_builder.run();

    tracing::info!(
        "{} {}. {} {}.",
        "Server listening on port".bright_green(),
        server_config.port.to_string().bright_blue(),
        "The Health Check port is".bright_green(),
        server_config.health_check_port.to_string().bright_blue()
    );

    let (_, _) = join!(health_server, main_server);

    Ok(compose)
    //})
}

/// Configures a Prometheus metrics collector based on the provided settings.
///
/// This function takes the application settings as input and returns a tuple
/// containing a boolean indicating whether metrics collection is enabled
/// and a `PrometheusMetrics` instance configured with the application
/// name and endpoint.
///
/// The `PrometheusMetrics` instance is configured with the following settings:
/// - The application name is used as the prefix for all exposed metrics.
/// - The `"/metrics"` endpoint is used to expose the metrics.
/// - The `^/swagger-ui/.*` regex is used to exclude the Swagger UI endpoint from
///   metrics collection.
///
/// # Errors
///
/// This function will return an error if either the ProcessCollector or
/// SysInfoCollector fails to initialize.
fn configure_prometheus(settings: &Settings, base: bool) -> Result<(bool, PrometheusMetrics)> {
    // Get metrics parameters
    let metrics_cfg = settings.metrics.as_ref();
    let metrics_enabled = metrics_cfg.and_then(|m| m.enabled).unwrap_or(false);
    let metrics_app_name = metrics_cfg
        .and_then(|m| m.app_name.clone())
        .unwrap_or_else(|| "api".to_string());

    // Metrics registry
    let registry = build_metrics_registry(&metrics_app_name)?;

    let endpoint = if base {
        "/actuator/metrics"
    } else {
        "/metrics"
    };
    let prometheus = PrometheusMetricsBuilder::new(&metrics_app_name)
        .endpoint(endpoint)
        .exclude_regex("^/swagger-ui/.*")
        .exclude_regex("^/actuator/swagger-ui/.*")
        .registry(registry)
        .build()
        .map_err(|e| HttpServerError::Bootstrap(e.to_string()))?;

    Ok((metrics_enabled, prometheus))
}

/// Builds a Prometheus registry with the given application name.
///
/// The registry is initialized with both the ProcessCollector and
/// SysInfoCollector. The ProcessCollector is used to expose process
/// metrics, such as memory and CPU usage. The SysInfoCollector is used
/// to expose system metrics, such as CPU count, memory usage, and
/// network connections.
///
/// # Errors
///
/// This function will return an error if either the ProcessCollector or
/// SysInfoCollector cannot be registered with the registry.
fn build_metrics_registry(app_name: &str) -> Result<Registry> {
    let pid = std::process::id() as i32;
    let registry = Registry::default();

    registry
        .register(Box::new(ProcessCollector::new(pid, app_name.to_string())))
        .map_err(|e| HttpServerError::Configuration(e.to_string()))?;

    let collector = SysInfoCollector::with_process_and_namespace(pid, app_name.to_string())
        .map_err(|e| HttpServerError::Configuration(e.to_string()))?;

    registry
        .register(Box::new(collector))
        .map_err(|e| HttpServerError::Configuration(e.to_string()))?;

    Ok(registry)
}

/// Starts the Docker Compose environment defined in the server settings.
///
/// This function loads the `docker-compose.yml` path from the application
/// configuration and attempts to start all declared services. After startup,
/// it prints the status of each container and returns a [`Compose`] handle
/// used for later shutdown.
///
/// # Parameters
/// - `settings`: Reference to the application configuration containing
///   Docker Compose settings.
///
/// # Returns
/// - `Ok(Compose)` if the Compose services start successfully.
/// - `Err(())` if a failure occurs during startup.
///
/// # Notes
/// - Panics if the Compose file is not found or if service startup fails.
/// - Should be used only when `use_docker_compose` is enabled.
fn run_docker_compose(settings: &Settings) -> Result<Compose> {
    let server_config = settings
        .server
        .as_ref()
        .ok_or_else(|| HttpServerError::Configuration("Server configuration is missing".into()))?;

    let compose_file = server_config.docker_compose_file.as_ref().ok_or_else(|| {
        HttpServerError::Configuration("Docker Compose file not configured".into())
    })?;

    let compose = Compose::builder()
        .path(compose_file)
        .build()
        .map_err(|e| HttpServerError::Custom(format!("Failed to build Docker Compose: {e}")))?;

    tracing::info!(
        "{} {}. {}",
        "Starting the docker compose from".to_string().green(),
        compose_file.to_string().bright_blue(),
        "Please wait...".to_string().green(),
    );

    if let Err(error) = compose.up().exec() {
        tracing::error!("Error starting Docker Compose: {error}");

        // Best-effort cleanup
        if let Err(down_error) = compose.down().exec() {
            tracing::warn!("Error while stopping Docker Compose after failure: {down_error}");
        }

        return Err(HttpServerError::Custom(format!(
            "Docker Compose startup failed: {error}"
        )));
    }

    log_compose_status(&compose);

    tracing::info!(
        "{}",
        "The Docker Compose containers are running! Starting the server...".green()
    );

    Ok(compose)
}

/// Logs the status of all containers in the given Docker Compose environment.
///
/// This function retrieves the status of all containers in the Compose environment
/// and logs their name, status, since when they were started, and exit code
/// if applicable. If the retrieval of the container status fails, it logs a
/// warning message with the error details.
///
/// # Parameters
/// - `compose`: Reference to the Docker Compose environment to retrieve
///   the container status from.
///
/// # Behavior
/// - Retrieves the status of all containers in the given Compose environment.
/// - Logs the name, status, since when they were started, and exit code of each
///   container if applicable.
/// - Logs a warning message with the error details if the retrieval of the
///   container status fails.
fn log_compose_status(compose: &Compose) {
    match compose.ps().exec() {
        Ok(services) => {
            tracing::info!("{}", "Containers:".bright_green());

            for service in services {
                let status = format!("{:?}", service.status.status);

                tracing::info!(
                    "  {} {:<25} {} {:?}, {} {}{}",
                    "Name:".white(),
                    service.name.bright_blue(),
                    "Status:".white().dimmed(),
                    service.status.status,
                    "Since:".white().dimmed(),
                    service.status.since.bright_blue().dimmed(),
                    service
                        .status
                        .exit_code
                        .filter(|_| status == "Exited")
                        .map(|code| {
                            format!(
                                "{} {}",
                                ", Exit Code:".bright_blue().dimmed(),
                                code.to_string().bright_blue().dimmed()
                            )
                        })
                        .unwrap_or_default()
                );
            }
        }
        Err(error) => {
            tracing::warn!("Failed to retrieve Docker Compose status: {error}");
        }
    }
}

/// Builds and returns a CORS configuration based on the server settings.
///
/// This function reads the CORS options defined in the application
/// configuration and applies rules for allowed origins and headers.
/// When no CORS configuration is provided, a permissive default policy
/// is applied.
///
/// # Parameters
/// - `settings`: Reference to the application settings used to load
///   CORS rules.
///
/// # Returns
/// A configured [`Cors`] instance ready to be applied to an Actix-Web
/// application.
fn configure_cors(settings: &Settings) -> Cors {
    if let Some(cors_config) = settings.server.as_ref().and_then(|sc| sc.cors.as_ref()) {
        let mut cors = Cors::default();

        // Configure CORS origins
        if let Some(pattern) = &cors_config.allowed_origins_pattern {
            let origins = pattern.split(',').collect::<Vec<&str>>();
            if origins.len() == 1 && origins[0].trim() == "*" {
                cors = cors.allow_any_origin();
            } else {
                for origin in origins {
                    cors = cors.allowed_origin(origin.trim());
                }
            }
        };

        // Configure CORS Allowed Headers
        if let Some(allowed_headers) = &cors_config.allowed_headers {
            let headers = allowed_headers.split(',').collect::<Vec<&str>>();
            if headers.len() == 1 && headers[0].trim() == "*" {
                cors = cors.allow_any_header()
            } else {
                for header in headers {
                    cors = cors.allowed_header(header.trim());
                }
            }
        }

        cors
    } else {
        Cors::permissive()
    }
}

/// Represents the middleware wrappers applied to the HTTP server.
///
/// This structure groups cross-cutting concerns that are attached to the
/// request/response pipeline, such as metrics collection and CORS handling.
///
/// # Fields
///
/// - `metrics_enabled`:
///   Indicates whether Prometheus metrics are exposed and collected.
///
/// - `prometheus`:
///   Configuration and instance responsible for exporting Prometheus metrics.
///
/// - `cors`:
///   Cross-Origin Resource Sharing (CORS) configuration applied to incoming requests.
///
/// # Usage
///
/// This struct is typically initialized during server bootstrap and injected
/// into the HTTP server builder to register middleware components.
///
/// The `rust_microservice::create_server_wrappers` function can be used to create an instance of
/// this struct.
///
/// # Example
///
/// ```rust
/// let wrappers = ServerWrappers {
///     metrics_enabled: true,
///     prometheus: prometheus_metrics,
///     cors: cors_config,
/// };
/// ```
pub struct ServerWrappers {
    pub metrics_enabled: bool,
    pub prometheus: PrometheusMetrics,
    pub cors: Cors,
}

/// Creates a `ServerWrappers` instance from the given `Settings`.
///
/// This function initializes the server wrappers, including:
///
/// - Prometheus metrics
/// - CORS configuration
///
/// # Parameters
///
/// - `settings`: Reference to the application's runtime configuration.
///
/// # Returns
///
/// Returns a `Result` containing a `ServerWrappers` instance if successful, or an error if configuration fails.
pub fn create_server_wrappers(settings: &Settings) -> Result<ServerWrappers> {
    let (metrics_enabled, prometheus) = configure_prometheus(settings, false)?;
    let cors = configure_cors(settings);

    Ok(ServerWrappers {
        metrics_enabled,
        prometheus,
        cors,
    })
}

/// A type alias for a `Result` with the `HttpServerError` error type.
pub type Result<T, E = HttpServerError> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum HttpServerError {
    #[error("Invalid HTTP server configuration: {0}")]
    Configuration(String),

    #[error("{0}")]
    Custom(String),

    #[error("Error on Docker Compose.")]
    Compose,

    #[error("Error initializing the HTTP server: {0}")]
    Bootstrap(String),
}
