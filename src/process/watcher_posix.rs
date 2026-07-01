use crate::error::FoundationError;
use crate::process_watcher::ProcessId;
use errno::errno;
use libc::{kill, ESRCH};

/// Watch a list of processes for termination.
///
/// # Arguments
///
/// * `processes` - A list of process IDs to watch.
///
/// # Returns
///
/// A list of process IDs that have terminated.
pub fn watch_processes_for_termination(
    processes: Vec<ProcessId>,
) -> Result<Vec<ProcessId>, FoundationError> {
    let mut dead_processes: Vec<ProcessId> = Vec::new();
    for process_id in processes {
        let pid = process_id as i32;

        if pid < 0 {
            continue;
        }

        // Sending the signal 0 to a process will check if the process is still alive.
        let result = unsafe { kill(pid, 0) };
        if result == -1 {
            let errno = errno();
            if errno.0 == ESRCH {
                dead_processes.push(process_id);
            }
        }
    }
    Ok(dead_processes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_live_process_not_reported_dead() {
        // Our own process is guaranteed to be alive, so it must not be reported as dead.
        let dead = watch_processes_for_termination(vec![std::process::id()]).unwrap();
        assert!(dead.is_empty());
    }

    #[test]
    fn test_dead_process_reported() {
        // 2147483647 (i32::MAX) is extremely unlikely to be a live PID.
        let dead = watch_processes_for_termination(vec![2147483647]).unwrap();
        assert_eq!(dead, vec![2147483647]);
    }

    #[test]
    fn test_negative_pid_skipped() {
        // u32::MAX casts to -1 as an i32, which the negative-PID guard must skip rather than
        // pass to kill(2) (where -1 would signal the whole process group).
        let dead = watch_processes_for_termination(vec![u32::MAX]).unwrap();
        assert!(dead.is_empty());
    }

    #[test]
    fn test_mixed_live_and_dead() {
        let dead = watch_processes_for_termination(vec![std::process::id(), 2147483647]).unwrap();
        assert_eq!(dead, vec![2147483647]);
    }

    #[test]
    fn test_empty_input() {
        let dead = watch_processes_for_termination(vec![]).unwrap();
        assert!(dead.is_empty());
    }
}
