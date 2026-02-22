//! # User Controller Integration Tests
//!
//! This module contains integration tests for the user management HTTP endpoints.
//! The tests validate the behavior of the REST API exposed by the user controller,
//! ensuring correct request handling, response structure, and error management.
//!
//! ## Covered Endpoints
//!
//! The test suite verifies the following operations:
//!
//! - **Create User** — Validates successful user creation and duplicate user handling.
//! - **Update User** — Ensures existing users can be updated and invalid updates are rejected.
//! - **Get User by ID** — Confirms user retrieval and proper handling of missing users.
//! - **Get All Users** — Verifies that the endpoint returns a collection of users.
//! - **Delete User** — Ensures users can be removed and invalid deletions are handled.
//!
//! ## Test Environment
//!
//! Tests are executed using the Actix Web test framework with an isolated application
//! instance per test. Each test:
//!
//! - Initializes a test server with the target endpoint registered
//! - Sends an HTTP request with appropriate headers and payload
//! - Asserts response status codes and response body content
//!
//! ## Authentication
//!
//! Some endpoints require an authorization token. The token is generated using a
//! helper function provided by the test crate.
//!
//! ## Response Validation
//!
//! Responses are deserialized into DTOs to validate:
//!
//! - Business data correctness
//! - Error codes and messages
//! - Collection sizes and constraints
//!
//! ## Purpose
//!
//! The goal of this module is to guarantee that the user API behaves according to
//! the expected contract, preventing regressions and ensuring reliability of CRUD
//! operations.

use actix_web::{
    App,
    http::{
        Method,
        header::{AUTHORIZATION, Accept, ContentType},
    },
    test,
};
use asserting::{
    assert_that,
    prelude::{AssertEquality, AssertOrder},
};
use serde_json::json;
use server_lib::{
    dto::{ResponseDTO, user::UserDTO},
    module::user::user_controller::{
        create_user_endpoint, delete_user_endpoint, get_all_users_endpoint, get_user_endpoint,
        update_user_endpoint,
    },
};

use crate::token;

/// Creates a new user in the database.
///
/// This function takes a `UserDTO` object and attempts to save it to the database.
/// If the user already exists, a `UserExists` error is returned.
/// If the user is successfully created, a `UserDTO` object is returned containing the newly
/// created user's information.
#[actix_web::test]
async fn test_create_user_endpoint() {
    let server = test::init_service(App::new().service(create_user_endpoint)).await;

    let json = json!({"name": "Teste", "email": "test@teste.com"});
    let request = test::TestRequest::default()
        .uri("/v1/user")
        .method(Method::POST)
        .insert_header(ContentType::json())
        .insert_header((AUTHORIZATION, token()))
        .set_json(json)
        .to_request();

    let res = test::call_service(&server, request).await;
    let status = res.status().as_u16();
    assert_eq!(status, 200);

    let body = test::read_body(res).await;
    let dto: UserDTO = serde_json::from_slice(&body).unwrap_or_default();
    assert_eq!(dto.name.unwrap_or("".to_string()), "Teste".to_string());
}

/// Tests the `create_user_endpoint` function, which is responsible for creating a new
/// user in the database. The test attempts to create a user with an existing name and
/// email address, and then asserts that the response status code is 400 (Bad Request),
/// and that the response body contains an error message indicating that the user already
/// exists.
#[actix_web::test]
async fn test_user_already_created_endpoint() {
    let server = test::init_service(App::new().service(create_user_endpoint)).await;

    let json = json!({"name": "Diana Miller", "email": "diana@example.com"});
    let request = test::TestRequest::default()
        .uri("/v1/user")
        .method(Method::POST)
        .insert_header(ContentType::json())
        .insert_header((AUTHORIZATION, token()))
        .set_json(json)
        .to_request();

    let res = test::call_service(&server, request).await;
    let status = res.status().as_u16();
    assert_eq!(status, 400);

    let body = test::read_body(res).await;
    let dto: ResponseDTO = serde_json::from_slice(&body).unwrap_or_default();
    assert_eq!(dto.code, 1002);
    assert_eq!(dto.message, "User already exists".to_string());
}

/// Tests the `update_user_endpoint` function, which is responsible for updating an existing
/// user in the database.
///
/// The test sends a PUT request to the `/v1/user/1` endpoint with a JSON body containing the
/// user's name and email address.
/// It then asserts that the response status code is 200 (OK), and that the response body
/// contains the updated user's information.
#[actix_web::test]
async fn test_update_user_endpoint() {
    let server = test::init_service(App::new().service(update_user_endpoint)).await;

    let json = json!({"name": "Kenniston", "email": "kenniston@kenniston.com"});
    let request = test::TestRequest::default()
        .uri("/v1/user/1")
        .method(Method::PUT)
        .insert_header(ContentType::json())
        .set_json(json)
        .to_request();

    let res = test::call_service(&server, request).await;
    let status = res.status().as_u16();
    assert_eq!(status, 200);

    let body = test::read_body(res).await;
    let dto: UserDTO = serde_json::from_slice(&body).unwrap_or_default();
    assert_eq!(dto.name.unwrap_or("".to_string()), "Kenniston".to_string());
}

