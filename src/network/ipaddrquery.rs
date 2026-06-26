//! The `ipaddrquery` module provides the `IpAddrQuery` trait that adds functionality to `IpAddr`,
//! `Ipv4Addr`, and `Ipv6Addr` from the `std::net` module.

use crate::error::FoundationError;
use crate::network::netmask::bits_in_mask;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

// A trait designed to add functionality to IpAddr, Ipv4Addr, and Ipv6Addr from the std::net module.
pub trait IpAddrQuery {
    /// The integer type capable of holding every value of the IP address.
    type Integer;

    /// Check if the IP address is a global address.
    ///
    /// # Returns
    ///
    /// `true` if the IP address is a global address, `false` otherwise.
    fn is_global_address(&self) -> bool;

    /// Create an IP address from an integer.
    ///
    /// # Arguments
    ///
    /// * `ip` - The integer to create the IP address from.
    fn from_integer(ip: Self::Integer) -> Self;

    /// Convert the IP address to an integer.
    ///
    /// # Returns
    ///
    /// The integer representation of the IP address.
    fn to_integer(&self) -> Self::Integer;

    fn from(s: &str) -> Result<Self, FoundationError>
    where
        Self: Sized;

    fn bits_in_mask(&self) -> u8;
}

impl IpAddrQuery for Ipv4Addr {
    type Integer = u32;

    fn is_global_address(&self) -> bool {
        let ip = self.to_integer();

        // Private subnets:
        // 2886729728 -> 172.16.0.0
        // 2887778303 -> 172.31.255.255
        // 167772160 -> 10.0.0.0
        // 184549375 -> 10.255.255.255
        // 3232235520 -> 192.168.0.0
        // 3232301055 -> 192.168.255.255

        // Addresses in the shared address space
        // 1681915904 -> 100.64.0.0
        // 1686110207 -> 100.127.255.255

        // Localhost addresses
        // 2130706432 -> 127.0.0.0
        // 2147483647 -> 127.255.255.255

        // Link local addresses
        // 2851995648 -> 169.254.0.0
        // 2852061183 -> 169.254.255.255

        // Documentation Addresses
        // 3221225984 -> 192.0.2.0
        // 3221226239 -> 192.0.2.255
        // 3325256704 -> 198.51.100.0
        // 3325256959 -> 198.51.100.255
        // 3405803776 -> 203.0.113.0
        // 3405804031 -> 203.0.113.255

        // Benchmarking Addresses
        // 3323068416 -> 198.18.0.0
        // 3323199487 -> 198.19.255.255

        // Reserved Addresses
        // 4026531840 -> 240.0.0.0
        // 4294967295 -> 255.255.255.255
        if (2886729728u32..=2887778303u32).contains(&ip)
            || (2130706432u32..=2147483647u32).contains(&ip)
            || (2851995648u32..=2852061183u32).contains(&ip)
            || (167772160u32..=184549375u32).contains(&ip)
            || (3232235520u32..=3232301055u32).contains(&ip)
            || (3221225984..=3221226239).contains(&ip)
            || (3325256704..=3325256959).contains(&ip)
            || (3405803776..=3405804031).contains(&ip)
            || (1681915904..=1686110207).contains(&ip)
            || (3323068416..=3323199487).contains(&ip)
            || (ip >= 4026531840)
            || ip == 0
        {
            return false;
        }
        true
    }

    fn from_integer(ip: Self::Integer) -> Ipv4Addr {
        // `to_integer` reads the octets in big-endian (network) order, so decode the
        // same way to ensure the two functions are inverses of each other.
        let bytes: [u8; 4] = ip.to_be_bytes();
        Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3])
    }

    fn to_integer(&self) -> Self::Integer {
        u32::from_be_bytes(self.octets())
    }

    fn from(s: &str) -> Result<Self, FoundationError>
    where
        Self: Sized,
    {
        match Ipv4Addr::from_str(s) {
            Ok(ip) => Ok(ip),
            Err(e) => Err(FoundationError::AddressParseError(e)),
        }
    }

    fn bits_in_mask(&self) -> u8 {
        bits_in_mask(&self.octets())
    }
}

