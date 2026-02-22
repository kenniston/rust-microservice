//! The `Settings` module loads application configuration using the
//! `config` crate, supporting YAML, TOML, and JSON files. It provides
//! a strongly typed structure for accessing configuration values
//! throughout the application.
use config::{Case, Config, ConfigError, Environment, File, FileFormat};
use jsonwebtoken::jwk::{Jwk, JwkSet};
#[allow(unused)]
use log::LevelFilter;
use serde::Deserialize;

/// Configuration for enabling or disabling data repositories.
///
/// This structure is usually used to control whether repository
/// layers backed by Redis or other data sources should be enabled.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Repositories {
    /// Enables or disables repositories.
    pub enabled: Option<bool>,
}

/// Connection pool configuration.
///
/// Defines limits and behavior for resource pooling,
/// such as database or Redis connections.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Pool {
    /// Enables or disables the connection pool.
    pub enabled: Option<bool>,

    /// Minimum number of idle connections kept in the pool.
    pub min_idle: Option<usize>,

    /// Maximum number of idle connections allowed.
    pub max_idle: Option<usize>,

    /// Maximum number of active connections allowed.
    pub max_active: Option<usize>,
}

/// Lettuce client configuration.
///
/// Represents advanced Redis client settings,
/// including connection pool configuration.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Lettuce {
    /// Connection pool configuration for the Lettuce client.
    pub pool: Option<Pool>,
}

/// Redis configuration.
///
/// Defines connection parameters and client behavior
/// for Redis-based integrations.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Redis {
    /// Redis server host.
    pub host: Option<String>,

    /// Redis server port.
    pub port: Option<u16>,

    /// Type of Redis client implementation.
    pub client_type: Option<String>,

    /// Lettuce client specific configuration.
    pub lettuce: Option<Lettuce>,

    /// Repository configuration related to Redis usage.
    pub repositories: Option<Repositories>,
}

/// Relational database configuration.
///
/// Controls connection details, pooling behavior,
/// timeouts, and logging options.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Database {
    /// Database connection name.
    pub name: String,

    /// Database connection enabled. Default true.
    pub enabled: Option<bool>,

    /// Database connection URL.
    pub url: Option<String>,

    /// Minimum number of connections in the pool.
    pub min_pool_size: Option<u32>,

    /// Maximum number of connections in the pool.
    pub max_pool_size: Option<u32>,

    /// Enables or disables database query logging.
    pub logging: Option<bool>,

    /// Timeout for acquiring a connection from the pool (in seconds).
    pub aquire_timeout: Option<u64>,

    /// Maximum lifetime of a connection (in seconds).
    pub max_lifetime: Option<u64>,

    /// Maximum idle time of a connection (in seconds).
    pub idle_timeout: Option<u64>,

    /// Timeout for establishing a new connection (in seconds).
    pub connect_timeout: Option<u64>,

    /// Logging level used by the database layer.
    pub logging_level: Option<String>,
}

/// Google BigQuery configuration.
///
/// Defines access parameters and datasets used
/// for analytics and data processing.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct BigQuery {
    /// BigQuery connection enabled. Defaults true.
    pub enabled: Option<bool>,

    /// Enables or disables printing available tables.
    pub print_tables: Option<bool>,

    /// BigQuery region.
    pub region: Option<String>,

    /// Google Cloud project identifier.
    pub project: Option<String>,

    /// Path or identifier for the credential file.
    pub credential: Option<String>,

    /// List of datasets available for querying.
    pub dataset: Option<Vec<String>>,
}

/// Data layer configuration.
///
/// Groups all data-related configurations,
/// such as Redis, BigQuery, and relational databases.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Data {
    /// Enables or disables redis connection.
    pub enabled: Option<bool>,

    /// Redis configuration.
    pub redis: Option<Redis>,

    /// BigQuery configuration.
    pub bigquery: Option<BigQuery>,

    /// Relational database configuration.
    pub databases: Option<Vec<Database>>,
}

/// CORS (Cross-Origin Resource Sharing) configuration.
///
/// Controls how the server handles cross-origin HTTP requests.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Cors {
    /// Maximum cache age for CORS preflight requests (in seconds).
    pub max_age: Option<u32>,

    /// Indicates whether credentials are allowed.
    pub allow_credentials: Option<bool>,

    /// Allowed HTTP methods.
    pub allowed_methods: Option<String>,

    /// Allowed HTTP headers.
    pub allowed_headers: Option<String>,

    /// Allowed origin patterns.
    pub allowed_origins_pattern: Option<String>,
}

/// Server configuration.
///
/// Defines network, runtime, and deployment-related settings.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Server {
    /// Server host address.
    pub host: Option<String>,

    /// Main server port.
    pub port: u16,

    /// Port used for health check endpoints.
    pub health_check_port: u16,

    /// Number of worker threads.
    pub workers: Option<usize>,

    /// Number of worker threads for health checks.
    pub health_check_workers: Option<usize>,

    /// Indicates whether Docker Compose is used.
    pub use_docker_compose: Option<bool>,

    /// Path to the Docker Compose file.
    pub docker_compose_file: Option<String>,

    /// CORS configuration.
    pub cors: Option<Cors>,
}

