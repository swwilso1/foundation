//! The `networkinterfaces` module provides the `NetworkInterfaces` struct to store network interfaces.

use crate::network::networkinterface::NetworkInterface;
use network_interface::NetworkInterfaceConfig;
use std::collections::HashMap;

/// The `NetworkInterfaces` struct stores network interfaces.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NetworkInterfaces {
    /// A HashMap that stores network interfaces. The key is the name of the interface.
    interfaces: HashMap<String, NetworkInterface>,
}

impl NetworkInterfaces {
    /// Create a new `NetworkInterfaces` instance.
    pub fn new() -> Self {
        NetworkInterfaces {
            interfaces: HashMap::new(),
        }
    }

    /// Get the number of network interfaces.
    pub fn len(&self) -> usize {
        self.interfaces.len()
    }

    /// Add a new interface to the container.
    pub fn add_interface(&mut self, interface: NetworkInterface) {
        self.interfaces.insert(interface.name.clone(), interface);
    }

    /// Get a reference to a named interface from the container.
    ///
    /// # Arguments
    ///
    /// * `name` - A string slice that holds the name of the interface.
    ///
    /// # Returns
    ///
    /// An Option containing a reference to the interface if it exists, or None if it does not.
    pub fn get_interface(&self, name: &str) -> Option<&NetworkInterface> {
        self.interfaces.get(name)
    }

    /// Get a mutable reference to a named interface from the container.
    ///
    /// # Arguments
    ///
    /// * `name` - A string slice that holds the name of the interface.
    ///
    /// # Returns
    ///
    /// An Option containing a mutable reference to the interface if it exists, or None if it does not.
    pub fn get_interface_mut(&mut self, name: &str) -> Option<&mut NetworkInterface> {
        self.interfaces.get_mut(name)
    }

    /// Get a vector of references to all interfaces in the container.
    ///
    /// # Returns
    ///
    /// A vector of references to all interfaces in the container.
    pub fn get_interfaces(&self) -> Vec<&NetworkInterface> {
        self.interfaces
            .values()
            .map(|interface| interface)
            .collect()
    }

    /// Get a vector of mutable references to all interfaces in the container.
    pub fn get_interfaces_mut(&mut self) -> Vec<&mut NetworkInterface> {
        self.interfaces
            .values_mut()
            .map(|interface| interface)
            .collect()
    }

    /// Get a vector of interface names.
    pub fn get_interface_names(&self) -> Vec<&str> {
        self.interfaces.keys().map(|name| name.as_str()).collect()
    }

    /// Get a vector of interface indexes.
    pub fn get_interface_indexes(&self) -> Vec<u32> {
        self.interfaces
            .values()
            .map(|interface| interface.index)
            .collect()
    }

    /// Get a reference to an interface by index.
    ///
    /// # Arguments
    ///
    /// * `index` - A u32 that holds the index of the interface.
    ///
    /// # Returns
    ///
    /// An Option containing a reference to the interface if it exists, or None if it does not.
    pub fn get_interface_by_index(&self, index: u32) -> Option<&NetworkInterface> {
        for interface in self.interfaces.values() {
            if interface.index == index {
                return Some(interface);
            }
        }
        None
    }

    /// Get a mutable reference to an interface by index.
    ///
    /// # Arguments
    ///
    /// * `index` - A u32 that holds the index of the interface.
    ///
    /// # Returns
    ///
    /// An Option containing a mutable reference to the interface if it exists, or None if it does not.
    pub fn get_interface_by_index_mut(&mut self, index: u32) -> Option<&mut NetworkInterface> {
        for interface in self.interfaces.values_mut() {
            if interface.index == index {
                return Some(interface);
            }
        }
        None
    }

    /// Get a reference to the loopback interface.
    ///
    /// # Returns
    ///
    /// An Option containing a reference to the loopback interface if it exists, or None if it does not.
    pub fn get_loopback_interface(&self) -> Option<&NetworkInterface> {
        for interface in self.interfaces.values() {
            if interface.is_loopback_interface() {
                return Some(interface);
            }
        }
        None
    }

