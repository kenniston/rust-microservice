//! # ServerDatabase Module
//!
//! This module provides a high-level abstraction for managing the application's
//! database connections and BigQuery clients. It is responsible for initializing
//! and managing the connections to the underlying databases and data warehouses.
//!
//! The module provides a `ServerDatabase` struct which encapsulates the connections
//! to the different databases and data warehouses. It also provides convenience functions
//! for initializing the connections with the provided settings.
//!
//! ## Responsibilities
//!
//! - Initialize and manage the connections to the underlying databases and data warehouses.
//! - Provide convenience functions for initializing the connections with the provided settings.
//!
//! ## Usage
//!
//! The `ServerDatabase` struct can be initialized using the `new_with_settings` function
//! which takes a `&Settings` as an argument. This function will initialize the connections
//! to the underlying databases and data warehouses based on the provided settings.

pub mod bigquery;
pub mod database;

use colored::Colorize;

use crate::settings::{BigQuery, Database, Settings};
use thiserror::Error;
use tracing::info;

#[cfg(feature = "memory-database")]
use std::sync::Arc;

#[cfg(feature = "memory-database")]
use sea_orm::MockDatabase;

/// Represents the application's database connections and BigQuery clients.
///
/// This struct encapsulates the connections to the different databases and data warehouses.
/// It provides convenience functions for initializing the connections with the provided settings.
///
/// ## Methods
///
/// `new_with_settings`: Initializes the connections to the underlying databases based on the
/// provided settings.
///
#[derive(Clone)]
pub struct ServerDatabase {
    /// An optional reference to the BigQuery client
    pub bigquery: Option<bigquery::BigQueryClient>,

    /// A vector of database clients, each representing a connection to a different database
    pub databases: Vec<database::DatabaseClient>,
}

/// Implementation of the `ServerDatabase` struct
impl ServerDatabase {
    /// Creates a new `ServerDatabase` instance with a mock database client.
    ///
    /// This function is used for testing purposes only and should not be used in production code.
    ///
    /// # Parameters
    ///
    /// - `name`: The name of the database client connection.
    /// - `database`: The mock database client instance.
    ///
    /// # Returns
    ///
    /// Returns a new `ServerDatabase` instance with the provided mock database client.
    #[cfg(feature = "memory-database")]
    pub fn new_with_mock_database(name: String, database: MockDatabase) -> Self {
        ServerDatabase {
            bigquery: None,
            databases: vec![database::DatabaseClient {
                name,
                connection: Arc::new(database.into_connection()),
            }],
        }
    }

    /// Initializes the connections to the underlying databases using a memory database.
    ///
    /// This function takes a `String` as an argument and initializes the connections to
    /// the underlying databases using a memory database. It returns a new `ServerDatabase`
    /// instance with Memory SQLite initialized connection.
    ///
    /// # Parameters
    ///
    /// - `name`: The name of the database client connection.
    ///
    /// # Returns
    ///
    /// Returns a new `ServerDatabase` instance with the initialized connections.
    #[cfg(feature = "memory-database")]
    pub async fn new_with_memory_database(name: String) -> Result<Self> {
        info!("Initializing ServerDatabase with memory database...");
        let database = database::DatabaseClient::new_with_memory_database(name.clone())
            .await
            .map_err(|e| DataError::Database(e.to_string()))?;

        Ok(ServerDatabase {
            bigquery: None,
            databases: vec![database],
        })
    }

    /// Initializes the connections to the underlying databases and data warehouses based on the
    /// provided settings.
    ///
    /// This function takes a `&Settings` as an argument and initializes the connections to the
    /// underlying databases and data warehouses based on the provided settings. It returns a new
    /// `ServerDatabase` instance with the initialized connections.
    ///
    /// # Parameters
    ///
    /// - `settings`: A reference to the application settings.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a new `ServerDatabase` instance with the initialized
    /// connections or a `DataError` if the connections cannot be established.
    pub async fn new_with_settings(settings: &Settings) -> Result<Self> {
        info!("Initializing ServerDatabase with provided settings...");

        let data = settings.data.as_ref();

        let is_enabled = |flag: Option<bool>| flag.unwrap_or(true);

        // Initialize BigQuery client
        let bigquery = match data.and_then(|d| d.bigquery.as_ref()) {
            Some(cfg) if is_enabled(cfg.enabled) => {
                Some(ServerDatabase::create_bigquery_client(cfg).await?)
            }
            _ => None,
        };

        // Initialize Database clients
        let mut databases = Vec::new();
        if let Some(configs) = data.and_then(|d| d.databases.as_ref()) {
            for config in configs {
                if is_enabled(config.enabled) {
                    databases.push(ServerDatabase::create_database_client(config).await?);
                }
            }
        }

        // Intialize other databases here as needed...
        //

        Ok(ServerDatabase {
            bigquery,
            databases,
        })
    }

