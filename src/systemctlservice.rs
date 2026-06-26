//! The `systemctlservice` module contains code that interacts with the SystemCTL service on a Linux
//! machine.

use crate::error::FoundationError;
use std::process::Command;

/// The `SystemCTLService` object is used to start, stop, and restart services on a Linux machine.
pub struct SystemCTLService {
    /// The name of the service.
    service_name: String,
}

impl SystemCTLService {
    /// Create a new `SystemCTLService` object.
    ///
    /// # Arguments
    ///
    /// * `service_name` - The name of the service.
    pub fn new(service_name: String) -> SystemCTLService {
        SystemCTLService { service_name }
    }

    /// Start the service.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the service was started successfully, otherwise returns a `FoundationError`.
    pub fn start(&self) -> Result<(), FoundationError> {
        let output = Command::new("systemctl")
            .arg("start")
            .arg(&self.service_name)
            .output()?;
        if !output.status.success() {
            return Err(FoundationError::OperationFailed(format!(
                "Failed to start service: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        Ok(())
    }

    /// Stop the service.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the service was stopped successfully, otherwise returns a `FoundationError`.
    pub fn stop(&self) -> Result<(), FoundationError> {
        let output = Command::new("systemctl")
            .arg("stop")
            .arg(&self.service_name)
            .output()?;
        if !output.status.success() {
            return Err(FoundationError::OperationFailed(format!(
                "Failed to stop service: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        Ok(())
    }

    /// Restart the service.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the service was restarted successfully, otherwise returns a `FoundationError`.
    pub fn restart(&self) -> Result<(), FoundationError> {
        let output = Command::new("systemctl")
            .arg("restart")
            .arg(&self.service_name)
            .output()?;
        if !output.status.success() {
            return Err(FoundationError::OperationFailed(format!(
                "Failed to restart service: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Returns true when the `systemctl` binary can be spawned on this host. On platforms
    /// without systemd (e.g. macOS) spawning fails, which drives the `start`/`stop`/`restart`
    /// methods down their `?`-propagated `FoundationError::IO` path rather than the
    /// `OperationFailed` status-check path.
    fn systemctl_available() -> bool {
        Command::new("systemctl").arg("--version").output().is_ok()
    }

    /// A service name that should never correspond to a real unit, so the systemctl
    /// operations are guaranteed to fail without touching anything real on the host.
    const BOGUS_SERVICE: &str = "foundation-nonexistent-service-12345.service";

    /// Every operation on the bogus service must return an error. Which error depends on the
    /// host: if `systemctl` is missing the spawn fails (`IO`); if it is present the command
    /// runs and exits non-zero (`OperationFailed`).
    fn assert_operation_errors(result: Result<(), FoundationError>) {
        let err = result.expect_err("operating on a bogus service should fail");
        match err {
            FoundationError::IO(_) => {
                assert!(
                    !systemctl_available(),
                    "got an IO error even though systemctl is available"
                );
            }
            FoundationError::OperationFailed(msg) => {
                assert!(
                    systemctl_available(),
                    "got OperationFailed even though systemctl is unavailable"
                );
                // The failure message embeds systemctl's stderr; it should not be blank.
                assert!(!msg.is_empty());
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn test_new_stores_service_name() {
        let service = SystemCTLService::new("my-service".to_string());
        assert_eq!(service.service_name, "my-service");
    }

    #[test]
    fn test_start_bogus_service_errors() {
        let service = SystemCTLService::new(BOGUS_SERVICE.to_string());
        assert_operation_errors(service.start());
    }

    #[test]
    fn test_stop_bogus_service_errors() {
        let service = SystemCTLService::new(BOGUS_SERVICE.to_string());
        assert_operation_errors(service.stop());
    }

    #[test]
    fn test_restart_bogus_service_errors() {
        let service = SystemCTLService::new(BOGUS_SERVICE.to_string());
        assert_operation_errors(service.restart());
    }
}