impl IpAddrQuery for Ipv6Addr {
    type Integer = u128;

    fn is_global_address(&self) -> bool {
        let ip = self.to_integer();

        // The unspecified address
        // 0 -> ::

        // The loopback address
        // 1 -> ::1

        // The ipv4-mapped address (0:0:0:0:0:ffff::/96)
        // 281470681743360 -> ::ffff:
        // 281474976710655 -> ::ffff:ffff:ffff

        // Addresses reserved for benchmarking (2001:2::/48)
        // 42540488320432167789079031612388147200 -> 2001:2::
        // 42540488320433376714898646241562853375 -> 2001:0002:0000:ffff:ffff:ffff:ffff:ffff

        // Addresses reserved for documentation (2001:db8::/32)
        // 42540766411282592856903984951653826560 -> 2001:db8::
        // 42540766490510755371168322545197776895 -> 2001:db8:ffff:ffff:ffff:ffff:ffff:ffff

        // Unique local addresses (fc00::/7)
        // 334965454937798799971759379190646833152 -> fc00::
        // 337623910929368631717566993311207522303 -> fdff:ffff:ffff:ffff:ffff:ffff:ffff:ffff

        // Unique addresses with link local scope (fe80::/10)
        // 338288524927261089654018896841347694592 -> fe80::
        // 338620831926207318622244848606417780735 -> febf:ffff:ffff:ffff:ffff:ffff:ffff:ffff
        if self.is_unspecified()
            || self.is_loopback()
            || (281470681743360..=281474976710655).contains(&ip)
            || (42540488320432167789079031612388147200..=42540488320433376714898646241562853375)
                .contains(&ip)
            || (42540766411282592856903984951653826560..=42540766490510755371168322545197776895)
                .contains(&ip)
            || (334965454937798799971759379190646833152..=337623910929368631717566993311207522303)
                .contains(&ip)
            || (338288524927261089654018896841347694592..=338620831926207318622244848606417780735)
                .contains(&ip)
        {
            return false;
        }
        true
    }

    fn from_integer(ip: Self::Integer) -> Self {
        // `to_integer` reads the octets in big-endian (network) order, so decode the
        // same way to ensure the two functions are inverses of each other.
        let bytes: [u8; 16] = ip.to_be_bytes();

        let u16_values: Vec<u16> = (0..8)
            .map(|i| u16::from_be_bytes([bytes[i * 2], bytes[i * 2 + 1]]))
            .collect::<Vec<u16>>();
        Self::new(
            u16_values[0],
            u16_values[1],
            u16_values[2],
            u16_values[3],
            u16_values[4],
            u16_values[5],
            u16_values[6],
            u16_values[7],
        )
    }

    fn to_integer(&self) -> Self::Integer {
        let bytes = self.octets();
        u128::from_be_bytes(bytes)
    }

    fn from(s: &str) -> Result<Self, FoundationError>
    where
        Self: Sized,
    {
        match Ipv6Addr::from_str(s) {
            Ok(ip) => Ok(ip),
            Err(e) => Err(FoundationError::AddressParseError(e)),
        }
    }

    fn bits_in_mask(&self) -> u8 {
        bits_in_mask(&self.octets())
    }
}

impl IpAddrQuery for IpAddr {
    type Integer = u128;

    fn is_global_address(&self) -> bool {
        match self {
            IpAddr::V4(ip) => ip.is_global_address(),
            IpAddr::V6(ip) => ip.is_global_address(),
        }
    }

    fn from_integer(ip: Self::Integer) -> Self {
        if ip <= u32::MAX as u128 {
            IpAddr::V4(Ipv4Addr::from_integer(ip as u32))
        } else {
            IpAddr::V6(Ipv6Addr::from_integer(ip))
        }
    }

    fn to_integer(&self) -> Self::Integer {
        match self {
            IpAddr::V4(ip) => ip.to_integer() as u128,
            IpAddr::V6(ip) => ip.to_integer(),
        }
    }

