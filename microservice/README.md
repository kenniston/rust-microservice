# Microservice Framework 

![rust](https://badgen.net/badge/Rust%20Edition/2024/red?scale=1.2) ![rust](https://badgen.net/badge/Rust/1.91.1/blue?scale=1.2) ![cargo](https://badgen.net/badge/Cargo/1.91.1/gray?scale=1.2) ![spring-boot](https://badgen.net/badge/Version/0.1.0/green?scale=1.2)

This crate provides a framework for building microservices in Rust. It follows the MVC 
(Model-View-Controller) architecture pattern and provides a strong focus on high performance, 
security, and scalability.

The framework is designed to be modular, allowing developers to register their own route 
handlers, serializers, and business logic. This enables a high degree of customization and 
flexibility, making it suitable for a wide range of use cases.

## Features

- Modular architecture with support for registering custom route handlers, serializers, 
  and business logic
- High performance HTTP server with support for asynchronous request handling
- Strong focus on security and scalability
- Support for JSON and XML serialization/deserialization
- Metrics and logging support using the `metrics` and `log` crates

## Modules

### cmd

The `cmd` module provides support for command-line parsing using the `clap` crate.
It allows developers to define custom command-line parameters and parse them into strongly typed
configuration objects.

### data

The `data` module provides support for loading configuration data from various sources
such as YAML files, environment variables, and command-line parameters.
It uses the `config` crate to provide a unified configuration system and supports loading
configuration data from multiple sources.

### http

The `http` module provides support for building high-performance HTTP servers using the
`actix-web` crate.
It provides a set of pre-built route handlers for common operations such as health checking and
version reporting.

### metrics

The `metrics` module provides support for metrics and logging using the `metrics` crate.
It allows developers to register custom metrics and log messages to track performance and
behavior of the application.

### security

The `security` module provides support for authentication and authorization using the
`oauth2` crate.
It provides a set of pre-built authentication and authorization handlers for common use cases
such as OAuth2 and JWT.

### server

The `server` module provides support for building and initializing the HTTP server.
It uses the `actix-web` crate to create a high-performance HTTP server and provides support
for registering custom route handlers and serializers.

### settings

The `settings` module provides support for loading configuration data from various sources
such as YAML files, environment variables, and command-line parameters.
It uses the `config` crate to provide a unified configuration system and supports loading
configuration data from multiple sources.

### test

The `test` module provides support for testing the application using the `actix-web` crate.
It provides a set of pre-built test helpers for common operations such as testing HTTP endpoints
and testing the application's configuration.
