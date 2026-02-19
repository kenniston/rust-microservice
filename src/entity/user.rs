//! User entity module.
//!
//! This module defines the **User** database entity using **SeaORM**.
//! It maps the `user` table to Rust structures, enabling type-safe
//! database operations such as queries, inserts, updates, and deletes.
//!
//! The entity supports serialization and deserialization via `serde`,
//! making it suitable for use in APIs and data transfer layers.

use sea_orm::{ActiveValue::Set, entity::prelude::*, sqlx::types::chrono};
use serde::{Deserialize, Serialize};

use crate::dto::user;

/// User model representing a row in the `user` table.
///
/// This struct is automatically mapped to the database table using
/// SeaORM's `DeriveEntityModel` macro.
///
/// # Fields
///
/// - `id`: Primary key identifier of the user.
/// - `name`: Full name of the user.
/// - `email`: Email address of the user.
/// - `created_at`: Timestamp indicating when the record was created.
/// - `updated_at`: Timestamp indicating when the record was last updated.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    /// Primary key of the user.
    #[sea_orm(primary_key)]
    pub id: i32,

    /// User's display name.
    pub name: String,

    /// User's email address.
    pub email: String,

    /// Timestamp when the user was created.
    pub created_at: DateTime,

    /// Timestamp of the last update.
    pub updated_at: DateTime,
}

/// Enumeration of entity relationships.
///
/// This entity currently does not define any relationships,
/// but this enum is required by SeaORM and can be extended
/// in the future to include relations such as `has_many` or `belongs_to`.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

/// Custom behavior for the ActiveModel.
///
/// This implementation uses the default SeaORM behavior.
/// Override methods here if you need hooks such as validation
/// or custom logic before insert/update operations.
impl ActiveModelBehavior for ActiveModel {}

/// Converts a `user::UserDTO` into a SeaORM `ActiveModel`.
///
/// This implementation performs a direct mapping of the DTO fields to the
/// corresponding entity fields, making the model ready for persistence.
/// The `created_at` and `updated_at` fields are initialized with the current
/// UTC timestamp at conversion time, while all remaining fields use their
/// default values.
impl From<user::UserDTO> for ActiveModel {
    fn from(model: user::UserDTO) -> Self {
        let now = chrono::Utc::now().naive_utc();
        Self {
            name: Set(model.name.unwrap_or_default()),
            email: Set(model.email.unwrap_or_default()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
    }
}
