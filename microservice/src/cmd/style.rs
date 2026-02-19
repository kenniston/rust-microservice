#![allow(unused)]

//! The `style` module provides utilities for building terminal styles used
//! across the application. It leverages `calp::builder::Styles` and
//! `anstyle::Style` to define two main style configurations:
//!
//! - **Cargo**: A style derived from the Cargo color theme
//!   allowing the application to apply distinct visual formatting on CLI texts.
//!
//! - **Default**: A basic styling configuration applied.
//!
//! These styles help ensure consistent and expressive formatting throughout
//! command-line output.
use anstyle::Style;
use clap::builder::Styles;

pub(crate) const CURRENT: Styles = default::DEFAULT;

pub(crate) const NOP: Style = Style::new();

/// Defines the Cargo color palette and style configuration for the CLI.
///
/// This module exposes a collection of color constants and formatting
/// styles used to render structured and visually consistent messages
/// in the command-line interface. It also defines `CARGO_STYLING`,
/// a ready-to-use `Styles` configuration compatible with `clap`.
pub(crate) mod cargo {
    use anstyle::{AnsiColor, Effects, Style};
    use clap::builder::Styles;

    pub(crate) const HEADER: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
    pub(crate) const USAGE: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
    pub(crate) const LITERAL: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
    pub(crate) const PLACEHOLDER: Style = AnsiColor::Cyan.on_default();
    pub(crate) const CONTEXT: Style = AnsiColor::White.on_default();
    pub(crate) const CONTEXT_VALUE: Style = AnsiColor::BrightWhite.on_default();
    pub(crate) const ERROR: Style = AnsiColor::Red.on_default().effects(Effects::BOLD);
    pub(crate) const WARN: Style = AnsiColor::Yellow.on_default().effects(Effects::BOLD);
    pub(crate) const NOTE: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
    pub(crate) const GOOD: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
    pub(crate) const VALID: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
    pub(crate) const INVALID: Style = AnsiColor::Yellow.on_default().effects(Effects::BOLD);

    /// Cargo's color style
    pub(crate) const CARGO_STYLING: Styles = Styles::styled()
        .header(HEADER)
        .usage(USAGE)
        .literal(LITERAL)
        .placeholder(PLACEHOLDER)
        .error(ERROR)
        .valid(VALID)
        .invalid(INVALID)
        .context(CONTEXT)
        .context_value(CONTEXT_VALUE);
}

/// Defines the default color palette and style configuration for the CLI.
///
/// This module offers a set of color and text-style constants used for
/// rendering help messages, errors, warnings, and contextual information.
/// It also provides `DEFAULT`, a predefined `Styles` set used by `clap`
/// for standard output formatting.
pub(crate) mod default {
    use anstyle::{AnsiColor, Effects, Style};
    use clap::builder::Styles;

    pub(crate) const HEADER: Style = AnsiColor::Yellow.on_default().effects(Effects::BOLD);
    pub(crate) const USAGE: Style = AnsiColor::BrightYellow.on_default().effects(Effects::BOLD);
    pub(crate) const LITERAL: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
    pub(crate) const PLACEHOLDER: Style = AnsiColor::BrightBlue.on_default();
    pub(crate) const CONTEXT: Style = AnsiColor::White.on_default();
    pub(crate) const CONTEXT_VALUE: Style = AnsiColor::BrightWhite.on_default();
    pub(crate) const ERROR: Style = AnsiColor::Red.on_default();
    pub(crate) const WARN: Style = AnsiColor::Yellow.on_default().effects(Effects::BOLD);
    pub(crate) const NOTE: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
    pub(crate) const GOOD: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
    pub(crate) const VALID: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
    pub(crate) const INVALID: Style = AnsiColor::Red.on_default().effects(Effects::BOLD);

    /// Cargo's color style
    pub(crate) const DEFAULT: Styles = Styles::styled()
        .header(HEADER)
        .usage(USAGE)
        .literal(LITERAL)
        .placeholder(PLACEHOLDER)
        .error(ERROR)
        .valid(VALID)
        .invalid(INVALID)
        .context(CONTEXT)
        .context_value(CONTEXT_VALUE);
}
