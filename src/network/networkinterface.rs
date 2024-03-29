//! The `networkinterface` module provides the `NetworkInterface` struct and its methods.

use crate::network::interfaceaddr::InterfaceAddr;
use crate::network::ipaddrquery::IpAddrQuery;
use crate::network::wireless::is_wireless_interface;
use network_interface::NetworkInterfaceConfig;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// The `NetworkInterface` struct represents a network interface.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NetworkInterface {
    /// The name of the network interface.
    pub name: String,

    /// The addresses of the network interface (including broadcast and netmask).
    pub addresses: Vec<InterfaceAddr>,

    /// The MAC address of the network interface.
    pub mac_addr: Option<String>,

    /// The index of the network interface.
    pub index: u32,

    /// The nameserver addresses of the network interface.
    pub nameserver_addresses: Vec<IpAddr>,

    /// The gateway addresses of the network interface.
    pub gateway_addresses: Vec<IpAddr>,
}

impl NetworkInterface {
    /// Create a default version of a `NetworkInterface`.
    /// All internals have a default value.
    pub fn default() -> NetworkInterface {
        NetworkInterface {
            name: String::new(),
            addresses: vec![],
            mac_addr: None,
            index: 0,
            nameserver_addresses: vec![],
            gateway_addresses: vec![],
        }
    }

    /// Create a new network interface
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the network interface.
    /// * `addresses` - A list of `InterfaceAddr` objects configured for the interface.
    /// * `mac_addr` - The MAC address of the interface or None.
    /// * `index` - The operating systems index for the interface.
    /// * `nameserver_addresses` - A list of IP address representing DNS nameservers
    ///    for the address.
    /// * `gateway_addresses` - A list of IP addresses representing gateways/routers
    ///   for the address.
    pub fn new(
        name: &str,
        addresses: Vec<InterfaceAddr>,
        mac_addr: Option<String>,
        index: u32,
        nameserver_addresses: Vec<IpAddr>,
        gateway_addresses: Vec<IpAddr>,
    ) -> Self {
        NetworkInterface {
            name: name.to_string(),
            addresses,
            mac_addr,
            index,
            nameserver_addresses,
            gateway_addresses,
        }
    }

    /// Create a new `NetworkInterface` instance.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the network interface.
    pub fn new_with_name(name: &str) -> Self {
        NetworkInterface::new(name, vec![], None, 0, vec![], vec![])
    }

    /// Remove all settings for the current interface except the name.
    pub fn clear(&mut self) {
        self.addresses.clear();
        self.mac_addr = None;
        self.index = 0;
        self.nameserver_addresses.clear();
        self.gateway_addresses.clear();
    }

    /// Get all the IPV4 addresses of the network interface along with broadcast address and netmasks.
    ///
    /// # Returns
    ///
    /// A vector of references to `InterfaceAddr` instances.
    pub fn get_ipv4_interface_addresses(&self) -> Vec<&InterfaceAddr> {
        self.addresses
            .iter()
            .filter(|addr| addr.ip.is_ipv4())
            .collect()
    }

    /// A function to get mutable references to all the IPV4 addresses of the network interface
    /// along with broadcast address and netmasks.
    ///
    /// # Returns
    ///
    /// A vector of mutable references to `InterfaceAddr` instances.
    pub fn get_ipv4_interface_addresses_mut(&mut self) -> Vec<&mut InterfaceAddr> {
        self.addresses
            .iter_mut()
            .filter(|addr| addr.ip.is_ipv4())
            .collect()
    }

    /// Get all the IPV6 addresses of the network interface along with broadcast address and netmasks.
    ///
    /// # Returns
    ///
    /// A vector of references to `InterfaceAddr` instances.
    pub fn get_ipv6_interface_addresses(&self) -> Vec<&InterfaceAddr> {
        self.addresses
            .iter()
            .filter(|addr| addr.ip.is_ipv6())
            .collect()
    }

    /// A function to get mutable references to all the IPV6 addresses of the network interface
    /// along with broadcast address and netmasks.
    ///
    /// # Returns
    ///
    /// A vector of mutable references to `InterfaceAddr` instances.
    pub fn get_ipv6_interface_addresses_mut(&mut self) -> Vec<&mut InterfaceAddr> {
        self.addresses
            .iter_mut()
            .filter(|addr| addr.ip.is_ipv6())
            .collect()
    }

