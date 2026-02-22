# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0] - Initial Release

### Added

#### Configuration & Core Server
- YAML-based configuration via `config.yaml`;
- Built-in API documentation with OpenAPI and interactive UI through Swagger UI powered by Utoipa Swagger UI;
- Built-in Health Check endpoint;
- Configurable server and actuator ports (defaults: `8080` and `7188`);
- Built-in CORS configuration support;
- MVC-based project architecture;
- Built-in metrics export compatible with Prometheus;
- Automatic database connectivity (SQLite, PostgreSQL, MySQL, and Google BigQuery) driven by YAML configuration;
- Local development and integration testing with Docker Compose and Testcontainers;
- Custom dashboard panel for metrics visualization in Grafana;
- CI/CD pipeline configured in GitLab;
- Integration test wrapper for initializing a global Testcontainers environment;
- Static code analysis via SonarQube;
- Multi-stage containerization: a slim image based on `BusyBox` with UPX compression, and a full-featured image based on `Alpine Linux` including network tools and the Nano editor.

#### Development Environment (example project)
- Docker Compose file to start the CI/CD environment containers: GitLab, GitLab Runner, Nexus Repository, and SonarQube;
- Docker Compose file to start runtime service containers: Keycloak, PostgreSQL, Mailpit, Grafana, Grafana Tempo, Prometheus, and Loki used during server execution;
- GitLab Runner configuration for integration with the Server project pipeline;
- Local Kubernetes setup using `K3D`;
- Kubernetes configuration manifest for the example Server project;
- Integration settings for the example Server project with SonarQube (`sonar-project.properties`).

#### Security
- Built-in integration with OAuth2 servers (Keycloak on the server example);
- Automatic discovery URL resolution for:
  issuer, jwks, token, authorization, introspection, user-info, and end-session endpoints;
- Endpoint-level security configuration using Rust macros.

Example:

```rust
#[utoipa::path(
    post,
    path = "/v1/user",
    tag = "Endpoint for creating a new user",
    responses(
        (status = 200, description= "The data structure representing a newly created user.", body = UserDTO),
        (status = 400, description= "The data structure representing an error message.", body = ResponseDTO),
    )
)]
#[secured(method = "post", path = "/v1/user", authorize = "ROLE_ADMIN")]
pub async fn create_user_endpoint(user: web::Json<UserDTO>) -> HttpResponse {
  ...
}