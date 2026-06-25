# Change Summary: multiqueue module test updates

**Branch:** `claude/test-updates`
**File touched:** `src/multiqueue.rs` (tests only — no production code changed)

## Tests added

| Test | Covers |
| --- | --- |
| `test_error_display` | `Display` for both `MultiQueueError::Push` and `::Fork` |
| `test_error_debug` | `Debug` impl (hides the contained object) |
| `test_error_is_std_error` | `MultiQueueError` usable as a boxed `std::error::Error` |
| `test_error_equality_and_copy` | `PartialEq`/`Eq`/`Copy` derives on the error |
| `test_core_shared_size_directly` | `Core::shared_size` empty/single-element branch unreachable via `MultiQueue` |
| `test_operations_on_fresh_queue` | Early `None`/no-op returns in `front`, `front_mut`, `pop_front` on an empty core |
| `test_front_mut_with_fork_at_end_of_queue` | `front_mut` when a fork is parked at the end of the queue (both the `None` and new-element paths) |
| `test_drop_with_remaining_elements` | `Drop` popping remaining elements and decrementing the final block's reference count |
| `test_poisoned_core_error_paths` | Lock-failure branches in `empty`, `size`, `shared_size`, `references`, `front`, `front_mut`, `pop_front`, `push_back`, `fork`, and `Drop` |
| `test_iterator_on_empty_queue` | `MultiQueueIterator::next` returning `None` immediately |

The poison helper poisons the core mutex on the current thread (via `catch_unwind` while
holding the guard — `Core` is not `Send`, so a second thread can't be used) to drive every
`Err(_)` lock-handling branch in the public API.

## Coverage

- Before: 96.88% functions / 92.75% lines (`cargo llvm-cov --lib test multiqueue::`).
- After: **100.00% functions / 97.13% lines**.

## Remaining uncovered lines (intentional)

`168–172`, `190–197`, `380`, `540` are defensive guards in the unsafe pointer code that
normal operation cannot reach:

- `168–172` / `190–197` (`Core::update`): a block only reaches reference count 0 after every
  fork has passed it, and forks pass earlier blocks first, so an earlier block always hits 0
  first — a zero-count block can never sit *behind* a non-zero one, nor can the tail be nulled
  while multiple live blocks remain. (The source comment already flags this as uncertain.)
- `380` / `540` (`front_mut` / `size`): the `head == null` while `at_end_of_queue == true`
  case, which is impossible because `at_end_of_queue` is only set in `pop_front` after `head`
  is made non-null.

These were left untested rather than forced with fabricated internal state.

## Status

- All 31 `multiqueue::` tests pass (`cargo test --lib multiqueue::`).
- Not committed; not pushed.