    /// Get a mutable reference to the loopback interface.
    ///
    /// # Returns
    ///
    /// An Option containing a mutable reference to the loopback interface if it exists, or None if it does not.
    pub fn get_loopback_interface_mut(&mut self) -> Option<&mut NetworkInterface> {
        for interface in self.interfaces.values_mut() {
            if interface.is_loopback_interface() {
                return Some(interface);
            }
        }
        None
    }

    /// Get a vector of references to interfaces with global addresses.
    ///
    /// # Returns
    ///
    /// A vector of references to interfaces with global addresses.
    pub fn get_interfaces_with_global_addresses(&self) -> Vec<&NetworkInterface> {
        self.interfaces
            .values()
            .filter(|interface| interface.has_global_address())
            .collect()
    }

    /// Get a vector of mutable references to interfaces with wireless addresses.
    pub async fn get_wireless_interfaces(&self) -> Vec<&NetworkInterface> {
        let mut wireless_interfaces: Vec<&NetworkInterface> = Vec::new();
        for interface in self.interfaces.values() {
            if interface.is_wireless_interface().await {
                wireless_interfaces.push(interface);
            }
        }
        wireless_interfaces
    }

    /// Get a vector of mutable references to interfaces with wireless addresses.
    pub async fn get_wireless_interfaces_mut(&mut self) -> Vec<&mut NetworkInterface> {
        let mut wireless_interfaces: Vec<&mut NetworkInterface> = Vec::new();
        for interface in self.interfaces.values_mut() {
            if interface.is_wireless_interface().await {
                wireless_interfaces.push(interface);
            }
        }
        wireless_interfaces
    }

    /// Get a vector of references to non-loopback, non-wireless interfaces.
    pub async fn get_nonloopback_nonwireless_interfaces(&self) -> Vec<&NetworkInterface> {
        let mut nonloopback_nonwireless_interfaces: Vec<&NetworkInterface> = Vec::new();
        for interface in self.interfaces.values() {
            if !interface.is_loopback_interface() && !interface.is_wireless_interface().await {
                nonloopback_nonwireless_interfaces.push(interface);
            }
        }
        nonloopback_nonwireless_interfaces
    }

    /// Get a vector of mutable references to non-loopback, non-wireless interfaces.
    pub async fn get_nonloopback_nonwireless_interfaces_mut(
        &mut self,
    ) -> Vec<&mut NetworkInterface> {
        let mut nonloopback_nonwireless_interfaces: Vec<&mut NetworkInterface> = Vec::new();
        for interface in self.interfaces.values_mut() {
            if !interface.is_loopback_interface() && !interface.is_wireless_interface().await {
                nonloopback_nonwireless_interfaces.push(interface);
            }
        }
        nonloopback_nonwireless_interfaces
    }

    /// Load the currently configured network interfaces from the running system.
    ///
    /// # Returns
    ///
    /// A `NetworkInterfaces` instance containing the currently configured network interfaces.
    pub fn load_interfaces() -> Self {
        let mut interfaces = NetworkInterfaces::new();
        if let Ok(system_interfaces) = network_interface::NetworkInterface::show() {
            for sys_interface in system_interfaces {
                interfaces.add_interface(NetworkInterface::from(sys_interface));
            }
        }
        interfaces
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::interfaceaddr::InterfaceAddr;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_network_interfaces() {
        let mut interfaces = NetworkInterfaces::new();
        assert_eq!(interfaces.len(), 0);

        let mut interface1 = NetworkInterface::new_with_name("eth0");
        interface1.index = 1;

        let mut interface2 = NetworkInterface::new_with_name("wlan0");
        interface2.index = 2;

        let mut interface3 = NetworkInterface::new_with_name("lo");
        interface3.index = 3;
        interface3.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            None,
            None,
        ));

        interfaces.add_interface(interface1.clone());
        interfaces.add_interface(interface2.clone());
        interfaces.add_interface(interface3.clone());

