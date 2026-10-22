//! The `netplanservice` module provides code that interacts with the Netplan service on a Linux
//! machine.
//!
//! Netplan reads every `.yaml` file in `/etc/netplan` in name order and merges the files. A later
//! file replaces the scalar values of a definition with the same id and appends to its lists.
//! `NetplanService::write_changes` uses that rule: it writes only the definitions that differ from
//! what the other files give, under the ids that those files use, so the other files keep their
//! settings.

use crate::error::FoundationError;
use crate::network::interfaceaddr::InterfaceAddr;
use crate::network::ipaddrquery::IpAddrQuery;
use crate::network::networkconfiguration::{AddressMode, NetworkConfiguration};
use crate::network::networkservice::NetworkService;
use crate::network::wireless::configuration::{WirelessConfiguration, WirelessMode};
use log::debug;
use serde::Deserialize;
use serde_yaml::{Mapping, Value};
use std::collections::HashMap;
use std::io::Write;
use std::net::IpAddr;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::PathBuf;
use std::process::Command;

/// The netplan section that holds Ethernet definitions.
const ETHERNETS: &str = "ethernets";

/// The netplan section that holds Wi-Fi client definitions.
const WIFIS: &str = "wifis";

/// The service object.
pub struct NetplanService {
    /// The path to the configuration file.
    filename: PathBuf,

    /// The netplan id of each interface, by interface name. A definition that picks its interface
    /// with a `match` rule has an id that differs from the interface name.
    interface_ids: HashMap<String, String>,
}

impl NetplanService {
    /// Create a new NetplanService object.
    pub fn new(filename: PathBuf) -> NetplanService {
        NetplanService {
            filename,
            interface_ids: HashMap::new(),
        }
    }

    /// Set the netplan ids that other files give each interface, by interface name. Loading then
    /// applies a definition with one of these ids to its interface, and `write_changes` writes
    /// each interface's definition under its id.
    pub fn set_interface_ids(&mut self, interface_ids: HashMap<String, String>) {
        self.interface_ids = interface_ids;
    }

    /// Return the netplan id of each interface, by interface name: the ids that
    /// `set_interface_ids` gave, and the ids of the definitions that loading found.
    pub fn interface_ids(&self) -> &HashMap<String, String> {
        &self.interface_ids
    }

    /// Return the name of the interface that the definition `id` configures, and record its id.
    fn resolve_interface(
        &mut self,
        id: &str,
        definition: &Value,
        config_map: &HashMap<String, NetworkConfiguration>,
    ) -> String {
        if let Some((name, _)) = self.interface_ids.iter().find(|(_, known)| *known == id) {
            return name.clone();
        }
        let name = matched_interface(definition, config_map).unwrap_or_else(|| id.to_string());
        self.interface_ids
            .entry(name.clone())
            .or_insert_with(|| id.to_string());
        name
    }

    /// Write the definitions of the configurations that differ from `base` to the configuration
    /// file, and remove the file when none differ.
    ///
    /// `base` holds the configurations that the system's other netplan files give. Each
    /// definition uses the id that `set_interface_ids` gave its interface, so netplan merges it
    /// with the other files' definition of that interface. A configuration that `base` enables
    /// and `configurations` disables gets `activation-mode: off`.
    ///
    /// Netplan appends lists from later files, so this file can add addresses and nameservers
    /// to an interface, and cannot remove the ones that another file gives it.
    ///
    /// # Arguments
    ///
    /// * `configurations` - A map of interface names to network configurations.
    /// * `base` - A map of interface names to the configurations that the other files give.
    ///
    /// # Returns
    ///
    /// Ok(()) on success or a FoundationError if a problem occurs.
    pub fn write_changes(
        &self,
        configurations: &HashMap<String, NetworkConfiguration>,
        base: &HashMap<String, NetworkConfiguration>,
    ) -> Result<(), FoundationError> {
        let mut sections = Sections::default();
        let mut names = configurations.keys().collect::<Vec<_>>();
        names.sort();
        for name in names {
            let current = definitions(&configurations[name]);
            let earlier = base.get(name).map(definitions).unwrap_or_default();
            if current == earlier {
                continue;
            }
            let id = self
                .interface_ids
                .get(name)
                .cloned()
                .unwrap_or_else(|| name.clone());
            if current.is_empty() {
                for (section, _) in &earlier {
                    let mut off = Mapping::new();
                    off.insert("activation-mode".into(), "off".into());
                    sections.add(section, &id, off);
                }
            }
            for (section, definition) in current {
                sections.add(section, &id, definition);
            }
        }

        if sections.is_empty() {
            debug!("No netplan changes for {}", self.filename.to_string_lossy());
            return match std::fs::remove_file(&self.filename) {
                Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(FoundationError::IO(e)),
                _ => Ok(()),
            };
        }
        write_document(&self.filename, sections.into_document(None))
    }
}

