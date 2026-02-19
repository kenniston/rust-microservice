//! # SERVER Framework
//!
//! **SERVER Framework** is a modular Rust crate designed to simplify the creation,
//! configuration, and initialization of microservice-oriented web servers.
//! It provides a unified system for handling configuration from multiple sources
//! and a web server bootstrap layer powered by `actix-web` and `tokio`.
//!
//! ## Features
//!
//! - **Multi-source configuration management**
//!   - YAML configuration file loading using [`serde_yaml`] and [`config`]
//!   - Environment variable overrides
//!   - Command-line parameter parsing using [`clap`]
//!   - Automatic deserialization into strongly typed structures using [`serde`]
//!
//! - **Web server initialization**
//!   - HTTP server powered by [`actix-web`]
//!   - Asynchronous runtime handled by [`tokio`]
//!   - Built-in routing integration for exposing microservice endpoints
//!   - Input/output serialization using [`serde_json`] and [`serde-xml-rs`]
//!
//! - **Extensible architecture**
//!   - Supports custom modules for controllers, services, repositories, and middleware
//!   - Easy integration with external crates and microservice ecosystems
//!
//! ## Configuration System
//!
//! The framework provides a configuration loader that merges settings from:
//!
//! 1. **YAML files** – typically `config.yaml` or environment-specific variations
//! 2. **Environment variables** – allowing container-friendly overrides
//! 3. **Command-line parameters** – using `clap` for high-level CLI ergonomics
//!
//! Configuration sources follow a layered precedence model:
//!
//! ```text
//! Environment variables > CLI parameters > YAML configuration file
//! ```
//!
//! A typical configuration structure might look like:
//!
//! ```rust
//! #[derive(Debug, Deserialize)]
//! #[serde(rename_all = "kebab-case")]
//! pub(crate) struct Settings {
//!     pub server: Option<SERVER>,
//!     pub data: Option<Data>
//! }
//!
//! #[derive(Debug, Deserialize)]
//! #[serde(rename_all = "kebab-case")]
//! pub(crate) struct SERVER {
//!     pub host: Option<String>,
//!     pub port: Option<u16>,
//!     pub use_docker_compose: Option<bool>,
//!     pub compose_file: Option<String>
//! }
//!
//! #[derive(Debug, Deserialize)]
//! #[serde(rename_all = "kebab-case")]
//! pub(crate) struct Data {
//!     pub redis: Option<Redis>
//! }
//! ```
//!
//! After loading the configuration, the framework exposes a typed `Settings`
//! instance for application modules and endpoint handlers.
//!
//! ## Web SERVER Module
//!
//! The server module is responsible for:
//!
//! - Initializing the `actix-web` HTTP server
//! - Registering routers and microservice endpoints
//! - Managing middlewares and shared application state
//! - Running the asynchronous runtime using `tokio::main` or a custom runtime
//!
//! Example of a minimal server bootstrap using this crate:
//!
//! ```rust
//! async fn bootstrap_server(settings: Settings) -> std::io::Result<()> {
//!     use actix_web::{App, HttpServer};
//!
//!     HttpServer::new(move || {
//!         App::new()
//!             .app_data(config.clone())
//!             .service(api::healthcheck)
//!             .service(api::version)
//!     })
//!     .bind((settings.server.host.as_str(), settings.server.port))?
//!     .run()
//!     .await
//! }
//! ```
//!
//! This framework encourages a modular architecture where each microservice
//! registers its own route handlers, serializers, and business logic.
//!
//! ## Serialization Support
//!
//! The crate includes seamless integration with:
//!
//! - **JSON** serialization/deserialization via `serde_json`
//! - **XML** handling via `serde-xml-rs`
//!
//! These serializers allow exposing multiple content-types or supporting legacy systems.
//!
//! ## Usage
//!
//! Add the crate dependency in your project's `Cargo.toml`:
//!
//! ```toml
//! server-framework = "0.1.0"
//! ```
#![deny(clippy::unwrap_used)]
#![deny(clippy::redundant_clone)]

mod cmd;
mod data;
mod http;
mod metrics;
mod security;
mod server;
pub mod settings;
pub mod test;

#[derive(rust_embed::Embed)]
#[folder = "assets"]
pub struct Asset;

pub use http::web::ServerWrappers;
pub use http::web::create_server_wrappers as server_wrappers;
pub use security::oauth2::LoginForm;
pub use security::oauth2::Token;
pub use server::Result;
pub use server::Server;
pub use server::ServerError;

/// Renames the main server macro and exports it.
pub use server_macros::api_server as ServerApi;
pub use server_macros::database;
pub use server_macros::secured;
