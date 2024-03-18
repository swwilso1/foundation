cfg_if! {
    if #[cfg(target_os = "linux")] {
        pub mod wireless_linux;

        pub use crate::network::wireless::wireless_linux::is_wireless_interface as is_wireless_interface;
    } else if #[cfg(target_os = "macos")] {
        pub mod wireless_macos;
        pub use crate::network::wireless::wireless_macos::is_wireless_interface as is_wireless_interface;
    }
}