/// The definitions of one netplan file, by section and then by id.
#[derive(Default)]
struct Sections {
    ethernets: Mapping,
    wifis: Mapping,
}

impl Sections {
    fn add(&mut self, section: &str, id: &str, definition: Mapping) {
        let map = if section == WIFIS {
            &mut self.wifis
        } else {
            &mut self.ethernets
        };
        map.insert(id.into(), Value::Mapping(definition));
    }

    fn is_empty(&self) -> bool {
        self.ethernets.is_empty() && self.wifis.is_empty()
    }

    /// Return the netplan document that holds these definitions, with `renderer` when given.
    fn into_document(self, renderer: Option<&str>) -> Value {
        let mut network = Mapping::new();
        network.insert("version".into(), 2.into());
        if let Some(renderer) = renderer {
            network.insert("renderer".into(), renderer.into());
        }
        if !self.ethernets.is_empty() {
            network.insert(ETHERNETS.into(), Value::Mapping(self.ethernets));
        }
        if !self.wifis.is_empty() {
            network.insert(WIFIS.into(), Value::Mapping(self.wifis));
        }
        let mut document = Mapping::new();
        document.insert("network".into(), Value::Mapping(network));
        Value::Mapping(document)
    }
}

/// Write a netplan document to `path`. The function writes a file beside it and renames that
/// file over `path`, so netplan never reads a partly written file. Netplan wants its files
/// readable only by root.
fn write_document(path: &PathBuf, document: Value) -> Result<(), FoundationError> {
    let text = serde_yaml::to_string(&document)?;
    let mut temporary = path.clone().into_os_string();
    temporary.push(".new");
    let temporary = PathBuf::from(temporary);
    let _ = std::fs::remove_file(&temporary);
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temporary)?;
    file.write_all(text.as_bytes())?;
    file.sync_all()?;
    drop(file);
    std::fs::set_permissions(&temporary, std::fs::Permissions::from_mode(0o600))?;
    std::fs::rename(&temporary, path)?;
    Ok(())
}

/// Return the netplan definitions that a configuration needs, with the section of each. A
/// disabled configuration and the loopback interface need none. An access point needs an
/// Ethernet definition for its address, since hostapd drives the radio. A Wi-Fi client needs a
/// Wi-Fi definition, and a static Wi-Fi client also an Ethernet definition for its address.
fn definitions(config: &NetworkConfiguration) -> Vec<(&'static str, Mapping)> {
    let mut definitions = vec![];
    if !config.enabled {
        return definitions;
    }
    let client = config
        .wifi_configuration
        .as_ref()
        .filter(|wifi| wifi.mode == WirelessMode::Client);
    if !config.interface.is_loopback_interface()
        && (client.is_none() || config.address_mode == AddressMode::Static)
    {
        definitions.push((ETHERNETS, ethernet_definition(config)));
    }
    if let Some(wifi) = client {
        definitions.push((WIFIS, wifi_definition(config, wifi)));
    }
    definitions
}

fn ethernet_definition(config: &NetworkConfiguration) -> Mapping {
    let mut definition = Mapping::new();
    if config.address_mode == AddressMode::DHCP {
        definition.insert("dhcp4".into(), true.into());
    } else {
        let addresses = config
            .interface
            .addresses
            .iter()
            .filter(|address| !address.ip.is_ipv6() || address.ip.is_global_address())
            .filter_map(|address| address.get_in_cidr_notation())
            .map(Value::String)
            .collect::<Vec<_>>();
        definition.insert("addresses".into(), Value::Sequence(addresses));

        if !config.interface.nameserver_addresses.is_empty() {
            let nameservers = config
                .interface
                .nameserver_addresses
                .iter()
                .map(|address| Value::String(address.to_string()))
                .collect::<Vec<_>>();
            let mut nameservers_map = Mapping::new();
            nameservers_map.insert("addresses".into(), Value::Sequence(nameservers));
            definition.insert("nameservers".into(), Value::Mapping(nameservers_map));
        }
    }
    definition.insert("optional".into(), true.into());
    definition
}

