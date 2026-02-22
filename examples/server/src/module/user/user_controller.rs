use actix_web::{HttpResponse, delete, get, http::StatusCode, post, put, web};
use colored::Colorize;
use rust_microservice::{Server, secured};
use serde::Deserialize;
use serde_json::to_string_pretty;
use tracing::{debug, error, info};

use crate::dto::{ResponseDTO, user::UserDTO};
use crate::module::user::{UserError, user_service as service};

/// Endpoint for creating a new user.
///
/// This endpoint is responsible for creating a new user.
///
/// # HTTP Request
///
/// `POST /v1/user HTTP/1.1`
///
/// # Request Body
///
/// The request body should contain a valid `UserDTO` object.
///
/// # Response
///
/// The response body will contain a `UserDTO` representing the newly created user.
///
/// # Response Status Codes
///
/// The following status codes are possible:
///
/// - `200 OK`: The user was created successfully.
/// - `400 Bad Request`: The request body was invalid.
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
    let result = service::create_user(user.0).await;

    match result {
        Ok(user) => {
            let body = to_string_pretty(&user);
            HttpResponse::Ok()
                .content_type("application/json")
                .body(body.unwrap_or("User not created.".to_string()))
        }
        Err(error) => create_error_response(error.code(), error.to_string()),
    }
}

/// Endpoint for retrieving a user by its ID.
///
/// This endpoint is responsible for retrieving a user from the database by its ID.
///
/// # HTTP Request
///
/// `GET /v1/user/{id} HTTP/1.1`
///
/// # Path Parameters
///
/// - `id`: The ID of the user to be retrieved.
///
/// # Response
///
/// The response body will contain a `UserDTO` object, representing the retrieved user.
///
/// # Response Status Codes
///
/// The following status codes are possible:
///
/// - `200 OK`: The user was retrieved successfully.
/// - `400 Bad Request`: An error occurred while attempting to retrieve the user.
#[utoipa::path(
    tag = "Retrieve a user by ID",
    responses(
        (status = 200, description= "The data structure representing the retrieved user.", body = UserDTO),
        (status = 400, description= "The data structure representing an error message.", body = ResponseDTO)
    )
)]
#[get("/v1/user/{id}")]
pub async fn get_user_endpoint(path: web::Path<i32>) -> HttpResponse {
    let id = path.into_inner();

    let result = service::find_user_by_id(id).await;

    match result {
        Ok(user) => {
            let body = to_string_pretty(&user);
            HttpResponse::Ok()
                .content_type("application/json")
                .body(body.unwrap_or("Error retrieving user.".to_string()))
        }
        Err(error) => create_error_response(error.code(), error.to_string()),
    }
}

/// Represents a user information object used to search users by name.
///
/// This struct is used as a query parameter in the `GET /v1/user` endpoint.
#[derive(Deserialize)]
struct UserInfo {
    /// The name of the user to be searched.
    pub name: String,
}

/// Endpoint for retrieving the list of users.
///
/// This endpoint is responsible for retrieving the list of users from the database.
///
/// # HTTP Request
///
/// `GET /v1/user HTTP/1.1`
///
/// # Response
///
/// The response body will contain a vector of `UserDTO` objects, representing the users in the system.
///
/// # Status Codes
///
/// The following status codes are possible:
///
/// - `200 OK`: The list of users was retrieved successfully.
/// - `400 Bad Request`: An error occurred while attempting to retrieve the list of users.
#[utoipa::path(
    tag = "User list endpoint",
    responses(
        (status = 200, body = Vec<UserDTO>, description = "Returns a list of users in the system."),
        (status = 400, body = ResponseDTO, description = "An error occurred while attempting to retrieve the list of users.")
    )
)]
#[get("/v1/user")]
pub async fn get_all_users_endpoint(info: Option<web::Query<UserInfo>>) -> HttpResponse {
    let name = info
        .as_ref()
        .map(|info| info.name.clone())
        .unwrap_or_default();

    let result = service::all_user(name).await;

    match result {
        Ok(users) => {
            let body = to_string_pretty(&users);
            HttpResponse::Ok()
                .content_type("application/json")
                .body(body.unwrap_or("Error retrieving users.".to_string()))
        }
        Err(error) => create_error_response(error.code(), error.to_string()),
    }
}

