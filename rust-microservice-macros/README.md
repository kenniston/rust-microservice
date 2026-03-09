# 🚀 Rust Microservice Macros

A procedural macro crate designed to power the
[`rust_microservice`](https://crates.io/crates/rust_microservice)
ecosystem with compile-time server generation, automatic controller
discovery, OpenAPI integration, authentication enforcement, and
database injection.

This crate eliminates runtime registration patterns by generating
deterministic, compile-time server bootstrap logic.

---

# 🎯 Design Goals

- ✅ Zero runtime reflection
- ✅ Compile-time controller discovery
- ✅ Deterministic OpenAPI generation
- ✅ Integrated JWT security middleware
- ✅ Declarative database injection
- ✅ Strict compile-time validation

All routing, OpenAPI metadata, middleware wrapping, and database
bindings are generated at compile time using Rust’s procedural macro
system.

---

# 🏗️ Architecture Overview

This crate is implemented using:

- `proc_macro`
- `proc_macro2`
- `syn` (AST parsing)
- `quote` (token generation)
- `walkdir` (controller discovery)

## Macro Expansion Pipeline

1. Parse attribute arguments (`key = value` pairs)
2. Parse annotated Rust items (`ItemFn`, modules, etc.)
3. Load and inspect controller files
4. Extract Actix-Web handlers
5. Generate:
   - Server bootstrap
   - Route registration
   - Swagger/OpenAPI specification
   - JWT middleware wrappers
   - Database injection logic

No runtime route aggregation occurs — all handlers are resolved
during compilation.

---

# 🧩 Provided Macros

This crate exposes three primary procedural attribute macros:

- `#[api_server]`
- `#[secured]`
- `#[database]`

---

# 🌐 `#[api_server]`

Generates the full HTTP server bootstrap and controller registration
logic for an `actix-web` application.

## Responsibilities

- Recursively scans controller directories
- Registers all HTTP handlers
- Generates Swagger UI configuration
- Generates OpenAPI documentation using `utoipa`
- Optionally initializes database connections
- Wraps the main function with `#[tokio::main]`
- Initializes and runs the global `Server`

## Supported Attributes

| Attribute                 | Type               | Description                                         |
| ------------------------- | ------------------ | --------------------------------------------------- |
| `controllers_path`        | `&str`             | Comma-separated directories containing controllers  |
| `openapi_title`           | `&str`             | OpenAPI title                                       |
| `openapi_api_name`        | `&str`             | OpenAPI tag name                                    |
| `openapi_api_description` | `&str`             | OpenAPI tag description                             |
| `openapi_auth_server`     | `&str`             | OAuth2 token URL fallback                           |
| `database`                | `"true" / "false"` | Enables SeaORM database initialization              |
| `banner`                  | `&str`             | Startup banner printed during server initialization |

## Example

```rust,ignore
use rust_microservice::ServerApi; // api_server was renamed to ServerApi for better ergonomics

#[ServerApi(
    controllers_path = "src/controllers",
    openapi_title = "🌍 My API",
    openapi_api_description = "Example API",
    database = "true"
)]
async fn start() -> rust_microservice::Result<(), String> {}
```

---

⚠️ `IMPORTANT`: The `server_api` (*ServerApi in rust_microservice*), `database` and `secured`
macros has been renamed or re-exported to improve ergonomics in the rust_microservice crate.
This crate provides only the macro implementation. The public API is re-exported by the
rust_microservice crate. Therefore, when using the macro in your project, prefer ServerApi
instead of api_server.

---

## Generated Behavior

- Wraps your function with `#[tokio::main]`
- Discovers all Actix-Web handlers
- Generates:
  - `register_endpoints`
  - `ApiDoc` (`utoipa::OpenApi`)
  - Swagger UI endpoint `/swagger-ui/*`

---

# 🔐 `#[secured]`

Protects an Actix-Web endpoint with JWT authentication and
role-based authorization.

Internally generates:

- A middleware module
- A wrapper using `actix_web::middleware::from_fn`
- Automatic role validation via `Server::validate_jwt`

## Supported Attributes

| Attribute   | Description                       |
| ----------- | --------------------------------- |
| `method`    | HTTP method (`get`, `post`, etc.) |
| `path`      | Route path                        |
| `authorize` | Role expression                   |

## Authorization Formats

### Single Role

```text
authorize = "ROLE_ADMIN"
```

### Any Role

```text
authorize = "hasAnyRole(ROLE_ADMIN, ROLE_USER)"
```

### All Roles

```text
authorize = "hasAllRoles(ROLE_ADMIN, ROLE_AUDITOR)"
```

## Example

```rust,ignore
use rust_microservice::secured;
use actix_web::HttpResponse;

#[secured(
    method = "get",
    path = "/v1/users",
    authorize = "hasAnyRole(ROLE_ADMIN, ROLE_USER)"
)]
pub async fn list_users() -> HttpResponse {
    HttpResponse::Ok().finish()
}
```

## Security Validation

The middleware validates:

- JWT presence
- Signature
- Expiration (`exp`)
- Issuer (`iss`)
- Required roles

If validation fails → `401 Unauthorized`.

---

# 🛢️ `#[database]`

Injects a SeaORM `DatabaseConnection` into a repository function.

## Required Attributes

| Attribute | Description                                         |
| --------- | --------------------------------------------------- |
| `name`    | Database configuration name                         |
| `error`   | Error variant returned if connection is unavailable |

The macro injects:

```rust,ignore
let db = Server::global()
    .database_with_name("name")?;
```

## Example

```rust,ignore
use rust_microservice::database;

#[database(name = "api", error = "UserError::DatabaseNotConfigured")]
pub async fn find_user(id: i32) -> Result<()> {
    // `db` is available here
    Ok(())
}
```

---

# 🔎 Controller Discovery

The `api_server` macro:

- Traverses `controllers_path`
- Parses each `.rs` file using `syn`
- Extracts functions annotated with:

```text
#[get]
#[post]
#[put]
#[delete]
#[patch]
#[head]
#[options]
#[trace]
#[connect]
#[secured]
```

These handlers are automatically registered into
`actix_web::web::ServiceConfig`.

---

# 📄 OpenAPI Generation

Uses `utoipa` to generate:

- `#[derive(OpenApi)]`
- Swagger UI configuration
- OAuth2 security scheme
- Global security requirements

The security scheme is dynamically configured from:

```rust,ignore
Server::global()?.settings().get_auth2_token_url()
```

---

# ⚙️ Internal Utility Structures

### `KeyValue`

Parses:

```text
key = value
```

### `ArgList`

Parses:

```text
key1 = value1, key2 = value2
```

These power all attribute parsing in this crate.

---

# 🧠 Compile-Time Guarantees

- Controllers must be valid Rust modules
- Handlers must use supported HTTP attributes
- Database names must exist at runtime
- Invalid macro parameters cause compile errors

---

# 🧪 Runtime Integration

Although this crate generates compile-time code,
runtime behavior depends on:

- `actix-web`
- `tokio`
- `utoipa`
- `sea-orm`
- `rust_microservice::Server`

---

# 📌 Summary

This macro crate transforms a modular Rust project into a fully
initialized HTTP API server with:

- Automatic route wiring
- JWT security enforcement
- OpenAPI documentation
- Swagger UI
- Database injection

All achieved with minimal boilerplate and strict compile-time guarantees.

---

🦀 Built for high-performance Rust microservices.
Deterministic. Secure. Compile-time powered.
