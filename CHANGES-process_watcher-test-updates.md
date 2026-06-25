# Change Summary: process_watcher module test updates

**Branch:** `claude/test-updates`
**Files touched:** `src/process_watcher.rs` and `src/process/watcher_posix.rs` (tests only — no production code changed)

## Tests added

### `src/process_watcher.rs`

| Test | Covers |
| --- | --- |
| `test_stop_without_start` | `stop()` when `thread_handle` is `None` (the `if let Some(..)` is skipped) |
| `test_remove_callback` | `remove_callback` removing a registered PID and being a no-op for an unregistered one |
| `test_multiple_dead_processes` | Several watched PIDs in one watcher; each dead PID's callback fires |
| `test_stop_reports_join_error_when_thread_panics` | The `FoundationError::JoinError` branch in `stop()`, driven by a panicking callback that unwinds the watcher thread |
| `test_live_process_then_terminates` | A real child process observed alive (callback must *not* fire), then killed and reaped, after which the callback fires |

### `src/process/watcher_posix.rs` (new `tests` module)

| Test | Covers |
| --- | --- |
| `test_live_process_not_reported_dead` | A live PID (`std::process::id()`) is not reported dead |
| `test_dead_process_reported` | A nonexistent PID (`i32::MAX`) is reported dead via the `ESRCH` branch |
| `test_negative_pid_skipped` | `u32::MAX` casts to `-1` as `i32` and is skipped by the negative-PID guard (never passed to `kill(2)`) |
| `test_mixed_live_and_dead` | A mix of live and dead PIDs returns only the dead one |
| `test_empty_input` | Empty input returns an empty result |

`test_multiple_dead_processes` deduplicates the fired PIDs: the watcher polls every 100ms and
never removes a callback on its own, so a still-dead PID can fire on multiple cycles.

`test_live_process_then_terminates` reaps the child with `wait()` after `kill()` — a zombie
process still reports as alive to `kill(pid, 0)`, so the PID must be fully reaped before the
watcher sees it as dead.

## Coverage

`cargo llvm-cov --lib -- process_watcher:: process::watcher`

- `process/watcher_posix.rs` — before: 88.89% lines / 100% functions; after: **100.00% lines / 100.00% functions**.
- `process_watcher.rs` — before: 87.69% lines / 88.89% functions; after: **96.99% lines / 94.44% functions**.

## Remaining uncovered lines (intentional)

`src/process_watcher.rs`:

- `64` — the `get_mut` returning `None` for a dead PID whose callback was removed between the
  key snapshot and the lookup. This is a narrow race that cannot be triggered deterministically.
- `72` — the `?` on `Builder::spawn`, i.e. an OS thread-spawn failure, which can't be induced
  reliably in a test.
- `219` — the `other => panic!(...)` assertion arm in `test_stop_reports_join_error_when_thread_panics`,
  only reached if `stop()` returned the wrong variant (it doesn't).

These were left untested rather than forced with fabricated state.

## Status

- All 12 `process_watcher::` / `process::watcher_posix::` tests pass.
- Not committed; not pushed.