    /// Creates a new BigQuery client based on the provided configuration.
    ///
    /// This function takes a `&BigQuery` as an argument and initializes a new BigQuery client
    /// with the provided credentials and dataset. It returns a `Result` containing the new
    /// BigQuery client instance, or a `DataError` if the client cannot be initialized.
    ///
    /// # Parameters
    ///
    /// - `settings`: A reference to the BigQuery configuration.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a new `BigQueryClient` instance, or a `DataError` if the
    /// client cannot be initialized.
    ///
    /// # Notes
    ///
    /// - If the `print_tables` field is set to `true`, the function will log the list of
    ///   available BigQuery tables after initializing the client.
    async fn create_bigquery_client(settings: &BigQuery) -> Result<bigquery::BigQueryClient> {
        info!("Setting up BigQuery client...");

        let credential = settings.credential.as_ref().ok_or_else(|| {
            DataError::Configuration("BigQuery credential is missing.".red().to_string())
        })?;

        let dataset = settings.dataset.as_ref().ok_or_else(|| {
            DataError::Configuration("BigQuery dataset is missing.".red().to_string())
        })?;

        let client =
            bigquery::BigQueryClient::new_with_credentials_and_dataset(credential, dataset)
                .await
                .map_err(|e| DataError::BigQuery(e.to_string().red().to_string()))?;

        info!("BigQuery client initialized.");

        if settings.print_tables.unwrap_or(false) {
            Self::log_bigquery_tables(&client);
        }

        Ok(client)
    }

    /// Prints the mapped tables when the BigQuery database has been
    /// successfully initialized.
    ///
    /// # Parameters
    ///
    /// - `client`: The BigQuery client instance.
    ///
    /// # Notes
    ///
    /// - This function will print the number of loaded BigQuery tables.
    /// - It will iterate and print the name of each table.
    fn log_bigquery_tables(client: &bigquery::BigQueryClient) {
        let tables = client.get_tables();

        info!(
            "{} {}",
            "Database initialized successfully.".white(),
            format!("Loaded {} BigQuery tables.", tables.len()).white()
        );

        for table in tables {
            info!(
                "\t{}",
                format!("Table: {}", table.resource()).white().dimmed()
            );
        }
    }

    /// Creates a new database client connection using the provided configuration.
    ///
    /// # Parameters
    ///
    /// - `settings`: A reference to the database settings.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a new `DatabaseClient` instance, or a `DataError` if the
    /// client cannot be initialized.
    ///
    /// # Notes
    ///
    /// - If the `logging` field is set to `true`, the function will enable SQLx logging.
    /// - If the `logging_level` field is set to a non-empty string, the function will set the
    ///   logging level to the specified value.
    /// - If the `acquire_timeout`, `max_lifetime`, `idle_timeout`, or `connect_timeout`
    ///   fields are set to a non-zero value, the function will set the corresponding timeout
    ///   values to the specified values.
    async fn create_database_client(
        settings: &Database,
    ) -> Result<database::DatabaseClient, DataError> {
        info!("Setting up [{}] Database client...", settings.name.blue());

        let url = settings.url.as_ref().ok_or_else(|| {
            DataError::Configuration(format!(
                "Database URL is missing for [{}].",
                settings.name.blue()
            ))
        })?;

        let database_client = database::DatabaseClient::new(
            settings.name.clone(),
            url.clone(),
            settings.min_pool_size.unwrap_or(1),
            settings.max_pool_size.unwrap_or(1),
            settings.logging,
            settings.logging_level.clone(),
            settings.aquire_timeout,
            settings.max_lifetime,
            settings.idle_timeout,
            settings.connect_timeout,
        )
        .await
        .map_err(|e| DataError::Database(e.to_string().red().to_string()))?;

        info!(
            "Database client initialized for [{}].",
            settings.name.blue()
        );

        Self::validate_connection(&database_client).await?;

        Ok(database_client)
    }

    /// Validates a database connection by pinging the connection and retrieving the database
    /// backend type.
    ///
    /// # Parameters
    ///
    /// - `client`: The database client instance to validate.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the database backend type if the connection is valid,
    /// otherwise a `DbErr` is returned.
    ///
    /// # Notes
    ///
    /// - This function will log information about the database connection and its backend type.
    async fn validate_connection(client: &database::DatabaseClient) -> Result<()> {
        client
            .connection
            .ping()
            .await
            .map_err(|e| DataError::Database(e.to_string().red().to_string()))?;

        let backend = client.connection.get_database_backend();

        info!(
            "The database connection is valid on the backend {:?} for [{}].",
            backend,
            client.name.blue(),
        );

        Ok(())
    }

    /// Closes all database connections in the server database client.
    ///
    /// This method is called when the server is shutting down
    /// and all database connections must be closed to prevent resource
    /// leaks.
    #[cfg(feature = "memory-database")]
    pub fn close(&self) {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async {
            for database in self.databases.iter() {
                let db = database.clone();
                if let Ok(conn) = Arc::try_unwrap(db.connection) {
                    let _ = conn.close().await;
                }
            }
        });
    }

    /// Closes all database connections in the server database client.
    ///
    /// This method is called when the server is shutting down
    /// and all database connections must be closed to prevent resource
    /// leaks.
    #[cfg(not(feature = "memory-database"))]
    pub fn close(&self) {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async {
            for database in self.databases.iter() {
                let _ = database.connection.clone().close().await;
            }
        });
    }
}

/// A type alias for a `Result` with the `DataError` error type.
pub type Result<T, E = DataError> = std::result::Result<T, E>;

/// Errors related to data operations.
///
/// This enum groups all data-domain errors in a single type, making error
/// handling consistent across the application. Each variant represents
/// a specific failure scenario and provides a human-readable error message.
///
/// # Variants
/// - `Configuration`: Invalid database configuration.
/// - `BigQuery`: An error occurred while interacting with BigQuery.
/// - `Database`: An error occurred while interacting with the database.
#[derive(Debug, Error)]
pub enum DataError {
    #[error("Invalid database configuration: {0}")]
    Configuration(String),

    #[error("BigQuery error: {0}")]
    BigQuery(String),

    #[error("Database error: {0}")]
    Database(String),
}
