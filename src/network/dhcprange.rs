//! The `dhcprange` module provides a structure to represent a range of IP addresses used for DHCP.

use crate::error::FoundationError;
use std::net::IpAddr;

/// The `DHCPRange` struct represents a range of IP addresses used for DHCP.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DHCPRange {
    /// The starting IP address of the range.
    pub start: IpAddr,

    /// The ending IP address of the range.
    pub end: IpAddr,
}

impl DHCPRange {
    /// Create a new `DHCPRange` instance.
    ///
    /// # Arguments
    ///
    /// * `start` - The starting IP address of the range.
    /// * `end` - The ending IP address of the range.
    pub fn new(start: IpAddr, end: IpAddr) -> Self {
        DHCPRange { start, end }
    }
}

impl TryFrom<&str> for DHCPRange {
    type Error = FoundationError;

    /// Attempt to parse a `DHCPRange` from a string.
    ///
    /// The string should be in the format `start,end`.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.contains(',') {
            let parts = value.split(',').collect::<Vec<&str>>();
            if parts.len() < 2 {
                return Err(FoundationError::OperationFailed(
                    "value does not contain a valid DHCP range".to_string(),
                ));
            }
            let start = parts[0].parse()?;
            let end = parts[1].parse()?;
            return Ok(DHCPRange::new(start, end));
        }
        Err(FoundationError::OperationFailed(
            "value does not contain a valid DHCP range".to_string(),
        ))
    }
}

impl std::fmt::Display for DHCPRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{}", self.start, self.end)
    }
}
