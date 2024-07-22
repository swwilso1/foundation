cfg_if! {
    if #[cfg(target_os = "linux")] {
        mod watcher_posix;
        pub use watcher_posix::watch_processes_for_termination;
    } else if #[cfg(target_os = "macos")] {
        mod watcher_posix;
        pub use watcher_posix::watch_processes_for_termination;
    }
}
