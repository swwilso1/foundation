//! The `shell` module contains code for interacting with a shell sub-process.

use crate::error::FoundationError;
use std::process::{Child, Command, Output};

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

    /// Executes a command with the given arguments and returns the stdout and stderr output.
    pub fn execute(command: &str, arguments: Vec<String>) -> (Option<String>, Option<String>) {
        if let Ok(output) = Shell::execute_command(command, arguments) {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                (Some(stdout), Some(stderr))
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                (None, Some(stderr))
            }
        } else {
            (None, None)
        }
    }

    /// Runs a command with the given arguments. The command will launch as a child
    /// of the currently running process.
    ///
    /// # Arguments
    ///
    /// * `command` - The command to run.
    /// * `arguments` - The arguments to pass to the command.
    ///
    /// # Returns
    ///
    /// A `Child` object on success or a `FoundationError` if an error occurs.
    pub fn spawn_command(command: &str, arguments: Vec<String>) -> Result<Child, FoundationError> {
        let args: Vec<&str> = arguments.iter().map(|s| s.as_str()).collect();
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .arg("/C")
                .arg(command)
                .args(args.iter().map(|arg| arg.to_string()))
                .spawn()
        } else {
            Command::new(command)
                .args(args.iter().map(|arg| arg.to_string()))
                .spawn()
        };

        match output {
            Ok(child) => Ok(child),
            Err(e) => Err(FoundationError::from(e)),
        }
    }
}
