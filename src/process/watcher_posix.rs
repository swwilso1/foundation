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
        // Sending the signal 0 to a process will check if the process is still alive.
        let result = unsafe { kill(process_id, 0) };
        if result == -1 {
            let errno = errno();
            if errno.0 == ESRCH {
                dead_processes.push(process_id);
            }
        }
    }
    Ok(dead_processes)
}
