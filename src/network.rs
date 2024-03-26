pub use dhcprange::DHCPRange as DHCPRange;
pub use interfaceaddr::InterfaceAddr as InterfaceAddr;
pub use ipaddrquery::IpAddrQuery as IPAddrQuery;
pub use networkconfiguration::NetworkConfiguration as NetworkConfiguration;
pub use networkconfiguration::AddressMode as AddressMode;
pub use networkinterface::NetworkInterface as NetworkInterface;
pub use networkinterfaces::NetworkInterfaces as NetworkInterfaces;
pub use networkmanager::NetworkManager as NetworkManager;
pub use networkservice::NetworkService as NetworkService;
pub use wireless::configuration::WirelessConfiguration as WirelessConfiguration;
pub use wireless::configuration::WirelessStandard as WirelessStandard;
pub use wireless::configuration::WirelessMode as WirelessMode;

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
