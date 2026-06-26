//! The `interfaceaddr` module provides the `InterfaceAddr` structure to represent an IP address
//! broadcast address and netmask for a network interface.

use crate::error::FoundationError;
use crate::network::ipaddrquery::IpAddrQuery;
use crate::network::netmask::{netmask_from_bits_ipv4, netmask_from_bits_ipv6};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// The `InterfaceAddr` struct represents an IP address, broadcast address, and netmask for a
/// network interface.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct InterfaceAddr {
    /// An IP address of a network interface.
    pub ip: IpAddr,

    /// The broadcast address of a network interface.
    pub broadcast: Option<IpAddr>,

    /// The netmask of a network interface.
    pub netmask: Option<IpAddr>,
}

impl InterfaceAddr {
    /// Create a new `InterfaceAddr` instance.
    ///
    /// # Arguments
    ///
    /// * `ip` - An IP address of a network interface.
    /// * `broadcast` - The broadcast address of a network interface.
    /// * `netmask` - The netmask of a network interface.
    pub fn new(ip: IpAddr, broadcast: Option<IpAddr>, netmask: Option<IpAddr>) -> Self {
        InterfaceAddr {
            ip,
            broadcast,
            netmask,
        }
    }

    /// Get the IP address in CIDR notation.
    ///
    /// # Returns
    ///
    /// An `Option` containing the IP address in CIDR notation if the interface address contains
    /// a netmask. Otherwise, `None` is returned.
    pub fn get_in_cidr_notation(&self) -> Option<String> {
        if let Some(netmask) = self.netmask {
            Some(format!("{}/{}", self.ip, netmask.bits_in_mask()))
        } else {
            None
        }
    }
}

impl From<network_interface::Addr> for InterfaceAddr {
    fn from(addr: network_interface::Addr) -> Self {
        match addr {
            network_interface::Addr::V4(v4addr) => {
                let broadcast = if let Some(ip) = v4addr.broadcast {
                    Some(IpAddr::V4(ip))
                } else {
                    None
                };

                let netmask = if let Some(ip) = v4addr.netmask {
                    Some(IpAddr::V4(ip))
                } else {
                    None
                };

                InterfaceAddr {
                    ip: IpAddr::V4(v4addr.ip),
                    broadcast,
                    netmask,
                }
            }
            network_interface::Addr::V6(v6addr) => {
                let broadcast = if let Some(ip) = v6addr.broadcast {
                    Some(IpAddr::V6(ip))
                } else {
                    None
                };

                let netmask = if let Some(ip) = v6addr.netmask {
                    Some(IpAddr::V6(ip))
                } else {
                    None
                };

                InterfaceAddr {
                    ip: IpAddr::V6(v6addr.ip),
                    broadcast,
                    netmask,
                }
            }
        }
    }
}

impl TryFrom<&str> for InterfaceAddr {
    type Error = FoundationError;

    /// Attempt to parse an `InterfaceAddr` from a string.
    ///
    /// The string should be in the format `ip[/netmask]`.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // Check to see if value is an IP address with a netmask in CIDR notation.
        if value.contains('/') {
            let parts = value.split('/').collect::<Vec<&str>>();
            if parts.len() != 2 {
                return Err(FoundationError::OperationFailed(format!(
                    "Failed to convert {} to InterfaceAddr",
                    value
                )));
            }
            let ip: IpAddr = parts[0].parse()?;
            let mask_bits: u8 = parts[1].parse()?;
            let netmask = match ip {
                IpAddr::V4(_) => {
                    let netmask = netmask_from_bits_ipv4(mask_bits);
                    Some(IpAddr::V4(<Ipv4Addr as From<[u8; 4]>>::from(netmask)))
                }
                IpAddr::V6(_) => {
                    let netmask = netmask_from_bits_ipv6(mask_bits);
                    Some(IpAddr::V6(<Ipv6Addr as From<[u8; 16]>>::from(netmask)))
                }
            };
            return Ok(InterfaceAddr::new(ip, None, netmask));
        }

