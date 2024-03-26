//! The `netmask` module contains functions to assist in converting netmask bytes to number
//! of bits and number of bits to netmask bytes.

/// The `BitMaskBits` struct contains a mask and the number of bits set in the mask.
struct BitMaskBits {
    pub mask: u8,
    pub bits: u8,
}

const BITS_IN_MASK_ARRAY: [BitMaskBits; 7] = [
    BitMaskBits {
        mask: 0xfe,
        bits: 7,
    },
    BitMaskBits {
        mask: 0xfc,
        bits: 6,
    },
    BitMaskBits {
        mask: 0xf8,
        bits: 5,
    },
    BitMaskBits {
        mask: 0xf0,
        bits: 4,
    },
    BitMaskBits {
        mask: 0xe0,
        bits: 3,
    },
    BitMaskBits {
        mask: 0xc0,
        bits: 2,
    },
    BitMaskBits {
        mask: 0x80,
        bits: 1,
    },
];

/// Return the number of bits set in the netmask.
///
/// # Arguments
///
/// * `bytes` - The netmask bytes.
///
/// # Returns
///
/// The number of bits set in the netmask.
pub fn bits_in_mask(bytes: &[u8]) -> u8 {
    let mut count = 0u8;

    for byte in bytes.iter() {
        if byte == &0xff {
            count += 8;
            continue;
        }

        if byte == &0 {
            break;
        }

        for mask in BITS_IN_MASK_ARRAY.iter() {
            if byte & mask.mask == mask.mask {
                count += mask.bits;
                break;
            }
        }
        break;
    }
    count
}

const NETMASK_FROM_BITS_MASK_ARRAY: [BitMaskBits; 7] = [
    BitMaskBits {
        mask: 0x80,
        bits: 1,
    },
    BitMaskBits {
        mask: 0xc0,
        bits: 2,
    },
    BitMaskBits {
        mask: 0xe0,
        bits: 3,
    },
    BitMaskBits {
        mask: 0xf0,
        bits: 4,
    },
    BitMaskBits {
        mask: 0xf8,
        bits: 5,
    },
    BitMaskBits {
        mask: 0xfc,
        bits: 6,
    },
    BitMaskBits {
        mask: 0xfe,
        bits: 7,
    },
];

/// Return the netmask bytes from the number of bits.
///
/// # Arguments
///
/// * `cidr` - The number of bits.
///
/// # Returns
///
/// The netmask bytes for an Ipv4Addr.
pub fn netmask_from_bits_ipv4(cidr: u8) -> [u8; 4] {
    let bytes = cidr / 8;
    let bits = cidr % 8;

    let mut octet = [0u8; 4];
    for i in 0..bytes {
        octet[i as usize] = 0xff;
    }
    if bytes < 4 && bits != 0 {
        octet[bytes as usize] = NETMASK_FROM_BITS_MASK_ARRAY[(bits - 1) as usize].mask;
    }
    octet
}

/// Return the netmask bytes from the number of bits.
///
/// # Arguments
///
/// * `cidr` - The number of bits.
///
/// # Returns
///
/// The netmask bytes for an Ipv6Addr.
pub fn netmask_from_bits_ipv6(cidr: u8) -> [u8; 16] {
    let bytes = cidr / 8;
    let bits = cidr % 8;

    let mut octet = [0u8; 16];
    for i in 0..bytes {
        octet[i as usize] = 0xff;
    }
    if bytes < 16 && bits != 0 {
        octet[bytes as usize] = NETMASK_FROM_BITS_MASK_ARRAY[(bits - 1) as usize].mask;
    }
    octet
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bits_in_mask() {
        for i in 1..33u8 {
            assert_eq!(bits_in_mask(&netmask_from_bits_ipv4(i)), i);
        }

        for i in 1..129u8 {
            assert_eq!(bits_in_mask(&netmask_from_bits_ipv6(i)), i);
        }
    }
}
