//! The `netplanservice` module provides code that interacts with the Netplan service on a Linux
//! machine.

use crate::error::FoundationError;
use crate::network::interfaceaddr::InterfaceAddr;
use crate::network::ipaddrquery::IpAddrQuery;
use crate::network::networkconfiguration::{AddressMode, NetworkConfiguration};
use crate::network::networkservice::NetworkService;
use crate::network::wireless::configuration::{WirelessConfiguration, WirelessMode};
use crate::systemctlservice::SystemCTLService;
use log::{debug, error};
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserialize, Serializer};
use serde_yaml::Value;
use std::collections::HashMap;
use std::net::IpAddr;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

/// The service object.
pub struct NetplanService {
    /// The path to the configuration file.
    filename: PathBuf,
    service: SystemCTLService,
}

impl NetplanService {
    /// Create a new NetplanService object.
    pub fn new(filename: PathBuf) -> NetplanService {
        NetplanService {
            filename,
            service: SystemCTLService::new("netplan".to_string()),
        }
    }
}

fn load_wifi_config_helper(
    config_map: &mut HashMap<String, NetworkConfiguration>,
    name: &str,
    wifis_value: &Value,
) -> String {
    // The keys for the wifis map might be the name of an interface,
    // or it might be the name of a configuration with a match key
    // that specifies the interface name.

    let mut interface_name = name.to_string();

    match config_map.get_mut(name) {
        None => match wifis_value.as_mapping() {
            Some(wifis_map) => match wifis_map.get("match") {
                Some(match_value) => match match_value.as_mapping() {
                    Some(match_map) => match match_map.get("name") {
                        Some(name_value) => match name_value.as_str() {
                            Some(name_value_str) => {
                                interface_name = name_value_str.to_string();
                            }
                            None => {}
                        },
                        None => {}
                    },
                    None => {}
                },
                None => {}
            },
            None => {}
        },
        _ => {}
    }

    match config_map.get_mut(&interface_name) {
        None => {
            let config = NetworkConfiguration::new_with_name(&interface_name);
            config_map.insert(interface_name.clone(), config);
        }
        _ => {}
    }

    return interface_name;
}

