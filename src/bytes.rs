//! The `bytes` module contains simple code for normalizing a byte size into a human-readable format.

use crate::constants::*;
use std::collections::HashMap;

/// The `ByteMetricBase` enum represents the base to use when converting bytes to a human-readable
/// format.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ByteMetricBase {
    /// Use 1024 for metric prefixes.
    Metric,

    /// Use 1000 for decimal prefixes.
    Decimal,
}

/// Normalize a byte size into a human-readable format.
///
/// # Arguments
///
/// * `size` - The size in bytes to normalize.
/// * `metric_base` - The base to use when converting bytes to a human-readable format.
///
/// # Returns
///
/// A string representing the normalized byte size.
pub fn normalize_byte_size(size: u128, metric_base: ByteMetricBase) -> String {
    let (divisor, suffix) = normalize_size_for_divisor_and_suffix(size, metric_base);
    format!("{:.2} {}", (size as f64) / (divisor as f64), suffix)
}

/// Convert a byte size into a normalized size and suffix.
///
/// # Arguments
///
/// * `size` - The size in bytes to normalize.
/// * `metric_base` - The base to use when converting bytes to a human-readable format.
///
/// # Returns
///
/// A tuple containing the normalized size and suffix.
pub fn normalize_size(size: u128, metric_base: ByteMetricBase) -> (f64, String) {
    let (divisor, suffix) = normalize_size_for_divisor_and_suffix(size, metric_base);
    ((size as f64) / (divisor as f64), suffix)
}

/// Calculate a divisor and suffix for a given size and metric base.
///
/// # Arguments
///
/// * `size` - The size in bytes to normalize.
/// * `metric_base` - The base to use when converting bytes to a human-readable format.
///
/// # Returns
///
/// A tuple containing the divisor and suffix.
fn normalize_size_for_divisor_and_suffix(
    size: u128,
    metric_base: ByteMetricBase,
) -> (u128, String) {
    let (suffix_map, divisor_map): (HashMap<u128, String>, HashMap<u128, u128>) = match metric_base
    {
        ByteMetricBase::Metric => (
            vec![
                (YOTTA, "Yb".to_string()),
                (ZETTA, "Zb".to_string()),
                (EXA as u128, "Eb".to_string()),
                (PETA as u128, "Pb".to_string()),
                (TERA as u128, "Tb".to_string()),
                (GIGA as u128, "Gb".to_string()),
                (MEGA as u128, "Mb".to_string()),
                (KILO as u128, "Kb".to_string()),
            ]
            .into_iter()
            .collect(),
            vec![
                (YOTTA, YOTTA),
                (ZETTA, ZETTA),
                (EXA as u128, EXA as u128),
                (PETA as u128, PETA as u128),
                (TERA as u128, TERA as u128),
                (GIGA as u128, GIGA as u128),
                (MEGA as u128, MEGA as u128),
                (KILO as u128, KILO as u128),
            ]
            .into_iter()
            .collect(),
        ),
        ByteMetricBase::Decimal => (
            vec![
                (YOTTA, "YB".to_string()),
                (ZETTA, "ZB".to_string()),
                (EXA as u128, "EB".to_string()),
                (PETA as u128, "PB".to_string()),
                (TERA as u128, "TB".to_string()),
                (GIGA as u128, "GB".to_string()),
                (MEGA as u128, "MB".to_string()),
                (KILO as u128, "KB".to_string()),
            ]
            .into_iter()
            .collect(),
            vec![
                (YOTTA, MYOTTA),
                (ZETTA, MZETTA),
                (EXA as u128, MEXA as u128),
                (PETA as u128, MPETA as u128),
                (TERA as u128, MTERA as u128),
                (GIGA as u128, MGIGA as u128),
                (MEGA as u128, MMEGA as u128),
                (KILO as u128, MKILO as u128),
            ]
            .into_iter()
            .collect(),
        ),
    };

    let (suffix, divisor) = if size < *divisor_map.get(&YOTTA).unwrap() {
        if size < *divisor_map.get(&ZETTA).unwrap() {
            if size < *divisor_map.get(&(EXA as u128)).unwrap() {
                if size < *divisor_map.get(&(PETA as u128)).unwrap() {
                    if size < *divisor_map.get(&(TERA as u128)).unwrap() {
                        if size < *divisor_map.get(&(GIGA as u128)).unwrap() {
                            if size < *divisor_map.get(&(MEGA as u128)).unwrap() {
                                if size < *divisor_map.get(&(KILO as u128)).unwrap() {
                                    ("bytes".to_string(), 1u128)
                                } else {
                                    (
                                        suffix_map.get(&(KILO as u128)).unwrap().to_string(),
                                        *divisor_map.get(&(KILO as u128)).unwrap(),
                                    )
                                }
                            } else {
                                (
                                    suffix_map.get(&(MEGA as u128)).unwrap().to_string(),
                                    *divisor_map.get(&(MEGA as u128)).unwrap(),
                                )
                            }
                        } else {
                            (
                                suffix_map.get(&(GIGA as u128)).unwrap().to_string(),
                                *divisor_map.get(&(GIGA as u128)).unwrap(),
                            )
                        }
                    } else {
                        (
                            suffix_map.get(&(TERA as u128)).unwrap().to_string(),
                            *divisor_map.get(&(TERA as u128)).unwrap(),
                        )
                    }
                } else {
                    (
                        suffix_map.get(&(PETA as u128)).unwrap().to_string(),
                        *divisor_map.get(&(PETA as u128)).unwrap(),
                    )
                }
            } else {
                (
                    suffix_map.get(&(EXA as u128)).unwrap().to_string(),
                    *divisor_map.get(&(EXA as u128)).unwrap(),
                )
            }
        } else {
            (
                suffix_map.get(&ZETTA).unwrap().to_string(),
                *divisor_map.get(&ZETTA).unwrap(),
            )
        }
    } else {
        (
            suffix_map.get(&YOTTA).unwrap().to_string(),
            *divisor_map.get(&YOTTA).unwrap(),
        )
    };

    (divisor, suffix)
}

