pub use dhcprange::DHCPRange;
pub use interfaceaddr::InterfaceAddr;
pub use ipaddrquery::IpAddrQuery as IPAddrQuery;
pub use networkconfiguration::AddressMode;
pub use networkconfiguration::NetworkConfiguration;
pub use networkinterface::NetworkInterface;
pub use networkinterfaces::NetworkInterfaces;
pub use networkmanager::NetworkManager;
pub use networkservice::NetworkService;
pub use wireless::configuration::WirelessConfiguration;
pub use wireless::configuration::WirelessMode;
pub use wireless::configuration::WirelessStandard;

pub mod dhcprange;
pub mod interfaceaddr;
pub mod ipaddrquery;
mod netmask;
pub mod networkconfiguration;
pub mod networkinterface;
pub mod networkinterfacequery;
pub mod networkinterfaces;
pub mod networkmanager;
pub mod networkservice;
pub mod wireless;

cfg_if! {
    if #[cfg(target_os = "linux")] {
        mod dhcpcdservice;
        mod dnsmasqservice;
        mod hostapdservice;
        mod netplanservice;
    }
}