        assert_eq!(interfaces.len(), 3);

        let eth0 = interfaces.get_interface("eth0").unwrap();
        assert_eq!(eth0.name, "eth0");
        assert_eq!(eth0.index, 1);

        let wlan0 = interfaces.get_interface("wlan0").unwrap();
        assert_eq!(wlan0.name, "wlan0");
        assert_eq!(wlan0.index, 2);

        let lo = interfaces.get_interface("lo").unwrap();
        assert_eq!(lo.name, "lo");
        assert_eq!(lo.index, 3);

        let eth0_mut = interfaces.get_interface_mut("eth0").unwrap();
        eth0_mut.index = 4;
        assert_eq!(eth0_mut.index, 4);

        let wlan0_mut = interfaces.get_interface_mut("wlan0").unwrap();
        wlan0_mut.index = 5;
        assert_eq!(wlan0_mut.index, 5);

        let lo_mut = interfaces.get_interface_mut("lo").unwrap();
        lo_mut.index = 6;
        assert_eq!(lo_mut.index, 6);

        let eth0_by_index = interfaces.get_interface_by_index(4).unwrap();
        assert_eq!(eth0_by_index.name, "eth0");
        assert_eq!(eth0_by_index.index, 4);

        let wlan0_by_index = interfaces.get_interface_by_index(5).unwrap();
        assert_eq!(wlan0_by_index.name, "wlan0");
        assert_eq!(wlan0_by_index.index, 5);

        let lo_by_index = interfaces.get_interface_by_index(6).unwrap();
        assert_eq!(lo_by_index.name, "lo");
        assert_eq!(lo_by_index.index, 6);

        let loopback = interfaces.get_loopback_interface().unwrap();
        assert_eq!(loopback.name, "lo");
        assert_eq!(loopback.index, 6);

