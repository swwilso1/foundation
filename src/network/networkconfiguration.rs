//! The `networkconfiguration` module contains the `NetworkConfiguration` struct and the
//! `AddressMode` enum. The `NetworkConfiguration` struct represents the configuration of a network
//! interface, including the address mode, the interface, whether the interface is enabled, the
//! wireless configuration, and the DHCP range. The `AddressMode` enum represents the address mode
//! of a network interface, which can be DHCP4, DHCP6, or Static.

use crate::error::FoundationError;
use crate::network::dhcprange::DHCPRange;
use crate::network::networkinterface::NetworkInterface;
use crate::network::wireless::configuration::WirelessConfiguration;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// The `AddressMode` enum represents the address mode of a network interface, which can be DHCP4,
/// DHCP6, or Static.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AddressMode {
    /// The interface receives an IPv4 address from a DHCP server.
    DHCP4,

    /// The interface receives an IPv6 address from a DHCP server.
    DHCP6,

    /// The interface has a static IP address.
    Static,
}

/// The `NetworkConfiguration` struct represents the configuration of a network interface.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NetworkConfiguration {
    /// The address mode of the network interface.
    pub address_mode: AddressMode,

    /// The network interface details.
    pub interface: NetworkInterface,

    /// Whether the network interface is enabled.
    pub enabled: bool,

    /// The wireless configuration of the network interface if configured.
    pub wifi_configuration: Option<WirelessConfiguration>,

    /// The DHCP range of the network interface if configured.
    pub dhcp_range: Option<DHCPRange>,
}

impl NetworkConfiguration {
    /// Creates a new `NetworkConfiguration` with the specified address mode, network interface,
    /// enabled status, wireless configuration, and DHCP range.
    ///
    /// # Arguments
    ///
    /// * `address_mode` - The address mode of the network interface.
    /// * `interface` - The network interface details.
    /// * `enabled` - Whether the network interface is enabled.
    /// * `wifi_configuration` - The wireless configuration of the network interface if configured.
    /// * `dhcp_range` - The DHCP range of the network interface if configured.
    pub fn new(
        address_mode: AddressMode,
        interface: NetworkInterface,
        enabled: bool,
        wifi_configuration: Option<WirelessConfiguration>,
        dhcp_range: Option<DHCPRange>,
    ) -> Self {
        NetworkConfiguration {
            address_mode,
            interface,
            enabled,
            wifi_configuration,
            dhcp_range,
        }
    }

    /// Creates a new `NetworkConfiguration` with the specified name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the network interface.
    ///
    /// # Returns
    ///
    /// A new `NetworkConfiguration` with the specified name. The address mode is set to DHCP4, the
    /// network interface is created with the specified name, and the network interface is enabled.
    /// The wireless configuration and DHCP range are not set.
    pub fn new_with_name(name: &str) -> Self {
        NetworkConfiguration::new(
            AddressMode::DHCP4,
            NetworkInterface::new_with_name(name),
            false,
            None,
            None,
        )
    }

    /// Creates a new `NetworkConfiguration` with the specified network interface.
    ///
    /// # Arguments
    ///
    /// * `interface` - The network interface details.
    ///
    /// # Returns
    ///
    /// A new `NetworkConfiguration` with the specified network interface. The address mode is set
    /// to DHCP4, the network interface is created with the specified details, and the network
    /// interface is enabled. The wireless configuration and DHCP range are not set.
    pub fn new_with_interface(interface: NetworkInterface) -> Self {
        NetworkConfiguration::new(AddressMode::DHCP4, interface, true, None, None)
    }

    /// Return the name of the network interface.
    pub fn get_name(&self) -> String {
        self.interface.name.clone()
    }

    /// Return whether the network interface is wireless.
    pub fn is_wireless_enabled(&self) -> bool {
        self.wifi_configuration.is_some()
    }
}

impl FromStr for AddressMode {
    type Err = FoundationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dhcp4" => Ok(AddressMode::DHCP4),
            "dhcp6" => Ok(AddressMode::DHCP6),
            "static" => Ok(AddressMode::Static),
            _ => Err(FoundationError::InvalidConversion(
                s.to_string(),
                "AddressMode",
            )),
        }
    }
}

impl Display for AddressMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AddressMode::DHCP4 => write!(f, "dhcp4"),
            AddressMode::DHCP6 => write!(f, "dhcp6"),
            AddressMode::Static => write!(f, "static"),
        }
    }
}
