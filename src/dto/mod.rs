pub mod user;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Data Transfer Object (DTO) used as a standard response for user-related operations.
///
/// This structure represents a simple API response containing a status code
/// and a human-readable message. It is typically returned by REST endpoints
/// to indicate the result of an operation.
///
/// # Fields
///
/// * `status` - Numeric status code representing the result of the operation
///   (e.g., HTTP status or application-specific code).
/// * `message` - Human-readable message describing the operation result.
#[derive(Serialize, Deserialize, ToSchema, Default)]
pub struct ResponseDTO {
    /// Code of the response.
    pub code: u32,

    /// Message describing the response.
    pub message: String,
}
