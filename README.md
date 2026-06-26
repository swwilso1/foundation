# Foundation

Foundation is a collection of lower-level utilities, and types used for making Rust applications.

--------

## Installation

Currently, Foundation is not available on crates.io, so you will need to add it to your `Cargo.toml` file manually.

```toml
foundation = { git = "https://github.com/swwilso1/foundation" }
```

Several modules in Foundation depend on the `tokio` crate. You will need to add it as a
dependency in your `Cargo.toml` file.

```toml
tokio = { version = "1", features = ["full", "fs", "io-util", "net"] }
```

Foundation will currently build using the stable version of Rust and has the most platform
support on Linux. It will also build on macOS, but some features may not be available. Windows
is not supported at this time.

## Features

- **`foundation::bytes`**: Utilities for working with capacity and size in bytes. Normalizes byte sizes into human-readable strings, with a `ByteMetricBase` to select metric (base-1000) or binary (base-1024) units.
- **`foundation::constants`**: Common constants used across the library.
- **`foundation::defer`**: Defer execution of a closure until the end of the current scope (RAII-style cleanup). Captured variables must be shareable (`Arc`, `Mutex`, etc.).
- **`foundation::delayed_handler`**: A mechanism for scheduling execution after some delay period. Handlers are registered by key and run later in a thread pool once the required data is available.
- **`foundation::dir_hasher`**: Efficiently compute the hashes of directories while avoiding recomputing the hash of the same file.
- **`foundation::error`**: Module error values (`FoundationError`).
- **`foundation::filesystem`**: Types for representing file systems (the `FileSystem` enum).
- **`foundation::filesystem_monitor`**: Monitor file system changes. A threaded, callback-based wrapper around the `notify` crate.
- **`foundation::fs`**: File-copy utilities, including synchronous `copy` and a platform-specific asynchronous `async_copy` (Linux and macOS).
- **`foundation::hash`**: Synchronous and asynchronous (tokio) BLAKE3 hashing of files and directories, with optional progress reporting and abort callbacks.
- **`foundation::interrupter`**: A mechanism for interrupting long-running tasks.
- **`foundation::keyvalueconfigfile`**: Read and write simple `key = value` configuration files.
- **`foundation::multiqueue`**: A multi-producer, multi-consumer queue that can be forked to create new queues sharing the same underlying data.
- **`foundation::network`**: Network utilities for managing network interfaces, addresses, and services. Cross-platform core types (`NetworkManager`, `NetworkConfiguration`, `NetworkInterface(s)`, `InterfaceAddr`, `DHCPRange`, the `IpAddrQuery`/`NetworkService` traits, and wireless configuration) plus Linux-only service integrations for `dhcpcd`, `dnsmasq`, `hostapd`, and `netplan`.
- **`foundation::partition`**: Types for representing disk partition tables (the `PartitionTable` enum).
- **`foundation::platformid`**: Platform identification, including processor architecture.
- **`foundation::process`** / **`foundation::process_watcher`**: Watch processes for termination, invoking a callback when a watched process exits.
- **`foundation::progressmeter`**: Progress meter for tracking progress of a task, with a notifier callback reporting percent complete.
- **`foundation::protected`**: A mechanism for protecting data from concurrent access (a wrapper over `Arc<Mutex<T>>`).
- **`foundation::result`**: Result types suitable for use with multithreaded apps (`DynResult` with `Send + Sync + 'static` errors).
- **`foundation::shell`**: Execute shell commands in a sub-process.
- **`foundation::substring`**: Simple trait to extract substrings from a string.
- **`foundation::sync`**: Synchronization primitives, including broadcast-style MPMC channels that deliver every message in order to all receivers.
- **`foundation::systemctlservice`** *(Linux only)*: Start, stop, and restart systemd services.
- **`foundation::threadcontroller`**: A mechanism for controlling standard threads (signal and shutdown).
- **`foundation::threadpool`**: A simple asynchronous thread pool.