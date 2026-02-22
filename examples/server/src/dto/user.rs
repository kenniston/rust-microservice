//! Data Transfer Objects (DTOs) for user-related operations.
//!
//! This module defines structures used to transfer user data between
//! application layers and to expose user schemas in the API documentation.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::entity::user;
use crate::module::user::{Result, UserError};

/// Represents a user data transfer object.
///
/// `UserDTO` is used to exchange basic user information such as
/// name and email between the API and its consumers.
///
/// # Fields
///
/// - `id`: The optional user's ID in the database.
/// - `name`: The user's full name.
/// - `email`: The user's email address.
///
/// # Usage
///
/// This struct supports:
/// - Serialization and deserialization via `serde`.
/// - Automatic OpenAPI schema generation via `utoipa`.
#[derive(Serialize, Deserialize, ToSchema, Default)]
pub struct UserDTO {
    /// The user's ID in the database.
    pub id: Option<i32>,

    /// The user's full name.
    pub name: Option<String>,

    /// The user's email address.
    pub email: Option<String>,
}

/// Attempts to convert a `user::ActiveModel` into a `UserDTO`.
///
/// This implementation validates that the required `id` field is set
/// before performing the conversion. Optional fields such as `name` and
/// `email` are copied only if they are present in the source model.
///
/// # Errors
///
/// Returns a `ConversionError` if the `id` field is not set in the
/// `user::ActiveModel`.
impl TryFrom<user::ActiveModel> for UserDTO {
    type Error = UserError;

    fn try_from(model: user::ActiveModel) -> Result<UserDTO> {
        if model.id.is_not_set() {
            return Err(UserError::Conversion(
                "User model conversion error. ID is not set".to_string(),
            ));
        }

        let mut dto = UserDTO {
            id: Some(model.id.unwrap()),
            ..Default::default()
        };

        if model.name.is_set() || model.name.is_unchanged() {
            dto.name = Some(model.name.unwrap());
        }

        if model.email.is_set() || model.email.is_unchanged() {
            dto.email = Some(model.email.unwrap());
        }

        Ok(dto)
    }
}

/// Converts a reference `user::Model` into a `UserDTO`.
///
/// This implementation simply copies the `id`, `name`, and `email`
/// fields from the source model into a new `UserDTO`.
impl From<&user::Model> for UserDTO {
    fn from(model: &user::Model) -> UserDTO {
        UserDTO {
            id: Some(model.id),
            name: Some(model.name.clone()),
            email: Some(model.email.clone()),
        }
    }
}

/// Converts `user::Model` into a `UserDTO`.
///
/// This implementation simply copies the `id`, `name`, and `email`
/// fields from the source model into a new `UserDTO`.
impl From<user::Model> for UserDTO {
    fn from(model: user::Model) -> UserDTO {
        UserDTO {
            id: Some(model.id),
            name: Some(model.name),
            email: Some(model.email),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, NaiveDateTime};
    use sea_orm::ActiveValue::Set;

    #[test]
    fn test_dto_from_invalid_active_model() {
        let model = crate::entity::user::ActiveModel {
            name: Set("Alice Johnson".to_string()),
            email: Set("alice@johnson".to_string()),
            ..Default::default()
        };
        let result = crate::dto::user::UserDTO::try_from(model);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(
                e.to_string(),
                "User conversion error: User model conversion error. ID is not set".to_string()
            );
        }
    }

    #[test]
    fn test_dto_from_active_model() {
        let model = crate::entity::user::ActiveModel {
            id: Set(1),
            name: Set("Alice Johnson".to_string()),
            email: Set("alice@johnson".to_string()),
            ..Default::default()
        };
        let dto = crate::dto::user::UserDTO::try_from(model).unwrap_or_default();
        assert_eq!(Some(1), dto.id);
        assert_eq!(Some("Alice Johnson".to_string()), dto.name);
        assert_eq!(Some("alice@johnson".to_string()), dto.email);
    }

    #[test]
    fn test_dto_from_usermodel() {
        let dt: NaiveDateTime = NaiveDate::from_ymd_opt(2016, 7, 8)
            .unwrap_or_default()
            .and_hms_opt(9, 10, 11)
            .unwrap_or_default();
        let model = crate::entity::user::Model {
            id: 1,
            name: "Alice Johnson".to_string(),
            email: "alice@johnson".to_string(),
            created_at: dt,
            updated_at: dt,
        };
        let dto = crate::dto::user::UserDTO::from(model);
        assert_eq!(Some(1), dto.id);
        assert_eq!(Some("Alice Johnson".to_string()), dto.name);
        assert_eq!(Some("alice@johnson".to_string()), dto.email);
    }

    #[test]
    fn test_dto_from_reference_usermodel() {
        let dt: NaiveDateTime = NaiveDate::from_ymd_opt(2016, 7, 8)
            .unwrap_or_default()
            .and_hms_opt(9, 10, 11)
            .unwrap_or_default();
        let model = crate::entity::user::Model {
            id: 1,
            name: "Alice Johnson".to_string(),
            email: "alice@johnson".to_string(),
            created_at: dt,
            updated_at: dt,
        };
        let dto = crate::dto::user::UserDTO::from(&model);
        assert_eq!(Some(1), dto.id);
        assert_eq!(Some("Alice Johnson".to_string()), dto.name);
        assert_eq!(Some("alice@johnson".to_string()), dto.email);
    }
}
