//! The `interfaceaddr` module provides the `InterfaceAddr` structure to represent an IP address
//! broadcast address and netmask for a network interface.

use crate::error::FoundationError;
use crate::network::netmask::{netmask_from_bits_ipv4, netmask_from_bits_ipv6};
use crate::network::ipaddrquery::IpAddrQuery;
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
