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
}