impl NetworkService for NetplanService {
    /// Load the network configurations from the Netplan configuration file.
    /// Insert a new configuration file in the configuration map or update the existing configuration
    /// if the map already has an entry for a given network interface.
    ///
    /// # Arguments
    ///
    /// * `config_map` - A map of configuration names to network configuration objects.
    ///
    /// # Returns
    ///
    /// Ok(()) on success for a FoundationError if an error occurs.
    fn load_configuration(
        &mut self,
        config_map: &mut HashMap<String, NetworkConfiguration>,
    ) -> Result<(), FoundationError> {
        match std::fs::File::open(&self.filename) {
            Ok(file) => {
                let deserializer = serde_yaml::Deserializer::from_reader(file);
                match Value::deserialize(deserializer) {
                    Ok(value) => {
                        // Now we suck out the data we need from the netplan YAML file.
                        if let Some(network) = value.get("network") {
                            if !network.as_mapping().is_some() {
                                return Err(FoundationError::OperationFailed(
                                    "The 'network' key is not a mapping".to_string(),
                                ));
                            }

                            if let Some(ethernets) = network.get("ethernets") {
                                if !ethernets.as_mapping().is_some() {
                                    return Err(FoundationError::OperationFailed(
                                        "The 'ethernets' key is not a mapping".to_string(),
                                    ));
                                }

                                // We just checked that ethernets *is* a mapping, so we can unwrap here.
                                for (name, ethernets_value) in ethernets.as_mapping().unwrap() {
                                    if !name.as_str().is_some() {
                                        debug!("The 'ethernets' mapping contains a key that is not a string {:?}", name);
                                        continue;
                                    }

                                    if !ethernets_value.as_mapping().is_some() {
                                        debug!(
                                            "The value for the '{}' key is not a mapping",
                                            name.as_str().unwrap()
                                        );
                                        continue;
                                    }

                                    let interface_name = name.as_str().unwrap();

                                    let configuration =
                                        if let Some(config) = config_map.get_mut(interface_name) {
                                            config
                                        } else {
                                            let config =
                                                NetworkConfiguration::new_with_name(interface_name);
                                            config_map.insert(interface_name.to_string(), config);
                                            config_map.get_mut(interface_name).unwrap()
                                        };

                                    for (inner_name, inner_value) in
                                        ethernets_value.as_mapping().unwrap()
                                    {
                                        if !inner_name.as_str().is_some() {
                                            debug!("The {} mapping contains a key that is not a string {:?}", interface_name, inner_name);
                                            continue;
                                        }

                                        let inner_key = inner_name.as_str().unwrap();

                                        if inner_key == "dhcp" {
                                            if !inner_value.as_str().is_some() {
                                                debug!("The {} mapping contains a 'dhcp4' key with a value that is not a string", interface_name);
                                                continue;
                                            }

                                            let dhcp_value = inner_value.as_str().unwrap();
                                            if dhcp_value == "true" {
                                                match inner_key {
                                                    "dhcp4" | "dhcp6" => {
                                                        configuration.address_mode =
                                                            AddressMode::DHCP
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        } else if inner_key == "addresses"
                                            && inner_value.as_sequence().is_some()
                                        {
                                            for address in inner_value.as_sequence().unwrap() {
                                                if !address.as_str().is_some() {
                                                    debug!("The {} mapping contains an 'addresses' key with a value that is not a string", interface_name);
                                                    continue;
                                                }
                                                let address_value = address.as_str().unwrap();
                                                if let Ok(address) =
                                                    InterfaceAddr::try_from(address_value)
                                                {
                                                    configuration.interface.addresses.push(address);
                                                }
                                            }
                                            configuration.address_mode = AddressMode::Static;
                                        } else if inner_key == "nameservers"
                                            && inner_value.as_mapping().is_some()
                                        {
                                            if let Some(address_value) =
                                                inner_value.as_mapping().unwrap().get("addresses")
                                            {
                                                if let Some(addresses) = address_value.as_sequence()
                                                {
                                                    for address in addresses {
                                                        if let Some(address_str) = address.as_str()
                                                        {
                                                            configuration
                                                                .interface
                                                                .nameserver_addresses
                                                                .push(
                                                                    <IpAddr as IpAddrQuery>::from(
                                                                        address_str,
                                                                    )?,
                                                                );
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    configuration.enabled = true;
                                }
                            }

                            if let Some(wifis) = network.get("wifis") {
                                if !wifis.as_mapping().is_some() {
                                    return Err(FoundationError::OperationFailed(
                                        "The 'wifis' key is not a mapping".to_string(),
                                    ));
                                }

                                for (name, wifis_value) in wifis.as_mapping().unwrap() {
                                    if !name.as_str().is_some() {
                                        debug!("The 'wifis' mapping contains a key that is not a string {:?}", name);
                                        continue;
                                    }

                                    if !wifis_value.as_mapping().is_some() {
                                        debug!(
                                            "The value for the '{}' key is not a mapping",
                                            name.as_str().unwrap()
                                        );
                                        continue;
                                    }

                                    // The keys for the wifis map might be the name of an interface,
                                    // or it might be the name of a configuration with a match key
                                    // that specifies the interface name.

                                    // Try to get a previously named configuration
                                    let temp_name = name.as_str().unwrap();

                                    let interface_name =
                                        load_wifi_config_helper(config_map, temp_name, wifis_value);

                                    let configuration =
                                        if let Some(config) = config_map.get_mut(&interface_name) {
                                            config
                                        } else {
                                            error!(
                                                "Failed to get valid configuration for {}",
                                                interface_name
                                            );
                                            continue;
                                        };

                                    for (inner_name, inner_value) in
                                        wifis_value.as_mapping().unwrap()
                                    {
                                        if !inner_name.as_str().is_some() {
                                            debug!("The {} mapping contains a key that is not a string {:?}", interface_name, inner_name);
                                            continue;
                                        }

                                        let inner_key = inner_name.as_str().unwrap();

                                        if inner_key == "dhcp4" || inner_key == "dhcp6" {
                                            if let Some(bool_value) = inner_value.as_str() {
                                                if bool_value == "true" {
                                                    match inner_key {
                                                        "dhcp4" | "dhcp6" => {
                                                            configuration.address_mode =
                                                                AddressMode::DHCP
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                            } else if let Some(bool_value) = inner_value.as_bool() {
                                                if bool_value {
                                                    match inner_key {
                                                        "dhcp4" | "dhcp6" => {
                                                            configuration.address_mode =
                                                                AddressMode::DHCP
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                            }
                                        } else if inner_key == "access-points" {
                                            if let Some(access_points) = inner_value.as_mapping() {
                                                let wireless_config = if let Some(config) =
                                                    &mut configuration.wifi_configuration
                                                {
                                                    config
                                                } else {
                                                    configuration.wifi_configuration =
                                                        Some(WirelessConfiguration::default());
                                                    configuration
                                                        .wifi_configuration
                                                        .as_mut()
                                                        .unwrap()
                                                };
                                                for (point_name, point_value) in access_points {
                                                    if let Some(point_str) = point_name.as_str() {
                                                        wireless_config.ssid =
                                                            point_str.to_string();
                                                    }
                                                    if let Some(ssid_map) = point_value.as_mapping()
                                                    {
                                                        for (ssid_key, ssid_value) in ssid_map {
                                                            if let Some(key_str) = ssid_key.as_str()
                                                            {
                                                                if key_str == "password" {
                                                                    if let Some(password_str) =
                                                                        ssid_value.as_str()
                                                                    {
                                                                        wireless_config.password =
                                                                            Some(
                                                                                password_str
                                                                                    .to_string(),
                                                                            );
                                                                        break;
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    configuration.enabled = true;
                                }
                            }
                        }
                        Ok(())
                    }
                    Err(error) => Err(FoundationError::SerdeYamlError(error)),
                }
            }
            Err(e) => Err(FoundationError::IO(e)),
        }
    }

    /// Write a set of network configuration settings to the Netplan configuration files.
    /// The function only writes the portions of the configuration that are handled by
    /// Netplan.
    ///
    /// # Arguments
    ///
    /// * `configurations` - A map of interface names to network configurations.
    ///
    /// # Returns
    ///
    /// Ok(()) on success or a FoundationError if a problem occurs.
    fn write_configuration(
        &self,
        configurations: &HashMap<String, NetworkConfiguration>,
    ) -> Result<(), FoundationError> {
        match std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&self.filename)
        {
            Ok(mut file) => {
                let should_use_config_for_ethernets = |config: &NetworkConfiguration| {
                    if config.interface.is_loopback_interface() {
                        return false;
                    }
                    (config.enabled && config.wifi_configuration.is_none())
                        || (config.enabled
                            && config.wifi_configuration.is_some()
                            && (config.wifi_configuration.as_ref().unwrap().mode
                                == WirelessMode::AccessPoint
                                || (config.wifi_configuration.as_ref().unwrap().mode
                                    == WirelessMode::Client
                                    && config.address_mode == AddressMode::Static)))
                };

                let needs_ethernet_section = configurations
                    .values()
                    .any(|c| should_use_config_for_ethernets(c));

                let needs_wifi_section = configurations.values().any(|c| {
                    c.enabled
                        && c.wifi_configuration.is_some()
                        && c.wifi_configuration.as_ref().unwrap().mode == WirelessMode::Client
                });

                let mut serializer = serde_yaml::Serializer::new(&mut file);
                let mut network_map = serializer.serialize_map(None)?;
                network_map.serialize_key("network")?;
                let mut netmap_inner_map = network_map.serialize_map(None)?;
                netmap_inner_map.serialize_entry("version", &2)?;
                netmap_inner_map.serialize_entry("renderer", "networkd")?;

                if needs_ethernet_section {
                    netmap_inner_map.serialize_key("ethernets")?;
                    let mut ethernets_map = netmap_inner_map.serialize_map(None)?;
                    for config in configurations.values() {
                        if should_use_config_for_ethernets(config) {
                            ethernets_map.serialize_key(&config.interface.name)?;
                            let mut inner_map = ethernets_map.serialize_map(None)?;
                            if config.address_mode == AddressMode::DHCP {
                                inner_map.serialize_entry("dhcp4", &true)?;
                            } else {
                                // Need to write out static addresses.
                                inner_map.serialize_key("addresses")?;
                                let mut addresses_array = inner_map.serialize_seq(None)?;
                                for address in &config.interface.addresses {
                                    if address.ip.is_ipv6() && !address.ip.is_global_address() {
                                        continue;
                                    }
                                    addresses_array
                                        .serialize_element(&address.get_in_cidr_notation())?;
                                }
                                SerializeSeq::end(addresses_array)?;

                                if config.interface.nameserver_addresses.len() > 0 {
                                    inner_map.serialize_key("nameservers")?;
                                    let mut nameservers_map = inner_map.serialize_map(None)?;
                                    nameservers_map.serialize_key("addresses")?;
                                    let mut addresses_array =
                                        nameservers_map.serialize_seq(None)?;
                                    for address in &config.interface.nameserver_addresses {
                                        addresses_array.serialize_element(&address.to_string())?;
                                    }
                                    SerializeSeq::end(addresses_array)?;
                                    SerializeMap::end(nameservers_map)?;
                                }
                            }
                            inner_map.serialize_entry("optional", &true)?;
                            SerializeMap::end(inner_map)?;
                        }
                    }
                    if let Err(e) = SerializeMap::end(ethernets_map) {
                        error!("Error end-serializing ethernets map: {:?}", e);
                        return Err(FoundationError::SerdeYamlError(e));
                    }
                }

                if needs_wifi_section {
                    netmap_inner_map.serialize_key("wifis")?;
                    let mut wifis_map = netmap_inner_map.serialize_map(None)?;
                    for config in configurations.values() {
                        if !config.enabled
                            || config.wifi_configuration.is_none()
                            || config.wifi_configuration.as_ref().unwrap().mode
                                != WirelessMode::Client
                        {
                            continue;
                        }
                        wifis_map.serialize_key(&config.interface.name)?;
                        let mut individual_wifi_map = wifis_map.serialize_map(None)?;
                        individual_wifi_map.serialize_entry("optional", &true)?;
                        if config.address_mode == AddressMode::DHCP {
                            individual_wifi_map
                                .serialize_entry(&format!("{}", config.address_mode), &true)?;
                        }
                        individual_wifi_map.serialize_key("access-points")?;
                        let mut access_points_map = individual_wifi_map.serialize_map(None)?;
                        if let Some(wifi_config) = config.wifi_configuration.as_ref() {
                            access_points_map.serialize_key(&wifi_config.ssid)?;

                            if let Some(password) = &wifi_config.password {
                                let mut ssid_map = access_points_map.serialize_map(None)?;
                                ssid_map.serialize_entry("password", password)?;
                                SerializeMap::end(ssid_map)?;
                            }
                        }

                        SerializeMap::end(access_points_map)?;
                        SerializeMap::end(individual_wifi_map)?;
                    }
                    SerializeMap::end(wifis_map)?;
                }

                SerializeMap::end(netmap_inner_map)?;
                SerializeMap::end(network_map)?;

                serializer.flush()?;

                let metadata = file.metadata()?;
                let mut permissions = metadata.permissions();

                // Set the permissions.
                permissions.set_mode(0o400);
                std::fs::set_permissions(&self.filename, permissions)?;

                Ok(())
            }
            Err(e) => Err(FoundationError::IO(e)),
        }
    }

    // Technically, netplan is not a service or daemon, but a configuration generator that converts
    // yaml files into configs for a backend renderer like systemd-networkd or NetworkManager. As
    // such we are not really starting or stopping the actual network layer, but rather applying
    // the configuration changes.

    /// Return the path to the service configuration file.
    fn get_configuration_file(&self) -> PathBuf {
        return self.filename.clone();
    }

    fn start(&self) -> Result<(), FoundationError> {
        let output = Command::new("/usr/sbin/netplan").arg("apply").output()?;
        if !output.status.success() {
            return Err(FoundationError::OperationFailed(format!(
                "Failed to start service: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        Ok(())
    }

    fn stop(&self) -> Result<(), FoundationError> {
        Ok(())
    }

    fn restart(&self) -> Result<(), FoundationError> {
        self.start()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::networkinterface::NetworkInterface;
    use std::net::Ipv4Addr;

    // Note that this service can lose configuration fidelity in the sense that the netplan configuration
    // file does not contain all settings supported by this library's notion of a network configuration.
    // When testing, be sure to understand what the service supports so that you only add enough to
    // configuration to test the service's ability to read and write the configuration file.  If you
    // add more, then the configurations will not match after you write the config, read it back and
    // then compare it to the read results (because the read config will contain less information).

    #[test]
    fn test_ethernet_configuration() {
        let mut config_map = HashMap::new();

        let mut interface = NetworkInterface::new_with_name("eth0");
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2)),
            None,
            Some(IpAddr::V4(Ipv4Addr::new(255, 255, 255, 0))),
        ));
        interface
            .nameserver_addresses
            .push(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)));
        let config = NetworkConfiguration::new(AddressMode::Static, interface, true, None, None);
        config_map.insert("eth0".to_string(), config);

        let eth1_interface = NetworkInterface::new_with_name("eth1");
        let config2 =
            NetworkConfiguration::new(AddressMode::DHCP, eth1_interface, true, None, None);
        config_map.insert("eth1".to_string(), config2);

        let mut netplan_service = NetplanService::new(PathBuf::from("/tmp/netplan.yaml"));
        let result = netplan_service.write_configuration(&config_map);
        assert!(result.is_ok());

        // Now try to read the configuration back in.
        let mut read_config_map: HashMap<String, NetworkConfiguration> = HashMap::new();
        let result = netplan_service.load_configuration(&mut read_config_map);
        assert!(result.is_ok());

        assert_eq!(read_config_map.len(), 2);
        assert_eq!(read_config_map, config_map);

        netplan_service.remove_config_file().unwrap();
    }

    #[test]
    fn test_wifi_configuration() {
        let mut config_map: HashMap<String, NetworkConfiguration> = HashMap::new();

        let mut interface = NetworkInterface::new_with_name("wlan0");
        interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 3)),
            None,
            Some(IpAddr::V4(Ipv4Addr::new(255, 255, 255, 0))),
        ));
        interface
            .nameserver_addresses
            .push(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)));
        let mut wifi_config = WirelessConfiguration::default();
        wifi_config.ssid = "PeanutButter".to_string();
        wifi_config.password = Some("Jelly Time".to_string());
        wifi_config.mode = WirelessMode::Client;
        let config = NetworkConfiguration::new(
            AddressMode::Static,
            interface,
            true,
            Some(wifi_config),
            None,
        );
        config_map.insert("wlan0".to_string(), config);

        let interface2 = NetworkInterface::new_with_name("wlan1");
        let mut wifi_config2 = WirelessConfiguration::default();
        wifi_config2.ssid = "HamSandwich".to_string();
        wifi_config2.password = Some("RhyBreadWithCrust".to_string());
        wifi_config2.mode = WirelessMode::Client;
        let config2 = NetworkConfiguration::new(
            AddressMode::DHCP,
            interface2,
            true,
            Some(wifi_config2),
            None,
        );
        config_map.insert("wlan1".to_string(), config2);

        let mut netplan_service = NetplanService::new(PathBuf::from("/tmp/wifi_netplan.yaml"));
        let result = netplan_service.write_configuration(&config_map);
        assert!(result.is_ok());

        // Now try to read the configuration back in.
        let mut read_config_map: HashMap<String, NetworkConfiguration> = HashMap::new();
        let result = netplan_service.load_configuration(&mut read_config_map);
        assert!(result.is_ok());

        assert_eq!(read_config_map.len(), 2);
        assert_eq!(read_config_map, config_map);

        netplan_service.remove_config_file().unwrap();
    }
}
