//! Database connection module.
//!
//! This module provides abstractions for database client connections. It
//! includes a `Database` struct, which represents a database client
//! connection, and utility functions for connecting to a database and
//! retrieving a database connection instance.
#![allow(clippy::too_many_arguments)]

use std::{str::FromStr, time::Duration};

use colored::Colorize;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use tracing::debug;

#[cfg(feature = "memory-database")]
use std::sync::Arc;

/// Represents a database client connection.
///
/// This struct holds the name and connection details of a database
/// client connection. It is used to interact with the database.
///
/// # Methods
///
/// - `new`: Creates a new database client connection with the provided configuration
///   parameters.
#[derive(Clone)]
pub struct DatabaseClient {
    /// The name of the database client connection
    pub name: String,

    /// The database connection object
    #[cfg(feature = "memory-database")]
    pub connection: Arc<DatabaseConnection>,

    #[cfg(not(feature = "memory-database"))]
    pub connection: DatabaseConnection,
}

impl DatabaseClient {
    #[cfg(feature = "memory-database")]
    pub async fn new_with_memory_database(name: String) -> Result<Self, DbErr> {
        Ok(DatabaseClient {
            name,
            connection: Arc::new(Database::connect("sqlite::memory:").await?),
        })
    }

    /// Creates a new database client connection with the provided configuration
    /// parameters.
    ///
    /// # Parameters
    ///
    /// - `name`: The name of the database client connection.
    /// - `dabaset_url`: The URL of the database connection.
    /// - `min_pool_size`: The minimum number of connections to maintain in
    ///   the pool.
    /// - `max_pool_size`: The maximum number of connections to maintain in
    ///   the pool.
    /// - `logging`: Enables or disables SQLx logging.
    /// - `logging_level`: Specifies the logging level for SQLx.
    /// - `acquire_timeout`: The maximum time to wait when acquiring a connection from
    ///   the pool.
    /// - `max_lifetime`: The maximum lifetime of a connection in the pool.
    /// - `idle_timeout`: The maximum time to wait before closing an idle connection in
    ///   the pool.
    /// - `connect_timeout`: The maximum time to wait when connecting to the database.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the newly created database client connection, or a
    /// `DbErr` if the connection cannot be established.
    pub async fn new(
        name: String,
        dabaset_url: String,

        min_pool_size: u32,
        max_pool_size: u32,
        logging: Option<bool>,
        logging_level: Option<String>,
        acquire_timeout: Option<u64>,
        max_lifetime: Option<u64>,
        idle_timeout: Option<u64>,
        connect_timeout: Option<u64>,
    ) -> Result<Self, DbErr> {
        let mut options = ConnectOptions::new(dabaset_url);

        options
            .min_connections(min_pool_size)
            .max_connections(max_pool_size);
        //.connect_lazy(true);

        if let Some(enabled) = logging {
            options.sqlx_logging(enabled);
        }

        if let Some(level) = logging_level
            && let Ok(level) = log::LevelFilter::from_str(&level)
        {
            options.sqlx_logging_level(level);
        }

        if let Some(seconds) = acquire_timeout {
            options.acquire_timeout(Duration::from_secs(seconds));
        }

        if let Some(seconds) = max_lifetime {
            options.max_lifetime(Duration::from_secs(seconds));
        }

        if let Some(seconds) = idle_timeout {
            options.idle_timeout(Duration::from_secs(seconds));
        }

        if let Some(seconds) = connect_timeout {
            options.connect_timeout(Duration::from_secs(seconds));
        }

        debug!("Database connection options: \n{:#?}", options);

        let connection = Database::connect(options)
            .await
            .map_err(|e| DbErr::Custom(e.to_string().red().to_string()))?;

        Ok(Self {
            name,
            #[cfg(feature = "memory-database")]
            connection: Arc::new(connection),
            #[cfg(not(feature = "memory-database"))]
            connection,
        })
    }
}
