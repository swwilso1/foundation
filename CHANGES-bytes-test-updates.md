# Change Summary: bytes module test updates

**Branch:** `claude/test-updates`
**Commit:** `34439f9`
**File touched:** `src/bytes.rs`

## Fix

- `bytes_from_string` now accepts the `"bytes"` unit (`"" | "b" | "B" | "bytes" => 1`).
  Previously the `"bytes"` suffix emitted by `normalize_byte_size` for sub-KiB
  values could not be parsed back, so those values did not round-trip.

## Tests added

| Test | Covers |
| --- | --- |
| `test_bytes_from_string_zero` | Zero values with/without units |
| `test_bytes_from_string_fractional` | `"1.5Kb"`, `"0.5GB"`, bare-fractional truncation |
| `test_bytes_from_string_whitespace_and_space_separated` | Trimming and space-separated unit (`"1 Kb"`) |
| `test_bytes_from_string_invalid` | `None` branch: empty, missing number, unknown unit, malformed numbers, negatives, case-sensitivity |
| `test_bytes_from_string_bytes_unit` | `"bytes"` suffix is parseable (the fix) |
| `test_normalize_byte_size_zero` | Zero edge for `normalize_byte_size` |
| `test_normalize_byte_size_sub_boundary` | Values straddling the kilo boundary (1023, 999) |
| `test_normalize_byte_size_rounding` | `{:.2}` rounding behavior |
| `test_normalize_size_zero` | Zero edge for `normalize_size` |
| `test_normalize_size_max_u128` | Top Yotta bucket / overflow boundary |
| `test_round_trip_metric` | `normalize_byte_size` output feeds back through `bytes_from_string`, incl. sub-KiB |

## Status

- All 16 `bytes::` tests pass (`cargo test --lib bytes::`).
- Committed to `claude/test-updates`; not pushed.

## Notes / behaviors pinned by tests

- `bytes_from_string` tolerates a space between number and unit (`"1 Kb"`),
  even though `normalize_byte_size` never emits that form.
- Unit casing is significant: `"kb"` / `"gb"` are not recognized.