    /// Get the first IPV4 address of the network interface along with broadcast address and netmask.
    ///
    /// # Returns
    ///
    /// A reference to the `InterfaceAddr` instance.
    pub fn get_ipv4_interface_address(&self) -> Option<&InterfaceAddr> {
        self.addresses.iter().find(|addr| addr.ip.is_ipv4())
    }

    /// A function to get a mutable reference to the first IPV4 address of the network interface
    ///
    /// # Returns
    ///
    /// A mutable reference to the `InterfaceAddr` instance.
    pub fn get_ipv4_interface_address_mut(&mut self) -> Option<&mut InterfaceAddr> {
        self.addresses.iter_mut().find(|addr| addr.ip.is_ipv4())
    }

    /// Get the first IPV6 address of the network interface along with broadcast address and netmask.
    ///
    /// # Returns
    ///
    /// A reference to the `InterfaceAddr` instance.
    pub fn get_ipv6_interface_address(&self) -> Option<&InterfaceAddr> {
        self.addresses.iter().find(|addr| addr.ip.is_ipv6())
    }

    /// A function to get a mutable reference to the first IPV6 address of the network interface
    /// along with broadcast address and netmask.
    ///
    /// # Returns
    ///
    /// A mutable reference to the `InterfaceAddr` instance.
    pub fn get_ipv6_interface_address_mut(&mut self) -> Option<&mut InterfaceAddr> {
        self.addresses.iter_mut().find(|addr| addr.ip.is_ipv6())
    }

    /// Get the first global address of the network interface along with broadcast address and netmask.
    ///
    /// # Returns
    ///
    /// A reference to the `InterfaceAddr` instance.
    pub fn get_global_interface_address(&self) -> Option<&InterfaceAddr> {
        self.addresses
            .iter()
            .find(|addr| addr.ip.is_global_address())
    }

    /// A function to get a mutable reference to the first global address of the network interface
    /// along with broadcast address and netmask.
    ///
    /// # Returns
    ///
    /// A mutable reference to the `InterfaceAddr` instance.
    pub fn get_global_interface_address_mut(&mut self) -> Option<&mut InterfaceAddr> {
        self.addresses
            .iter_mut()
            .find(|addr| addr.ip.is_global_address())
    }

    /// Get the first global IPV4 address of the network interface along with broadcast address and netmask.
    ///
    /// # Returns
    ///
    /// A reference to the `InterfaceAddr` instance.
    pub fn get_global_ipv4_interface_address(&self) -> Option<&InterfaceAddr> {
        self.addresses
            .iter()
            .find(|addr| addr.ip.is_global_address() && addr.ip.is_ipv4())
    }

    /// A function to get a mutable reference to the first global IPV4 address of the network interface
    /// along with broadcast address and netmask.
    ///
    /// # Returns
    ///
    /// A mutable reference to the `InterfaceAddr` instance.
    pub fn get_global_ipv4_interface_address_mut(&mut self) -> Option<&mut InterfaceAddr> {
        self.addresses
            .iter_mut()
            .find(|addr| addr.ip.is_global_address() && addr.ip.is_ipv4())
    }

    /// Get the first global IPV6 address of the network interface along with broadcast address and netmask.
    ///
    /// # Returns
    ///
    /// A reference to the `InterfaceAddr` instance.
    pub fn get_global_ipv6_interface_address(&self) -> Option<&InterfaceAddr> {
        self.addresses
            .iter()
            .find(|addr| addr.ip.is_global_address() && addr.ip.is_ipv6())
    }

    /// A function to get a mutable reference to the first global IPV6 address of the network interface
    ///
    /// # Returns
    ///
    /// A mutable reference to the `InterfaceAddr` instance.
    pub fn get_global_ipv6_interface_address_mut(&mut self) -> Option<&mut InterfaceAddr> {
        self.addresses
            .iter_mut()
            .find(|addr| addr.ip.is_global_address() && addr.ip.is_ipv6())
    }

