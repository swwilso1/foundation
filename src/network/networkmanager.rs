//! The `networkmanager` module provides the `NetworkManager` type, which is responsible for
//! managing network configurations and services on a machine.

use crate::network::networkconfiguration::NetworkConfiguration;
use crate::network::networkinterface::NetworkInterface;
use log::debug;
use std::collections::HashMap;

cfg_if! {
    if #[cfg(target_os = "linux")] {
        use crate::network::dhcpcdservice::DHCPCDService;
        use crate::network::dnsmasqservice::DNSMasqService;
        use crate::network::hostapdservice::HostAPDService;
        use crate::network::netplanservice::NetplanService;
        use crate::network::networkservice::NetworkService;
        use crate::platformid::{PlatformId, ProcessorArchitecture};
        use crate::shell::Shell;
        use log::error;

        const NETPLAN_DIR: &str = "/etc/netplan";
        const NETPLAN_CONF: &str = "/etc/netplan/99-network-manager-config.yaml";
        const NETPLAN_COMMAND: &str = "/usr/sbin/netplan";
        const DHCPCD_CONF: &str = "/etc/dhcpcd.conf";
        const DNSMASQ_CONF: &str = "/etc/dnsmasq.conf";
        const HOSTAPD_CONF: &str = "/etc/hostapd/hostapd.conf";
        const SYSTEMCTL_COMMAND: &str = "/usr/bin/systemctl";
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
/// The `NetworkManager` struct is responsible for managing network configurations and services
/// on a machine.
pub struct NetworkManager {
    /// A map of network configurations by name.
    configurations: HashMap<String, NetworkConfiguration>,
}

impl NetworkManager {
    /// Constructs a new `NetworkManager`.
    pub fn new() -> Self {
        NetworkManager {
            configurations: HashMap::new(),
        }
    }

    /// Adds a network configuration to the manager.
    ///
    /// # Arguments
    ///
    /// * `configuration` - The network configuration to add.
    pub fn add_configuration(&mut self, configuration: NetworkConfiguration) {
        self.configurations
            .insert(configuration.get_name(), configuration);
    }

    /// Check if the network manager has a configuration for an interface with the specified name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the network configuration for which to check.
    ///
    /// # Returns
    ///
    /// `true` if the network manager has a configuration for the specified name, `false` otherwise.
    pub fn has_configuration_for_name(&self, name: &str) -> bool {
        self.configurations.contains_key(name)
    }

    /// Get a network configuration by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the network configuration to get.
    ///
    /// # Returns
    ///
    /// A reference to the network configuration with the specified name, or `None` if no such
    /// configuration exists.
    pub fn get_configuration(&self, name: &str) -> Option<&NetworkConfiguration> {
        self.configurations.get(name)
    }

    /// Get a mutable network configuration by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the network configuration to get.
    ///
    /// # Returns
    ///
    /// A reference to the mutable network configuration with the specified name, or `None` if no such
    /// configuration exists.
    pub fn get_configuration_mut(&mut self, name: &str) -> Option<&mut NetworkConfiguration> {
        self.configurations.get_mut(name)
    }

    /// Remove a network configuration by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the network configuration to remove.
    pub fn remove_configuration(&mut self, name: &str) {
        self.configurations.remove(name);
    }

    /// Return true if any network configuration has an enabled wifi configuration.
    pub fn is_wireless_enabled(&self) -> bool {
        self.configurations
            .values()
            .any(|c| c.enabled && c.is_wireless_enabled())
    }

    /// Return true if any network configuration has an enabled ethernet configuration.
    pub fn is_ethernet_enabled(&self) -> bool {
        self.configurations
            .values()
            .any(|c| c.enabled && !c.is_wireless_enabled())
    }

    /// Return the number of interfaces with wireless configurations.
    pub fn get_number_of_wireless_configurations(&self) -> usize {
        self.configurations
            .values()
            .filter(|c| c.is_wireless_enabled())
            .count()
    }

    /// Return the number of interfaces with ethernet configurations.
    pub fn get_number_of_ethernet_configurations(&self) -> usize {
        self.configurations
            .values()
            .filter(|c| !c.is_wireless_enabled())
            .count()
    }

    /// Return the interface names of all wireless configurations.
    pub fn get_wireless_configuration_names(&self) -> Vec<String> {
        self.configurations
            .values()
            .filter(|c| c.is_wireless_enabled())
            .map(|c| c.get_name())
            .collect()
    }

    /// Return the interface names of all ethernet configurations.
    pub fn get_ethernet_configuration_names(&self) -> Vec<String> {
        self.configurations
            .values()
            .filter(|c| !c.is_wireless_enabled())
            .map(|c| c.get_name())
            .collect()
    }

    /// Return the name of the primary wireless interface.
    pub fn get_primary_wireless_configuration_name(&self) -> Option<String> {
        self.configurations
            .values()
            .filter(|c| c.is_wireless_enabled())
            .filter(|c| c.interface.has_ipv4_address())
            .find(|c| c.enabled)
            .map(|c| c.get_name())
    }

    /// Return the name of the primary ethernet interface.
    pub fn get_primary_ethernet_configuration_name(&self) -> Option<String> {
        self.configurations
            .values()
            .filter(|c| !c.is_wireless_enabled() && !c.interface.is_loopback_interface())
            .filter(|c| c.interface.has_ipv4_address())
            .find(|c| c.enabled)
            .map(|c| c.get_name())
    }

    /// Remove all the network configurations from the manager.
    pub fn clear(&mut self) {
        self.configurations.clear();
    }

    /// Load network settings from the system configuration into the manager.
    pub fn load_settings_from_system(&mut self) {
        // Load network interfaces currently running on the system.
        let interfaces = NetworkInterface::load();

        for interface in interfaces {
            let mut configuration = NetworkConfiguration::new_with_interface(interface.clone());
            if configuration.interface.addresses.is_empty() {
                configuration.enabled = false;
            }

            self.configurations
                .insert(interface.name.clone(), configuration);
        }

        cfg_if! {
            if #[cfg(target_os = "linux")] {
                let platform_id = PlatformId::new();
                if platform_id.vendor == "Ubuntu" &&
                    platform_id.processor_architecture == ProcessorArchitecture::X86_64 {
                    // We are running on Ubuntu 64-bit, assume we have access to the Netplan service.

                    // Get the netplan .yaml files.
                    let mut netplan_yaml_files = match std::fs::read_dir(NETPLAN_DIR) {
                        Ok(entries) => {
                            entries.into_iter()
                                .filter(|entry| entry.is_ok())
                                .filter(|entry| entry.as_ref().unwrap().path().extension().unwrap_or_default() == "yaml")
                                .map(|entry| entry.unwrap().path())
                                .collect::<Vec<_>>()
                        },
                        Err(_) => return,
                    };

                    netplan_yaml_files.sort();

                    for yaml_path in netplan_yaml_files {
                        debug!("Loading {:?}", yaml_path);
                        let mut netplan_service = NetplanService::new(yaml_path.clone());
                        if let Err(e) = netplan_service.load_configuration(&mut self.configurations) {
                            error!("Failed to load Netplan configuration from {}: {}", yaml_path.to_string_lossy(), e);
                        }
                    }
                } else if platform_id.name == "Debian" &&
                    (platform_id.processor_architecture == ProcessorArchitecture::ARM64 || platform_id.processor_architecture == ProcessorArchitecture::ARM) {
                    // We are running on Debian ARM box, probably a Raspberry Pi. Assume we have access to the dhcpcd service.

                    let config_file = std::path::PathBuf::from(DHCPCD_CONF);
                    if config_file.exists() {
                        let mut dhcpcd_service = DHCPCDService::new(config_file.clone());
                        if let Err(e) = dhcpcd_service.load_configuration(&mut self.configurations)  {
                            error!("Failed to load DHCPCD configuration from {}: {}", config_file.to_string_lossy(), e);
                        }
                    }
                }

                let dnsmasq_config_file = std::path::PathBuf::from(DNSMASQ_CONF);
                if dnsmasq_config_file.exists() {
                    let mut dnsmasq_service = DNSMasqService::new(dnsmasq_config_file.clone());
                    if let Err(e) = dnsmasq_service.load_configuration(&mut self.configurations) {
                        error!("Failed to load DNSMasq configuration from {}: {}", dnsmasq_config_file.to_string_lossy(), e);
                    }
                }

                let hostapd_config_file = std::path::PathBuf::from(HOSTAPD_CONF);
                if hostapd_config_file.exists() {
                    let mut hostapd_service = HostAPDService::new(hostapd_config_file.clone());
                    if let Err(e) = hostapd_service.load_configuration(&mut self.configurations) {
                        error!("Failed to load HostAPD configuration from {}: {}", hostapd_config_file.to_string_lossy(), e);
                    }
                }
            }
        }
    }

    /// Save network settings from the manager to the system configuration.
    ///
    /// This method will write the network configurations to the system configuration files and
    /// restart the necessary services to apply the changes.
    pub fn save_settings_to_system(&self) {
        cfg_if! {
            if #[cfg(target_os = "linux")] {
                let dnsmasq_config_file = std::path::PathBuf::from(DNSMASQ_CONF);
                let dnsmasq_service = DNSMasqService::new(dnsmasq_config_file.clone());
                if let Err(e) = dnsmasq_service.write_configuration(&self.configurations) {
                    error!("Failed to write DNSMasq configuration to {}: {}", dnsmasq_config_file.to_string_lossy(), e);
                }

                Shell::execute(SYSTEMCTL_COMMAND, vec!["restart".to_string(), "dnsmasq".to_string()]);

                let hostapd_config_file = std::path::PathBuf::from(HOSTAPD_CONF);
                let hostapd_service = HostAPDService::new(hostapd_config_file.clone());
                if let Err(e) = hostapd_service.write_configuration(&self.configurations) {
                    error!("Failed to write HostAPD configuration to {}: {}", hostapd_config_file.to_string_lossy(), e);
                }

                Shell::execute(SYSTEMCTL_COMMAND, vec!["restart".to_string(), "hostapd".to_string()]);

                let platform_id = PlatformId::new();

                if platform_id.name == "Ubuntu" &&
                    platform_id.processor_architecture == ProcessorArchitecture::X86_64 {

                    // Find the .yaml netplan files.
                    let netplan_yaml_files = match std::fs::read_dir(NETPLAN_DIR) {
                        Ok(entries) => {
                            entries.into_iter()
                                .filter(|entry| entry.is_ok())
                                .filter(|entry| entry.as_ref().unwrap().path().extension().unwrap_or_default() == "yaml")
                                .filter(|entry| entry.as_ref().unwrap().path().exists())
                                .map(|entry| entry.unwrap().path())
                                .collect::<Vec<_>>()
                        },
                        Err(e) => {
                            error!("Failed to read directory {}: {}", NETPLAN_DIR, e);
                            vec![]
                        },
                    };

                    for yaml_path in netplan_yaml_files {
                        let new_yaml_path = yaml_path.with_extension("yaml.orig");
                        if let Err(e) = std::fs::rename(&yaml_path, &new_yaml_path) {
                            error!("Failed to rename {} to {}: {}", yaml_path.to_string_lossy(), new_yaml_path.to_string_lossy(), e);
                            continue;
                        }
                    }

                    let netplan_config_file = std::path::PathBuf::from(NETPLAN_CONF);
                    let netplan_service = NetplanService::new(netplan_config_file.clone());
                    if let Err(e) = netplan_service.write_configuration(&self.configurations) {
                        error!("Failed to write Netplan configuration to {}: {}", netplan_config_file.to_string_lossy(), e);
                    }

                    Shell::execute(NETPLAN_COMMAND, vec!["apply".to_string()]);
                } else if platform_id.name == "Debian" &&
                    (platform_id.processor_architecture == ProcessorArchitecture::ARM64 || platform_id.processor_architecture == ProcessorArchitecture::ARM) {
                    let dhcpcd_config_file = std::path::PathBuf::from(DHCPCD_CONF);
                    let dhcpcd_service = DHCPCDService::new(dhcpcd_config_file.clone());
                    if let Err(e) = dhcpcd_service.write_configuration(&self.configurations) {
                        error!("Failed to write DHCPCD configuration to {}: {}", dhcpcd_config_file.to_string_lossy(), e);
                    }

                    Shell::execute(SYSTEMCTL_COMMAND, vec!["restart".to_string(), "dhcpcd".to_string()]);
                }
            }
        }
    }
}