/// Metrics configuration.
///
/// Controls application metrics exposure and identification.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Metrics {
    /// Enables or disables metrics collection.
    pub enabled: Option<bool>,

    /// Application name used in metrics labels.
    pub app_name: Option<String>,
}

/// OAuth2 configuration.
///
/// Controls authentication and authorization settings.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct OAuth2Configuration {
    /// Enables or disables OAuth2 integration.
    pub enabled: Option<bool>,

    /// Enables or disables automatic OAuth2 discovery.
    pub discovery_enabled: Option<bool>,

    /// OAuth2 discovery URL for automatic configuration.
    #[serde(alias = "discovery_endpoint")]
    pub discovery_url: Option<String>,

    /// OAuth2 configuration issuer URL.
    #[serde(alias = "issuer", alias = "issuer_endpoint")]
    pub issuer_uri: Option<String>,

    /// OAuth2 configuration JSON Web Key Set URI.
    #[serde(alias = "jwks_endpoint", alias = "jwks_uri")]
    pub jwks_uri: Option<String>,

    /// OAuth2 configuration token endpoint URI.
    #[serde(alias = "token_endpoint")]
    pub token_uri: Option<String>,

    /// OAuth2 configuration authorization endpoint URI.
    #[serde(alias = "authorization_endpoint")]
    pub authorization_uri: Option<String>,

    /// OAuth2 configuration introspection endpoint URI.
    #[serde(alias = "introspection_endpoint")]
    pub introspection_uri: Option<String>,

    /// OAuth2 configuration user info endpoint URI.
    #[serde(alias = "userinfo_endpoint")]
    pub user_info_uri: Option<String>,

    /// OAuth2 configuration end session endpoint URI.
    #[serde(alias = "end_session_endpoint")]
    pub end_session_uri: Option<String>,

    /// OAuth2 client configuration.
    pub client: Option<OAuth2Client>,

    /// OAuth2 JSON Web Key Set. This list of keys is used to validate tokens.
    pub jwks: Option<JwkSet>,
}

/// OAuth2 client configuration.

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct OAuth2Client {
    /// OAuth2 client ID.
    pub id: Option<String>,

    /// OAuth2 client secret.
    pub secret: Option<String>,

    /// OAuth2 client scopes.
    pub scope: Option<String>,
}

/// Server security configuration.
///
/// This structure aggregates all security-related settings.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Security {
    /// OAuth2 configuration.
    pub oauth2: Option<OAuth2Configuration>,
}

/// Global application settings.
///
/// Root configuration structure that aggregates
/// server, data, and metrics configurations.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Settings {
    /// Server configuration.
    pub server: Option<Server>,

    /// Data layer configuration.
    pub data: Option<Data>,

    /// Metrics configuration.
    pub metrics: Option<Metrics>,

    /// Security configuration.
    pub security: Option<Security>,
}

impl Settings {
    /// Loads the application settings from a configuration file.
    ///
    /// This function reads the configuration from the specified file path,
    /// deserializes it into the `Settings` structure, and returns it.
    ///
    /// # Arguments
    ///
    /// * `config_path` - A string slice that holds the path to the configuration file.
    ///
    /// # Returns
    ///
    /// * `Result<Settings, ConfigError>` - The loaded settings or an error if loading fails.
    pub fn new(config_path: &str) -> Result<Self> {
        let mut builder = Config::builder().add_source(File::new(config_path, FileFormat::Yaml));

        builder =
            builder.add_source(Environment::with_convert_case(Case::UpperSnake).separator("_"));

        let config = builder.build()?;
        config.try_deserialize::<Settings>()
    }

    /// Returns the OAuth2 configuration object if available.
    ///
    /// # Returns
    ///
    /// * `Option<OAuth2Configuration>` - The OAuth2 configuration if present, `None` otherwise.
    pub fn get_oauth2_config(&self) -> Option<OAuth2Configuration> {
        self.security.as_ref()?.oauth2.clone()
    }

    /// Returns a JWK (JSON Web Key) object if available for the given kid.
    ///
    /// # Arguments
    ///
    /// * `kid` - A string slice that holds the key ID of the JWK object.
    ///
    /// # Returns
    ///
    /// * `Option<Jwk>` - The JWK object if present, `None` otherwise.
    pub fn get_auth2_public_key(&self, kid: &str) -> Option<Jwk> {
        self.security
            .as_ref()?
            .oauth2
            .as_ref()?
            .jwks
            .as_ref()?
            .find(kid)
            .cloned()
    }

    /// Returns the OAuth2 token endpoint URL if available.
    ///
    /// # Returns
    ///
    /// * `Option<String>` - The OAuth2 token endpoint URL if present, `None` otherwise.
    pub fn get_auth2_token_url(&self) -> Option<String> {
        self.security
            .as_ref()?
            .oauth2
            .as_ref()?
            .token_uri
            .as_ref()
            .cloned()
    }
}

/// A type alias for a `Result` with the `ConfigError` error type.
pub type Result<T, E = ConfigError> = std::result::Result<T, E>;
