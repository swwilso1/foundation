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

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn test_new() {
        let start = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10));
        let end = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));
        let range = DHCPRange::new(start, end);
        assert_eq!(range.start, start);
        assert_eq!(range.end, end);
    }

    #[test]
    fn test_try_from_valid_ipv4() {
        let range = DHCPRange::try_from("192.168.1.10,192.168.1.100").unwrap();
        assert_eq!(range.start, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)));
        assert_eq!(range.end, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)));
    }

    #[test]
    fn test_try_from_valid_ipv6() {
        let range = DHCPRange::try_from("fc00::1,fc00::ff").unwrap();
        assert_eq!(range.start, IpAddr::V6(Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 1)));
        assert_eq!(
            range.end,
            IpAddr::V6(Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 0xff))
        );
    }

    #[test]
    fn test_try_from_missing_comma() {
        let result = DHCPRange::try_from("192.168.1.10");
        assert!(matches!(result, Err(FoundationError::OperationFailed(_))));
    }

    #[test]
    fn test_try_from_invalid_address() {
        let result = DHCPRange::try_from("not_an_ip,192.168.1.100");
        assert!(matches!(result, Err(FoundationError::AddressParseError(_))));
    }

    #[test]
    fn test_display() {
        let range = DHCPRange::new(
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 254)),
        );
        assert_eq!(range.to_string(), "10.0.0.2,10.0.0.254");
    }

    #[test]
    fn test_display_try_from_roundtrip() {
        let original = DHCPRange::new(
            IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1)),
            IpAddr::V4(Ipv4Addr::new(172, 16, 0, 50)),
        );
        let parsed = DHCPRange::try_from(original.to_string().as_str()).unwrap();
        assert_eq!(original, parsed);
    }
}