    fn from(s: &str) -> Result<Self, FoundationError>
    where
        Self: Sized,
    {
        if let Ok(ipv4) = Ipv4Addr::from_str(s) {
            return Ok(IpAddr::V4(ipv4));
        } else if let Ok(ipv6) = Ipv6Addr::from_str(s) {
            return Ok(IpAddr::V6(ipv6));
        }
        Err(FoundationError::OperationFailed(
            "Failed to convert {} to either an IPv4 or IPv6 address".to_string(),
        ))
    }

    fn bits_in_mask(&self) -> u8 {
        match self {
            IpAddr::V4(ip) => ip.bits_in_mask(),
            IpAddr::V6(ip) => ip.bits_in_mask(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::netmask::{netmask_from_bits_ipv4, netmask_from_bits_ipv6};

    #[test]
    fn test_bits_in_mask() {
        for i in 1..33u8 {
            assert_eq!(
                <Ipv4Addr as From<[u8; 4]>>::from(netmask_from_bits_ipv4(i)).bits_in_mask(),
                i
            );
        }

        for i in 1..129u8 {
            assert_eq!(
                <Ipv6Addr as From<[u8; 16]>>::from(netmask_from_bits_ipv6(i)).bits_in_mask(),
                i
            );
        }
    }

    #[test]
    fn test_ipv4_is_global_address() {
        assert!(Ipv4Addr::new(8, 8, 8, 8).is_global_address());
        assert!(!Ipv4Addr::UNSPECIFIED.is_global_address());

        // Addresses reserved for private use: (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16)
        assert!(!Ipv4Addr::new(10, 254, 0, 0).is_global_address());
        assert!(!Ipv4Addr::new(192, 168, 10, 65).is_global_address());
        assert!(!Ipv4Addr::new(172, 16, 0, 65).is_global_address());

        // Addresses in the shared address space (100.64.0.0/10)
        assert!(!Ipv4Addr::new(100, 100, 0, 0).is_global_address());

        // The loopback addresses (127.0.0.0/8)
        assert!(!Ipv4Addr::new(127, 0, 0, 55).is_global_address());
        assert!(!Ipv4Addr::LOCALHOST.is_global_address());

        // The link-local addresses (169.254.0.0/16)
        assert!(!Ipv4Addr::new(169, 254, 45, 1).is_global_address());

        // Addresses reserved for documentation (192.0.2.0/24, 198.51.100.0/24, 203.0.113.0/24)
        assert!(!Ipv4Addr::new(192, 0, 2, 255).is_global_address());
        assert!(!Ipv4Addr::new(198, 51, 100, 65).is_global_address());
        assert!(!Ipv4Addr::new(203, 0, 113, 6).is_global_address());

        // Addresses reserved for benchmarking (198.18.0.0/15)
        assert!(!Ipv4Addr::new(198, 18, 0, 0).is_global_address());

        // Reserved addresses (240.0.0.0/4)
        assert!(!Ipv4Addr::new(250, 10, 20, 30).is_global_address());

        // The broadcast address (255.255.255.255)
        assert!(!Ipv4Addr::BROADCAST.is_global_address());
    }

    #[test]
    fn test_ipv6_is_global_address() {
        assert!(Ipv6Addr::new(0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8888).is_global_address());
        assert!(!Ipv6Addr::UNSPECIFIED.is_global_address());
        assert!(!Ipv6Addr::LOCALHOST.is_global_address());

        // The ipv4-mapped address (0:0:0:0:0:ffff::/96)
        assert!(!Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0, 0).is_global_address());

        // Addresses reserved for benchmarking (2001:2::/48)
        assert!(!Ipv6Addr::new(0x2001, 0x2, 0, 0, 0, 0, 0, 0).is_global_address());

        // Addresses reserved for documentation (2001:db8::/32)
        assert!(!Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0).is_global_address());

        // Unique local addresses (fc00::/7)
        assert!(!Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 0).is_global_address());

        // Unique addresses with link local scope (fe80::/10)
        assert!(!Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 0).is_global_address());
    }

    #[test]
    fn test_ipv4_to_integer() {
        // `to_integer` interprets the octets as a big-endian (network-order) number.
        assert_eq!(Ipv4Addr::new(8, 8, 8, 8).to_integer(), 0x0808_0808);
        assert_eq!(Ipv4Addr::new(192, 168, 1, 1).to_integer(), 0xC0A8_0101);
        assert_eq!(Ipv4Addr::UNSPECIFIED.to_integer(), 0);
        assert_eq!(Ipv4Addr::BROADCAST.to_integer(), u32::MAX);
    }

    #[test]
    fn test_ipv6_to_integer() {
        assert_eq!(Ipv6Addr::LOCALHOST.to_integer(), 1u128);
        assert_eq!(Ipv6Addr::UNSPECIFIED.to_integer(), 0u128);
    }

    #[test]
    fn test_ipv4_integer_roundtrip() {
        for addr in [
            Ipv4Addr::new(8, 8, 8, 8),
            Ipv4Addr::new(192, 168, 1, 1),
            Ipv4Addr::new(203, 0, 113, 42),
            Ipv4Addr::UNSPECIFIED,
            Ipv4Addr::BROADCAST,
        ] {
            assert_eq!(Ipv4Addr::from_integer(addr.to_integer()), addr);
        }

        // `from_integer` decodes a known big-endian value correctly.
        assert_eq!(
            Ipv4Addr::from_integer(0xC0A8_0101),
            Ipv4Addr::new(192, 168, 1, 1)
        );
    }

    #[test]
    fn test_ipv6_integer_roundtrip() {
        for addr in [
            Ipv6Addr::new(0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8888),
            Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 1),
            Ipv6Addr::LOCALHOST,
            Ipv6Addr::UNSPECIFIED,
        ] {
            assert_eq!(Ipv6Addr::from_integer(addr.to_integer()), addr);
        }

        // `from_integer` decodes a known big-endian value correctly.
        assert_eq!(Ipv6Addr::from_integer(1u128), Ipv6Addr::LOCALHOST);
    }

    #[test]
    fn test_ipv4_from_str() {
        assert_eq!(
            <Ipv4Addr as IpAddrQuery>::from("1.2.3.4").unwrap(),
            Ipv4Addr::new(1, 2, 3, 4)
        );
        assert!(matches!(
            <Ipv4Addr as IpAddrQuery>::from("not_an_ip"),
            Err(FoundationError::AddressParseError(_))
        ));
    }

    #[test]
    fn test_ipv6_from_str() {
        assert_eq!(
            <Ipv6Addr as IpAddrQuery>::from("::1").unwrap(),
            Ipv6Addr::LOCALHOST
        );
        assert!(matches!(
            <Ipv6Addr as IpAddrQuery>::from("not_an_ip"),
            Err(FoundationError::AddressParseError(_))
        ));
    }

    #[test]
    fn test_ipaddr_from_str_dispatches_on_family() {
        assert_eq!(
            <IpAddr as IpAddrQuery>::from("10.0.0.1").unwrap(),
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))
        );
        assert_eq!(
            <IpAddr as IpAddrQuery>::from("fc00::1").unwrap(),
            IpAddr::V6(Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 1))
        );
        assert!(matches!(
            <IpAddr as IpAddrQuery>::from("garbage"),
            Err(FoundationError::OperationFailed(_))
        ));
    }

    #[test]
    fn test_ipaddr_from_integer_selects_family() {
        // Values that fit in a u32 map to IPv4.
        assert_eq!(
            IpAddr::from_integer(0x0808_0808u128),
            IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))
        );
        // Values larger than u32::MAX map to IPv6.
        let big = (u32::MAX as u128) + 1;
        assert!(matches!(IpAddr::from_integer(big), IpAddr::V6(_)));
    }

    #[test]
    fn test_ipaddr_to_integer_and_global() {
        let v4 = IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8));
        assert_eq!(v4.to_integer(), 0x0808_0808u128);
        assert!(v4.is_global_address());

        let v6 = IpAddr::V6(Ipv6Addr::LOCALHOST);
        assert_eq!(v6.to_integer(), 1u128);
        assert!(!v6.is_global_address());
    }
}
