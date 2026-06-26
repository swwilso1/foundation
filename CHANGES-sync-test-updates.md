# Change Summary: sync module test updates

**Branch:** `claude/test-updates`
**Files touched:** `src/sync/error.rs`, `src/sync/mpmc/sender.rs`, `src/sync/mpmc/receiver.rs`
(tests only — no production code changed)

## Tests added

### `src/sync/error.rs` (new `tests` module)

| Test | Covers |
| --- | --- |
| `test_display` | `Display` for `SendError` ("channel closed", value never revealed) |
| `test_debug` | `Debug` impl hiding the contained value via `finish_non_exhaustive` |
| `test_is_std_error` | `SendError` usable as a boxed `std::error::Error` |
| `test_equality_and_copy` | `PartialEq`/`Eq`/`Clone`/`Copy` derives |

### `src/sync/mpmc/sender.rs` (new `tests` module)

| Test | Covers |
| --- | --- |
| `test_clone_shares_channel` | `Sender::clone` shares the channel; cloned + original both send, sender count increments to 2 |
| `test_clone_with_poisoned_channel` | `increment_senders` lock-error branch and `clone` still returning a `Sender` |
| `test_send_on_poisoned_channel_errors` | `get_send_space` lock-error path and `send` returning `Err(SendError(thing))` with the original message |
| `test_drop_on_poisoned_channel` | `Drop` lock-error branch (`decrement_senders` cannot run) |

### `src/sync/mpmc/receiver.rs` (new `tests` module)

| Test | Covers |
| --- | --- |
| `test_recv_on_poisoned_empty_channel` | `get_something_to_receive` lock-error path on an empty queue → `recv` returns `None` |
| `test_recv_on_poisoned_channel_with_message` | lock-error path when a message *is* queued → `recv` returns `None` |
| `test_subscribe_on_poisoned_channel_panics` | `Receiver::new` "Failed to lock channel" panic when the shared channel is poisoned |

The lock-error tests use a `poison_channel` helper that poisons the channel mutex on the
current thread (via `catch_unwind` while holding the guard), mirroring the existing
`poison_queue` helper in `src/multiqueue.rs`.

## Coverage (`cargo llvm-cov --lib test 'sync::'`)

| File | Before | After |
| --- | --- | --- |
| `sync/error.rs` | 0.00% lines / 0.00% fns | **100.00% / 100.00%** |
| `sync/mpmc/receiver.rs` | 85.96% lines / 100.00% fns | **94.57% / 100.00%** |
| `sync/mpmc/sender.rs` | 78.12% lines / 90.00% fns | **98.20% / 100.00%** |
| `sync/mpmc/channel.rs` | 92.16% / 100.00% | unchanged |
| `sync/mpmc/unbounded.rs` | 100% | unchanged |

Every function in the module is now covered (100% function coverage).

## Remaining uncovered lines (intentional)

These are defensive branches that normal operation cannot reach and that cannot be forced
without fabricating internal state:

- **`channel.rs` 59–64** — the `push_back` error path in `Channel::send`. `MultiQueue::push_back`
  only errors when its internal `Core` mutex is poisoned, and that mutex is private to the
  `multiqueue` module (the channel only exposes the `MultiQueue`, not its `Core`), so it cannot
  be poisoned from the `sync` tests. Line 64 is an explicit `panic!` for a `MultiQueueError`
  variant `push_back` never returns.
- **`receiver.rs` 40–41** — the "Failed to fork queue" panic in `Receiver::new`. Requires the
  channel lock to succeed *and* the inner `MultiQueue` core fork to fail, i.e. the same
  unreachable inner-core poison as above. (Line 45's "Failed to lock channel" branch *is* now
  covered.)
- **`receiver.rs` 118, 130, 136** — the lock-error branches in `recv` after
  `get_something_to_receive` has already succeeded, plus the `front() == None` arm when the queue
  reported a non-zero size. The channel uses a single mutex, so a lock that just succeeded inside
  `get_something_to_receive` cannot then fail on the next line, and a non-empty queue always
  yields a `front`.
- **`sender.rs` 138–139** — the inner `channel.lock()` error branch in `send`, unreachable for the
  same single-mutex reason: `get_send_space` would have already returned the lock error first.

These were left untested rather than forced with fabricated internal state.

## Status

- All 17 `sync::` tests pass (`cargo test --lib 'sync::'`): 4 error + 4 sender + 3 receiver
  (new) plus the 6 pre-existing channel/bounded/unbounded tests.
- Tests only; no production code changed. Not committed; not pushed.
