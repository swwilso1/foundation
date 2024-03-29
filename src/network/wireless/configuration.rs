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

    /// The WPA key management setting of the wireless network.
    pub wpa_key_mgmt: Option<String>,

    /// The WPA pairwise setting of the wireless network.
    pub wpa_pairwise: Option<String>,

    /// The RSN pairwise setting of the wireless network.
    pub rsn_pairwise: Option<String>,
}

impl WirelessConfiguration {
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
    pub fn default() -> WirelessConfiguration {
        WirelessConfiguration {
            ssid: String::new(),
            standard: WirelessStandard::N,
            mode: WirelessMode::Client,
            password: None,
            channel: 1,
            wpa_mode: 1,
            wpa_key_mgmt: None,
            wpa_pairwise: None,
            rsn_pairwise: None,
        }
    }

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
    pub fn new(
        ssid: String,
        standard: WirelessStandard,
        mode: WirelessMode,
        password: Option<String>,
        channel: u32,
        wpa_mode: u32,
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
