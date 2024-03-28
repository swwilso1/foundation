//! The `dnsmasqservice` module contains code that interacts with the DNSMasq service on a Linux
//! machine.

use crate::error::FoundationError;
use crate::keyvalueconfigfile::KeyValueConfigFile;
use crate::network::dhcprange::DHCPRange;
use crate::network::networkconfiguration::NetworkConfiguration;
use crate::network::networkservice::NetworkService;
use crate::systemctlservice::SystemCTLService;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct DNSMasqService {
    filename: PathBuf,
    service: SystemCTLService,
}

impl DNSMasqService {
    pub fn new(filename: PathBuf) -> DNSMasqService {
        DNSMasqService {
            filename,
            service: SystemCTLService::new("dnsmasq".to_string()),
        }
    }
}

impl NetworkService for DNSMasqService {
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

            if let Some(dhcp_range) = configuration.get("dhcp-range") {
                if let Ok(drange) = DHCPRange::try_from(dhcp_range.as_str()) {
                    config.dhcp_range = Some(drange);
                }
            }
        }

        Ok(())
    }

    fn write_configuration(
        &self,
        configurations: &HashMap<String, NetworkConfiguration>,
    ) -> Result<(), FoundationError> {
        for (name, config) in configurations {
            if config.enabled && config.wifi_configuration.is_some() && config.dhcp_range.is_some()
            {
                let key_value_config = KeyValueConfigFile::new(self.filename.clone());
                let mut config_map: HashMap<String, String> = HashMap::new();
                config_map.insert("interface".to_string(), name.clone());
                if let Some(dhcp_range) = &config.dhcp_range {
                    config_map.insert(
                        "dhcp-range".to_string(),
                        format!("{},{},12h", dhcp_range.start, dhcp_range.end),
                    );
                }
                config_map.insert("port".to_string(), "0".to_string());
                config_map.insert("bogus-priv".to_string(), String::new());
                config_map.insert("dnssec".to_string(), String::new());

                key_value_config.save_configuration(&config_map)?;
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
    use crate::network::wireless::configuration::WirelessConfiguration;

    // Note that this service can lose configuration fidelity in the sense that the dnsmasq configuration
    // file does not contain all settings supported by this library's notion of a network configuration.
    // When testing, be sure to understand what the service supports so that you only add enough to
    // configuration to test the service's ability to read and write the configuration file.  If you
    // add more, then the configurations will not match after you write the config, read it back and
    // then compare it to the read results (because the read config will contain less information).

    #[test]
    fn test_dnsmasq_service() {
        let interface = NetworkInterface::new_with_name("eth0");
        let wifi_config = WirelessConfiguration::default();
        let config = NetworkConfiguration::new(
            AddressMode::DHCP,
            interface,
            true,
            Some(wifi_config),
            Some(DHCPRange::new(
                "192.168.1.10".parse().unwrap(),
                "192.168.1.20".parse().unwrap(),
            )),
        );
        let mut config_map: HashMap<String, NetworkConfiguration> = HashMap::new();
        config_map.insert("eth0".to_string(), config);

        let mut dnsmasq_service = DNSMasqService::new(PathBuf::from("/tmp/dnsmasq.conf"));
        let result = dnsmasq_service.write_configuration(&config_map);
        assert!(result.is_ok());

        let mut other_config_map: HashMap<String, NetworkConfiguration> = HashMap::new();
        let result = dnsmasq_service.load_configuration(&mut other_config_map);
        assert!(result.is_ok());

        // Because of default values the two configs will not be exactly identical here, but they
        // *should* have the same dhcp-range.
        assert_eq!(
            config_map.get("eth0").unwrap().dhcp_range.as_ref().unwrap(),
            other_config_map
                .get("eth0")
                .unwrap()
                .dhcp_range
                .as_ref()
                .unwrap()
        );

        dnsmasq_service.remove_config_file().unwrap();
    }
}
