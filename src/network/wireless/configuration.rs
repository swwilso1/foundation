//! The `configuration` module contains the `WirelessConfiguration` struct and its associated enums.

use crate::error::FoundationError;
use std::fmt::Display;
use std::str::FromStr;

/// The `WirelessStandard` enum represents the wireless standards used by a wireless network.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WirelessStandard {
    A,
    B,
    G,
    N,
}

/// The `WirelessMode` enum represents the wireless modes used by a wireless network.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WirelessMode {
    Client,
    AccessPoint,
}

/// The `WirelessConfiguration` struct represents the configuration of a wireless network.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WirelessConfiguration {
    /// The SSID of the wireless network.
    pub ssid: String,

    /// The wireless standard used by the wireless network.
    pub standard: WirelessStandard,

    /// The wireless mode used by the wireless network.
    pub mode: WirelessMode,

    /// The password of the wireless network.
    pub password: Option<String>,

    /// The channel of the wireless network.
    pub channel: u32,

    /// The WPA mode of the wireless network.
    pub wpa_mode: u32,

    /// The IEEE 802.11n setting of the wireless network.
    pub ieee80211n: bool,

    /// The WMM enabled setting of the wireless network.
    pub wmm_enabled: bool,

    /// The WPA key management setting of the wireless network.
    pub wpa_key_mgmt: Option<String>,

    /// The WPA pairwise setting of the wireless network.
    pub wpa_pairwise: Option<String>,

    /// The RSN pairwise setting of the wireless network.
    pub rsn_pairwise: Option<String>,
}

impl Default for WirelessConfiguration {
    /// Returns a new `WirelessConfiguration` instance with default values.
    ///
    /// # Returns
    ///
    /// A new `WirelessConfiguration` instance with default values. The default values are as follows:
    /// ssid - An empty string.
    /// standard - WirelessStandard::N.
    /// mode - WirelessMode::Client.
    /// password - None.
    /// channel - 1.
    /// wpa_mode - 1.
    /// wpa_key_mgmt - None.
    /// wpa_pairwise - None.
    /// rsn_pairwise - None.
    fn default() -> WirelessConfiguration {
        WirelessConfiguration {
            ssid: String::new(),
            standard: WirelessStandard::N,
            mode: WirelessMode::Client,
            password: None,
            channel: 1,
            wpa_mode: 1,
            ieee80211n: false,
            wmm_enabled: false,
            wpa_key_mgmt: None,
            wpa_pairwise: None,
            rsn_pairwise: None,
        }
    }
}

impl WirelessConfiguration {
    /// Returns a new `WirelessConfiguration` instance with the specified values.
    ///
    /// # Arguments
    ///
    /// * `ssid` - The SSID of the wireless network.
    /// * `standard` - The wireless standard used by the wireless network.
    /// * `mode` - The wireless mode used by the wireless network.
    /// * `password` - The password of the wireless network.
    /// * `channel` - The channel of the wireless network.
    /// * `wpa_mode` - The WPA mode of the wireless network.
    /// * `wpa_key_mgmt` - The WPA key management setting of the wireless network.
    /// * `wpa_pairwise` - The WPA pairwise setting of the wireless network.
    /// * `rsn_pairwise` - The RSN pairwise setting of the wireless network.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ssid: String,
        standard: WirelessStandard,
        mode: WirelessMode,
        password: Option<String>,
        channel: u32,
        wpa_mode: u32,
        ieee80211n: bool,
        wmm_enabled: bool,
        wpa_key_mgmt: Option<String>,
        wpa_pairwise: Option<String>,
        rsn_pairwise: Option<String>,
    ) -> Self {
        WirelessConfiguration {
            ssid,
            standard,
            mode,
            password,
            channel,
            wpa_mode,
            ieee80211n,
            wmm_enabled,
            wpa_key_mgmt,
            wpa_pairwise,
            rsn_pairwise,
        }
    }

    /// Clear the current settings from the configuration and restore to default values.
    pub fn clear(&mut self) {
        *self = WirelessConfiguration::default();
    }
}

impl Display for WirelessStandard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WirelessStandard::A => write!(f, "A"),
            WirelessStandard::B => write!(f, "B"),
            WirelessStandard::G => write!(f, "G"),
            WirelessStandard::N => write!(f, "N"),
        }
    }
}

impl FromStr for WirelessStandard {
    type Err = FoundationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "A" => Ok(WirelessStandard::A),
            "B" => Ok(WirelessStandard::B),
            "G" => Ok(WirelessStandard::G),
            "N" => Ok(WirelessStandard::N),
            _ => Err(FoundationError::UnknownWirelessStandard(s.to_string())),
        }
    }
}

impl Display for WirelessMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WirelessMode::Client => write!(f, "client"),
            WirelessMode::AccessPoint => write!(f, "access_point"),
        }
    }
}

