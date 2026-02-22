//! Module providing database operations for user-related entities.
//!
//! This module contains database operations for creating, updating, deleting, and retrieving
//! user-related entities. It uses the SeaORM library to interact with the database.
//!
//! The module provides error handling and conversion functions to map between
//! the `user::ActiveModel` and `UserDTO` structures.
//!
//! # Functions
//!
//! - `create_user`: Creates a new user in the database.
//! - `update_user`: Updates an existing user in the database.
//! - `delete_user`: Deletes a user from the database.
//! - `all_user`: Retrieves all users from the database.
//!
//! # Errors
//!
//! The module defines a set of error codes for user-related operations,
//! such as user creation, update, deletion, and retrieval. The error codes
//! are defined in the `UserErros` enumeration.
//!
//! # Conversions
//!
//! The module provides conversion functions to map between the `user::ActiveModel`
//! and `UserDTO` structures. The conversion functions are defined in the
//! `try_from` and `try_into` functions.

use std::sync::Arc;

use crate::module::user::{Result, UserError};
use google_cloud_bigquery::client::google_cloud_auth::token;
use rust_microservice::{Server, database};
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::Set,
    ColumnTrait, DatabaseConnection, DbBackend, EntityTrait, ExprTrait, QueryFilter, QueryTrait,
    SqlErr,
    prelude::Expr,
    sea_query::{Func, extension::postgres::PgExpr},
};
use tracing::info;

use crate::{dto::user::UserDTO, entity::user};

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
#[database(name = "api", error = "UserError::DatabaseNotConfigured")]
pub async fn create_user(dto: UserDTO) -> Result<UserDTO> {
    let model = user::ActiveModel::from(dto);

    db.ping()
        .await
        .expect("CONNECTION DROPPED!!!!!!!!!!!!!!!!!!!!");

    let saved = model.save(&db).await.map_err(|e| match e.sql_err() {
        Some(SqlErr::UniqueConstraintViolation(_)) => UserError::AlreadyExists,
        _ => UserError::Create(e.to_string()),
    })?;

    UserDTO::try_from(saved).map_err(|e| UserError::Conversion(e.to_string()))
}

/// Updates an existing user in the database.
///
/// This function takes a `UserDTO` object and attempts to update the corresponding user
/// in the database.
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
#[database(name = "api", error = "UserError::DatabaseNotConfigured")]
pub async fn update_user(dto: UserDTO) -> Result<UserDTO> {
    let id = dto.id.ok_or(UserError::InvalidUserId)?;

    db.ping()
        .await
        .expect("CONNECTION DROPPED!!!!!!!!!!!!!!!!!!!!");

    let model = user::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|e| UserError::Update(e.to_string()))?
        .ok_or(UserError::NotFound)?;

    let mut active: user::ActiveModel = model.into();

    if let Some(name) = dto.name {
        active.name = Set(name);
    }
    if let Some(email) = dto.email {
        active.email = Set(email);
    }

    active
        .clone()
        .update(&db)
        .await
        .map_err(|e| UserError::Update(e.to_string()))?;

    UserDTO::try_from(active).map_err(|e| UserError::Conversion(e.to_string()))
}

/// Deletes a user from the database.
///
/// This function takes a user ID and attempts to delete the corresponding user from
/// the database.
/// If the user does not exist, a `UserNotFound` error is returned.
/// If the user is successfully deleted, a `Result` object is returned indicating success.
///
/// # Errors
///
/// The following errors are possible:
///
/// - `UserNotFound`: The user with the given ID does not exist.
/// - `UserDeleteError`: An error occurred while attempting to delete the user.
#[database(name = "api", error = "UserError::DatabaseNotConfigured")]
pub async fn delete_user(user_id: i32) -> Result<u64> {
    db.ping()
        .await
        .expect("CONNECTION DROPPED!!!!!!!!!!!!!!!!!!!!");

    let result = user::Entity::delete_by_id(user_id)
        .exec(&db)
        .await
        .map_err(|e| UserError::Delete(e.to_string()))?;

    Ok(result.rows_affected)
}

/// Retrieves all users from the database with names that match the given name.
///
/// # Parameters
///
/// - `name`: The name to search for in the database.
///
/// # Errors
///
/// The following errors are possible:
///
/// - `UserListError`: An error occurred while attempting to retrieve the list of users.
///
/// # Returns
///
/// A `Result` containing a vector of `UserDTO` objects, representing the users in the system.
#[database(name = "api", error = "UserError::DatabaseNotConfigured")]
pub async fn all_user(name: String) -> Result<Vec<UserDTO>> {
    db.ping()
        .await
        .expect("CONNECTION DROPPED!!!!!!!!!!!!!!!!!!!!");

    let users = user::Entity::find()
        .filter(
            sea_orm::Condition::all().add(
                Expr::expr(Func::lower(Expr::col(user::Column::Name)))
                    .like(format!("%{}%", name.to_lowercase())),
            ),
        )
        .all(&db)
        .await
        .map_err(|e| UserError::List(e.to_string()))?;

    Ok(users.into_iter().map(Into::into).collect())
}

/// Retrieves a user by its ID from the database.
///
/// # Parameters
///
/// - `user_id`: The ID of the user to be retrieved.
///
/// # Errors
///
/// The following errors are possible:
///
/// - `UserGetError`: An error occurred while attempting to retrieve the user.
/// - `UserNotFound`: The user with the given ID does not exist.
///
/// # Returns
///
/// A `Result` containing a `UserDTO` object, representing the retrieved user.
#[database(name = "api", error = "UserError::DatabaseNotConfigured")]
pub async fn get_user_by_id(user_id: i32) -> Result<UserDTO> {
    db.ping()
        .await
        .expect("CONNECTION DROPPED!!!!!!!!!!!!!!!!!!!!");

    user::Entity::find_by_id(user_id)
        .one(&db)
        .await
        .map_err(|_| UserError::NotFound)?
        .ok_or(UserError::NotFound)
        .map(Into::into)
}