    /// Get all the IP addresses of the network interface.
    ///
    /// # Returns
    ///
    /// A vector of references to `IpAddr` instances.
    pub fn get_addresses(&self) -> Vec<&IpAddr> {
        self.addresses.iter().map(|addr| &addr.ip).collect()
    }

    /// Get all the IPV4 addresses of the network interface.
    ///
    /// # Returns
    ///
    /// A vector of `Ipv4Addr` instances.
    pub fn get_ipv4_addresses(&self) -> Vec<Ipv4Addr> {
        self.addresses
            .iter()
            .filter(|addr| addr.ip.is_ipv4())
            .map(|addr| match addr.ip {
                IpAddr::V4(ip) => ip,
                _ => unreachable!(),
            })
            .collect()
    }

    /// Get all the IPV6 addresses of the network interface.
    ///
    /// # Returns
    ///
    /// A vector of `Ipv6Addr` instances.
    pub fn get_ipv6_addresses(&self) -> Vec<Ipv6Addr> {
        self.addresses
            .iter()
            .filter(|addr| addr.ip.is_ipv6())
            .map(|addr| match addr.ip {
                IpAddr::V6(ip) => ip,
                _ => unreachable!(),
            })
            .collect()
    }

    /// Get the first IPV4 address of the network interface.
    ///
    /// # Returns
    ///
    /// An `Ipv4Addr` instance.
    pub fn get_ipv4_address(&self) -> Option<Ipv4Addr> {
        self.addresses
            .iter()
            .find(|addr| addr.ip.is_ipv4())
            .map(|addr| match addr.ip {
                IpAddr::V4(ip) => ip,
                _ => unreachable!(),
            })
    }

    /// Get the first IPV6 address of the network interface.
    ///
    /// # Returns
    ///
    /// An `Ipv6Addr` instance.
    pub fn get_ipv6_address(&self) -> Option<Ipv6Addr> {
        self.addresses
            .iter()
            .find(|addr| addr.ip.is_ipv6())
            .map(|addr| match addr.ip {
                IpAddr::V6(ip) => ip,
                _ => unreachable!(),
            })
    }

    /// Get the first global address of the network interface.
    ///
    /// # Returns
    ///
    /// An optional `IpAddr` instance.
    pub fn get_global_address(&self) -> Option<IpAddr> {
        self.addresses
            .iter()
            .find(|addr| addr.ip.is_global_address())
            .map(|addr| addr.ip)
    }

    /// Get the first global IPV4 address of the network interface.
    ///
    /// # Returns
    ///
    /// An optional `Ipv4Addr` instance.
    pub fn get_global_ipv4_address(&self) -> Option<Ipv4Addr> {
        self.addresses
            .iter()
            .find(|addr| addr.ip.is_global_address() && addr.ip.is_ipv4())
            .map(|addr| match addr.ip {
                IpAddr::V4(ip) => ip,
                _ => unreachable!(),
            })
    }

    /// Get the first global IPV6 address of the network interface.
    ///
    /// # Returns
    ///
    /// An optional `Ipv6Addr` instance.
    pub fn get_global_ipv6_address(&self) -> Option<Ipv6Addr> {
        self.addresses
            .iter()
            .find(|addr| addr.ip.is_global_address() && addr.ip.is_ipv6())
            .map(|addr| match addr.ip {
                IpAddr::V6(ip) => ip,
                _ => unreachable!(),
            })
    }

    /// Check if the network interface is a loopback interface.
    ///
    /// # Returns
    ///
    /// True if the interface is the loopback interface, otherwise false.
    pub fn is_loopback_interface(&self) -> bool {
        self.addresses.iter().any(|addr| addr.ip.is_loopback())
    }

    /// Check if the network interface has a global address.
    ///
    /// # Returns
    ///
    /// True if the interface has a global address, otherwise false.
    pub fn has_global_address(&self) -> bool {
        self.addresses
            .iter()
            .any(|addr| addr.ip.is_global_address())
    }

    /// Check if the network interface has a global IPV4 address.
    ///
    /// # Returns
    ///
    /// True if the interface has a global IPV4 address, otherwise false.
    pub fn has_global_ipv4_address(&self) -> bool {
        self.addresses
            .iter()
            .any(|addr| addr.ip.is_global_address() && addr.ip.is_ipv4())
    }

