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
support on Linux. It will also buildon macOS, but some features may not be available. Windows
is not supported at this time.

## Features

- **`foundation::bytes`**: Utilities for working with capacity and size in bytes.
- **`foundation::constants`**: Common constants.
- **`foundation::defer`**: Defer execution of a closure until the end of the current scope.
- **`foundation::delayed_handler`**: A mechanism for scheduling execution after some delay period.
- **`foundation::error`**: Module error values.
- **`foundation::filesystem`**: Types for representing file systems.
- **`foundation::filesystem_monitor`**: Monitor file system changes.
- **`foundation::hash`**: Utilities for hashing file and directory contents.
- **`foundation::interrupter`**: A mechanism for interrupting long-running tasks.
- **`foundation::keyvalueconfigfile`**: Read and write key-value configuration files.
- **`foundation::multiqueue`**: A multi-producer, multi-consumer queue.
- **`foundation::network`**: Network utilities for managing network interfaces.
- **`foundation::partition`**: Types for representing disk partitions.
- **`foundation::platformid`**: Platform identification.
- **`foundation::progressmeter`**: Progress meter for tracking progress of a task.
- **`foundation::protected`**: A mechanism for protecting data from concurrent access.
- **`foundation::result`**: Result types suitable for use with multithreaded apps.
- **`foundation::shell`**: Execute shell commands.
- **`foundation::substring`**: Simple trait to extract substrings from a string.
- **`foundation::sync`**: Synchronization primitives.
- **`foundation::threadcontroller`**: A mechanism for controlling standard threads.
- **`foundation::threadpool`**: A simple thread pool.