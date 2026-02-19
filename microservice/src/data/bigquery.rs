//! # BigQuery Database Module
//!
//! This module provides the BigQuery database integration layer for the application.
//! It is responsible for managing the connection to Google BigQuery and executing
//! SQL queries in a safe, reliable, and efficient way.
//!
//! ## Responsibilities
//!
//! - Initialize and manage the BigQuery client and authentication.
//! - Execute SQL queries against BigQuery datasets and tables.
//! - Handle query jobs, including submission, polling, and result retrieval.
//! - Map BigQuery results to application-friendly data structures.
//! - Centralize error handling related to BigQuery operations.
//!
//! ## Usage
//!
//! This module should be used by repository or service layers that require access
//! to analytical data stored in BigQuery. It abstracts BigQuery-specific details
//! and provides a clean API for executing queries.
//!
//! ## Notes
//!
//! - Authentication is typically performed using a service account.
//! - Long-running queries are executed as jobs and may require polling for completion.
//! - All BigQuery interactions should go through this module to ensure consistency
//!   and observability.
//!
//! ## See Also
//!
//! - Google BigQuery Documentation: https://cloud.google.com/bigquery/docs
//! - Google Cloud Rust libraries

use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use colored::Colorize;
use google_cloud_bigquery::client::google_cloud_auth::credentials::CredentialsFile;
use google_cloud_bigquery::client::{Client, ClientConfig, QueryError};
use google_cloud_bigquery::http::job::query::QueryRequest;
use google_cloud_bigquery::http::table::TableReference;
use google_cloud_bigquery::http::table::list::ListTablesRequest;
use google_cloud_bigquery::http::types::QueryParameter;
use google_cloud_bigquery::query::row::Row;
use google_cloud_bigquery::query::{self, ExponentialBuilder, QueryOption};
use tracing::error;

use crate::data::DataError;

/// BigQuery client abstraction responsible for authentication,
/// dataset inspection, and query execution.
///
/// This struct encapsulates the Google BigQuery client and stores
/// metadata such as the project ID and available tables from the
/// configured datasets.
///
/// It is designed to be cloned and shared safely across async
/// contexts.
#[derive(Clone)]
pub struct BigQueryClient {
    project_id: String,
    client: Client,
    tables: Vec<TableReference>,
}

impl BigQueryClient {
    pub async fn new_with_credentials_and_dataset(
        encoded_credentials: &str,
        datasets: &Vec<String>,
    ) -> Result<Self, DataError> {
        let normalized_credentials = encoded_credentials.replace(" ", "").replace("\n", "");
        let decoded = BASE64_STANDARD
            .decode(normalized_credentials)
            .map_err(|e| {
                DataError::Configuration(
                    format!("Failed to decode credentials: {}", e)
                        .red()
                        .to_string(),
                )
            })?;

        let json = std::str::from_utf8(&decoded).map_err(|e| {
            DataError::Configuration(
                format!("Decoded credentials are not valid UTF-8: {:?}", e)
                    .red()
                    .to_string(),
            )
        })?;

        let credential = CredentialsFile::new_from_str(json).await.map_err(|e| {
            DataError::Configuration(
                format!("Failed to create credentials from decoded string: {:?}", e)
                    .red()
                    .to_string(),
            )
        })?;

        let (config, project_id) = ClientConfig::new_with_credentials(credential)
            .await
            .map_err(|e| {
                DataError::BigQuery(
                    format!("Failed to create ClientConfig with credentials: {:?}", e)
                        .red()
                        .to_string(),
                )
            })?;

        let project_id = project_id.ok_or_else(|| {
            DataError::BigQuery("Project ID not found in credentials".red().to_string())
        })?;

        let client = Client::new(config).await.map_err(|e| {
            DataError::BigQuery(
                format!("Failed to create BigQuery client: {:?}", e)
                    .red()
                    .to_string(),
            )
        })?;

        let mut tables = Vec::new();

        for dataset in datasets {
            let table_overviews = client
                .table()
                .list(
                    &project_id,
                    dataset,
                    &ListTablesRequest {
                        max_results: Some(500),
                    },
                )
                .await
                .map_err(|e| {
                    DataError::BigQuery(
                        format!("Failed to list tables in dataset {}: {:?}", dataset, e)
                            .red()
                            .to_string(),
                    )
                })?;

            tables.extend(
                table_overviews
                    .into_iter()
                    .map(|overview| overview.table_reference),
            );
        }

        Ok(BigQueryClient {
            project_id,
            client,
            tables,
        })
    }

    pub async fn query(
        &self,
        query: &str,
        query_parameters: Vec<QueryParameter>,
        max_results: Option<i64>,
        legacy_sql: Option<bool>,
    ) -> Result<query::Iterator<Row>, QueryError> {
        let project_id = self.project_id.as_str();
        let request = QueryRequest {
            max_results,
            query_parameters,
            query: query.to_string(),
            use_legacy_sql: legacy_sql.unwrap_or(false),
            use_query_cache: Some(true),
            parameter_mode: Some("NAMED".to_string()),
            ..Default::default()
        };

        let retry = ExponentialBuilder::default().with_max_times(10);
        let option = QueryOption::default().with_retry(retry); //.with_enable_storage_read(true);

        self.client
            .query_with_option::<Row>(project_id, request, option)
            .await
            .map_err(|e| {
                error!("{}", format!("Failed to execute query: {:?}", e).red());
                e
            })
    }

    /// Returns the cached list of BigQuery tables.
    ///
    /// The tables are loaded during client initialization from the
    /// configured datasets.
    ///
    /// # Returns
    ///
    /// An optional reference to a vector of [`TableReference`].
    /// Returns `None` if the client was not properly initialized.
    pub fn get_tables(&self) -> &Vec<TableReference> {
        self.tables.as_ref()
    }
}
