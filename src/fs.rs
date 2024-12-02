pub mod copy;

pub use copy::copy;

cfg_if! {
    if #[cfg(target_os = "linux")] {
        pub use crate::linux_copy::async_copy as async_copy;

        mod linux_copy;
    } else if #[cfg(target_os = "macos")] {
        pub use crate::fs::macos_copy::async_copy as async_copy;

        mod macos_copy;
    }
}
