//! The `hostapdservice` module contains code that interacts with the HostAPD service on a Linux
//! machine.

use crate::error::FoundationError;
use crate::keyvalueconfigfile::KeyValueConfigFile;
use crate::network::networkconfiguration::NetworkConfiguration;
use crate::network::networkservice::NetworkService;
use crate::network::wireless::configuration::{
    WirelessConfiguration, WirelessMode, WirelessStandard,
};
use crate::systemctlservice::SystemCTLService;
use std::collections::HashMap;
use std::path::PathBuf;

/// The `HostAPDService` object is used to start, stop, and restart the HostAPD service on a Linux
/// machine.
pub struct HostAPDService {
    /// The path to the configuration file.
    filename: PathBuf,

    /// The `SystemCTLService` object used to start, stop, and restart the HostAPD service.
    service: SystemCTLService,
}

impl HostAPDService {
    /// Create a new `HostAPDService` object.
    ///
    /// # Arguments
    ///
    /// * `filename` - The path to the configuration file.
    pub fn new(filename: PathBuf) -> HostAPDService {
        HostAPDService {
            filename,
            service: SystemCTLService::new("hostapd".to_string()),
        }
    }
}

impl NetworkService for HostAPDService {
    fn load_configuration(
        &mut self,
        config_map: &mut HashMap<String, NetworkConfiguration>,
    ) -> Result<(), FoundationError> {
        let key_value_config = KeyValueConfigFile::new(self.filename.clone());

        if !key_value_config.file_exists() {
            return Err(FoundationError::OperationFailed(format!(
                "Configuration file does not exist: {}",
                self.filename.to_string_lossy()
            )));
        }

        let configuration = key_value_config.load_configuration()?;

        if let Some(interface_name) = configuration.get("interface") {
            let config = if let Some(config) = config_map.get_mut(interface_name) {
                config
            } else {
                let config = NetworkConfiguration::new_with_name(interface_name);
                config_map.insert(interface_name.to_string(), config);
                config_map.get_mut(interface_name).unwrap()
            };

            let mut wifi_config = WirelessConfiguration::default();
            wifi_config.mode = WirelessMode::AccessPoint;

            if let Some(ssid_str) = configuration.get("ssid") {
                wifi_config.ssid = ssid_str.to_string();
            }

            if let Some(hw_mode_str) = configuration.get("hw_mode") {
                match hw_mode_str.as_str() {
                    "a" => wifi_config.standard = WirelessStandard::A,
                    "b" => wifi_config.standard = WirelessStandard::B,
                    "g" => wifi_config.standard = WirelessStandard::G,
                    "n" => wifi_config.standard = WirelessStandard::N,
                    _ => {}
                }
            }

            if let Some(channel_str) = configuration.get("channel") {
                wifi_config.channel = channel_str.parse()?;
            }

            if let Some(password_str) = configuration.get("wpa_passphrase") {
                wifi_config.password = Some(password_str.to_string());
            }

            if let Some(wpa_mode_str) = configuration.get("wpa") {
                wifi_config.wpa_mode = wpa_mode_str.parse()?;
            }

            if let Some(ieee80211n_str) = configuration.get("ieee80211n") {
                wifi_config.ieee802111n = ieee80211n_str.parse()? == 1;
            }

            if let Some(wmm_enabled_str) = configuration.get("wmm_enabled") {
                wifi_config.wmm_enabled = wmm_enabled_str.parse()? == 1;
            }

            if let Some(wpa_key_mgmt_str) = configuration.get("wpa_key_mgmt") {
                wifi_config.wpa_key_mgmt = Some(wpa_key_mgmt_str.to_string());
            }

            if let Some(wpa_pairwise_str) = configuration.get("wpa_pairwise") {
                wifi_config.wpa_pairwise = Some(wpa_pairwise_str.to_string());
            }

            if let Some(rsn_pairwise_str) = configuration.get("rsn_pairwise") {
                wifi_config.rsn_pairwise = Some(rsn_pairwise_str.to_string());
            }

            config.wifi_configuration = Some(wifi_config);
        }
        Ok(())
    }