pub fn bytes_from_string(s: &str) -> Option<u128> {
    let s = s.trim();

    // split numeric and unit parts
    let idx = s
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(s.len());

    let (num, unit) = s.split_at(idx);

    let value: f64 = num.parse().ok()?;
    let multiplier: u128 = match unit.trim() {
        "" | "b" | "B" => 1,
        "Kb" => 1024_u128,
        "KB" => 1000_u128,
        "Mb" => 1024_u128.pow(2),
        "MB" => 1000_u128.pow(2),
        "Gb" => 1024_u128.pow(3),
        "GB" => 1000_u128.pow(3),
        "Tb" => 1024_u128.pow(4),
        "TB" => 1000_u128.pow(4),
        "Pb" => 1024_u128.pow(5),
        "PB" => 1000_u128.pow(5),
        "Eb" => 1024_u128.pow(6),
        "EB" => 1000_u128.pow(6),
        "Zb" => 1024_u128.pow(7),
        "ZB" => 1000_u128.pow(7),
        "Yb" => 1024_u128.pow(8),
        "YB" => 1000_u128.pow(8),
        _ => return None,
    };

    Some((value * multiplier as f64) as u128)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalization_of_metrics() {
        assert_eq!(
            normalize_byte_size(10, ByteMetricBase::Metric),
            "10.00 bytes"
        );
        assert_eq!(
            normalize_byte_size(10, ByteMetricBase::Decimal),
            "10.00 bytes"
        );
        assert_eq!(normalize_byte_size(1024, ByteMetricBase::Metric), "1.00 Kb");
        assert_eq!(
            normalize_byte_size(1024, ByteMetricBase::Decimal),
            "1.02 KB"
        );
        assert_eq!(
            normalize_byte_size(1048576, ByteMetricBase::Metric),
            "1.00 Mb"
        );
        assert_eq!(
            normalize_byte_size(1048576, ByteMetricBase::Decimal),
            "1.05 MB"
        );
        assert_eq!(
            normalize_byte_size(1073741824, ByteMetricBase::Metric),
            "1.00 Gb"
        );
        assert_eq!(
            normalize_byte_size(1073741824, ByteMetricBase::Decimal),
            "1.07 GB"
        );
        assert_eq!(
            normalize_byte_size(1099511627776, ByteMetricBase::Metric),
            "1.00 Tb"
        );
        assert_eq!(
            normalize_byte_size(1099511627776, ByteMetricBase::Decimal),
            "1.10 TB"
        );
        assert_eq!(
            normalize_byte_size(1125899906842624, ByteMetricBase::Metric),
            "1.00 Pb"
        );
        assert_eq!(
            normalize_byte_size(1125899906842624, ByteMetricBase::Decimal),
            "1.13 PB"
        );
        assert_eq!(
            normalize_byte_size(1152921504606846976, ByteMetricBase::Metric),
            "1.00 Eb"
        );
        assert_eq!(
            normalize_byte_size(1152921504606846976, ByteMetricBase::Decimal),
            "1.15 EB"
        );
        assert_eq!(
            normalize_byte_size(1180591620717411303424, ByteMetricBase::Metric),
            "1.00 Zb"
        );
        assert_eq!(
            normalize_byte_size(1180591620717411303424, ByteMetricBase::Decimal),
            "1.18 ZB"
        );
        assert_eq!(
            normalize_byte_size(1208925819614629174706176, ByteMetricBase::Metric),
            "1.00 Yb"
        );
        assert_eq!(
            normalize_byte_size(1208925819614629174706176, ByteMetricBase::Decimal),
            "1.21 YB"
        );
    }

    #[test]
    fn test_normalization_of_decimals() {
        assert_eq!(
            normalize_byte_size(1000, ByteMetricBase::Metric),
            "1000.00 bytes"
        );
        assert_eq!(
            normalize_byte_size(1000, ByteMetricBase::Decimal),
            "1.00 KB"
        );
        assert_eq!(
            normalize_byte_size(1000000, ByteMetricBase::Metric),
            "976.56 Kb"
        );
        assert_eq!(
            normalize_byte_size(1000000, ByteMetricBase::Decimal),
            "1.00 MB"
        );
        assert_eq!(
            normalize_byte_size(1000000000, ByteMetricBase::Metric),
            "953.67 Mb"
        );
        assert_eq!(
            normalize_byte_size(1000000000, ByteMetricBase::Decimal),
            "1.00 GB"
        );
        assert_eq!(
            normalize_byte_size(1000000000000, ByteMetricBase::Metric),
            "931.32 Gb"
        );
        assert_eq!(
            normalize_byte_size(1000000000000, ByteMetricBase::Decimal),
            "1.00 TB"
        );
        assert_eq!(
            normalize_byte_size(1000000000000000, ByteMetricBase::Metric),
            "909.49 Tb"
        );
        assert_eq!(
            normalize_byte_size(1000000000000000, ByteMetricBase::Decimal),
            "1.00 PB"
        );
        assert_eq!(
            normalize_byte_size(1000000000000000000, ByteMetricBase::Metric),
            "888.18 Pb"
        );
        assert_eq!(
            normalize_byte_size(1000000000000000000, ByteMetricBase::Decimal),
            "1.00 EB"
        );
        assert_eq!(
            normalize_byte_size(1000000000000000000000, ByteMetricBase::Metric),
            "867.36 Eb"
        );
        assert_eq!(
            normalize_byte_size(1000000000000000000000, ByteMetricBase::Decimal),
            "1.00 ZB"
        );
        assert_eq!(
            normalize_byte_size(1000000000000000000000000, ByteMetricBase::Metric),
            "847.03 Zb"
        );
        assert_eq!(
            normalize_byte_size(1000000000000000000000000, ByteMetricBase::Decimal),
            "1.00 YB"
        );
    }

    #[test]
    fn test_normalize_size_of_metric() {
        assert_eq!(
            normalize_size(10, ByteMetricBase::Metric),
            (10.0, "bytes".to_string())
        );
        assert_eq!(
            normalize_size(10, ByteMetricBase::Decimal),
            (10.0, "bytes".to_string())
        );
        assert_eq!(
            normalize_size(1024, ByteMetricBase::Metric),
            (1.0, "Kb".to_string())
        );
        assert_eq!(
            normalize_size(1024, ByteMetricBase::Decimal),
            (1.024, "KB".to_string())
        );
        assert_eq!(
            normalize_size(1048576, ByteMetricBase::Metric),
            (1.0, "Mb".to_string())
        );
        assert_eq!(
            normalize_size(1048576, ByteMetricBase::Decimal),
            (1.048576, "MB".to_string())
        );
        assert_eq!(
            normalize_size(1073741824, ByteMetricBase::Metric),
            (1.0, "Gb".to_string())
        );
        assert_eq!(
            normalize_size(1073741824, ByteMetricBase::Decimal),
            (1.073741824, "GB".to_string())
        );
        assert_eq!(
            normalize_size(1099511627776, ByteMetricBase::Metric),
            (1.0, "Tb".to_string())
        );
        assert_eq!(
            normalize_size(1099511627776, ByteMetricBase::Decimal),
            (1.099511627776, "TB".to_string())
        );
        assert_eq!(
            normalize_size(1125899906842624, ByteMetricBase::Metric),
            (1.0, "Pb".to_string())
        );
        assert_eq!(
            normalize_size(1125899906842624, ByteMetricBase::Decimal),
            (1.125899906842624, "PB".to_string())
        );
        assert_eq!(
            normalize_size(1152921504606846976, ByteMetricBase::Metric),
            (1.0, "Eb".to_string())
        );
        assert_eq!(
            normalize_size(1152921504606846976, ByteMetricBase::Decimal),
            (1.152921504606847, "EB".to_string())
        );
        assert_eq!(
            normalize_size(1180591620717411303424, ByteMetricBase::Metric),
            (1.0, "Zb".to_string())
        );
        assert_eq!(
            normalize_size(1180591620717411303424, ByteMetricBase::Decimal),
            (1.1805916207174113, "ZB".to_string())
        );
        assert_eq!(
            normalize_size(1208925819614629174706176, ByteMetricBase::Metric),
            (1.0, "Yb".to_string())
        );
        assert_eq!(
            normalize_size(1208925819614629174706176, ByteMetricBase::Decimal),
            (1.2089258196146292, "YB".to_string())
        );
    }

    #[test]
    fn test_normalize_size_of_decimal() {
        assert_eq!(
            normalize_size(1000, ByteMetricBase::Metric),
            (1000.0, "bytes".to_string())
        );
        assert_eq!(
            normalize_size(1000, ByteMetricBase::Decimal),
            (1.0, "KB".to_string())
        );
        assert_eq!(
            normalize_size(1000000, ByteMetricBase::Metric),
            (976.5625, "Kb".to_string())
        );
        assert_eq!(
            normalize_size(1000000, ByteMetricBase::Decimal),
            (1.0, "MB".to_string())
        );
        assert_eq!(
            normalize_size(1000000000, ByteMetricBase::Metric),
            (953.67431640625, "Mb".to_string())
        );
        assert_eq!(
            normalize_size(1000000000, ByteMetricBase::Decimal),
            (1.0, "GB".to_string())
        );
        assert_eq!(
            normalize_size(1000000000000, ByteMetricBase::Metric),
            (931.3225746154785, "Gb".to_string())
        );
        assert_eq!(
            normalize_size(1000000000000, ByteMetricBase::Decimal),
            (1.0, "TB".to_string())
        );
        assert_eq!(
            normalize_size(1000000000000000, ByteMetricBase::Metric),
            (909.4947017729282, "Tb".to_string())
        );
        assert_eq!(
            normalize_size(1000000000000000, ByteMetricBase::Decimal),
            (1.0, "PB".to_string())
        );
        assert_eq!(
            normalize_size(1000000000000000000, ByteMetricBase::Metric),
            (888.1784197001252, "Pb".to_string())
        );
        assert_eq!(
            normalize_size(1000000000000000000, ByteMetricBase::Decimal),
            (1.0, "EB".to_string())
        );
        assert_eq!(
            normalize_size(1000000000000000000000, ByteMetricBase::Metric),
            (867.3617379884035, "Eb".to_string())
        );
        assert_eq!(
            normalize_size(1000000000000000000000, ByteMetricBase::Decimal),
            (1.0, "ZB".to_string())
        );
        assert_eq!(
            normalize_size(1000000000000000000000000, ByteMetricBase::Metric),
            (847.0329472543003, "Zb".to_string())
        );
        assert_eq!(
            normalize_size(1000000000000000000000000, ByteMetricBase::Decimal),
            (1.0, "YB".to_string())
        );
    }

    #[test]
    fn test_bytes_from_string() {
        assert_eq!(bytes_from_string("1024"), Some(1024_u128));
        assert_eq!(bytes_from_string("1024b"), Some(1024_u128));
        assert_eq!(bytes_from_string("1024B"), Some(1024_u128));

        assert_eq!(bytes_from_string("1Kb"), Some(1024_u128));
        assert_eq!(bytes_from_string("1KB"), Some(1000_u128));
        assert_eq!(bytes_from_string("15Kb"), Some(15360_u128));
        assert_eq!(bytes_from_string("15KB"), Some(15000_u128));

        assert_eq!(bytes_from_string("1Mb"), Some(1048576_u128));
        assert_eq!(bytes_from_string("1MB"), Some(1000000_u128));
        assert_eq!(bytes_from_string("17Mb"), Some(17825792_u128));
        assert_eq!(bytes_from_string("17MB"), Some(17000000_u128));

        assert_eq!(bytes_from_string("1Gb"), Some(1073741824_u128));
        assert_eq!(bytes_from_string("1GB"), Some(1000000000_u128));
        assert_eq!(bytes_from_string("18Gb"), Some(19327352832_u128));
        assert_eq!(bytes_from_string("18GB"), Some(18000000000_u128));

        assert_eq!(bytes_from_string("1Tb"), Some(1099511627776_u128));
        assert_eq!(bytes_from_string("1TB"), Some(1000000000000_u128));
        assert_eq!(bytes_from_string("82Tb"), Some(90159953477632_u128));
        assert_eq!(bytes_from_string("82TB"), Some(82000000000000_u128));

        assert_eq!(bytes_from_string("1Pb"), Some(1125899906842624_u128));
        assert_eq!(bytes_from_string("1PB"), Some(1000000000000000_u128));
        assert_eq!(bytes_from_string("4Pb"), Some(4503599627370496_u128));
        assert_eq!(bytes_from_string("4PB"), Some(4000000000000000_u128));

        assert_eq!(bytes_from_string("1Eb"), Some(1152921504606846976_u128));
        assert_eq!(bytes_from_string("1EB"), Some(1000000000000000000_u128));
        assert_eq!(bytes_from_string("8Eb"), Some(9223372036854775808_u128));
        assert_eq!(bytes_from_string("8EB"), Some(8000000000000000000_u128));

        assert_eq!(bytes_from_string("1Zb"), Some(1180591620717411303424_u128));
        assert_eq!(bytes_from_string("1ZB"), Some(1000000000000000000000_u128));
        assert_eq!(
            bytes_from_string("12Zb"),
            Some(14167099448608935641088_u128)
        );
        assert_eq!(
            bytes_from_string("12ZB"),
            Some(12000000000000000000000_u128)
        );

        assert_eq!(
            bytes_from_string("1Yb"),
            Some(1208925819614629174706176_u128)
        );
        // bytes_from_string uses f64 to represent its multiplier. 1000^8 exceeds
        // the range of f64. In the future we will use arbitrary precision floats
        // to make this work. For you Yotta scale is not practical.
        // assert_eq!(bytes_from_string("1YB"), Some(1000000000000000000000000_u128));
        // assert_eq!(bytes_from_string("17Yb"), Some(20551738933448695970004992_u128));
        // assert_eq!(bytes_from_string("17YB"), Some(17000000000000000000000000_u128));
    }
}
