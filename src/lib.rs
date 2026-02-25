//! # 📦 Rust Microservice
//!
//! **Rust Microservice Framework** is a high-performance Rust crate designed to simplify
//! the creation, configuration, and initialization of microservice-oriented web servers.
//! It provides a unified system for handling configuration from multiple sources
//! and a web server bootstrap layer powered by `actix-web` and `tokio`.
//!
//! ## ✨ Features
//!
//! - **Multi-source configuration management**
//!   - YAML configuration file loading using `serde_yaml` and `config`
//!   - Environment variable overrides
//!   - Command-line parameter parsing using `clap`
//!   - Automatic deserialization into strongly typed structures using `serde`
//!
//! - **Web server initialization**
//!   - HTTP server powered by `actix-web`
//!   - Asynchronous runtime handled by `tokio`
//!   - Built-in routing integration for exposing microservice endpoints
//!   - Input/output serialization using `serde_json` and `serde-xml-rs`
//!
//! - **Extensible architecture**
//!   - Supports custom modules for controllers, services, repositories, and middleware
//!   - Easy integration with external crates and microservice ecosystems
//!
//! ## 🛠️ Configuration System
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
//! After loading the configuration, the framework exposes a typed `Settings`
//! instance for application modules and endpoint handlers.
//!
//! ## 🌐 Web SERVER Module
//!
//! The server module is responsible for:
//!
//! - Initializing the `actix-web` HTTP server
//! - Registering routers and microservice endpoints
//! - Managing middlewares and shared application state
//! - Running the asynchronous runtime using `tokio::main` or a custom runtime
//!
//! This framework encourages a modular architecture where each microservice
//! registers its own route handlers, serializers, and business logic.
//!
//! ## ⚙️ Serialization Support
//!
//! The crate includes seamless integration with:
//!
//! - **JSON** serialization/deserialization via `serde_json`
//! - **XML** handling via `serde-xml-rs`
//!
//! These serializers allow exposing multiple content-types or supporting legacy systems.
//!
//! ## ⚡ Usage
//!
//! Add the crate dependency in your project's `Cargo.toml`:
//!
//! ```toml
//! rust-microservice = "0.1.0"
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
pub(crate) struct Asset;

pub use http::web::ServerWrappers;
pub use http::web::create_server_wrappers as server_wrappers;
pub use security::oauth2::LoginForm;
pub use security::oauth2::Token;
pub use server::Result;
pub use server::Server;
pub use server::ServerError;

/// # 🔗 API Server Macro
///
/// The `api_server` macro is a procedural macro that generates the code necessary to
/// start an `actix-web` HTTP server with support for OpenAPI documentation and
/// a health check endpoint.
///
/// The `api_server` macro takes the following attributes:
///
/// - `controllers_path`: A comma-separated list of paths to modules containing
///   controllers. The macro will recursively traverse the directories and generate
///   code to register the controllers with the HTTP server.
///
/// - `openapi_title`: A string used as the title of the OpenAPI documentation.
///
/// - `openapi_api_description`: A string used as the description of the OpenAPI
///   documentation.
///
/// - `database`: A boolean indicating whether the microservice should enable database
///   integration. If set to `true`, the macro will generate code to initialize the
///   database connection pool using the `sea_orm` crate.
///
/// - `banner`: A string used as the banner of the microservice. The banner is displayed
///   in the server logs during startup.
///
/// Example of a minimal server bootstrap using this crate:
///
/// ```rust
/// use rust_microservice::ServerApi;
///
/// #[ServerApi(
///     controllers_path = "src/module, src/controllers",
///     openapi_title = "🌐 Rest API Server",
///     openapi_api_description = "Rest API OpenApi Documentation built with Rust 🦀.",
///     database = "true",
///     banner = r#"
///             _~^~^~_         ___    ___   ____    ____
///         \) /  o o  \ (/    / _ |  / _ \ /  _/   / __/___  ____ _  __ ___  ____
///           '_   -   _'     / __ | / ___/_/ /    _\ \ / -_)/ __/| |/ //! -_)/ __/
///           / '-----' \    /_/ |_|/_/   /___/   /___/ \__//!_/   |___/ \__//!_/
///     "#
/// )]
/// async fn start_server() -> rust_microservice::Result<(), String> {}
/// ```
pub use server_macros::api_server as ServerApi;

