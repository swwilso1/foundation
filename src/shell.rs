//! The `shell` module contains code for interacting with a shell sub-process.

use crate::error::FoundationError;
use std::process::{Command, Output};

/// The `Shell` struct represents a shell sub-process.
pub struct Shell {}

impl Shell {
    /// Executes a command with the given arguments.
    ///
    /// # Arguments
    ///
    /// * `command` - The command to execute.
    /// * `arguments` - The arguments to pass to the command.
    ///
    /// # Returns
    ///
    /// A Result containing the output of the command if successful, or a `FoundationError` if an error occurred.
    pub fn execute_command(
        command: &str,
        arguments: Vec<String>,
    ) -> Result<Output, FoundationError> {
        let args: Vec<&str> = arguments.iter().map(|s| s.as_str()).collect();
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .arg("/C")
                .arg(command)
                .args(args.iter().map(|arg| arg.to_string()))
                .output()
        } else {
            Command::new(command)
                .args(args.iter().map(|arg| arg.to_string()))
                .output()
        };

        match output {
            Ok(o) => Ok(o),
            Err(e) => Err(FoundationError::from(e)),
        }
    }
}
