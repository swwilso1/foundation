//! The `wireless_linux` module provides function implementations about WiFi features
//! specific to Linux.

use crate::error::FoundationError;
use futures::TryStreamExt;
use wl_nl80211::{new_connection, Nl80211Attr};

/// Check if the given interface is a wireless interface using the Netlink socket protocol.
///
/// # Arguments
///
/// * `name` - A string slice that holds the name of the interface.
///
/// # Returns
///
/// A Result containing a boolean value. If the interface is a wireless interface, the result
/// will be `Ok(true)`. If the interface is not a wireless interface, the result will be `Ok(false)`.
/// If an error occurs, the result will be `Err(FoundationError)`.
async fn is_wireless_interface_netlink(name: &str) -> Result<bool, FoundationError> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let mut interfaces = handle.interface().get(vec![]).execute().await;
    while let Ok(Some(interface)) = interfaces.try_next().await {
        if !interface
            .payload
            .attributes
            .iter()
            .any(|nla| matches!(nla, Nl80211Attr::IfName(n) if n == name))
        {
            continue;
        }

        return Ok(true);
    }
    Ok(false)
}

/// Check if the given interface is a wireless interface.
///
/// # Arguments
///
/// * `name` - A string slice that holds the name of the interface.
///
/// # Returns
///
/// A boolean value. If the interface is a wireless interface, the result will be `true`. If the
/// interface is not a wireless interface, the result will be `false`.
pub async fn is_wireless_interface(name: &str) -> bool {
    if let Ok(result) = is_wireless_interface_netlink(name).await {
        return result;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use network_interface::{NetworkInterface, NetworkInterfaceConfig};

    #[tokio::test]
    async fn test_is_wireless_interface() {
        let interfaces = NetworkInterface::show().unwrap();
        let mut found_wireless = false;
        for interface in interfaces {
            println!("{}", interface.name);
            if is_wireless_interface(&interface.name).await {
                found_wireless = true;
                break;
            }
        }
        assert!(found_wireless);

        let eth0_wireless = is_wireless_interface("eth0").await;
        assert!(!eth0_wireless);
    }
}
