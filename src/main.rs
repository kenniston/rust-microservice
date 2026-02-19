//! Application entry point module.
//!
//! This module defines the executable entry point of the application.
//! Its sole responsibility is to bootstrap the runtime and delegate
//! execution to the public API exposed by the `lib.rs` crate.
//!
//! ## Design Rationale
//!
//! All application logic, including configuration loading, dependency
//! initialization, domain services, and business rules, is intentionally
//! implemented in `lib.rs`. This architectural choice provides the
//! following benefits:
//!
//! - **Testability**: By centralizing logic in `lib.rs`, the application
//!   can be exercised through integration and unit tests without relying
//!   on the binary entry point.
//! - **Reusability**: The library crate can be reused by other binaries
//!   or tools if necessary.
//! - **Separation of concerns**: The `main.rs` file remains minimal and
//!   focused solely on starting the application.
//!
//! ## Integration Testing Strategy
//!
//! Integration tests are executed against the public interfaces exposed
//! by `lib.rs`, allowing tests to run in isolation from the binary startup
//! logic. This avoids side effects and simplifies test setup.
//!
//! As a result, `main.rs` should not contain any business logic or
//! application-specific behavior beyond invoking the library entry point.
use server_lib as lib;

fn main() -> microservice::Result<(), String> {
    // Delegates execution to the library crate
    lib::start_server()
}