fn wifi_definition(config: &NetworkConfiguration, wifi: &WirelessConfiguration) -> Mapping {
    let mut definition = Mapping::new();
    definition.insert("optional".into(), true.into());
    if config.address_mode == AddressMode::DHCP {
        definition.insert("dhcp4".into(), true.into());
    }
    let mut access_point = Mapping::new();
    if let Some(password) = &wifi.password {
        access_point.insert("password".into(), password.as_str().into());
    }
    let mut access_points = Mapping::new();
    access_points.insert(wifi.ssid.as_str().into(), Value::Mapping(access_point));
    definition.insert("access-points".into(), Value::Mapping(access_points));
    definition
}

/// Return the interface that a definition's `match` rule picks: the first interface on the
/// machine, by name, whose name fits the rule's `name` pattern and whose MAC address equals the
/// rule's `macaddress`. A `name` without wildcards names its interface even when the machine
/// does not have it.
fn matched_interface(
    definition: &Value,
    config_map: &HashMap<String, NetworkConfiguration>,
) -> Option<String> {
    let rule = definition.get("match")?.as_mapping()?;
    let name_pattern = rule.get("name").and_then(Value::as_str);
    let mac_address = rule.get("macaddress").and_then(Value::as_str);
    if name_pattern.is_none() && mac_address.is_none() {
        return None;
    }
    let mut names = config_map
        .iter()
        .filter(|(_, config)| config.interface.exists)
        .map(|(name, _)| name)
        .collect::<Vec<_>>();
    names.sort();
    let found = names.into_iter().find(|name| {
        let interface = &config_map[*name].interface;
        name_pattern.map_or(true, |pattern| glob_matches(pattern, &interface.name))
            && mac_address.map_or(true, |mac| {
                interface
                    .mac_addr
                    .as_deref()
                    .is_some_and(|own| own.eq_ignore_ascii_case(mac))
            })
    });
    match (found, name_pattern) {
        (Some(name), _) => Some(name.clone()),
        (None, Some(pattern)) if !pattern.contains(['*', '?']) => Some(pattern.to_string()),
        _ => None,
    }
}

/// Return true if `text` fits a shell pattern, in which `*` stands for any run of characters
/// and `?` for any one character.
fn glob_matches(pattern: &str, text: &str) -> bool {
    fn fits(pattern: &[char], text: &[char]) -> bool {
        match pattern.first() {
            None => text.is_empty(),
            Some('*') => (0..=text.len()).any(|skip| fits(&pattern[1..], &text[skip..])),
            Some('?') => !text.is_empty() && fits(&pattern[1..], &text[1..]),
            Some(c) => text.first() == Some(c) && fits(&pattern[1..], &text[1..]),
        }
    }
    let pattern = pattern.chars().collect::<Vec<_>>();
    let text = text.chars().collect::<Vec<_>>();
    fits(&pattern, &text)
}

/// Return true for a netplan boolean, which a file may write as a YAML boolean or as a string.
fn is_true(value: &Value) -> bool {
    value
        .as_bool()
        .unwrap_or_else(|| matches!(value.as_str(), Some("true" | "yes" | "on")))
}

impl NetworkService for NetplanService {
    /// Load the network configurations from the Netplan configuration file.
    /// Insert a new configuration file in the configuration map or update the existing configuration
    /// if the map already has an entry for a given network interface. A definition with a `match`
    /// rule applies to the interface that the rule picks.
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
        let file = std::fs::File::open(&self.filename).map_err(FoundationError::IO)?;
        let value = Value::deserialize(serde_yaml::Deserializer::from_reader(file))
            .map_err(FoundationError::SerdeYamlError)?;

        let Some(network) = value.get("network") else {
            return Ok(());
        };
        if network.as_mapping().is_none() {
            return Err(FoundationError::OperationFailed(
                "The 'network' key is not a mapping".to_string(),
            ));
        }

