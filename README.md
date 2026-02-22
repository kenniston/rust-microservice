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


# Microservice Framework <!-- omit from toc -->

A Rust crate whichs provides a framework for building microservices. It follows the MVC 
(Model-View-Controller) architecture pattern and provides a strong focus on high performance, 
security, and scalability.

## 📋 Table of Contents <!-- omit from toc -->

- [� Overview](#-overview)
- [✨ Features](#-features)
- [🛠️ Installation](#️-installation)
- [⚡ Quick Start](#-quick-start)
- [💡 Usage Examples](#-usage-examples)
- [🔧 Development Setup](#-development-setup)
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
  and business logic
- High performance HTTP server with support for asynchronous request handling
- Strong focus on security and scalability
- Support for JSON and XML serialization/deserialization
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
discovers all actix-web handlers defined in the `controllers_path` attribute and initializes the 
databases specified in the YAML configuration file.

The configuration file can be provided to the framework using the `--config-file` command-line 
parameter (or the `CONFIG_FILE` environment variable), or as a Base64-encoded file via the 
`--b64-config-file` parameter (or the `B64_CONFIG_FILE` environment variable).

The ServerApi macro attributes also allow customization of the OpenAPI metadata and the 
server banner.

```rust
pub mod module;

use rust_microservice::ServerApi;

#[ServerApi(
    controllers_path = "src/module, src/controllers",
    openapi_title = "🌐 Rest API Server",
    openapi_api_description = "Rest API OpenApi Documentation built with Rust 🦀.",
    database = "true",
    banner = r#"
            _~^~^~_         ___    ___   ____    ____
        \) /  o o  \ (/    / _ |  / _ \ /  _/   / __/___  ____ _  __ ___  ____
          '_   -   _'     / __ | / ___/_/ /    _\ \ / -_)/ __/| |/ // -_)/ __/
          / '-----' \    /_/ |_|/_/   /___/   /___/ \__//_/   |___/ \__//_/
    "#
)]
fn main() -> rust_microservice::Result<(), String> {}

```

## 💡 Usage Examples


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