/// Tests the `update_user_endpoint` function, which is responsible for updating an existing
/// user in the database, with an invalid user (with duplicated email).
///
/// The test sends a PUT request to the `/v1/user/1` endpoint with a JSON body containing the
/// user's name and email address.
/// It then asserts that the response status code is 400 (Bad Request), and that the response body
/// contains an error message indicating that the user is invalid.
#[actix_web::test]
async fn test_update_invalid_user_endpoint() {
    let server = test::init_service(App::new().service(update_user_endpoint)).await;

    let json = json!({"name": "Kenniston", "email": "diana@example.com"});
    let request = test::TestRequest::default()
        .uri("/v1/user/1")
        .method(Method::PUT)
        .insert_header(ContentType::json())
        .set_json(json)
        .to_request();

    let res = test::call_service(&server, request).await;
    let status = res.status().as_u16();
    assert_eq!(status, 400);

    let body = test::read_body(res).await;
    let dto: ResponseDTO = serde_json::from_slice(&body).unwrap_or_default();
    assert_eq!(dto.code, 1003);
}

/// Tests the `get_user_endpoint` function, which is responsible for retrieving an existing
/// user from the database by its ID.
///
/// The test sends a GET request to the `/v1/user/3` endpoint, and then asserts that the
/// response status code is 200 (OK), and that the response body contains the user's information.
///
/// # Assertions
///
/// - The response status code is 200 (OK).
/// - The response body contains the user's information.
#[actix_web::test]
async fn test_find_user_by_id() {
    let server = test::init_service(App::new().service(get_user_endpoint)).await;

    let request = test::TestRequest::default()
        .uri("/v1/user/3")
        .method(Method::GET)
        .insert_header(Accept::json())
        .to_request();

    let res = test::call_service(&server, request).await;
    let status = res.status().as_u16();
    assert_eq!(status, 200);

    let body = test::read_body(res).await;
    let dto: UserDTO = serde_json::from_slice(&body).unwrap_or_default();
    assert_eq!(dto.name.unwrap_or("".to_string()), "Bob Smith".to_string());
}

/// Tests the `get_user_endpoint` function, which is responsible for retrieving an existing
/// user from the database by its ID.
///
/// The test sends a GET request to the `/v1/user/999` endpoint, and then asserts that the
/// response status code is 400 (Bad Request), and that the response body contains an error
/// message indicating that the user was not found.
#[actix_web::test]
async fn test_user_by_id_not_found() {
    let server = test::init_service(App::new().service(get_user_endpoint)).await;

    let request = test::TestRequest::default()
        .uri("/v1/user/999")
        .method(Method::GET)
        .insert_header(Accept::json())
        .to_request();

    let res = test::call_service(&server, request).await;
    let status = res.status().as_u16();
    assert_eq!(status, 400);

    let body = test::read_body(res).await;
    let dto: ResponseDTO = serde_json::from_slice(&body).unwrap_or_default();
    assert_eq!(dto.code, 1001);
    assert_eq!(dto.message, "User not found");
}

/// Tests the `get_all_users_endpoint` function, which is responsible for retrieving all
/// existing users from the database.
///
/// The test sends a GET request to the `/v1/user` endpoint, and then asserts that the
/// response status code is 200 (OK), and that the response body contains a list of users
/// with at least 3 elements.
#[actix_web::test]
async fn test_find_all_users() {
    let server = test::init_service(App::new().service(get_all_users_endpoint)).await;

    let request = test::TestRequest::default()
        .uri("/v1/user")
        .method(Method::GET)
        .insert_header(Accept::json())
        .to_request();

    let res = test::call_service(&server, request).await;
    let status = res.status().as_u16();
    assert_eq!(status, 200);

    let body = test::read_body(res).await;
    let dto: Vec<UserDTO> = serde_json::from_slice(&body).unwrap_or_default();

    assert_that!(dto.len()).is_greater_than(2);
}

/// Tests the `delete_user_endpoint` function, which is responsible for deleting an existing
/// user from the database by its ID.
///
/// The test sends a DELETE request to the `/v1/user/4` endpoint, and then asserts that the
/// response status code is 200 (OK), and that the response body contains a success message.
///
/// # Assertions
///
/// - The response status code is 200 (OK).
/// - The response body contains a success message.
#[actix_web::test]
async fn test_delete_user_by_id() {
    let server = test::init_service(App::new().service(delete_user_endpoint)).await;

    let request = test::TestRequest::default()
        .uri("/v1/user/4")
        .method(Method::DELETE)
        .insert_header(Accept::json())
        .to_request();

    let res = test::call_service(&server, request).await;
    let status = res.status().as_u16();
    assert_eq!(status, 200);

    let body = test::read_body(res).await;
    let dto: ResponseDTO = serde_json::from_slice(&body).unwrap_or_default();

    assert_that!(dto.message).is_equal_to("User deleted successfully.");
}

/// Tests the `delete_user_endpoint` function, which is responsible for deleting an existing
/// user from the database by its ID.
///
/// The test sends a DELETE request to the `/v1/user/999` endpoint, which is an invalid user ID,
/// and then asserts that the response status code is 400 (Bad Request), and that the response body
/// contains an error message indicating that the user was not found.
///
/// # Assertions
///
/// - The response status code is 400 (Bad Request).
/// - The response body contains an error message indicating that the user was not found.
#[actix_web::test]
async fn test_delete_invalid_user() {
    let server = test::init_service(App::new().service(delete_user_endpoint)).await;

    let request = test::TestRequest::default()
        .uri("/v1/user/999")
        .method(Method::DELETE)
        .insert_header(Accept::json())
        .to_request();

    let res = test::call_service(&server, request).await;
    let status = res.status().as_u16();
    assert_eq!(status, 400);

    let body = test::read_body(res).await;
    let dto: ResponseDTO = serde_json::from_slice(&body).unwrap_or_default();
    assert_that!(dto.code).is_equal_to(1004);
    assert_that!(dto.message).is_equal_to("Error deleting user: User not found.");
}