        if let Some(ethernets) = network.get(ETHERNETS) {
            let Some(ethernets) = ethernets.as_mapping() else {
                return Err(FoundationError::OperationFailed(
                    "The 'ethernets' key is not a mapping".to_string(),
                ));
            };

            for (name, ethernets_value) in ethernets {
                let Some(id) = name.as_str() else {
                    debug!(
                        "The 'ethernets' mapping contains a key that is not a string {:?}",
                        name
                    );
                    continue;
                };
                let Some(definition) = ethernets_value.as_mapping() else {
                    debug!("The value for the '{}' key is not a mapping", id);
                    continue;
                };

                let interface_name = self.resolve_interface(id, ethernets_value, config_map);
                let configuration = config_map
                    .entry(interface_name.clone())
                    .or_insert_with(|| NetworkConfiguration::new_with_name(&interface_name));
                let mut enabled = true;

                for (inner_name, inner_value) in definition {
                    let Some(inner_key) = inner_name.as_str() else {
                        debug!(
                            "The {} mapping contains a key that is not a string {:?}",
                            id, inner_name
                        );
                        continue;
                    };

                    if inner_key == "dhcp4" || inner_key == "dhcp6" {
                        if is_true(inner_value) {
                            configuration.address_mode = AddressMode::DHCP;
                        }
                    } else if inner_key == "activation-mode" {
                        enabled = inner_value.as_str() != Some("off");
                    } else if inner_key == "addresses" && inner_value.as_sequence().is_some() {
                        // The netplan file is authoritative for the configured
                        // static addresses. Clear any addresses already present
                        // on this configuration (e.g. seeded from a live system
                        // query in load_settings_from_system) so that we do not
                        // accumulate duplicate addresses across a
                        // load/modify/write cycle.
                        configuration.interface.addresses.clear();
                        for address in inner_value.as_sequence().unwrap() {
                            let Some(address_value) = address.as_str() else {
                                debug!("The {} mapping contains an 'addresses' key with a value that is not a string", id);
                                continue;
                            };
                            if let Ok(address) = InterfaceAddr::try_from(address_value) {
                                configuration.interface.addresses.push(address);
                            }
                        }
                        configuration.address_mode = AddressMode::Static;
                    } else if inner_key == "nameservers" && inner_value.as_mapping().is_some() {
                        if let Some(addresses) = inner_value
                            .as_mapping()
                            .unwrap()
                            .get("addresses")
                            .and_then(Value::as_sequence)
                        {
                            // As with the interface addresses above, the netplan file is
                            // authoritative for the configured nameservers. Clear any existing
                            // entries before populating so we do not duplicate them across a
                            // load/modify/write cycle.
                            configuration.interface.nameserver_addresses.clear();
                            for address in addresses {
                                if let Some(address_str) = address.as_str() {
                                    configuration
                                        .interface
                                        .nameserver_addresses
                                        .push(<IpAddr as IpAddrQuery>::from(address_str)?);
                                }
                            }
                        }
                    }
                }

                configuration.enabled = enabled;
            }
        }

        if let Some(wifis) = network.get(WIFIS) {
            let Some(wifis) = wifis.as_mapping() else {
                return Err(FoundationError::OperationFailed(
                    "The 'wifis' key is not a mapping".to_string(),
                ));
            };

            for (name, wifis_value) in wifis {
                let Some(id) = name.as_str() else {
                    debug!(
                        "The 'wifis' mapping contains a key that is not a string {:?}",
                        name
                    );
                    continue;
                };
                let Some(definition) = wifis_value.as_mapping() else {
                    debug!("The value for the '{}' key is not a mapping", id);
                    continue;
                };

                let interface_name = self.resolve_interface(id, wifis_value, config_map);
                let configuration = config_map
                    .entry(interface_name.clone())
                    .or_insert_with(|| NetworkConfiguration::new_with_name(&interface_name));
                let mut enabled = true;

                for (inner_name, inner_value) in definition {
                    let Some(inner_key) = inner_name.as_str() else {
                        debug!(
                            "The {} mapping contains a key that is not a string {:?}",
                            id, inner_name
                        );
                        continue;
                    };

                    if inner_key == "dhcp4" || inner_key == "dhcp6" {
                        if is_true(inner_value) {
                            configuration.address_mode = AddressMode::DHCP;
                        }
                    } else if inner_key == "activation-mode" {
                        enabled = inner_value.as_str() != Some("off");
                    } else if inner_key == "access-points" {
                        if let Some(access_points) = inner_value.as_mapping() {
                            let wireless_config = configuration
                                .wifi_configuration
                                .get_or_insert_with(WirelessConfiguration::default);
                            for (point_name, point_value) in access_points {
                                if let Some(point_str) = point_name.as_str() {
                                    wireless_config.ssid = point_str.to_string();
                                }
                                if let Some(password) = point_value
                                    .as_mapping()
                                    .and_then(|point| point.get("password"))
                                    .and_then(Value::as_str)
                                {
                                    wireless_config.password = Some(password.to_string());
                                }
                            }
                        }
                    }
                }

                configuration.enabled = enabled;
            }
        }

