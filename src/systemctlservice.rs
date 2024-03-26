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
