//! The `networkinterfacequery` module provides a trait for querying network interfaces from the
//! `network_interface` crate.  The `network_interface` crate provides functionality for inspecting
//! network interfaces on a system. This module leverages that functionality and provides the
//! `NetworkInterfaceQuery` trait for querying data from a `network_interface::NetworkInterface`
//! object.

use crate::network::wireless::is_wireless_interface;
use crate::network::ipaddrquery::IpAddrQuery;

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