        Ok(())
    }

    /// Write a set of network configuration settings to the Netplan configuration file, as full
    /// definitions under the interface names. The function only writes the portions of the
    /// configuration that are handled by Netplan.
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
        let mut sections = Sections::default();
        let mut configs = configurations.values().collect::<Vec<_>>();
        configs.sort_by_key(|config| config.interface.name.clone());
        for config in configs {
            for (section, definition) in definitions(config) {
                sections.add(section, &config.interface.name, definition);
            }
        }
        write_document(&self.filename, sections.into_document(Some("networkd")))
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
    use tempfile::TempDir;

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

    // Regression test for the address-duplication bug. When the configuration map is seeded from a
    // live system query (as load_settings_from_system does) before the netplan file is loaded, the
    // interface already carries its current addresses. Loading the netplan file must not append the
    // file's static addresses on top of those, otherwise a load/modify/write cycle duplicates the
    // IP addresses in the netplan configuration file.
    #[test]
    fn test_load_does_not_duplicate_seeded_addresses() {
        let mut netplan_service =
            NetplanService::new(PathBuf::from("/tmp/wifi_netplan_dedup.yaml"));

        // Build and write a static Wi-Fi client configuration to disk.
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

        let result = netplan_service.write_configuration(&config_map);
        assert!(result.is_ok());

        // Seed a fresh map the way load_settings_from_system does: the live interface already has
        // its current address. Note the live address carries a broadcast value while the file-parsed
        // one does not, so the duplicate would not be caught by a simple equality dedup.
        let mut seeded_map: HashMap<String, NetworkConfiguration> = HashMap::new();
        let mut live_interface = NetworkInterface::new_with_name("wlan0");
        live_interface.addresses.push(InterfaceAddr::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 3)),
            Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 255))),
            Some(IpAddr::V4(Ipv4Addr::new(255, 255, 255, 0))),
        ));
        live_interface
            .nameserver_addresses
            .push(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)));
        seeded_map.insert(
            "wlan0".to_string(),
            NetworkConfiguration::new_with_interface(live_interface),
        );

        // Now load the netplan file on top of the seeded map.
        let result = netplan_service.load_configuration(&mut seeded_map);
        assert!(result.is_ok());

        let loaded = seeded_map.get("wlan0").unwrap();
        assert_eq!(
            loaded.interface.addresses.len(),
            1,
            "netplan load must not duplicate addresses already present from a live query"
        );
        assert_eq!(
            loaded.interface.nameserver_addresses.len(),
            1,
            "netplan load must not duplicate nameservers already present from a live query"
        );

        // A subsequent write must not duplicate the address in the file either. Read it back into a
        // fresh map and confirm there is still exactly one address.
        let result = netplan_service.write_configuration(&seeded_map);
        assert!(result.is_ok());

        let mut reread_map: HashMap<String, NetworkConfiguration> = HashMap::new();
        let result = netplan_service.load_configuration(&mut reread_map);
        assert!(result.is_ok());
        assert_eq!(
            reread_map.get("wlan0").unwrap().interface.addresses.len(),
            1
        );

        netplan_service.remove_config_file().unwrap();
    }

    // An access-point wifi configuration is rendered into the `ethernets` section (netplan manages
    // the underlying link; hostapd drives the AP itself). This exercises the AccessPoint branch of
    // should_use_config_for_ethernets. Reloading reads the entry back as a plain ethernet, so the
    // wifi-specific fields are intentionally not preserved.
    #[test]
    fn test_access_point_written_as_ethernet() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("ap_netplan.yaml");

        let mut config_map: HashMap<String, NetworkConfiguration> = HashMap::new();
        let interface = NetworkInterface::new_with_name("wlan0");
        let mut wifi_config = WirelessConfiguration::default();
        wifi_config.mode = WirelessMode::AccessPoint;
        wifi_config.ssid = "MyAccessPoint".to_string();
        let config =
            NetworkConfiguration::new(AddressMode::DHCP, interface, true, Some(wifi_config), None);
        config_map.insert("wlan0".to_string(), config);

        let mut netplan_service = NetplanService::new(path);
        netplan_service.write_configuration(&config_map).unwrap();

        let mut read_config_map: HashMap<String, NetworkConfiguration> = HashMap::new();
        netplan_service
            .load_configuration(&mut read_config_map)
            .unwrap();

        let loaded = read_config_map
            .get("wlan0")
            .expect("wlan0 present in ethernets");
        assert!(loaded.enabled);
        assert_eq!(loaded.address_mode, AddressMode::DHCP);
        // The wifi configuration is not round-tripped through the ethernets section.
        assert!(loaded.wifi_configuration.is_none());
    }

    #[test]
    fn test_write_configuration_overwrites_existing_read_only_file() {
        // Netplan files belong to root and are often read-only (0o400). The writer replaces the
        // file instead of writing into it, so a read-only file does not stop it.
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("overwrite_netplan.yaml");
        std::fs::write(&path, "network: {version: 2}\n").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o400)).unwrap();

        let mut config_map: HashMap<String, NetworkConfiguration> = HashMap::new();
        let interface = NetworkInterface::new_with_name("eth0");
        let config = NetworkConfiguration::new(AddressMode::DHCP, interface, true, None, None);
        config_map.insert("eth0".to_string(), config);

        let netplan_service = NetplanService::new(path.clone());
        netplan_service.write_configuration(&config_map).unwrap();
        netplan_service.write_configuration(&config_map).unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[test]
    fn test_load_configuration_missing_file_errors() {
        let dir = TempDir::new().unwrap();
        let mut netplan_service = NetplanService::new(dir.path().join("missing_netplan.yaml"));
        let mut config_map: HashMap<String, NetworkConfiguration> = HashMap::new();
        let result = netplan_service.load_configuration(&mut config_map);
        assert!(matches!(result, Err(FoundationError::IO(_))));
    }

    #[test]
    fn test_load_configuration_malformed_yaml_errors() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("bad_netplan.yaml");
        std::fs::write(&path, "network: [this: is, not: valid: yaml\n").unwrap();

        let mut netplan_service = NetplanService::new(path);
        let mut config_map: HashMap<String, NetworkConfiguration> = HashMap::new();
        let result = netplan_service.load_configuration(&mut config_map);
        assert!(matches!(result, Err(FoundationError::SerdeYamlError(_))));
    }

    #[test]
    fn test_get_configuration_file() {
        let path = PathBuf::from("/tmp/some_netplan_path.yaml");
        let netplan_service = NetplanService::new(path.clone());
        assert_eq!(netplan_service.get_configuration_file(), path);
    }

    /// A live interface, as load_settings_from_system finds it on the machine.
    fn live(name: &str, mac: &str) -> NetworkConfiguration {
        NetworkConfiguration::new_with_interface(NetworkInterface::new(
            name,
            vec![],
            Some(mac.to_string()),
            2,
            vec![],
            vec![],
            true,
        ))
    }

    fn static_address(text: &str) -> InterfaceAddr {
        InterfaceAddr::try_from(text).unwrap()
    }

    /// A netplan file like the one that sets the test VM's Ethernet address.
    const MATCHED_ETHERNET: &str = "network:
  version: 2
  ethernets:
    ethernet:
      match:
        name: \"en*\"
      addresses: [10.0.2.15/24]
      routes:
        - to: default
          via: 10.0.2.2
      nameservers:
        addresses: [10.0.2.3]
