#![allow(unused)]

pub mod user_controller;
pub mod user_repository;
pub mod user_service;

use thiserror::Error;

/// A type alias for a `Result` with the `UserError` error type.
pub type Result<T, E = UserError> = std::result::Result<T, E>;

/// Errors related to user operations.
///
/// This enum groups all user-domain errors in a single type, making error
/// handling consistent across the application. Each variant represents
/// a specific failure scenario and provides a human-readable error message.
///
/// # Variants
/// - `DatabaseNotConfigured`: The database connection or configuration is missing.
/// - `Conversion`: Failed to convert a user model or DTO.
/// - `NotFound`: The requested user does not exist.
/// - `AlreadyExists`: The user already exists.
/// - `Create`: An error occurred while creating a user.
/// - `Update`: An error occurred while updating a user.
/// - `Delete`: An error occurred while deleting a user.
/// - `List`: An error occurred while listing users.
/// - `Get`: An error occurred while retrieving a user.
/// - `InvalidUserId`: The provided user ID is not valid.
#[derive(Debug, Error)]
pub enum UserError {
    /// Database is not configured or initialized.
    #[error("Database is not configured")]
    DatabaseNotConfigured,

    /// Database connection error.
    #[error("Database error: {0}")]
    Database(String),

    /// Error during user conversion (e.g., model to DTO).
    #[error("User conversion error: {0}")]
    Conversion(String),

    /// User was not found.
    #[error("User not found")]
    NotFound,

    /// User already exists.
    #[error("User already exists")]
    AlreadyExists,

    /// Error while creating a user.
    #[error("Error creating user: {0}")]
    Create(String),

    /// Error while updating a user.
    #[error("Error updating user: {0}")]
    Update(String),

    /// Error while deleting a user.
    #[error("Error deleting user: {0}")]
    Delete(String),

    /// Error while listing users.
    #[error("Error listing users: {0}")]
    List(String),

    /// Error while retrieving a user.
    #[error("Error retrieving user: {0}")]
    Get(String),

    /// User ID is not valid.
    #[error("User ID is not valid")]
    InvalidUserId,
}

impl UserError {
    /// Returns a numeric error code associated with the error.
    ///
    /// These codes can be used for API responses, logs, or metrics,
    /// providing a stable and machine-readable way to identify errors.
    ///
    /// # Error Codes
    /// - `1000`: Conversion error
    /// - `1001`: User not found
    /// - `1002`: User already exists
    /// - `1003`: Update error
    /// - `1004`: Delete error
    /// - `1005`: Create error
    /// - `1006`: Get error
    /// - `1007`: List error
    /// - `1008`: Invalid user ID
    /// - `1500`: Database not configured
    /// - `1501`: Custom database error
    pub fn code(&self) -> u32 {
        match self {
            Self::Conversion(_) => 1000,
            Self::NotFound => 1001,
            Self::AlreadyExists => 1002,
            Self::Update(_) => 1003,
            Self::Delete(_) => 1004,
            Self::Create(_) => 1005,
            Self::Get(_) => 1006,
            Self::List(_) => 1007,
            Self::InvalidUserId => 1008,
            Self::DatabaseNotConfigured => 1500,
            Self::Database(_) => 1501,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::UserError;

    #[test]
    fn user_error_code_should_return_expected_values() {
        assert_eq!(UserError::Conversion("err".into()).code(), 1000);
        assert_eq!(UserError::NotFound.code(), 1001);
        assert_eq!(UserError::AlreadyExists.code(), 1002);
        assert_eq!(UserError::Update("err".into()).code(), 1003);
        assert_eq!(UserError::Delete("err".into()).code(), 1004);
        assert_eq!(UserError::Create("err".into()).code(), 1005);
        assert_eq!(UserError::Get("err".into()).code(), 1006);
        assert_eq!(UserError::List("err".into()).code(), 1007);
        assert_eq!(UserError::InvalidUserId.code(), 1008);
        assert_eq!(UserError::DatabaseNotConfigured.code(), 1500);
        assert_eq!(UserError::Database("err".into()).code(), 1501);
    }
}