/// Endpoint for updating a user.
///
/// This endpoint is responsible for updating a user by its ID.
///
/// # HTTP Request
///
/// `PUT /v1/user/{id} HTTP/1.1`
///
/// # Request Body
///
/// The request body should contain a valid `UserDTO` object.
///
/// # Response
///
/// The response body will contain a `UserDTO` representing the updated user.
///
/// # Response Status Codes
///
/// The following status codes are possible:
///
/// - `200 OK`: The user was updated successfully.
/// - `400 Bad Request`: The request body was invalid.
#[utoipa::path(
    tag = "Update user endpoint",
    responses(
        (status = 200, description= "Returns the updated user, with the new data.", body = UserDTO),
        (status = 400, description= "Returns an error response if the update fails.", body = ResponseDTO),
    )
)]
#[put("/v1/user/{id}")]
pub async fn update_user_endpoint(path: web::Path<i32>, user: web::Json<UserDTO>) -> HttpResponse {
    let id = path.into_inner();

    let result = service::update_user(UserDTO {
        id: Some(id),
        ..user.0
    })
    .await;

    match result {
        Ok(user) => {
            let body = to_string_pretty(&user);
            HttpResponse::Ok()
                .content_type("application/json")
                .body(body.unwrap_or("User not updated.".to_string()))
        }
        Err(error) => create_error_response(error.code(), error.to_string()),
    }
}

/// Endpoint for deleting a user by its ID.
///
/// This endpoint is responsible for deleting a user by its ID.
///
/// # HTTP Request
///
/// `DELETE /v1/user/{id} HTTP/1.1`
///
/// # Response
///
/// The response body will contain a `ResponseDTO` containing an error message
/// if the deletion fails.
///
/// # Response Status Codes
///
/// The following status codes are possible:
///
/// - `400 Bad Request`: The request body was invalid.
#[utoipa::path(
    tag = "Delete a user by ID",
    responses(
        (status = 400, description = "Returns an error response if the deletion fails.", body = ResponseDTO),
    )
)]
#[delete("/v1/user/{id}")]
pub async fn delete_user_endpoint(path: web::Path<i32>) -> HttpResponse {
    let user_id = path.into_inner();

    let result = service::delete_user(user_id).await;

    match result {
        Ok(rows_affected) => {
            if rows_affected == 0 {
                let error = UserError::Delete("User not found.".to_string());
                return create_error_response(error.code(), error.to_string());
            }

            let body = to_string_pretty(&ResponseDTO {
                code: StatusCode::OK.as_u16() as u32,
                message: "User deleted successfully.".to_string(),
            });
            HttpResponse::Ok()
                .content_type("application/json")
                .body(body.unwrap_or("Error deleting user.".to_string()))
        }
        Err(error) => create_error_response(error.code(), error.to_string()),
    }
}

/// Creates an error response with the given code and message.
///
/// # Parameters
///
/// * `code`: The error code to be included in the response.
/// * `message`: The error message to be included in the response.
///
/// # Returns
///
/// A `HttpResponse` object with the error response.
fn create_error_response(code: u32, message: String) -> HttpResponse {
    debug!(
        "Server Error. Detail: Code: {} Message: {}",
        code.to_string().bright_blue(),
        message.bright_blue()
    );

    let body = to_string_pretty(&ResponseDTO { code, message });
    HttpResponse::BadRequest()
        .content_type("application/json")
        .body(body.unwrap_or("Unknown server error.".to_string()))
}

#[cfg(test)]
mod tests {
    use actix_web::{body, http::StatusCode, test};

    use crate::dto::ResponseDTO;

    #[test]
    async fn test_create_error_response() {
        let code = 1001;
        let message = "User not found".to_string();
        let response = super::create_error_response(code, message.clone());

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = body::to_bytes(response.into_body())
            .await
            .unwrap_or_default();
        let response_dto: ResponseDTO = serde_json::from_slice(&body).unwrap_or_default();

        assert_eq!(response_dto.code, code);
        assert_eq!(response_dto.message, message);
    }
}