    fn write_configuration(
        &self,
        configurations: &HashMap<String, NetworkConfiguration>,
    ) -> Result<(), FoundationError> {
        for (name, configuration) in configurations {
            if !configuration.enabled {
                continue;
            }

            if let Some(wifi_config) = &configuration.wifi_configuration {
                if wifi_config.mode == WirelessMode::Client {
                    continue;
                }

                let mut value_map: HashMap<String, String> = HashMap::new();

                value_map.insert("interface".to_string(), name.clone());
                value_map.insert("driver".to_string(), "nl80211".to_string());
                value_map.insert("ssid".to_string(), wifi_config.ssid.clone());

                let hw_mode = "hw_mode".to_string();
                match wifi_config.standard {
                    WirelessStandard::A => value_map.insert(hw_mode, "a".to_string()),
                    WirelessStandard::B => value_map.insert(hw_mode, "b".to_string()),
                    WirelessStandard::G => value_map.insert(hw_mode, "g".to_string()),
                    WirelessStandard::N => value_map.insert(hw_mode, "n".to_string()),
                };

                value_map.insert("channel".to_string(), wifi_config.channel.to_string());
                value_map.insert("macaddr_acl".to_string(), "0".to_string());
                value_map.insert("auth_algs".to_string(), "1".to_string());
                value_map.insert("ignore_broadcast_ssid".to_string(), "0".to_string());
                value_map.insert("wpa".to_string(), wifi_config.wpa_mode.to_string());
                if wifi_config.ieee802111n {
                    value_map.insert("ieee80211n".to_string(), "1".to_string());
                }
                if wifi_config.wmm_enabled {
                    value_map.insert("wmm_enabled".to_string(), "1".to_string());
                }
                if let Some(password_str) = &wifi_config.password {
                    value_map.insert("wpa_passphrase".to_string(), password_str.clone());
                }

                if let Some(wpa_key_management_str) = &wifi_config.wpa_key_mgmt {
                    value_map.insert("wpa_key_mgmt".to_string(), wpa_key_management_str.clone());
                } else {
                    value_map.insert("wpa_key_mgmt".to_string(), "WPA-PSK".to_string());
                }

                if let Some(wpa_pairwise_str) = &wifi_config.wpa_pairwise {
                    value_map.insert("wpa_pairwise".to_string(), wpa_pairwise_str.clone());
                } else {
                    value_map.insert("wpa_pairwise".to_string(), "TKIP".to_string());
                }

                if let Some(rsn_pairwise_str) = &wifi_config.rsn_pairwise {
                    value_map.insert("rsn_pairwise".to_string(), rsn_pairwise_str.clone());
                } else {
                    value_map.insert("rsn_pairwise".to_string(), "CCMP".to_string());
                }

                let key_value_config = KeyValueConfigFile::new(self.filename.clone());
                key_value_config.save_configuration(&value_map)?;
            }
        }
        Ok(())
    }

    fn get_configuration_file(&self) -> PathBuf {
        self.filename.clone()
    }

    fn start(&self) -> Result<(), FoundationError> {
        self.service.start()
    }

    fn stop(&self) -> Result<(), FoundationError> {
        self.service.stop()
    }

    fn restart(&self) -> Result<(), FoundationError> {
        self.service.restart()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::networkconfiguration::AddressMode;
    use crate::network::networkinterface::NetworkInterface;

    // Note that this service can lose configuration fidelity in the sense that the hostapd configuration
    // file does not contain all settings supported by this library's notion of a network configuration.
    // When testing, be sure to understand what the service supports so that you only add enough to
    // configuration to test the service's ability to read and write the configuration file.  If you
    // add more, then the configurations will not match after you write the config, read it back and
    // then compare it to the read results (because the read config will contain less information).

    #[test]
    fn test_hostapd_service() {
        let interface = NetworkInterface::new_with_name("wlan0");
        let mut wifi_config = WirelessConfiguration::default();
        wifi_config.mode = WirelessMode::AccessPoint;
        wifi_config.ssid = "HoneyBadgerHut".to_string();
        wifi_config.password = Some("NUTHUT".to_string());
        wifi_config.standard = WirelessStandard::N;
        wifi_config.channel = 5;
        wifi_config.wpa_mode = 8;
        wifi_config.wpa_key_mgmt = Some("WPA-PSK".to_string());
        wifi_config.wpa_pairwise = Some("BUBBA".to_string());
        wifi_config.rsn_pairwise = Some("FLUBBA".to_string());
        let config =
            NetworkConfiguration::new(AddressMode::DHCP, interface, true, Some(wifi_config), None);
        let mut config_map: HashMap<String, NetworkConfiguration> = HashMap::new();
        config_map.insert("wlan0".to_string(), config);

        let mut hostapd_service = HostAPDService::new(PathBuf::from("/tmp/hostapd.conf"));
        let result = hostapd_service.write_configuration(&config_map);
        assert!(result.is_ok());

        // We create a second configuration map with an empty wlan0 configuration.  This makes
        // sure all the settings in the configuration match the first configuration default values
        // that hostapd does not use or change. This setup allows us to compare the configurations
        // after loading them from the file.
        let mut other_config_map: HashMap<String, NetworkConfiguration> = HashMap::new();
        let other_interface = NetworkInterface::new_with_name("wlan0");
        let other_config =
            NetworkConfiguration::new(AddressMode::DHCP, other_interface, true, None, None);
        other_config_map.insert("wlan0".to_string(), other_config);
        let result = hostapd_service.load_configuration(&mut other_config_map);
        assert!(result.is_ok());
    }
}
