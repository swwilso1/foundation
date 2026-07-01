//! The `networkinterfacequery` module provides a trait for querying network interfaces from the
//! `network_interface` crate.  The `network_interface` crate provides functionality for inspecting
//! network interfaces on a system. This module leverages that functionality and provides the
//! `NetworkInterfaceQuery` trait for querying data from a `network_interface::NetworkInterface`
//! object.

use crate::network::ipaddrquery::IpAddrQuery;
use crate::network::wireless::is_wireless_interface;

use network_interface::{Addr, NetworkInterface};

pub trait NetworkInterfaceQuery {
    fn get_global_address(&self) -> Option<Addr>;
    fn get_global_ipv4_address(&self) -> Option<Addr>;
    fn get_global_ipv6_address(&self) -> Option<Addr>;
    fn get_ipv4_addresses(&self) -> Vec<Addr>;
    fn get_ipv6_addresses(&self) -> Vec<Addr>;
    fn has_global_address(&self) -> bool;
    fn has_global_ipv4_address(&self) -> bool;
    fn has_global_ipv6_address(&self) -> bool;
    fn has_ipv4_address(&self) -> bool;
    fn has_ipv6_address(&self) -> bool;
    fn is_loopback_interface(&self) -> bool;
    fn is_wireless_interface(&self) -> impl std::future::Future<Output = bool> + Send;
}

impl NetworkInterfaceQuery for NetworkInterface {
    fn get_global_address(&self) -> Option<Addr> {
        for addr in &self.addr {
            match addr {
                Addr::V4(v4addr) => {
                    if v4addr.ip.is_global_address() {
                        return Some(Addr::V4(*v4addr));
                    }
                }
                Addr::V6(v6addr) => {
                    if v6addr.ip.is_global_address() {
                        return Some(Addr::V6(*v6addr));
                    }
                }
            }
        }
        None
    }

    fn get_global_ipv4_address(&self) -> Option<Addr> {
        for addr in &self.addr {
            match addr {
                Addr::V4(v4addr) => {
                    if v4addr.ip.is_global_address() {
                        return Some(Addr::V4(*v4addr));
                    }
                }
                Addr::V6(_) => {}
            }
        }
        None
    }

    fn get_global_ipv6_address(&self) -> Option<Addr> {
        for addr in &self.addr {
            match addr {
                Addr::V4(_) => {}
                Addr::V6(v6addr) => {
                    if v6addr.ip.is_global_address() {
                        return Some(Addr::V6(*v6addr));
                    }
                }
            }
        }
        None
    }

    fn get_ipv4_addresses(&self) -> Vec<Addr> {
        let mut v4addrs = Vec::new();
        for addr in &self.addr {
            match addr {
                Addr::V4(v4addr) => {
                    v4addrs.push(Addr::V4(*v4addr));
                }
                Addr::V6(_) => {}
            }
        }
        v4addrs
    }

    fn get_ipv6_addresses(&self) -> Vec<Addr> {
        let mut v6addrs = Vec::new();
        for addr in &self.addr {
            match addr {
                Addr::V4(_) => {}
                Addr::V6(v6addr) => {
                    v6addrs.push(Addr::V6(*v6addr));
                }
            }
        }
        v6addrs
    }

