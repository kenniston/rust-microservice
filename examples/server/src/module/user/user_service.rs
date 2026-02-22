//! User Service Module
//!
//! This module provides user-related operations such as user creation, update, deletion,
//! and retrieval.
//! It uses the SeaORM library to interact with the database and provides error handling
//! and conversion functions to map between the `user::ActiveModel` and `UserDTO` structures.
//!
//! # Functions
//!
//! - `create_user`: Creates a new user in the database.
//! - `update_user`: Updates an existing user in the database.
//! - `delete_user`: Deletes a user from the database.
//! - `find_user_by_id`: Retrieves a user by its ID.
//!
//! # Errors
//!
//! The module defines a set of error codes for user-related operations,
//! such as user creation, update, deletion, and retrieval. The error codes
//! are defined in the `UserErros` enumeration.

use crate::dto::user::UserDTO;
use crate::module::user::Result;
use crate::module::user::user_repository as repository;

/// Creates a new user in the database.
///
/// This function takes a `UserDTO` object and attempts to save it to the database.
/// If the user already exists, a `UserExists` error is returned.
/// If the user is successfully created, a `UserDTO` object is returned containing the newly
/// created user's information.
///
/// # Errors
///
/// The following errors are possible:
///
/// - `UserConversionError`: The conversion of the `UserDTO` into a `user::ActiveModel` failed.
/// - `UserExists`: The user with the given name and email already exists in the database.
/// - `UserCreateError`: An error occurred while attempting to create the user.
pub async fn create_user(user: UserDTO) -> Result<UserDTO> {
    repository::create_user(user).await
}

/// Updates an existing user in the database.
///
/// This function takes a `UserDTO` object and attempts to save it to the database.
/// If the user does not exist, a `UserNotFound` error is returned.
/// If the user is successfully updated, a `UserDTO` object is returned containing the newly
/// updated user's information.
///
/// # Errors
///
/// The following errors are possible:
///
/// - `UserConversionError`: The conversion of the `UserDTO` into a `user::ActiveModel` failed.
/// - `UserNotFound`: The user with the given ID does not exist.
/// - `UserUpdateError`: An error occurred while attempting to update the user.
pub async fn update_user(user: UserDTO) -> Result<UserDTO> {
    repository::update_user(user).await
}

/// Deletes a user from the database.
///
/// This function takes a user ID and attempts to delete the corresponding user from the database.
/// If the user does not exist, a `UserNotFound` error is returned.
/// If the user is successfully deleted, a `Result` object is returned indicating success.
///
/// # Errors
///
/// The following errors are possible:
///
/// - `UserNotFound`: The user with the given ID does not exist.
/// - `UserDeleteError`: An error occurred while attempting to delete the user.
pub async fn delete_user(user_id: i32) -> Result<u64> {
    repository::delete_user(user_id).await
}

/// Retrieves the list of all users from the database.
///
/// # Errors
///
/// Returns a `UserListError` if an error occurs while attempting to retrieve the list of users.
///
/// # Panics
///
/// Panics if the database is not configured.
pub async fn all_user(name: String) -> Result<Vec<UserDTO>> {
    repository::all_user(name).await
}

/// Retrieves a user by its ID.
///
/// This function takes a user ID and attempts to retrieve the corresponding user from
/// the database.
/// If the user does not exist, a `UserNotFound` error is returned.
/// If the user is successfully retrieved, a `UserDTO` object is returned containing the
/// retrieved user's information.
///
/// # Errors
///
/// The following errors are possible:
///
/// - `UserNotFound`: The user with the given ID does not exist.
/// - `UserGetError`: An error occurred while attempting to retrieve the user.
pub async fn find_user_by_id(user_id: i32) -> Result<UserDTO> {
    repository::get_user_by_id(user_id).await
}
