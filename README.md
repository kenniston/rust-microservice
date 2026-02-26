<div align="center">

  <img alt="Rust Microservice" src="assets/rust-microservice.jpg"/>

  <h1></h1>
  <h3>Rust Microservice is a powerful framework for building web services in Rust</h3>

  ![rust](https://badgen.net/badge/Rust%20Edition/2024/red?scale=1.0)
  ![rust](https://badgen.net/badge/Rust/1.91.1/blue?scale=1.0)
  ![crate](https://badgen.net/badge/crates.io/v0.1.0/orange?scale=1.0)
  <!--[![build status](https://github.com/kenniston/rust-microservice/actions/workflows/rust.yml/badge.svg)](https://github.com/kenniston/rust-microservice/actions/workflows/rust.yml)-->
  [![GitHub stars](https://img.shields.io/github/stars/kenniston/rust-microservice.svg?style=social&label=Star&maxAge=1)](https://github.com/kenniston/rust-microservice/)
  <br>Support us with a ⭐ !

</div>


# 📦 Rust Microservice Framework <!-- omit from toc -->

A Rust crate whichs provides a framework for building microservices. It follows the MVC 
(Model-View-Controller) architecture pattern and provides a strong focus on high performance, 
security, and scalability.

## 📋 Table of Contents <!-- omit from toc -->

- [📖 Overview](#-overview)
- [✨ Features](#-features)
- [🛠️ Installation](#️-installation)
- [⚡ Quick Start](#-quick-start)
- [💡 Usage Examples](#-usage-examples)
  - [🖥️ Simple Server](#️-simple-server)
  - [🔗 ServerApi Macro](#-serverapi-macro)
  - [🛢️ Database Macro](#️-database-macro)
  - [🔐 Secured Macro](#-secured-macro)
  - [Attribute Reference](#attribute-reference)
    - [**`method`**](#method)
    - [**`path`**](#path)
    - [**`authorize`**](#authorize)
  - [Examples](#examples)
    - [**`Single role`**](#single-role)
    - [**`Any role`**](#any-role)
    - [**`All roles`**](#all-roles)
- [YAML-based server configuration file](#yaml-based-server-configuration-file)
  - [Server](#server)
  - [CORS](#cors)
  - [Security — OAuth2 / OpenID Connect](#security--oauth2--openid-connect)
  - [OAuth2 Client](#oauth2-client)
  - [JWKS](#jwks)
  - [Data Sources](#data-sources)
  - [*Redis*](#redis)
  - [*Relational Databases*](#relational-databases)
  - [*BigQuery Database Connection*](#bigquery-database-connection)
  - [Metrics](#metrics)
  - [Runtime Notes](#runtime-notes)
- [🔧 Development Setup](#-development-setup)
- [🔬 Test Environment Infrastructure](#-test-environment-infrastructure)
  - [Architecture Overview](#architecture-overview)
  - [Blocking Execution Helper](#blocking-execution-helper)
  - [Container Utilities](#container-utilities)
    - [Supported Services](#supported-services)
  - [Usage](#usage)
  - [Concurrency Model](#concurrency-model)
  - [Safety Guarantees](#safety-guarantees)
- [📦 Project Dependencies](#-project-dependencies)
- [📄 License](#-license)
- [🤝 Contributing](#-contributing)
- [📝 Changelog](#-changelog)


## 📖 Overview

The framework is designed to be modular, allowing developers to register their own route 
handlers, serializers, and business logic. This enables a high degree of customization and 
flexibility, making it suitable for a wide range of use cases.

## ✨ Features

- Modular architecture with support for registering custom route handlers, serializers, 
  and business logic;
- High performance HTTP server with support for asynchronous request handling;
- Strong focus on security and scalability;
- Support for JSON and XML serialization/deserialization;
- Built-in metrics export compatible with Prometheus;
- YAML-based configuration via `config.yaml`;
- Built-in API documentation with OpenAPI and interactive UI through Swagger UI powered 
  by Utoipa Swagger UI;
- Built-in Health Check endpoint;
- Configurable server and actuator ports (defaults: `8080` and `7188`);
- Native CORS configuration support;
- MVC-based project architecture;
- Automatic database connectivity (SQLite, PostgreSQL, MySQL, and Google BigQuery) driven 
  by YAML configuration;
- Local development and integration testing with Docker Compose and Testcontainers;
- Integration test wrapper for initializing a global Testcontainers environment;
- Built-in integration with OAuth2 servers (Keycloak on the server example);
- Automatic discovery URL resolution for:
  issuer, jwks, token, authorization, introspection, user-info, and end-session endpoints;
- Endpoint-level security configuration using Rust macros.
  
## 🛠️ Installation

Add the Rust Microservice to Cargo.toml:

```toml

[dependencies]
rust-microservice = "0.1.0"

```

> The `Rust Microservice` framework relies on several core crates to bootstrap and run projects.  
> As a result, the following dependencies are required in any application built with the framework:
>
> - `actix-web`
> - `tokio`
> - `utoipa`
> - `sea-orm`
> - `utoipa-swagger-ui`

## ⚡ Quick Start

To configure the server, apply the ServerApi macro to the main function. The macro automatically
discovers all actix-web handlers defined in the `controllers_path` ([ServerApi Attributes](#serverapi-macro)) 
attribute and initializes the databases specified in the YAML configuration file.

The configuration file can be provided to the framework using the `--config-file` command-line 
parameter (or the `CONFIG_FILE` environment variable), or as a Base64-encoded file via the 
`--b64-config-file` parameter (or the `B64_CONFIG_FILE` environment variable).

```rust
use rust_microservice::ServerApi;

#[ServerApi]
fn main() -> rust_microservice::Result<(), String> {}

```

## 💡 Usage Examples

### 🖥️ Simple Server

The Rust Microservice framework has a default config wich starts a simple server on port `8080`.
This server also enable a health check and monitoring feature. The health check can be reach
on default port `7188` and `/health` endpoint.

Default server configuration:

```yaml
server:
  host: "0.0.0.0"
  port: 8080
  health-check-port: 7188
```
---

### 🔗 ServerApi Macro

The `ServerApi` macro is a procedural macro that generates the code necessary to
start an `actix-web` HTTP server with support for OpenAPI documentation and
a health check endpoint.

The `ServerApi` macro takes the following attributes:

- `controllers_path`: A comma-separated list of paths to modules containing
  controllers. The macro will recursively traverse the directories and generate
  code to register the controllers with the HTTP server.

- `openapi_title`: A string used as the title of the OpenAPI documentation.

- `openapi_api_description`: A string used as the description of the OpenAPI
  documentation.

- `database`: A boolean indicating whether the microservice should enable database
  integration. If set to `true`, the macro will generate code to initialize the
  database connection pool using the `sea_orm` crate.

- `banner`: A string used as the banner of the microservice. The banner is displayed
  in the server logs during startup.

Example of a minimal server bootstrap using this crate:

```rust
use rust_microservice::ServerApi;

#[ServerApi(
    controllers_path = "src/module, src/controllers",
    openapi_title = "🌐 Rest API Server",
    openapi_api_description = "Rest API OpenApi Documentation built with Rust 🦀.",
    database = "true",
    banner = r#"
            _~^~^~_         ___    ___   ____    ____
        \) /  o o  \ (/    / _ |  / _ \ /  _/   / __/___  ____ _  __ ___  ____
          '_   -   _'     / __ | / ___/_/ /    _\ \ / -_)/ __/| |/ //! -_)/ __/
          / '-----' \    /_/ |_|/_/   /___/   /___/ \__//!_/   |___/ \__//!_/
    "#
)]
async fn start_server() -> rust_microservice::Result<(), String> {}
```
---

### 🛢️ Database Macro

The `database` macro is a procedural macro that injects a database connection
into repository methods.

It expects two mandatory attributes:
- `name`: selects which configured database connection will be used.
- `error`: defines the error variant returned when the database is not configured or
  database connection cannot be found.

The macro injects a variable named `db` with type `&DatabaseConnection` (seaorm),
so the function body can execute queries directly.

Example:

```rust
use rust_microservice::{Server, database};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UserError {
    #[error("Database is not configured")]
    DatabaseNotConfigured,

    #[error("User not found")]
    NotFound,
}

pub type Result<T, E = UserError> = std::result::Result<T, E>;

#[database(name = "api", error = "UserError::DatabaseNotConfigured")]
pub async fn get_user_by_id(user_id: i32) -> Result<()> {
    user::Entity::find_by_id(user_id)
       .one(&db)
       .await
       .map_err(|_| UserError::NotFound)?
       .ok_or(UserError::NotFound)
       .map(Into::into)
}
```
---

### 🔐 Secured Macro

The `Secured` macro protects `actix-web` endpoints by attaching an authentication middleware.

When applied to an endpoint, it validates:

- JWT presence in the request.
- JWT signature.
- JWT expiration time (`exp` claim).
- JWT issuer (`iss` claim).
- Required roles from the `authorize` expression.

### Attribute Reference

Macro usage format:

```rust
#[secured(method = "...", path = "...", authorize = "...")]
```

#### **`method`**

Defines the HTTP method used to map the endpoint in Actix-Web.

Supported values: 

- `get`
- `post`
- `put`
- `delete`
- `head`
- `connect`
- `options`
- `trace`
- `patch`

#### **`path`**

Defines the endpoint path to be registered by Actix-Web.

Example:

```rust
path = "/v1/user/{id}"
```

#### **`authorize`**

Defines the required role rule that must be satisfied by roles present in the JWT.

Supported formats:

1. `Single role`: validates one role in the token.

```rust
authorize = "ROLE_ADMIN"
```

2. `hasAnyRole`: validates that at least one role in the list exists in the token.

```rust
authorize = "hasAnyRole(ROLE_ADMIN, ROLE_USER)"
```

3. `hasAllRoles`: validates that all roles in the list exist in the token.

```rust
authorize = "hasAllRoles(ROLE_ADMIN, ROLE_USER)"
```

### Examples

#### **`Single role`**

```rust
#[secured(method = "post", path = "/v1/user", authorize = "ROLE_ADMIN")]
pub async fn create_user_endpoint(...) -> HttpResponse {
    // handler body
}
```

#### **`Any role`**

```rust
#[secured(
    method = "get",
    path = "/v1/user/{id}",
    authorize = "hasAnyRole(ROLE_ADMIN, ROLE_USER)"
)]
pub async fn get_user_endpoint(...) -> HttpResponse {
    // handler body
}
```

#### **`All roles`**

```rust
#[secured(
    method = "delete",
    path = "/v1/user/{id}",
    authorize = "hasAllRoles(ROLE_ADMIN, ROLE_AUDITOR)"
)]
pub async fn delete_user_endpoint(...) -> HttpResponse {
    // handler body
}
```


## YAML-based server configuration file
The server behavior is fully driven by a YAML configuration file. This file defines network 
settings, security providers, data sources, and observability integrations used at runtime.

The configuration is loaded during application startup and applied automatically by the framework.

### Server

Defines how the HTTP service is exposed and how it interacts with the runtime environment.

| Field                 | Description                                                               |
| --------------------- | ------------------------------------------------------------------------- |
| `host`                | Network interface where the server binds.                                 |
| `port`                | Main HTTP port used by the API.                                           |
| `health-check-port`   | Dedicated port exposing the health endpoint.                              |
| `use-docker-compose`  | Enables orchestration of dependencies via Docker Compose.                 |
| `docker-compose-file` | Path to the Docker Compose definition used when orchestration is enabled. |

### CORS

Controls cross-origin access policies.

| Field                     | Description                                                        |
| ------------------------- | ------------------------------------------------------------------ |
| `max-age`                 | Duration (seconds) browsers cache preflight responses.             |
| `allow-credentials`       | Allows cookies and authorization headers in cross-origin requests. |
| `allowed-methods`         | HTTP methods allowed for cross-origin calls.                       |
| `allowed-headers`         | Headers accepted from clients.                                     |
| `allowed-origins_pattern` | Comma-separated list of allowed origin patterns.                   |

### Security — OAuth2 / OpenID Connect

Enables authentication and token validation using an OAuth2 provider.

| Field                     | Description                                                        |
| ------------------------- | ------------------------------------------------------------------ |
| `enabled`                 | Activates OAuth2 protection for secured endpoints.                 |
| `load-from-discovery-url` | Automatically loads provider metadata from the discovery endpoint. |
| `discovery-url`           | OpenID Provider discovery document.                                |
| `issuer-uri`              | Expected token issuer identifier.                                  |
| `jwks-uri`                | JSON Web Key Set endpoint used to validate tokens.                 |
| `token-uri`               | Endpoint for obtaining access tokens.                              |
| `authorization-uri`       | Authorization endpoint for login flows.                            |
| `introspection-uri`       | Endpoint for validating opaque tokens.                             |
| `user_info-uri`           | Endpoint returning authenticated user claims.                      |
| `end_session-uri`         | Logout endpoint for session termination.                           |

### OAuth2 Client

Credentials used by the server when interacting with the identity provider.

| Field    | Description                             |
| -------- | --------------------------------------- |
| `id`     | OAuth2 client identifier.               |
| `secret` | OAuth2 client secret.                   |
| `scope`  | Requested scopes during authentication. |


### JWKS

Defines local JSON Web Keys used for token signing or validation.

Each key entry contains:

- kid — Key identifier
- kty — Key type
- alg — Signing algorithm
- use — Key usage
- e — Public exponent
- n — RSA modulus
- x5c — X.509 certificate chain

This section is typically used when keys are managed internally or cached locally.

### Data Sources

### *Redis*

Configuration for distributed cache and key-value storage.

| Field                  | Description                                     |
| ---------------------- | ----------------------------------------------- |
| `enabled`              | Enables Redis integration.                      |
| `host` / `port`        | Connection settings.                            |
| `client-type`          | Redis client implementation.                    |
| `lettuce.pool`         | Connection pool configuration.                  |
| `repositories.enabled` | Enables repository abstraction backed by Redis. |


### *Relational Databases*

Defines a list of database connections used by the application.

Each database entry supports:

- Connection pooling configuration
- Timeouts and lifecycle settings
- SQL logging control
- Independent enable/disable toggle

This allows multiple data sources (e.g., API DB, job processing DB) to coexist in the same runtime.

| Field            | Description                                                                                       |
| ---------------- | ------------------------------------------------------------------------------------------------- |
| `name`           | Logical name of the database connection used by the server.                                       |
| `enabled`        | Enables or disables this database configuration. When `false`, the connection is not initialized. |
| `url`            | Database connection string used to establish the connection.                                      |
| `min-pool-size`  | Minimum number of connections maintained in the pool.                                             |
| `max-pool-size`  | Maximum number of connections allowed in the pool.                                                |
| `logging`        | Enables query and connection logging for this database.                                           |
| `aquire-timeout` | Maximum time (in seconds) to wait when acquiring a connection from the pool.                      |
| `max-lifetime`   | Maximum lifetime (in minutes) of                                                                  |

> Important: The framework currently supports only SQLite, PostgreSQL, MySQL, MariaDB, and 
> Microsoft SQL Server databases.

### *BigQuery Database Connection*

This section defines the configuration parameters required to establish a secure connection
to Google BigQuery, the fully managed data warehouse provided by Google. These settings allow
the server to authenticate, select the target project and dataset, and control execution
behavior for queries. .

| Field          | Description                                 |
| -------------- | ------------------------------------------- |
| `enabled`      | Enables BigQuery access.                    |
| `print-tables` | Logs available tables during startup.       |
| `region`       | Dataset region.                             |
| `project`      | Google Cloud project identifier.            |
| `credential`   | Base64-encoded service account credentials. |
| `dataset`      | List of datasets used by the application.   |


### Metrics

Controls application observability and monitoring integration.

| Field      | Description                             |
| ---------- | --------------------------------------- |
| `enabled`  | Enables metrics collection.             |
| `app-name` | Identifier used when exporting metrics. |


### Runtime Notes

- Disabled components remain configured but inactive.
- Secrets should be externalized in production environments.
- Configuration values can be overridden via environment variables or CLI parameters.
- The configuration is validated during server startup.

## 🔧 Development Setup

```bash
# Clone the repository
git clone https://github.com/kenniston/rust-microservice
cd rust-microservice

# Build the server example
cargo build -p server

# Run the server example
cargo run -p server

# Format code
cargo fmt

# Check for issues
cargo clippy
```

## 🔬 Test Environment Infrastructure

The framework provides utilities for bootstrapping and managing an isolated
integration test environment powered by Docker containers and an async
server runtime.

It is designed to:

- Provision and manage test containers (e.g., Postgres, Keycloak);
- Coordinate initialization and shutdown across threads;
- Provide global access to running containers;
- Ensure deterministic teardown after tests complete.

The module is intended for integration and end-to-end testing scenarios
where external dependencies must be provisioned dynamically.

### Architecture Overview

The test environment follows a controlled lifecycle with three main phases:

1. **Setup Phase**
  - Initializes logging
  - Starts required containers
  - Executes optional post-initialization tasks
  - Signals readiness to the test runtime

2. **Execution Phase**
  - Tests run against the live server and provisioned services
  - Containers remain active and globally accessible

3. **Teardown Phase**
  - Receives shutdown signal
  - Stops and removes all registered containers
  - Releases global resources

Synchronization between phases is handled using global channels and locks.

### Blocking Execution Helper

The module provides an internal utility for executing async code from
synchronous contexts by creating a dedicated Tokio runtime. This is used
primarily during container shutdown and cleanup.

---

### Container Utilities

The `containers` submodule provides helpers for starting commonly used
infrastructure services.

#### Supported Services

- **Postgres**
  Starts a database container with optional initialization scripts,
  network configuration, and credentials.

- **Keycloak**
  Starts an identity provider container with realm import support and
  readiness checks.

Each container:

- Waits for readiness before returning
- Registers itself for automatic teardown
- Returns a connection URI for test usage

---

### Usage

```rust
#[ctor::ctor]
pub fn setup() {
    rust_microservice::test::setup(
        async || {
            let mut settings = load_test_settings();

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
 ```

Teardown is handled automatically by the `dtor` attribute.
```rust
#[ctor::dtor]
pub fn teardown() {
    rust_microservice::test::teardown();
}
```


### Concurrency Model

The environment runs inside a dedicated multi-thread Tokio runtime
spawned in a background thread. This allows synchronous test code to
coordinate with async infrastructure without requiring async test
functions.

Communication is performed via channels that coordinate:

- Initialization completion
- Container stop commands
- Shutdown confirmation


### Safety Guarantees

- Containers remain alive for the full test lifecycle
- Teardown is deterministic and blocking
- Global state is initialized exactly once
- Async resources are properly awaited before shutdown

>The environment assumes Docker is available and reachable using default
>configuration. Failure to connect to the Docker daemon will cause setup
>to abort.

>All containers are forcefully removed during teardown to ensure a clean
>test environment for subsequent runs.


## 📦 Project Dependencies

This section describes the common dependencies used by projects built with the Rust 
Microservice Framework. These libraries provide the foundation for web handling, 
serialization, asynchronous execution, database access, observability, and API 
documentation.

The dependencies listed below represent a typical setup and may be adjusted according 
to project requirements.

- `rust-microservice` – Core framework providing the microservice structure and shared components.
- `rust-embed` – Embeds static assets directly into the binary.
- `actix-web` – High-performance asynchronous web framework.
- `serde` – Serialization and deserialization framework.
- `serde_json` – JSON support for Serde.
- `tokio` – Asynchronous runtime for non-blocking operations.
- `sea-orm` – Async and dynamic ORM for database access.
- `utoipa` – OpenAPI specification generation.
- `utoipa-swagger-ui` *(actix-web feature)* – Embedded Swagger UI for API exploration.
- `google-cloud-bigquery` – Client library for Google BigQuery integration.
- `base64` – Base64 encoding and decoding utilities.
- `tracing` – Structured, event-based logging and diagnostics.
- `thiserror` – Ergonomic error definitions.
- `derive_more` – Additional derive macros to reduce boilerplate.
- `colored` – Colored terminal output.
- `env_logger` – Environment-based logger implementation.
- `log` *(serde feature)* – Logging facade with serialization support.
- `reqwest` – HTTP client for outbound requests.
- `reqwest-tracing` *(optional)* – Tracing instrumentation for HTTP client requests.
- `reqwest-middleware` *(optional)* – Middleware support for Reqwest clients.


## 📄 License
  
Licensed under either of

- Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.


## 🤝 Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

Contributions are welcome! Join us — let’s build the future of Rust together.

Here's how to get started:

1. **🍴 Fork** the repository
2. **🔧 Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **💾 Commit** your changes (`git commit -m 'Add amazing feature'`)
4. **📤 Push** to the branch (`git push origin feature/amazing-feature`)
5. **🔀 Open** a Pull Request


## 📝 Changelog
  
  👉 See [CHANGELOG.md](CHANGELOG.md) for version history and updates.