    fn has_global_address(&self) -> bool {
        for addr in &self.addr {
            match addr {
                Addr::V4(v4addr) => {
                    if v4addr.ip.is_global_address() {
                        return true;
                    }
                }
                Addr::V6(v6addr) => {
                    if v6addr.ip.is_global_address() {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn has_global_ipv4_address(&self) -> bool {
        for addr in &self.addr {
            match addr {
                Addr::V4(v4addr) => {
                    if v4addr.ip.is_global_address() {
                        return true;
                    }
                }
                Addr::V6(_) => {}
            }
        }
        false
    }

    fn has_global_ipv6_address(&self) -> bool {
        for addr in &self.addr {
            match addr {
                Addr::V4(_) => {}
                Addr::V6(v6addr) => {
                    if v6addr.ip.is_global_address() {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn has_ipv4_address(&self) -> bool {
        for addr in &self.addr {
            match addr {
                Addr::V4(_) => {
                    return true;
                }
                Addr::V6(_) => {}
            }
        }
        false
    }

    fn has_ipv6_address(&self) -> bool {
        for addr in &self.addr {
            match addr {
                Addr::V4(_) => {}
                Addr::V6(_) => {
                    return true;
                }
            }
        }
        false
    }

    fn is_loopback_interface(&self) -> bool {
        for addr in &self.addr {
            match addr {
                Addr::V4(vf4addr) => {
                    if vf4addr.ip.is_loopback() {
                        return true;
                    }
                }
                Addr::V6(vf6addr) => {
                    if vf6addr.ip.is_loopback() {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn is_wireless_interface(&self) -> impl std::future::Future<Output = bool> + Send {
        is_wireless_interface(&self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use network_interface::{V4IfAddr, V6IfAddr};
    use std::net::{Ipv4Addr, Ipv6Addr};

    fn v4(addr: Addr) -> Ipv4Addr {
        match addr {
            Addr::V4(a) => a.ip,
            Addr::V6(_) => panic!("expected an IPv4 address"),
        }
    }

    fn v6(addr: Addr) -> Ipv6Addr {
        match addr {
            Addr::V6(a) => a.ip,
            Addr::V4(_) => panic!("expected an IPv6 address"),
        }
    }

    fn make_interface(addrs: Vec<Addr>) -> NetworkInterface {
        NetworkInterface {
            name: "test0".to_string(),
            addr: addrs,
            mac_addr: None,
            index: 0,
        }
    }

    fn v4_addr(ip: Ipv4Addr) -> Addr {
        Addr::V4(V4IfAddr {
            ip,
            broadcast: None,
            netmask: None,
        })
    }

    fn v6_addr(ip: Ipv6Addr) -> Addr {
        Addr::V6(V6IfAddr {
            ip,
            broadcast: None,
            netmask: None,
        })
    }

    #[test]
    fn test_get_global_address_prefers_first_global() {
        let interface = make_interface(vec![
            v4_addr(Ipv4Addr::new(192, 168, 1, 1)),
            v4_addr(Ipv4Addr::new(8, 8, 8, 8)),
        ]);
        assert_eq!(
            v4(interface.get_global_address().unwrap()),
            Ipv4Addr::new(8, 8, 8, 8)
        );
    }

    #[test]
    fn test_get_global_address_none_when_all_private() {
        let interface = make_interface(vec![v4_addr(Ipv4Addr::new(10, 0, 0, 1))]);
        assert!(interface.get_global_address().is_none());
    }

    #[test]
    fn test_get_global_ipv4_and_ipv6_address() {
        let interface = make_interface(vec![
            v4_addr(Ipv4Addr::new(10, 0, 0, 1)),
            v4_addr(Ipv4Addr::new(8, 8, 8, 8)),
            v6_addr(Ipv6Addr::new(0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8888)),
            v6_addr(Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 1)),
        ]);
        assert_eq!(
            v4(interface.get_global_ipv4_address().unwrap()),
            Ipv4Addr::new(8, 8, 8, 8)
        );
        assert_eq!(
            v6(interface.get_global_ipv6_address().unwrap()),
            Ipv6Addr::new(0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8888)
        );
    }

    #[test]
    fn test_get_ipv4_and_ipv6_addresses() {
        let interface = make_interface(vec![
            v4_addr(Ipv4Addr::new(10, 0, 0, 1)),
            v4_addr(Ipv4Addr::new(8, 8, 8, 8)),
            v6_addr(Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 1)),
        ]);
        assert_eq!(interface.get_ipv4_addresses().len(), 2);
        assert_eq!(interface.get_ipv6_addresses().len(), 1);
    }

    #[test]
    fn test_has_global_addresses() {
        let global = make_interface(vec![v4_addr(Ipv4Addr::new(8, 8, 8, 8))]);
        assert!(global.has_global_address());
        assert!(global.has_global_ipv4_address());
        assert!(!global.has_global_ipv6_address());

        let private = make_interface(vec![v4_addr(Ipv4Addr::new(192, 168, 0, 1))]);
        assert!(!private.has_global_address());
        assert!(!private.has_global_ipv4_address());

        let global_v6 = make_interface(vec![v6_addr(Ipv6Addr::new(
            0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8888,
        ))]);
        assert!(global_v6.has_global_address());
        assert!(global_v6.has_global_ipv6_address());
        assert!(!global_v6.has_global_ipv4_address());
    }

    #[test]
    fn test_has_ipv4_and_ipv6_address() {
        let interface = make_interface(vec![
            v4_addr(Ipv4Addr::new(10, 0, 0, 1)),
            v6_addr(Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 1)),
        ]);
        assert!(interface.has_ipv4_address());
        assert!(interface.has_ipv6_address());

        let v4_only = make_interface(vec![v4_addr(Ipv4Addr::new(10, 0, 0, 1))]);
        assert!(v4_only.has_ipv4_address());
        assert!(!v4_only.has_ipv6_address());

        let empty = make_interface(vec![]);
        assert!(!empty.has_ipv4_address());
        assert!(!empty.has_ipv6_address());
    }

    #[test]
    fn test_is_loopback_interface() {
        let loopback_v4 = make_interface(vec![v4_addr(Ipv4Addr::new(127, 0, 0, 1))]);
        assert!(loopback_v4.is_loopback_interface());

        let loopback_v6 = make_interface(vec![v6_addr(Ipv6Addr::LOCALHOST)]);
        assert!(loopback_v6.is_loopback_interface());

        let not_loopback = make_interface(vec![v4_addr(Ipv4Addr::new(8, 8, 8, 8))]);
        assert!(!not_loopback.is_loopback_interface());
    }
}