        // The value is not a string with CIDR notation, just try to parse the value
        // as an IP address.
        let ip = value.parse()?;
        Ok(InterfaceAddr::new(ip, None, None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use network_interface::{V4IfAddr, V6IfAddr};

    #[test]
    fn test_new() {
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 5));
        let broadcast = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 255));
        let netmask = IpAddr::V4(Ipv4Addr::new(255, 255, 255, 0));
        let addr = InterfaceAddr::new(ip, Some(broadcast), Some(netmask));
        assert_eq!(addr.ip, ip);
        assert_eq!(addr.broadcast, Some(broadcast));
        assert_eq!(addr.netmask, Some(netmask));
    }

    #[test]
    fn test_get_in_cidr_notation_with_netmask() {
        let addr = InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 5)),
            None,
            Some(IpAddr::V4(Ipv4Addr::new(255, 255, 255, 0))),
        );
        assert_eq!(addr.get_in_cidr_notation(), Some("192.168.1.5/24".to_string()));
    }

    #[test]
    fn test_get_in_cidr_notation_without_netmask() {
        let addr = InterfaceAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 5)), None, None);
        assert_eq!(addr.get_in_cidr_notation(), None);
    }

    #[test]
    fn test_try_from_cidr_ipv4() {
        let addr = InterfaceAddr::try_from("10.0.0.1/8").unwrap();
        assert_eq!(addr.ip, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));
        assert_eq!(addr.broadcast, None);
        assert_eq!(
            addr.netmask,
            Some(IpAddr::V4(Ipv4Addr::new(255, 0, 0, 0)))
        );
    }

    #[test]
    fn test_try_from_cidr_ipv6() {
        let addr = InterfaceAddr::try_from("fc00::1/64").unwrap();
        assert_eq!(
            addr.ip,
            IpAddr::V6(Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 1))
        );
        assert_eq!(
            addr.netmask,
            Some(IpAddr::V6(Ipv6Addr::new(
                0xffff, 0xffff, 0xffff, 0xffff, 0, 0, 0, 0
            )))
        );
    }

    #[test]
    fn test_try_from_plain_ip() {
        let addr = InterfaceAddr::try_from("172.16.0.1").unwrap();
        assert_eq!(addr.ip, IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1)));
        assert_eq!(addr.broadcast, None);
        assert_eq!(addr.netmask, None);
    }

    #[test]
    fn test_try_from_too_many_slashes() {
        let result = InterfaceAddr::try_from("10.0.0.1/8/16");
        assert!(matches!(result, Err(FoundationError::OperationFailed(_))));
    }

    #[test]
    fn test_try_from_invalid_mask_bits() {
        let result = InterfaceAddr::try_from("10.0.0.1/abc");
        assert!(matches!(result, Err(FoundationError::ParseIntError(_))));
    }

    #[test]
    fn test_try_from_invalid_ip() {
        let result = InterfaceAddr::try_from("not_an_ip");
        assert!(matches!(result, Err(FoundationError::AddressParseError(_))));
    }

    #[test]
    fn test_from_network_interface_addr_v4() {
        let v4addr = network_interface::Addr::V4(V4IfAddr {
            ip: Ipv4Addr::new(192, 168, 1, 5),
            broadcast: Some(Ipv4Addr::new(192, 168, 1, 255)),
            netmask: Some(Ipv4Addr::new(255, 255, 255, 0)),
        });
        let addr = InterfaceAddr::from(v4addr);
        assert_eq!(addr.ip, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 5)));
        assert_eq!(
            addr.broadcast,
            Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 255)))
        );
        assert_eq!(
            addr.netmask,
            Some(IpAddr::V4(Ipv4Addr::new(255, 255, 255, 0)))
        );
    }

    #[test]
    fn test_from_network_interface_addr_v4_no_optionals() {
        let v4addr = network_interface::Addr::V4(V4IfAddr {
            ip: Ipv4Addr::new(192, 168, 1, 5),
            broadcast: None,
            netmask: None,
        });
        let addr = InterfaceAddr::from(v4addr);
        assert_eq!(addr.ip, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 5)));
        assert_eq!(addr.broadcast, None);
        assert_eq!(addr.netmask, None);
    }

    #[test]
    fn test_from_network_interface_addr_v6() {
        let v6addr = network_interface::Addr::V6(V6IfAddr {
            ip: Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 1),
            broadcast: None,
            netmask: Some(Ipv6Addr::new(0xffff, 0xffff, 0xffff, 0xffff, 0, 0, 0, 0)),
        });
        let addr = InterfaceAddr::from(v6addr);
        assert_eq!(
            addr.ip,
            IpAddr::V6(Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 1))
        );
        assert_eq!(addr.broadcast, None);
        assert_eq!(
            addr.netmask,
            Some(IpAddr::V6(Ipv6Addr::new(
                0xffff, 0xffff, 0xffff, 0xffff, 0, 0, 0, 0
            )))
        );
    }
}
