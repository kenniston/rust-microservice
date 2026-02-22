use std::{any::Any, sync::OnceLock};

use colored::Colorize;
use reqwest::header::CONTENT_TYPE;
use reqwest_middleware::ClientBuilder;
use rust_microservice::{
    LoginForm, Server, Token,
    settings::Settings,
    test::{
        Result, TestError,
        containers::{keycloak, postgres},
    },
};
use testcontainers::{ContainerAsync, GenericImage};
use testcontainers_modules::postgres::Postgres;
use tracing::{error, info};

mod module;

/// Sets up the test environment by starting Containers and
/// updating the application configuration with the connection details.
///
/// This function is automatically invoked by the `ctor` attribute before any tests
/// are executed. It is responsible for setting up the test environment and
/// ensuring that the test database is properly configured.
///
/// The function starts a Postgres test container with a unique name and updates the
/// application configuration with the connection details. This allows the tests
/// to use the test database instead of the production database.
#[ctor::ctor]
pub fn setup() {
    rust_microservice::test::setup(
        async || {
            let mut settings = load_test_settings();

            // This vector serves as a workaround for Testcontainers’ automatic cleanup,
            // ensuring that containers remain available until all tests have completed.
            let mut containers: Vec<Box<dyn Any + Send>> = vec![];

            let postgres = start_postgres_container(&mut settings).await;
            if let Ok(postgres) = postgres {
                containers.push(Box::new(postgres.0));
            }

            let keycloak = start_keycloak_container(&mut settings).await;
            if let Ok(keycloak) = keycloak {
                containers.push(Box::new(keycloak.0));
            }

            (containers, settings)
        },
        || async {
            info!("Getting authorization token ...");
            let oauth2_token = get_auth_token().await.unwrap_or("".to_string());
            TOKEN.set(oauth2_token);
            info!("Authorization token: {}...", token()[..50].bright_blue());
        },
    );
}

/// Starts a Postgres test container with a unique name and
/// updates the provided application configuration with the connection details.
///
/// # Parameters
/// - `config`: Reference to the application configuration that
///   will be updated with the connection details of the Postgres test container.
///
/// # Notes
/// - Panics if the `docker` command is not found or if the container fails to start.
/// - The test container will be stopped when the test suite is finished.
async fn start_postgres_container(
    config: &mut Settings,
) -> Result<(ContainerAsync<Postgres>, String)> {
    let result = postgres(
        Some("etc/.api-initdb".to_string()),
        Some("api_database".to_string()),
        None,
        Some("user_api".to_string()),
        Some("secret".to_string()),
    )
    .await;

    match result {
        Ok((container, postgres_uri)) => {
            info!(
                "Using Postgres test container at {}.",
                &postgres_uri.bright_blue()
            );

            if !postgres_uri.is_empty()
                && let Some(postgres_config) = config
                    .data
                    .as_mut()
                    .and_then(|data| data.databases.as_mut())
                    .and_then(|databases| databases.iter_mut().find(|db| db.name == "api"))
            {
                postgres_config.url = Some(postgres_uri.clone());
            }
            Ok((container, postgres_uri))
        }
        Err(e) => {
            error!("Failed to start Postgres container: {}", e);
            Err(e)
        }
    }
}

/// Starts a Keycloak test container with a unique name and updates the provided
/// application configuration with the connection details.
///
/// # Parameters
/// - `config`: Reference to the application configuration that will be updated
///   with the connection details of the Keycloak test container.
///
/// # Notes
/// - Panics if the `docker` command is not found or if the container fails to start.
/// - The test container will be stopped when the test suite is finished.
async fn start_keycloak_container(
    config: &mut Settings,
) -> Result<(ContainerAsync<GenericImage>, String)> {
    let result = keycloak("assets/tests/realm-export-users.json", "test_network").await;
    match result {
        Ok((container, keycloak_uri)) => {
            info!(
                "Using Keycloak test container at {}.",
                &keycloak_uri.bright_blue()
            );

            if !keycloak_uri.is_empty()
                && let Some(oauth2_config) = config
                    .security
                    .as_mut()
                    .and_then(|security| security.oauth2.as_mut())
            {
                oauth2_config.discovery_enabled = Some(true);
                oauth2_config.discovery_url = Some(
                    keycloak_uri.clone() + "/realms/rust-api/.well-known/openid-configuration",
                );
            }

            Ok((container, keycloak_uri))
        }
        Err(err) => {
            error!("Failed to start Keycloak container: {}", err);
            Err(err)
        }
    }
}

/// Loads test application settings from a configuration file.
///
/// This function loads the configuration from the specified file path,
/// deserializes it into the `Settings` structure, and returns it.
///
/// If the loading process fails, it panics with a formatted error message.
///
/// # Returns
///
/// * `Settings` - The loaded settings or panics with an error message on failure.
fn load_test_settings() -> Settings {
    let result = Settings::new("assets/tests/test-config.yaml");
    match result {
        Ok(settings) => settings,
        Err(err) => panic!("Failed to load server settings: {}", err),
    }
}

/// Tears down the test environment by stopping all running containers
/// This function is automatically called after all tests have been executed.
#[ctor::dtor]
pub fn teardown() {
    rust_microservice::test::teardown();
}

/// Global static instance of the OAuth2 token.
static TOKEN: OnceLock<String> = OnceLock::new();

pub fn token() -> String {
    TOKEN.get().unwrap_or(&"".to_string()).clone()
}

/// Retrieves an OAuth2 token for use in testing.
///
/// This function retrieves an OAuth2 token from the configured token URI.
/// The token is retrieved using the `reqwest` library and the `Token` struct is used
/// to decode the JSON response.
///
/// If the token URI is not configured, a `TestError::Custom` is returned with the message
/// "Security not configured!".
///
/// If the token is successfully retrieved, the token is returned as a `Result<String>`.
/// If an error occurs during the retrieval of the token, a `TestError::Custom` is returned
/// with the error message.
///
/// # Returns
///
/// A `Result<String>` containing the OAuth2 token.
async fn get_auth_token() -> Result<String> {
    let oauth2 = Server::global()
        .map_err(|e| TestError::Custom(e.to_string()))?
        .settings()
        .security
        .as_ref()
        .and_then(|s| s.oauth2.as_ref())
        .ok_or(TestError::Custom("Security not configured!".to_string()))?;

    if let Some(token_uri) = &oauth2.token_uri {
        let client = ClientBuilder::new(reqwest::Client::new())
            //.with(TracingMiddleware::default())
            .build();

        let form = LoginForm {
            grant_type: "password".to_string(),
            username: Some("kenniston".to_string()),
            password: Some("123456".to_string()),
            client_id: Some("rust-client".to_string()),
            client_secret: Some("pLcQStmQ9HUyp75MFGZoIgyyfS2jmEkr".to_string()),
            scope: Some("openid".to_string()),
        };

        let response = client
            .post(token_uri)
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(form.to_urlencoded())
            .send()
            .await
            .map_err(|e| TestError::Custom(e.to_string()))?;

        let token = response
            .json::<Token>()
            .await
            .map_err(|e| {
                info!("Failed to decode token: {}", e);
                TestError::Custom(e.to_string())
            })?
            .access_token
            .ok_or(TestError::Custom("Token not found!".to_string()))?;

        return Ok(format!("Bearer {}", token));
    }

    Ok("".to_string())
}