    /// Check if the network interface has a global IPV6 address.
    ///
    /// # Returns
    ///
    /// True if the interface has a global IPV6 address, otherwise false.
    pub fn has_global_ipv6_address(&self) -> bool {
        self.addresses
            .iter()
            .any(|addr| addr.ip.is_global_address() && addr.ip.is_ipv6())
    }

    /// Check if the network interface has an IPV4 address.
    ///
    /// # Returns
    ///
    /// True if the interface has an IPV4 address, otherwise false.
    pub fn has_ipv4_address(&self) -> bool {
        self.addresses.iter().any(|addr| addr.ip.is_ipv4())
    }

    /// Check if the network interface has an IPV6 address.
    ///
    /// # Returns
    ///
    /// True if the interface has an IPV6 address, otherwise false.
    pub fn has_ipv6_address(&self) -> bool {
        self.addresses.iter().any(|addr| addr.ip.is_ipv6())
    }

    /// Check if the network interface is a wireless interface.
    ///
    /// # Returns
    ///
    /// True if the interface is a wireless interface, otherwise false.
    pub async fn is_wireless_interface(&self) -> bool {
        is_wireless_interface(&self.name).await
    }

    /// Load the network interfaces on the running system.
    ///
    /// # Returns
    ///
    /// A vector of `NetworkInterface` instances.
    pub fn load() -> Vec<NetworkInterface> {
        if let Ok(interfaces) = network_interface::NetworkInterface::show() {
            interfaces
                .into_iter()
                .map(|interface| NetworkInterface::from(interface))
                .collect()
        } else {
            vec![]
        }
    }
}

