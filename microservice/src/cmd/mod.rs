/// Cmd module provides the command-line functions and structures for
/// this application. It defines all supported flags, arguments, and
/// subcommands.
///
/// # Additional Modules
///
/// - `root` — module for the CLI root command.
/// - `run` — module for the run command that starts the API server.
/// - `style` — module style and themes used by CLI.
pub mod root;
pub mod run;
mod style;