/// # 🛢️ Database Macro
///
/// The `database` macro is a procedural macro that injects a database connection
/// into repository methods.
///
/// It expects two mandatory attributes:
/// - `name`: selects which configured database connection will be used.
/// - `error`: defines the error variant returned when the database is not configured or
///   database connection cannot be found.
///
/// The macro injects a variable named `db` with type `&DatabaseConnection` (seaorm),
/// so the function body can execute queries directly.
///
/// Example:
///
/// ```rust
/// use rust_microservice::{Server, database};
/// use thiserror::Error;
///
/// #[derive(Debug, Error)]
/// pub enum UserError {
///     #[error("Database is not configured")]
///     DatabaseNotConfigured,
///
///     #[error("User not found")]
///     NotFound,
/// }
///
/// pub type Result<T, E = UserError> = std::result::Result<T, E>;
///
/// #[database(name = "api", error = "UserError::DatabaseNotConfigured")]
/// pub async fn get_user_by_id(user_id: i32) -> Result<()> {
///
///     // Database will be injected here as `db`
///
///     //user::Entity::find_by_id(user_id)
///     //    .one(&db)
///     //    .await
///     //    .map_err(|_| UserError::NotFound)?
///     //    .ok_or(UserError::NotFound)
///     //    .map(Into::into)
///
///     Ok(())
/// }
/// ```
pub use server_macros::database;

/// # 🔐 Secured Macro
///
/// The `Secured` macro protects `actix-web` endpoints by attaching an authentication middleware.
///
/// When applied to an endpoint, it validates:
///
/// - JWT presence in the request.
/// - JWT signature.
/// - JWT expiration time (`exp` claim).
/// - JWT issuer (`iss` claim).
/// - Required roles from the `authorize` expression.
///
/// ## Attribute Reference
//
/// Macro usage format:
//
/// ```rust
/// #[secured(method = "...", path = "...", authorize = "...")]
/// ```
///
/// ### **`method`**
///
/// Defines the HTTP method used to map the endpoint in Actix-Web.
///
/// Supported values:
///
/// - `get`
/// - `post`
/// - `put`
/// - `delete`
/// - `head`
/// - `connect`
/// - `options`
/// - `trace`
/// - `patch`
///
/// ### **`path`**
///
/// Defines the endpoint path to be registered by Actix-Web.
///
/// Example:
///
/// ```rust
/// path = "/v1/user/{id}"
/// ```
///
/// ### **`authorize`**
///
/// Defines the required role rule that must be satisfied by roles present in the JWT.
///
/// Supported formats:
///
/// 1. `Single role`: validates one role in the token.
///
/// ```rust
/// authorize = "ROLE_ADMIN"
/// ```
///
/// 2. `hasAnyRole`: validates that at least one role in the list exists in the token.
///
/// ```rust
/// authorize = "hasAnyRole(ROLE_ADMIN, ROLE_USER)"
/// ```
///
/// 3. `hasAllRoles`: validates that all roles in the list exist in the token.
///
/// ```rust
/// authorize = "hasAllRoles(ROLE_ADMIN, ROLE_USER)"
/// ```
///
/// ## Examples
///
/// ### **`Single role`**:
///
/// ```rust
/// #[secured(method = "post", path = "/v1/user", authorize = "ROLE_ADMIN")]
/// pub async fn create_user_endpoint(...) -> HttpResponse {
///     // handler body
/// }
/// ```
///
/// ### **`Any role`**:
///
/// ```rust
/// #[secured(
///     method = "get",
///     path = "/v1/user/{id}",
///     authorize = "hasAnyRole(ROLE_ADMIN, ROLE_USER)"
/// )]
/// pub async fn get_user_endpoint(...) -> HttpResponse {
///     // handler body
/// }
/// ```
///
/// ### **`All roles`**:
///
/// ```rust
/// #[secured(
///     method = "delete",
///     path = "/v1/user/{id}",
///     authorize = "hasAllRoles(ROLE_ADMIN, ROLE_AUDITOR)"
/// )]
/// pub async fn delete_user_endpoint(...) -> HttpResponse {
///     // handler body
/// }
/// ```
pub use server_macros::secured;