impl From<network_interface::NetworkInterface> for NetworkInterface {
    fn from(value: network_interface::NetworkInterface) -> Self {
        let addresses = value
            .addr
            .iter()
            .map(|addr| InterfaceAddr::from(*addr))
            .collect();

        NetworkInterface {
            name: value.name.clone(),
            addresses,
            mac_addr: value.mac_addr.clone(),
            index: value.index,
            nameserver_addresses: vec![],
            gateway_addresses: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let name = "eth0";
        let interface = NetworkInterface::new_with_name(name);
        assert_eq!(interface.name, name.to_string());
        assert_eq!(interface.addresses, vec![]);
        assert_eq!(interface.mac_addr, None);
        assert_eq!(interface.index, 0);
        assert_eq!(interface.nameserver_addresses, Vec::<IpAddr>::new());
        assert_eq!(interface.gateway_addresses, Vec::<IpAddr>::new());
    }

    #[test]
    fn test_clear() {
        let mut interface = NetworkInterface::new_with_name("eth0");
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            None,
            None,
        ));
        interface.mac_addr = Some("00:00:00:00:00:00".to_string());
        interface.index = 1;
        interface
            .nameserver_addresses
            .push(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)));
        interface
            .gateway_addresses
            .push(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));
        interface.clear();
        assert_eq!(&interface.name, "eth0");
        assert_eq!(interface.addresses, vec![]);
        assert_eq!(interface.mac_addr, None);
        assert_eq!(interface.index, 0);
        assert_eq!(interface.nameserver_addresses, Vec::<IpAddr>::new());
        assert_eq!(interface.gateway_addresses, Vec::<IpAddr>::new());
    }

    #[test]
    fn test_get_ipv4_interface_addresses() {
        let mut interface = NetworkInterface::new_with_name("eth0");
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2)),
            None,
            None,
        ));
        let ipv4_addresses = interface.get_ipv4_interface_addresses();
        assert_eq!(ipv4_addresses.len(), 2);
        assert_eq!(
            ipv4_addresses[0].ip,
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))
        );
        assert_eq!(
            ipv4_addresses[1].ip,
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2))
        );
    }

    #[test]
    fn test_get_ipv4_interface_addresses_mut() {
        let mut interface = NetworkInterface::new_with_name("eth0");
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2)),
            None,
            None,
        ));
        let mut ipv4_addresses = interface.get_ipv4_interface_addresses_mut();
        assert_eq!(ipv4_addresses.len(), 2);
        assert_eq!(
            ipv4_addresses[0].ip,
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))
        );
        assert_eq!(
            ipv4_addresses[1].ip,
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2))
        );
        ipv4_addresses[0].ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 3));

        let other_ipv4_addresses = interface.get_ipv4_interface_addresses();
        assert_eq!(other_ipv4_addresses.len(), 2);
        assert_eq!(
            other_ipv4_addresses[0].ip,
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 3))
        );
        assert_eq!(
            other_ipv4_addresses[1].ip,
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2))
        );
    }

    #[test]
    fn test_get_ipv6_interface_addresses() {
        let mut interface = NetworkInterface::new_with_name("eth0");
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 2)),
            None,
            None,
        ));
        let ipv6_addresses = interface.get_ipv6_interface_addresses();
        assert_eq!(ipv6_addresses.len(), 2);
        assert_eq!(
            ipv6_addresses[0].ip,
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1))
        );
        assert_eq!(
            ipv6_addresses[1].ip,
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 2))
        );
    }

    #[test]
    fn test_get_ipv6_interface_addresses_mut() {
        let mut interface = NetworkInterface::new_with_name("eth0");
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 2)),
            None,
            None,
        ));
        let mut ipv6_addresses = interface.get_ipv6_interface_addresses_mut();
        assert_eq!(ipv6_addresses.len(), 2);
        assert_eq!(
            ipv6_addresses[0].ip,
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1))
        );
        assert_eq!(
            ipv6_addresses[1].ip,
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 2))
        );
        ipv6_addresses[0].ip = IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 3));

        let other_ipv6_addresses = interface.get_ipv6_interface_addresses();
        assert_eq!(other_ipv6_addresses.len(), 2);
        assert_eq!(
            other_ipv6_addresses[0].ip,
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 3))
        );
        assert_eq!(
            other_ipv6_addresses[1].ip,
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 2))
        );
    }

    #[test]
    fn test_get_ipv4_interface_address() {
        let mut interface = NetworkInterface::new_with_name("eth0");
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2)),
            None,
            None,
        ));
        let ipv4_address = interface.get_ipv4_interface_address();
        assert_eq!(
            ipv4_address.unwrap().ip,
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))
        );
    }

    #[test]
    fn test_get_ipv4_interface_address_mut() {
        let mut interface = NetworkInterface::new_with_name("eth0");
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2)),
            None,
            None,
        ));
        let ipv4_address = interface.get_ipv4_interface_address_mut();
        if let Some(iaddr) = ipv4_address {
            assert_eq!(iaddr.ip, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));
            iaddr.ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 3));
        } else {
            assert!(false);
        }
        let other_ipv4_address = interface.get_ipv4_interface_address();
        if let Some(iaddr) = other_ipv4_address {
            assert_eq!(iaddr.ip, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 3)));
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_get_ipv6_interface_address() {
        let mut interface = NetworkInterface::new_with_name("eth0");
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 2)),
            None,
            None,
        ));
        let ipv6_address = interface.get_ipv6_interface_address();
        assert_eq!(
            ipv6_address.unwrap().ip,
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1))
        );
    }

    #[test]
    fn test_get_ipv6_interface_address_mut() {
        let mut interface = NetworkInterface::new_with_name("eth0");
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 2)),
            None,
            None,
        ));
        let ipv6_address = interface.get_ipv6_interface_address_mut();
        if let Some(iaddr) = ipv6_address {
            assert_eq!(
                iaddr.ip,
                IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1))
            );
            iaddr.ip = IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 3));
        } else {
            assert!(false);
        }
        let other_ipv6_address = interface.get_ipv6_interface_address();
        if let Some(iaddr) = other_ipv6_address {
            assert_eq!(
                iaddr.ip,
                IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 3))
            );
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_get_global_interface_address() {
        let mut interface = NetworkInterface::new_with_name("eth0");
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 2)),
            None,
            None,
        ));
        let global_address = interface.get_global_interface_address();
        assert_eq!(
            global_address.unwrap().ip,
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1))
        );
    }

    #[test]
    fn test_get_global_interface_address_mut() {
        let mut interface = NetworkInterface::new_with_name("eth0");
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 2)),
            None,
            None,
        ));
        let global_address = interface.get_global_interface_address_mut();
        if let Some(iaddr) = global_address {
            assert_eq!(
                iaddr.ip,
                IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1))
            );
            iaddr.ip = IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 3));
        } else {
            assert!(false);
        }
        let other_global_address = interface.get_global_interface_address();
        if let Some(iaddr) = other_global_address {
            assert_eq!(
                iaddr.ip,
                IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 3))
            );
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_get_global_ipv4_interface_address() {
        let mut interface = NetworkInterface::new_with_name("eth0");
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(11, 168, 1, 2)),
            None,
            None,
        ));
        let global_ipv4_address = interface.get_global_ipv4_interface_address();
        assert_eq!(
            global_ipv4_address.unwrap().ip,
            IpAddr::V4(Ipv4Addr::new(11, 168, 1, 2))
        );
    }

    #[test]
    fn test_get_global_ipv4_interface_address_mut() {
        let mut interface = NetworkInterface::new_with_name("eth0");
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(11, 168, 1, 2)),
            None,
            None,
        ));
        let global_ipv4_address = interface.get_global_ipv4_interface_address_mut();
        if let Some(iaddr) = global_ipv4_address {
            assert_eq!(iaddr.ip, IpAddr::V4(Ipv4Addr::new(11, 168, 1, 2)));
            iaddr.ip = IpAddr::V4(Ipv4Addr::new(11, 168, 1, 3));
        } else {
            assert!(false);
        }
        let other_global_ipv4_address = interface.get_global_ipv4_interface_address();
        if let Some(iaddr) = other_global_ipv4_address {
            assert_eq!(iaddr.ip, IpAddr::V4(Ipv4Addr::new(11, 168, 1, 3)));
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_get_global_ipv6_interface_address() {
        let mut interface = NetworkInterface::new_with_name("eth0");
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 2)),
            None,
            None,
        ));
        let global_ipv6_address = interface.get_global_ipv6_interface_address();
        assert_eq!(
            global_ipv6_address.unwrap().ip,
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1))
        );
    }

    #[test]
    fn test_get_global_ipv6_interface_address_mut() {
        let mut interface = NetworkInterface::new_with_name("eth0");
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 2)),
            None,
            None,
        ));
        let global_ipv6_address = interface.get_global_ipv6_interface_address_mut();
        if let Some(iaddr) = global_ipv6_address {
            assert_eq!(
                iaddr.ip,
                IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1))
            );
            iaddr.ip = IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 3));
        } else {
            assert!(false);
        }
        let other_global_ipv6_address = interface.get_global_ipv6_interface_address();
        if let Some(iaddr) = other_global_ipv6_address {
            assert_eq!(
                iaddr.ip,
                IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 3))
            );
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_get_addresses() {
        let mut interface = NetworkInterface::new_with_name("eth0");
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(11, 168, 1, 2)),
            None,
            None,
        ));
        let addresses = interface.get_addresses();
        assert_eq!(addresses.len(), 3);
        assert_eq!(
            addresses[0],
            &IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1))
        );
        assert_eq!(addresses[1], &IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));
        assert_eq!(addresses[2], &IpAddr::V4(Ipv4Addr::new(11, 168, 1, 2)));
    }

    #[test]
    fn test_get_ipv4_addresses() {
        let mut interface = NetworkInterface::new_with_name("eth0");
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(11, 168, 1, 2)),
            None,
            None,
        ));
        let ipv4_addresses = interface.get_ipv4_addresses();
        assert_eq!(ipv4_addresses.len(), 2);
        assert_eq!(ipv4_addresses[0], Ipv4Addr::new(192, 168, 1, 1));
        assert_eq!(ipv4_addresses[1], Ipv4Addr::new(11, 168, 1, 2));
    }

    #[test]
    fn test_get_ipv6_addresses() {
        let mut interface = NetworkInterface::new_with_name("eth0");
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 2)),
            None,
            None,
        ));
        let ipv6_addresses = interface.get_ipv6_addresses();
        assert_eq!(ipv6_addresses.len(), 2);
        assert_eq!(ipv6_addresses[0], Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1));
        assert_eq!(ipv6_addresses[1], Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 2));
    }

    #[test]
    fn test_get_ipv4_address() {
        let mut interface = NetworkInterface::new_with_name("eth0");
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(11, 168, 1, 2)),
            None,
            None,
        ));
        let ipv4_address = interface.get_ipv4_address();
        assert_eq!(ipv4_address.unwrap(), Ipv4Addr::new(192, 168, 1, 1));
    }

    #[test]
    fn test_get_ipv6_address() {
        let mut interface = NetworkInterface::new_with_name("eth0");
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 2)),
            None,
            None,
        ));
        let ipv6_address = interface.get_ipv6_address();
        assert_eq!(
            ipv6_address.unwrap(),
            Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1)
        );
    }

    #[test]
    fn test_get_global_address() {
        let mut interface = NetworkInterface::new_with_name("eth0");
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(11, 168, 1, 2)),
            None,
            None,
        ));
        let global_address = interface.get_global_address();
        assert_eq!(
            global_address.unwrap(),
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1))
        );
    }

    #[test]
    fn test_get_global_ipv4_address() {
        let mut interface = NetworkInterface::new_with_name("eth0");
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(11, 168, 1, 2)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            None,
            None,
        ));
        let global_ipv4_address = interface.get_global_ipv4_address();
        assert_eq!(global_ipv4_address.unwrap(), Ipv4Addr::new(11, 168, 1, 2));
    }

    #[test]
    fn test_get_global_ipv6_address() {
        let mut interface = NetworkInterface::new_with_name("eth0");
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(11, 168, 1, 2)),
            None,
            None,
        ));
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 2)),
            None,
            None,
        ));
        let global_ipv6_address = interface.get_global_ipv6_address();
        assert_eq!(
            global_ipv6_address.unwrap(),
            Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1)
        );
    }

    #[test]
    fn test_is_loopback_interface() {
        let mut interface = NetworkInterface::new_with_name("eth0");
        assert_eq!(interface.is_loopback_interface(), false);
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            None,
            None,
        ));
        assert_eq!(interface.is_loopback_interface(), true);
    }

    #[test]
    fn test_has_global_address() {
        let mut interface = NetworkInterface::new_with_name("eth0");
        assert_eq!(interface.has_global_address(), false);
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2)),
            None,
            None,
        ));
        assert_eq!(interface.has_global_address(), false);
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1)),
            None,
            None,
        ));
        assert_eq!(interface.has_global_address(), true);
    }

    #[test]
    fn test_has_global_ipv4_address() {
        let mut interface = NetworkInterface::new_with_name("eth0");
        assert_eq!(interface.has_global_ipv4_address(), false);
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2)),
            None,
            None,
        ));
        assert_eq!(interface.has_global_ipv4_address(), false);
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(11, 168, 1, 2)),
            None,
            None,
        ));
        assert_eq!(interface.has_global_ipv4_address(), true);
    }

    #[test]
    fn test_has_global_ipv6_address() {
        let mut interface = NetworkInterface::new_with_name("eth0");
        assert_eq!(interface.has_global_ipv6_address(), false);
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1)),
            None,
            None,
        ));
        assert_eq!(interface.has_global_ipv6_address(), true);
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 2)),
            None,
            None,
        ));
        assert_eq!(interface.has_global_ipv6_address(), true);
    }

    #[test]
    fn test_has_ipv4_address() {
        let mut interface = NetworkInterface::new_with_name("eth0");
        assert_eq!(interface.has_ipv4_address(), false);
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1)),
            None,
            None,
        ));
        assert_eq!(interface.has_ipv4_address(), false);
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(11, 168, 1, 2)),
            None,
            None,
        ));
        assert_eq!(interface.has_ipv4_address(), true);
    }

    #[test]
    fn test_has_ipv6_address() {
        let mut interface = NetworkInterface::new_with_name("eth0");
        assert_eq!(interface.has_ipv6_address(), false);
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(11, 168, 1, 2)),
            None,
            None,
        ));
        assert_eq!(interface.has_ipv6_address(), false);
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, 1)),
            None,
            None,
        ));
        assert_eq!(interface.has_ipv6_address(), true);
    }

    cfg_if! {
        if #[cfg(target_os = "linux")] {
            #[tokio::test]
            async fn test_is_wireless_interface() {
                let interface = NetworkInterface::new_with_name("eth0");
                assert_eq!(interface.is_wireless_interface().await, false);
            }
        }
    }
}
