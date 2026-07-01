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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_command_success() {
        let output =
            Shell::execute_command("echo", vec!["hello".to_string()]).expect("echo should succeed");
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("hello"));
    }

    #[test]
    fn test_execute_command_passes_arguments() {
        // Pass multiple arguments and confirm they all appear in stdout.
        let args = vec!["one".to_string(), "two".to_string(), "three".to_string()];
        let output = Shell::execute_command("echo", args).expect("echo should succeed");
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("one"));
        assert!(stdout.contains("two"));
        assert!(stdout.contains("three"));
    }

    #[test]
    fn test_execute_command_nonexistent_returns_error() {
        // On non-Windows the command is invoked directly, so a bogus binary yields an Err.
        // On Windows the command runs via `cmd /C`, which succeeds while the inner command
        // fails, so only assert the error path off Windows.
        let result = Shell::execute_command("this_command_should_not_exist_anywhere_12345", vec![]);
        if cfg!(target_os = "windows") {
            // `cmd /C bogus` still spawns cmd successfully.
            assert!(result.is_ok());
        } else {
            assert!(matches!(result, Err(FoundationError::IO(_))));
        }
    }

    #[test]
    fn test_execute_success_returns_stdout_and_stderr() {
        let (stdout, stderr) = Shell::execute("echo", vec!["hello".to_string()]);
        let stdout = stdout.expect("stdout should be Some on success");
        let stderr = stderr.expect("stderr should be Some on success");
        assert!(stdout.contains("hello"));
        // echo writes nothing to stderr.
        assert!(stderr.is_empty());
    }

    #[test]
    fn test_execute_nonexistent_command_returns_none_none() {
        // A command that cannot be spawned makes execute_command return Err, so execute
        // yields (None, None). This only holds where the command is invoked directly.
        if !cfg!(target_os = "windows") {
            let (stdout, stderr) =
                Shell::execute("this_command_should_not_exist_anywhere_12345", vec![]);
            assert!(stdout.is_none());
            assert!(stderr.is_none());
        }
    }

    #[test]
    fn test_execute_failing_command_returns_none_stdout_some_stderr() {
        // `false` exits non-zero with no output; on most shells stderr is empty but the
        // status is unsuccessful, so stdout must be None and stderr must be Some.
        if !cfg!(target_os = "windows") {
            let (stdout, stderr) = Shell::execute("false", vec![]);
            assert!(stdout.is_none());
            assert!(stderr.is_some());
        }
    }

    #[test]
    fn test_execute_command_failure_captures_stderr() {
        // `ls` of a missing path exits non-zero and writes a diagnostic to stderr.
        if !cfg!(target_os = "windows") {
            let (stdout, stderr) = Shell::execute(
                "ls",
                vec!["/this/path/definitely/does/not/exist/12345".to_string()],
            );
            assert!(stdout.is_none());
            let stderr = stderr.expect("stderr should be Some on failure");
            assert!(!stderr.is_empty());
        }
    }

    #[test]
    fn test_spawn_command_success() {
        let mut child =
            Shell::spawn_command("echo", vec!["hello".to_string()]).expect("echo should spawn");
        let status = child.wait().expect("child should be waitable");
        assert!(status.success());
    }

    #[test]
    fn test_spawn_command_with_arguments() {
        // Spawn a process that exits non-zero and confirm we observe that status.
        if !cfg!(target_os = "windows") {
            let mut child =
                Shell::spawn_command("sh", vec!["-c".to_string(), "exit 3".to_string()])
                    .expect("sh should spawn");
            let status = child.wait().expect("child should be waitable");
            assert_eq!(status.code(), Some(3));
        }
    }

    #[test]
    fn test_spawn_command_nonexistent_returns_error() {
        if cfg!(target_os = "windows") {
            // `cmd /C bogus` spawns cmd successfully; just reap the child.
            if let Ok(mut child) =
                Shell::spawn_command("this_command_should_not_exist_anywhere_12345", vec![])
            {
                let _ = child.wait();
            }
        } else {
            let result =
                Shell::spawn_command("this_command_should_not_exist_anywhere_12345", vec![]);
            assert!(matches!(result, Err(FoundationError::IO(_))));
        }
    }
}