        let loopback_mut = interfaces.get_loopback_interface_mut().unwrap();
        loopback_mut.index = 7;
        let other_loopback = interfaces.get_loopback_interface().unwrap();
        assert_eq!(other_loopback.index, 7);
    }

    #[test]
    fn test_get_interface_names() {
        let mut interfaces = NetworkInterfaces::new();
        let interface1 = NetworkInterface::new_with_name("eth0");
        let interface2 = NetworkInterface::new_with_name("wlan0");
        let interface3 = NetworkInterface::new_with_name("lo");
        interfaces.add_interface(interface1.clone());
        interfaces.add_interface(interface2.clone());
        interfaces.add_interface(interface3.clone());
        let interface_names = interfaces.get_interface_names();
        assert_eq!(interface_names.len(), 3);
        assert!(interface_names.contains(&"eth0"));
        assert!(interface_names.contains(&"wlan0"));
        assert!(interface_names.contains(&"lo"));
    }

    #[test]
    fn test_get_interface_indexes() {
        let mut interfaces = NetworkInterfaces::new();
        let mut interface1 = NetworkInterface::new_with_name("eth0");
        interface1.index = 1;
        let mut interface2 = NetworkInterface::new_with_name("wlan0");
        interface2.index = 2;
        let mut interface3 = NetworkInterface::new_with_name("lo");
        interface3.index = 3;
        interfaces.add_interface(interface1.clone());
        interfaces.add_interface(interface2.clone());
        interfaces.add_interface(interface3.clone());
        let interface_indexes = interfaces.get_interface_indexes();
        assert_eq!(interface_indexes.len(), 3);
        assert!(interface_indexes.contains(&1));
        assert!(interface_indexes.contains(&2));
        assert!(interface_indexes.contains(&3));
    }

    #[test]
    fn test_get_interface_by_index() {
        let mut interfaces = NetworkInterfaces::new();
        let mut interface1 = NetworkInterface::new_with_name("eth0");
        interface1.index = 1;
        let mut interface2 = NetworkInterface::new_with_name("wlan0");
        interface2.index = 2;
        let mut interface3 = NetworkInterface::new_with_name("lo");
        interface3.index = 3;
        interfaces.add_interface(interface1.clone());
        interfaces.add_interface(interface2.clone());
        interfaces.add_interface(interface3.clone());
        let eth0 = interfaces.get_interface_by_index(1).unwrap();
        assert_eq!(eth0.name, "eth0");
        assert_eq!(eth0.index, 1);
        let wlan0 = interfaces.get_interface_by_index(2).unwrap();
        assert_eq!(wlan0.name, "wlan0");
        assert_eq!(wlan0.index, 2);
        let lo = interfaces.get_interface_by_index(3).unwrap();
        assert_eq!(lo.name, "lo");
        assert_eq!(lo.index, 3);
    }

    #[test]
    fn test_get_interfaces_by_index_mut() {
        let mut interfaces = NetworkInterfaces::new();
        let mut interface1 = NetworkInterface::new_with_name("eth0");
        interface1.index = 1;
        let mut interface2 = NetworkInterface::new_with_name("wlan0");
        interface2.index = 2;
        let mut interface3 = NetworkInterface::new_with_name("lo");
        interface3.index = 3;
        interfaces.add_interface(interface1.clone());
        interfaces.add_interface(interface2.clone());
        interfaces.add_interface(interface3.clone());
        let eth0 = interfaces.get_interface_by_index_mut(1).unwrap();
        eth0.index = 4;
        let eth0_2 = interfaces.get_interface_by_index(4).unwrap();
        assert_eq!(eth0_2.name, "eth0");
        let wlan0 = interfaces.get_interface_by_index_mut(2).unwrap();
        wlan0.index = 5;
        let wlan0_2 = interfaces.get_interface_by_index(5).unwrap();
        assert_eq!(wlan0_2.name, "wlan0");
        let lo = interfaces.get_interface_by_index_mut(3).unwrap();
        lo.index = 6;
        let lo_2 = interfaces.get_interface_by_index(6).unwrap();
        assert_eq!(lo_2.name, "lo");
    }

    #[test]
    fn test_get_loopback_interface() {
        let mut interfaces = NetworkInterfaces::new();
        let mut interface = NetworkInterface::new_with_name("lo");
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            None,
            None,
        ));
        interfaces.add_interface(NetworkInterface::new_with_name("eth0"));
        interfaces.add_interface(NetworkInterface::new_with_name("wlan0"));
        interfaces.add_interface(interface.clone());
        let loopback = interfaces.get_loopback_interface().unwrap();
        assert_eq!(loopback.name, "lo");
    }

    #[test]
    fn test_get_loopback_interface_mut() {
        let mut interfaces = NetworkInterfaces::new();
        let mut interface = NetworkInterface::new_with_name("lo");
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            None,
            None,
        ));
        interfaces.add_interface(NetworkInterface::new_with_name("eth0"));
        interfaces.add_interface(NetworkInterface::new_with_name("wlan0"));
        interfaces.add_interface(interface.clone());
        let loopback = interfaces.get_loopback_interface_mut().unwrap();
        loopback.index = 1;
        let other_loopback = interfaces.get_loopback_interface().unwrap();
        assert_eq!(other_loopback.index, 1);
    }

    #[test]
    fn test_get_interfaces_with_global_addresses() {
        let mut interfaces = NetworkInterfaces::new();
        let mut interface1 = NetworkInterface::new_with_name("eth0");
        interface1.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            None,
            None,
        ));
        let mut interface2 = NetworkInterface::new_with_name("wlan0");
        interface2.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)),
            None,
            None,
        ));
        let mut interface3 = NetworkInterface::new_with_name("lo");
        interface3.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            None,
            None,
        ));
        interfaces.add_interface(interface1.clone());
        interfaces.add_interface(interface2.clone());
        interfaces.add_interface(interface3.clone());
        let interfaces_with_global_addresses = interfaces.get_interfaces_with_global_addresses();
        assert_eq!(interfaces_with_global_addresses.len(), 1);
        assert!(interfaces_with_global_addresses.contains(&&interface2));
    }
}