";

    /// Load `text` as a netplan file on top of a machine with the interfaces in `machine`.
    fn load_text(
        dir: &TempDir,
        text: &str,
        machine: Vec<NetworkConfiguration>,
    ) -> (NetplanService, HashMap<String, NetworkConfiguration>) {
        let path = dir.path().join("01-base.yaml");
        std::fs::write(&path, text).unwrap();
        let mut config_map = machine
            .into_iter()
            .map(|config| (config.get_name(), config))
            .collect::<HashMap<_, _>>();
        let mut service = NetplanService::new(path);
        service.load_configuration(&mut config_map).unwrap();
        (service, config_map)
    }

    fn read_yaml(path: &std::path::Path) -> Value {
        serde_yaml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    #[test]
    fn test_glob_matches() {
        assert!(glob_matches("en*", "enp0s2"));
        assert!(glob_matches("en*", "en"));
        assert!(glob_matches("eth?", "eth0"));
        assert!(glob_matches("eth0", "eth0"));
        assert!(!glob_matches("en*", "wlan0"));
        assert!(!glob_matches("eth?", "eth10"));
        assert!(glob_matches("*0", "wlan0"));
    }

    #[test]
    fn test_load_applies_a_match_rule_to_the_matched_interface() {
        let dir = TempDir::new().unwrap();
        let (service, config_map) = load_text(
            &dir,
            MATCHED_ETHERNET,
            vec![
                live("enp0s2", "52:54:00:12:34:56"),
                live("wlan0", "02:00:00:00:00:00"),
            ],
        );

        assert!(!config_map.contains_key("ethernet"));
        let ethernet = &config_map["enp0s2"];
        assert!(ethernet.enabled);
        assert_eq!(ethernet.address_mode, AddressMode::Static);
        assert_eq!(
            ethernet.interface.addresses,
            vec![static_address("10.0.2.15/24")]
        );
        assert_eq!(service.interface_ids()["enp0s2"], "ethernet");
    }

    #[test]
    fn test_load_applies_a_mac_address_rule() {
        let dir = TempDir::new().unwrap();
        let (service, config_map) = load_text(
            &dir,
            "network:
  version: 2
  ethernets:
    uplink:
      match:
        macaddress: 52:54:00:AA:BB:CC
      dhcp4: true
",
            vec![
                live("enp0s2", "52:54:00:12:34:56"),
                live("enp0s3", "52:54:00:aa:bb:cc"),
            ],
        );

        assert_eq!(service.interface_ids()["enp0s3"], "uplink");
        assert!(!service.interface_ids().contains_key("enp0s2"));
        assert_eq!(config_map["enp0s3"].address_mode, AddressMode::DHCP);
    }

    #[test]
    fn test_load_reads_dhcp4_and_activation_mode() {
        let dir = TempDir::new().unwrap();
        let mut machine = live("eth0", "52:54:00:12:34:56");
        machine.address_mode = AddressMode::Static;
        let (_, config_map) = load_text(
            &dir,
            "network:
  version: 2
  ethernets:
    eth0:
      dhcp4: true
      activation-mode: \"off\"
",
            vec![machine],
        );

        assert_eq!(config_map["eth0"].address_mode, AddressMode::DHCP);
        assert!(!config_map["eth0"].enabled);
    }

    #[test]
    fn test_write_changes_without_changes_removes_the_file() {
        let dir = TempDir::new().unwrap();
        let (mut service, config_map) = load_text(
            &dir,
            MATCHED_ETHERNET,
            vec![live("enp0s2", "52:54:00:12:34:56")],
        );
        let path = dir.path().join("99-changes.yaml");
        std::fs::write(&path, "network: {version: 2}\n").unwrap();
        let ids = service.interface_ids().clone();
        service = NetplanService::new(path.clone());
        service.set_interface_ids(ids);

        service
            .write_changes(&config_map, &config_map.clone())
            .unwrap();
        assert!(!path.exists());
        // Without a file to remove, the write still succeeds.
        service
            .write_changes(&config_map, &config_map.clone())
            .unwrap();
    }

    /// The base configurations from MATCHED_ETHERNET, and a service that writes changes to
    /// 99-changes.yaml with the base file's ids.
    fn matched_base(dir: &TempDir) -> (NetplanService, HashMap<String, NetworkConfiguration>) {
        let (base_service, base) = load_text(
            dir,
            MATCHED_ETHERNET,
            vec![live("enp0s2", "52:54:00:12:34:56")],
        );
        let mut service = NetplanService::new(dir.path().join("99-changes.yaml"));
        service.set_interface_ids(base_service.interface_ids().clone());
        (service, base)
    }

    #[test]
    fn test_write_changes_turns_off_a_disabled_interface_under_its_id() {
        let dir = TempDir::new().unwrap();
        let (service, base) = matched_base(&dir);
        let mut current = base.clone();
        current.get_mut("enp0s2").unwrap().enabled = false;

        service.write_changes(&current, &base).unwrap();
        let path = dir.path().join("99-changes.yaml");
        let text = std::fs::read_to_string(&path).unwrap();
        let ethernets = &read_yaml(&path)["network"]["ethernets"];
        assert_eq!(ethernets.as_mapping().unwrap().len(), 1, "{}", text);
        assert_eq!(ethernets["ethernet"]["activation-mode"], Value::from("off"));
        assert!(read_yaml(&path)["network"].get("renderer").is_none());

        // Loading the base file and then the change gives the disabled interface back.
        let mut reloaded =
            HashMap::from([("enp0s2".to_string(), live("enp0s2", "52:54:00:12:34:56"))]);
        let mut base_service = NetplanService::new(dir.path().join("01-base.yaml"));
        base_service.load_configuration(&mut reloaded).unwrap();
        let mut change_service = NetplanService::new(path);
        change_service.set_interface_ids(base_service.interface_ids().clone());
        change_service.load_configuration(&mut reloaded).unwrap();
        assert!(!reloaded["enp0s2"].enabled);
        assert!(!reloaded.contains_key("ethernet"));
    }

    #[test]
    fn test_write_changes_writes_a_changed_interface_under_its_id() {
        let dir = TempDir::new().unwrap();
        let (service, base) = matched_base(&dir);
        let mut current = base.clone();
        current.get_mut("enp0s2").unwrap().address_mode = AddressMode::DHCP;

        service.write_changes(&current, &base).unwrap();
        let document = read_yaml(&dir.path().join("99-changes.yaml"));
        let ethernet = &document["network"]["ethernets"]["ethernet"];
        assert_eq!(ethernet["dhcp4"], Value::from(true));
        assert!(ethernet.get("match").is_none());
        assert!(document["network"]["ethernets"].get("enp0s2").is_none());
    }

    #[test]
    fn test_write_changes_turns_a_disabled_interface_back_on() {
        // After a change turns the interface off, enabling it again writes a definition
        // without activation-mode, so the interface comes back.
        let dir = TempDir::new().unwrap();
        let (service, base) = matched_base(&dir);
        let mut current = base.clone();
        current.get_mut("enp0s2").unwrap().enabled = false;
        service.write_changes(&current, &base).unwrap();

        let path = dir.path().join("99-changes.yaml");
        let mut change_service = NetplanService::new(path.clone());
        change_service.set_interface_ids(service.interface_ids().clone());
        let mut reloaded = base.clone();
        change_service.load_configuration(&mut reloaded).unwrap();
        assert!(!reloaded["enp0s2"].enabled);

        let ethernet = reloaded.get_mut("enp0s2").unwrap();
        ethernet.enabled = true;
        ethernet.address_mode = AddressMode::DHCP;
        service.write_changes(&reloaded, &base).unwrap();
        let ethernet = read_yaml(&path)["network"]["ethernets"]["ethernet"].clone();
        assert!(ethernet.get("activation-mode").is_none());
        assert_eq!(ethernet["dhcp4"], Value::from(true));
    }

    #[test]
    fn test_write_changes_writes_a_new_interface_under_its_name() {
        let dir = TempDir::new().unwrap();
        let (service, base) = matched_base(&dir);
        let mut current = base.clone();
        let mut added = live("enp0s3", "52:54:00:aa:bb:cc");
        added
            .interface
            .addresses
            .push(static_address("192.168.5.2/24"));
        current.insert("enp0s3".to_string(), added);

        service.write_changes(&current, &base).unwrap();
        let ethernets =
            read_yaml(&dir.path().join("99-changes.yaml"))["network"]["ethernets"].clone();
        assert_eq!(ethernets.as_mapping().unwrap().len(), 1);
        assert_eq!(ethernets["enp0s3"]["dhcp4"], Value::from(true));
    }

    #[test]
    fn test_write_changes_leaves_out_unchanged_live_interfaces() {
        // A live interface with an address and no netplan definition, such as a container
        // bridge, gets no definition.
        let dir = TempDir::new().unwrap();
        let mut bridge = live("docker0", "02:42:00:00:00:01");
        bridge
            .interface
            .addresses
            .push(static_address("172.17.0.1/16"));
        let (base_service, base) = load_text(
            &dir,
            MATCHED_ETHERNET,
            vec![live("enp0s2", "52:54:00:12:34:56"), bridge],
        );
        let path = dir.path().join("99-changes.yaml");
        let mut service = NetplanService::new(path.clone());
        service.set_interface_ids(base_service.interface_ids().clone());

        service.write_changes(&base, &base.clone()).unwrap();
        assert!(!path.exists());
    }

    #[test]
    fn test_wifi_client_definition_uses_dhcp4() {
        let mut wifi_config = WirelessConfiguration::default();
        wifi_config.ssid = "Library".to_string();
        wifi_config.mode = WirelessMode::Client;
        let config = NetworkConfiguration::new(
            AddressMode::DHCP,
            NetworkInterface::new_with_name("wlan1"),
            true,
            Some(wifi_config),
            None,
        );
        let definitions = definitions(&config);
        assert_eq!(definitions.len(), 1);
        assert_eq!(definitions[0].0, WIFIS);
        assert_eq!(definitions[0].1.get("dhcp4"), Some(&Value::from(true)));
        assert!(definitions[0].1.get("dhcp").is_none());
    }
}
