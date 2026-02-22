//! This program provides the command-line interface for this application.
//! It defines all supported flags, arguments, and subcommands.
//!
//! # Command-Line Parameters
//!
//! The application exposes several optional and required command-line
//! parameters. These parameters are parsed at runtime and control the
//! behavior of the tool.
//!
//! ## Global Flags
//!
//! `--config <path>`
//! Specifies the path to the configuration file. If omitted, the
//! application attempts to load a default configuration from common
//! lookup locations.
//!
//! `--log-level <level>`
//! Defines the logging verbosity. Valid levels include `"debug"`,
//! `"info"`, `"warn"`, and `"error"`. Defaults to `"info"`.
//!
//! ## Additional Subcommands
//!
//! Other subcommands may be declared in separate files under the
//! `cmd/` package. Each subcommand may define its own flags and
//! execution logic. Refer to the corresponding file for detailed
//! documentation.
//!
//! ## Notes
//!
//! - All commands support the standard `--help` flag, which displays
//!   contextual help.
//! - Flags must be placed after the subcommand unless they are global
//!   flags available to the command.
#![deny(clippy::unwrap_used)]
#![deny(clippy::redundant_clone)]
#![deny(clippy::all)]

pub mod dto;
mod entity;
pub mod module;

use rust_embed::Embed;
use rust_microservice::ServerApi;

#[derive(Embed)]
#[folder = "assets"]
pub struct Asset;

#[ServerApi(
    controllers_path = "src/module, src/controllers, teste/controllers, src/poc_bigquery/module",
    openapi_title = "🌐 Rest API Server",
    openapi_api_description = "Rest API OpenApi Documentation built with Rust 🦀.",
    database = "true",
    banner = r#"
            _~^~^~_         ___    ___   ____    ____
        \) /  o o  \ (/    / _ |  / _ \ /  _/   / __/___  ____ _  __ ___  ____
          '_   -   _'     / __ | / ___/_/ /    _\ \ / -_)/ __/| |/ // -_)/ __/
          / '-----' \    /_/ |_|/_/   /___/   /___/ \__//_/   |___/ \__//_/
    "#
)]
pub fn start_server() {}