impl FromStr for WirelessMode {
    type Err = FoundationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "client" => Ok(WirelessMode::Client),
            "access_point" => Ok(WirelessMode::AccessPoint),
            _ => Err(FoundationError::UnknownWirelessMode(s.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let config = WirelessConfiguration::default();
        assert_eq!(config.ssid, "");
        assert_eq!(config.standard, WirelessStandard::N);
        assert_eq!(config.mode, WirelessMode::Client);
        assert_eq!(config.password, None);
        assert_eq!(config.channel, 1);
        assert_eq!(config.wpa_mode, 1);
        assert!(!config.ieee80211n);
        assert!(!config.wmm_enabled);
        assert_eq!(config.wpa_key_mgmt, None);
        assert_eq!(config.wpa_pairwise, None);
        assert_eq!(config.rsn_pairwise, None);
    }

    #[test]
    fn test_new() {
        let config = WirelessConfiguration::new(
            "my-network".to_string(),
            WirelessStandard::G,
            WirelessMode::AccessPoint,
            Some("secret".to_string()),
            6,
            2,
            true,
            true,
            Some("WPA-PSK".to_string()),
            Some("TKIP".to_string()),
            Some("CCMP".to_string()),
        );
        assert_eq!(config.ssid, "my-network");
        assert_eq!(config.standard, WirelessStandard::G);
        assert_eq!(config.mode, WirelessMode::AccessPoint);
        assert_eq!(config.password, Some("secret".to_string()));
        assert_eq!(config.channel, 6);
        assert_eq!(config.wpa_mode, 2);
        assert!(config.ieee80211n);
        assert!(config.wmm_enabled);
        assert_eq!(config.wpa_key_mgmt, Some("WPA-PSK".to_string()));
        assert_eq!(config.wpa_pairwise, Some("TKIP".to_string()));
        assert_eq!(config.rsn_pairwise, Some("CCMP".to_string()));
    }

    #[test]
    fn test_clear() {
        let mut config = WirelessConfiguration::new(
            "my-network".to_string(),
            WirelessStandard::A,
            WirelessMode::AccessPoint,
            Some("secret".to_string()),
            11,
            2,
            true,
            true,
            Some("WPA-PSK".to_string()),
            Some("TKIP".to_string()),
            Some("CCMP".to_string()),
        );

        config.clear();

        assert_eq!(config, WirelessConfiguration::default());
    }

    #[test]
    fn test_wireless_standard_display() {
        assert_eq!(WirelessStandard::A.to_string(), "A");
        assert_eq!(WirelessStandard::B.to_string(), "B");
        assert_eq!(WirelessStandard::G.to_string(), "G");
        assert_eq!(WirelessStandard::N.to_string(), "N");
    }

    #[test]
    fn test_wireless_standard_from_str() {
        assert_eq!("A".parse::<WirelessStandard>().unwrap(), WirelessStandard::A);
        assert_eq!("B".parse::<WirelessStandard>().unwrap(), WirelessStandard::B);
        assert_eq!("G".parse::<WirelessStandard>().unwrap(), WirelessStandard::G);
        assert_eq!("N".parse::<WirelessStandard>().unwrap(), WirelessStandard::N);
    }

    #[test]
    fn test_wireless_standard_from_str_roundtrip() {
        for standard in [
            WirelessStandard::A,
            WirelessStandard::B,
            WirelessStandard::G,
            WirelessStandard::N,
        ] {
            assert_eq!(
                standard.to_string().parse::<WirelessStandard>().unwrap(),
                standard
            );
        }
    }

    #[test]
    fn test_wireless_standard_from_str_invalid() {
        let result = "AC".parse::<WirelessStandard>();
        assert!(matches!(
            result,
            Err(FoundationError::UnknownWirelessStandard(s)) if s == "AC"
        ));
    }

    #[test]
    fn test_wireless_mode_display() {
        assert_eq!(WirelessMode::Client.to_string(), "client");
        assert_eq!(WirelessMode::AccessPoint.to_string(), "access_point");
    }

    #[test]
    fn test_wireless_mode_from_str() {
        assert_eq!("client".parse::<WirelessMode>().unwrap(), WirelessMode::Client);
        assert_eq!(
            "access_point".parse::<WirelessMode>().unwrap(),
            WirelessMode::AccessPoint
        );
    }

    #[test]
    fn test_wireless_mode_from_str_roundtrip() {
        for mode in [WirelessMode::Client, WirelessMode::AccessPoint] {
            assert_eq!(mode.to_string().parse::<WirelessMode>().unwrap(), mode);
        }
    }

    #[test]
    fn test_wireless_mode_from_str_invalid() {
        let result = "bridge".parse::<WirelessMode>();
        assert!(matches!(
            result,
            Err(FoundationError::UnknownWirelessMode(s)) if s == "bridge"
        ));
    }
}
