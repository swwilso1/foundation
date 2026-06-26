# Change Summary: systemctlservice module test updates

**Branch:** `claude/test-updates`
**Files touched:** `src/systemctlservice.rs` (tests only — no production code changed)

## Context

`systemctlservice` is gated to Linux only (`src/lib.rs` compiles `pub mod systemctlservice`
inside `cfg_if! { if #[cfg(target_os = "linux")] { ... } }`). It is therefore **not built on
this macOS host**, so the tests below run on Linux CI rather than locally. They were verified
to compile for Linux with `cargo check --lib --tests --target x86_64-unknown-linux-gnu`.

Every method (`start`, `stop`, `restart`) shells out to the real `systemctl` binary. To avoid
mutating any real unit, the tests operate on a deliberately bogus service name. The expected
error depends on the host:

- `systemctl` missing → the spawn fails and the `?` operator returns `FoundationError::IO`.
- `systemctl` present → the command runs and exits non-zero → `FoundationError::OperationFailed`.

A shared helper, `assert_operation_errors`, accepts either outcome and cross-checks it against
`systemctl --version` availability, so the suite is correct whether or not the CI container has
a working systemd.

## Tests added

### `src/systemctlservice.rs` (new `tests` module)

| Test | Covers |
| --- | --- |
| `test_new_stores_service_name` | `new` stores the supplied service name on the struct |
| `test_start_bogus_service_errors` | `start` error path (`IO` spawn failure or `OperationFailed` non-zero status) |
| `test_stop_bogus_service_errors` | `stop` error path (same two branches) |
| `test_restart_bogus_service_errors` | `restart` error path (same two branches) |

## Coverage

Cannot be measured on darwin (module is Linux-gated and not compiled here). On a Linux runner
the tests exercise `new` and the full body of `start`/`stop`/`restart`, including the
`!output.status.success()` → `OperationFailed` branch and the `?`-propagated `IO` branch.

The happy-path `Ok(())` arms are intentionally not forced: actually starting/stopping a real
service would require root and would mutate host state, which the tests deliberately avoid.

## Status

- Tests compile clean for `x86_64-unknown-linux-gnu` (`cargo check --lib --tests`).
- Not run locally (Linux-only module); not committed; not pushed.
