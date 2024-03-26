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
        if (ip >= 2886729728u32 && ip <= 2887778303u32)
            || (ip >= 2130706432u32 && ip <= 2147483647u32)
            || (ip >= 2851995648u32 && ip <= 2852061183u32)
            || (ip >= 167772160u32 && ip <= 184549375u32)
            || (ip >= 3232235520u32 && ip <= 3232301055u32)
            || (ip >= 3221225984 && ip <= 3221226239)
            || (ip >= 3325256704 && ip <= 3325256959)
            || (ip >= 3405803776 && ip <= 3405804031)
            || (ip >= 1681915904 && ip <= 1686110207)
            || (ip >= 3323068416 && ip <= 3323199487)
            || (ip >= 4026531840)
            || ip == 0
        {
            return false;
        }
        true
    }

    fn from_integer(ip: Self::Integer) -> Ipv4Addr {
        let bytes: [u8; 4] = if cfg!(target_endian = "little") {
            ip.to_le_bytes()
        } else {
            ip.to_be_bytes()
        };
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
            || (ip >= 281470681743360 && ip <= 281474976710655)
            || (ip >= 42540488320432167789079031612388147200
                && ip <= 42540488320433376714898646241562853375)
            || (ip >= 42540766411282592856903984951653826560
                && ip <= 42540766490510755371168322545197776895)
            || (ip >= 334965454937798799971759379190646833152
                && ip <= 337623910929368631717566993311207522303)
            || (ip >= 338288524927261089654018896841347694592
                && ip <= 338620831926207318622244848606417780735)
        {
            return false;
        }
        true
    }

    fn from_integer(ip: Self::Integer) -> Self {
        let bytes: [u8; 16] = if cfg!(target_endian = "little") {
            ip.to_le_bytes()
        } else {
            ip.to_be_bytes()
        };

        let u16_values: Vec<u16> = (0..8)
            .map(|i| {
                if cfg!(target_endian = "little") {
                    u16::from_le_bytes([bytes[i * 2], bytes[i * 2 + 1]])
                } else {
                    u16::from_be_bytes([bytes[i * 2], bytes[i * 2 + 1]])
                }
            })
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
        assert_eq!(Ipv4Addr::new(8, 8, 8, 8).is_global_address(), true);
        assert_eq!(Ipv4Addr::UNSPECIFIED.is_global_address(), false);

        // Addresses reserved for private use: (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16)
        assert_eq!(Ipv4Addr::new(10, 254, 0, 0).is_global_address(), false);
        assert_eq!(Ipv4Addr::new(192, 168, 10, 65).is_global_address(), false);
        assert_eq!(Ipv4Addr::new(172, 16, 0, 65).is_global_address(), false);

        // Addresses in the shared address space (100.64.0.0/10)
        assert_eq!(Ipv4Addr::new(100, 100, 0, 0).is_global_address(), false);

        // The loopback addresses (127.0.0.0/8)
        assert_eq!(Ipv4Addr::new(127, 0, 0, 55).is_global_address(), false);
        assert_eq!(Ipv4Addr::LOCALHOST.is_global_address(), false);

        // The link-local addresses (169.254.0.0/16)
        assert_eq!(Ipv4Addr::new(169, 254, 45, 1).is_global_address(), false);

        // Addresses reserved for documentation (192.0.2.0/24, 198.51.100.0/24, 203.0.113.0/24)
        assert_eq!(Ipv4Addr::new(192, 0, 2, 255).is_global_address(), false);
        assert_eq!(Ipv4Addr::new(198, 51, 100, 65).is_global_address(), false);
        assert_eq!(Ipv4Addr::new(203, 0, 113, 6).is_global_address(), false);

        // Addresses reserved for benchmarking (198.18.0.0/15)
        assert_eq!(Ipv4Addr::new(198, 18, 0, 0).is_global_address(), false);

        // Reserved addresses (240.0.0.0/4)
        assert_eq!(Ipv4Addr::new(250, 10, 20, 30).is_global_address(), false);

        // The broadcast address (255.255.255.255)
        assert_eq!(Ipv4Addr::BROADCAST.is_global_address(), false);
    }

    #[test]
    fn test_ipv6_is_global_address() {
        assert_eq!(
            Ipv6Addr::new(0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8888).is_global_address(),
            true
        );
        assert_eq!(Ipv6Addr::UNSPECIFIED.is_global_address(), false);
        assert_eq!(Ipv6Addr::LOCALHOST.is_global_address(), false);

        // The ipv4-mapped address (0:0:0:0:0:ffff::/96)
        assert_eq!(
            Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0, 0).is_global_address(),
            false
        );

        // Addresses reserved for benchmarking (2001:2::/48)
        assert_eq!(
            Ipv6Addr::new(0x2001, 0x2, 0, 0, 0, 0, 0, 0).is_global_address(),
            false
        );

        // Addresses reserved for documentation (2001:db8::/32)
        assert_eq!(
            Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0).is_global_address(),
            false
        );

        // Unique local addresses (fc00::/7)
        assert_eq!(
            Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 0).is_global_address(),
            false
        );

        // Unique addresses with link local scope (fe80::/10)
        assert_eq!(
            Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 0).is_global_address(),
            false
        );
    }
}
