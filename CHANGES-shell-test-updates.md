# Change Summary: shell module test updates

**Branch:** `claude/test-updates`
**Files touched:** `src/shell.rs` (tests only — no production code changed)

## Tests added

### `src/shell.rs` (new `tests` module)

| Test | Covers |
| --- | --- |
| `test_execute_command_success` | `execute_command` happy path: `echo hello` succeeds and stdout contains the argument |
| `test_execute_command_passes_arguments` | Multiple arguments are all forwarded to the spawned process |
| `test_execute_command_nonexistent_returns_error` | The `Err(FoundationError::IO(_))` branch when a bogus binary cannot be spawned (non-Windows); on Windows `cmd /C` spawns successfully |
| `test_execute_success_returns_stdout_and_stderr` | `execute` success branch returning `(Some(stdout), Some(stderr))` |
| `test_execute_nonexistent_command_returns_none_none` | `execute` returning `(None, None)` when `execute_command` errors |
| `test_execute_failing_command_returns_none_stdout_some_stderr` | `execute` returning `(None, Some(stderr))` for a command that exits non-zero (`false`) |
| `test_execute_command_failure_captures_stderr` | Non-zero exit (`ls` of a missing path) produces non-empty stderr |
| `test_spawn_command_success` | `spawn_command` returns a waitable `Child` that exits successfully |
| `test_spawn_command_with_arguments` | Arguments are forwarded; a spawned `sh -c 'exit 3'` reports exit code 3 |
| `test_spawn_command_nonexistent_returns_error` | The `Err(FoundationError::IO(_))` branch for `spawn_command` (non-Windows) |

## Coverage

`cargo llvm-cov --lib -- shell::`

- `shell.rs` — before: 0% (no tests); after: **83.33% lines / 89.47% functions** on darwin.

## Remaining uncovered lines (intentional)

All uncovered lines are platform-gated `cfg!(target_os = "windows")` branches that cannot run
on this host:

- `26–30`, `73–77` — the production `cmd /C` Windows code paths in `execute_command` and
  `spawn_command`.
- `127`, `152`, `163`, `177`, `197`, `204–208` — the Windows-only assertion arms inside the
  test module's `if cfg!(target_os = "windows")` guards.

These would be exercised on a Windows runner; nothing was forced with fabricated state.

## Status

- All 10 `shell::` tests pass.
- Not committed; not pushed.
