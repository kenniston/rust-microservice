use serde::Serialize;
use utoipa::{OpenApi, ToSchema};

use actix_web::{HttpResponse, get, web::ServiceConfig};
use serde_json::to_string_pretty;

/// Basic health-check response model.
///
/// Returned by the `/health` endpoint to indicate the
/// operational status of the service.
#[derive(Serialize, ToSchema)]
struct HealthResponse {
    status: String,
}

/// OpenAPI documentation definition for the service.
///
/// This object aggregates exposed paths, components and
/// tags used to generate the API specification.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "🌐 API Server",
    ),
    paths(
        health
    ),
    components(
        schemas(HealthResponse)
    ),
    tags(
        (name = "⚙️ Rest API", description = "Rest API OpenApi Documentation.")
    ),
)]
pub(crate) struct HealthApiDoc;

/// Health-check endpoint for the API.
///
/// This endpoint provides a simple mechanism for external systems or load
/// balancers to verify that the server is running correctly. When invoked,
/// it returns a JSON payload indicating the application's status, along
/// with an additional custom header.
///
/// # Response
/// Returns an **HTTP 200 OK** response with:
///
/// - **Content-Type:** `application/json`  
/// - **Header:** `api-server: on-line`  
/// - **Body:** A JSON object in the form:
///
/// ```json
/// {
///   "status": "UP"
/// }
/// ```
///
/// # Usage
/// Typically used for availability checks, readiness probes, or uptime
/// monitoring systems.
///
/// # Example
/// ```text
/// GET /health
/// → 200 OK
/// → {
///     "status": "UP"
///   }
/// ```
#[utoipa::path(
    //get,
    //path = "/actuator/health",
    tag = "✅ Server Health Check",
    responses(
        (status = 200, description= "API Server Health Check Status", body = HealthResponse),       
    )
)]
#[get("/actuator/health")]
async fn health() -> HttpResponse {
    let body = to_string_pretty(&HealthResponse {
        status: "UP".to_string(),
    });
    HttpResponse::Ok()
        .content_type("application/json")
        .append_header(("api-server", "on-line"))
        .body(body.unwrap_or_default())
}

/// Configures the base server settings by registering core services.
///
/// This function adds essential endpoints required for the API to operate
/// correctly. Currently, it registers the health-check route, which provides
/// a lightweight way for monitoring systems to verify that the server is
/// running.
///
/// # Parameters
/// - `cfg`: Mutable reference to the Actix-Web [`ServiceConfig`], where
///   routes and services are registered.
///
/// # Behavior
/// - Registers the `/health` endpoint via the `health` service.
/// - Intended as the foundation for additional globally available routes.
///
/// # Notes
/// This function is typically called during the server initialization phase
/// and should contain only fundamental, globally accessible routes.
pub(crate) fn configure_server_base(cfg: &mut ServiceConfig) {
    // Add health check endpoint
    cfg.service(health);
}
