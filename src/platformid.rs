//! The `platformid` module contains code that provides information about the platform on which the
//! application is running.

use std::path::PathBuf;
use versions::SemVer;

/// The `ProcessorArchitecture` enum represents the processor architecture of the platform.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ProcessorArchitecture {
    X86,
    X86_64,
    ARM,
    ARM64,
}

cfg_if! {
    if #[cfg(target_os = "linux")] {
        use crate::shell::Shell;
        use std::env;

        /// Find the path to the requested binary using the PATH environment variable.
        ///
        /// # Arguments
        ///
        /// * `binary` - The name of the binary to find.
        ///
        /// # Returns
        ///
        /// An `Option` containing the path to the binary if found, or `None` if the binary was not found.
        fn find_path_to_binary(binary: &str) -> Option<PathBuf> {
            env::var("PATH").ok().and_then(|paths| {
                env::split_paths(&paths)
                    .map(|path| path.join(binary))
                    .find(|path| path.is_file())
            })
        }

        /// Get the vendor and version of the platform.
        ///
        /// # Returns
        ///
        /// A tuple containing the vendor and version of the platform.
        fn get_vendor_version() -> (String, SemVer) {
                let rpm_path = find_path_to_binary("rpm");
                let lsb_release_path = find_path_to_binary("lsb_release");

                if let Some(rpm) = rpm_path {
                    let centos_query_result = Shell::execute(rpm.to_str().unwrap(), vec!["-q".to_string(), "centos-release".to_string()]);
                    let fedora_query_result = Shell::execute(rpm.to_str().unwrap(), vec!["-q".to_string(), "fedora-release".to_string()]);
                    let sles_query_result = Shell::execute(rpm.to_str().unwrap(), vec!["-q".to_string(), "sles-release".to_string()]);
                    let system_release_query_result = Shell::execute(rpm.to_str().unwrap(), vec!["-q".to_string(), "system-release".to_string()]);

                    let mut vendor = String::new();
                    let mut release_string = String::new();


                    let release_helper_strings = vec![
                        "redhat-release".to_string(),
                        "redhat-release-server".to_string(),
                        "redhat-release-client".to_string(),
                        "redhat-release-computenode".to_string(),
                        "redhat-release-workstation".to_string(),
                    ];

                    for helper in release_helper_strings {
                        let result = Shell::execute_command(rpm.to_str().unwrap(), vec!["-q".to_string(), helper.clone()]);
                        if result.is_ok() {
                            vendor = "RedHat".to_string();
                            release_string = helper;
                        }
                    }

                    if let (Some(output), _) = centos_query_result {
                        if !output.contains("not installed") {
                            vendor = "CentOS".to_string();
                            release_string = "centos-release".to_string();
                        }
                    }

                    if let (Some(output), _) = fedora_query_result {
                        if !output.contains("not installed") {
                            vendor = "Fedora".to_string();
                            release_string = "fedora-release".to_string();
                        }
                    }

                    if let (Some(output), _) = sles_query_result {
                        if !output.contains("not installed") {
                            vendor = "Suse".to_string();
                            release_string = "sles-release".to_string();
                        }
                    }

                    if let (Some(_output), _) = system_release_query_result {
                        let system_query_result = Shell::execute(&rpm.to_string_lossy(), vec![
                            "-q".to_string(),
                            "--qf".to_string(),
                            "\"%{VENDOR}\"".to_string(),
                            "system-release".to_string()]);
                        if let (Some(output), _) = system_query_result {
                            if output.contains("Amazon") {
                                vendor = "Amazon".to_string();
                                release_string = "system-release".to_string();
                            }
                        }
                    }

                    let major_version = Shell::execute(&rpm.to_string_lossy(), vec![
                        "-q".to_string(),
                        "--qf".to_string(),
                        "\"%{VERSION}\"".to_string(),
                        release_string.clone()]);

                    let minor_version = Shell::execute(&rpm.to_string_lossy(), vec![
                        "-q".to_string(),
                        "--qf".to_string(),
                        "\"%{RELEASE}\"".to_string(),
                        release_string]);

                    let mut version_string = if let (Some(output), _) = major_version {
                        format!("{}.", output)
                    } else {
                        "0".to_string()
                    };

                    let minor = if let (Some(output), _) = minor_version {
                        output
                    } else {
                        "0".to_string()
                    };

                    version_string = format!("{}.{}", version_string, minor);

                    (vendor, SemVer::new(&version_string).unwrap())
                } else if let Some(lsb_release) = lsb_release_path {
                    let distribution_result = Shell::execute(&lsb_release.to_string_lossy(), vec!["-i".to_string()]);
                    let release_result = Shell::execute(&lsb_release.to_string_lossy(), vec!["-r".to_string()]);

                    let vendor = if let (Some(output), _) = distribution_result {
                        let parts = output.split(':').collect::<Vec<&str>>();
                        if parts.len() > 1 {
                            if parts[1].contains("Ubuntu") {
                                "Ubuntu".to_string()
                            } else if parts[1].contains("Debian") {
                                "Debian".to_string()
                            } else if parts[1].contains("Pop") {
                                "Pop".to_string()
                            } else if parts[1].contains("Raspbian") {
                                "Raspbian".to_string()
                            } else if parts[1].contains("Mint") {
                                "Mint".to_string()
                            } else if parts[1].contains("Kali") {
                                "Kali".to_string()
                            } else {
                                "Unknown".to_string()
                            }
                        } else {
                            "Unknown".to_string()
                        }
                    } else {
                        "Unknown".to_string()
                    };

                    let version = if let (Some(output), _) = release_result {
                        let parts = output.split(':').collect::<Vec<&str>>();
                        if parts.len() > 1 {
                            let version_string = parts[1].trim();

                            let s = match vendor.as_str() {
                                "Ubuntu" => format!("{}.0", version_string),
                                _ => version_string.to_string(),
                            };

                            if let Some(v) = SemVer::new(&s) {
                                v
                            } else {
                                SemVer::new("0.0.0").unwrap()
                            }
                        } else {
                            SemVer::new("0.0.0").unwrap()
                        }
                    } else {
                        SemVer::new("0.0.0").unwrap()
                    };

                    (vendor, version)
                } else {
                    ("Unknown".to_string(), SemVer::new("0.0.0").unwrap())
            }
        }
    }
}

lazy_static! {
    static ref NAME: String = {
        cfg_if! {
            if #[cfg(target_os = "linux")] {
                "Linux".to_string()
            } else if #[cfg(target_os = "macos")] {
                "macOS".to_string()
            } else if #[cfg(target_os = "windows")] {
                "Windows".to_string()
            } else if #[cfg(target_os = "freebsd")] {
                "FreeBSD".to_string()
            } else {
                "Unknown".to_string()
            }
        }
    };

    static ref VENDOR: String = {
        cfg_if! {
            if #[cfg(target_os = "linux")] {
                get_vendor_version().0
            } else if #[cfg(target_os = "macos")] {
                "Apple".to_string()
            } else if #[cfg(target_os = "windows")] {
                "Microsoft".to_string()
            } else if #[cfg(target_os = "freebsd")] {
                "FreeBSD".to_string()
            } else {
                "Unknown".to_string()
            }
        }
    };

    static ref VERSION: SemVer = {
        cfg_if! {
            if #[cfg(target_os = "linux")] {
                get_vendor_version().1
            } else {
                SemVer::new("0.0.0").unwrap()
            }
        }
    };

    static ref NUMBER_OF_PROCESSORS: usize = {
        num_cpus::get()
    };

    static ref PROCESSOR_ARCHITECTURE: ProcessorArchitecture = {
        cfg_if ! {
            if #[cfg(target_arch = "x86")] {
                ProcessorArchitecture::X86
            } else if #[cfg(target_arch = "x86_64")] {
                ProcessorArchitecture::X86_64
            } else if #[cfg(target_arch = "arm")] {
                ProcessorArchitecture::ARM
            } else if #[cfg(target_arch = "aarch64")] {
                ProcessorArchitecture::ARM64
            } else {
                // Default to x86_64
                ProcessorArchitecture::X86_64
            }
        }
    };
}

/// The `PlatformId` struct represents the platform on which the application is running.
pub struct PlatformId {
    /// The name of the platform.
    pub name: String,

    /// The vendor of the platform.
    pub vendor: String,

    /// The version number of the platform.
    pub version: SemVer,

    /// The number of processors on the platform.
    pub number_of_processors: usize,

    /// The processor architecture of the platform.
    pub processor_architecture: ProcessorArchitecture,
}

impl PlatformId {
    /// Create a new `PlatformId` instance.
    pub fn new() -> PlatformId {
        PlatformId {
            name: NAME.to_string(),
            vendor: VENDOR.to_string(),
            version: VERSION.to_owned(),
            number_of_processors: NUMBER_OF_PROCESSORS.to_owned(),
            processor_architecture: PROCESSOR_ARCHITECTURE.to_owned(),
        }
    }
}

// Testing code that is disabled for now.
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use env_logger::Builder;
//     use log::LevelFilter;
//     use std::io::Write;
//
//     #[test]
//     fn test_platform_id() {
//         let target =
//             Box::new(std::fs::File::create("platformid.log").expect("Could not create file"));
//         let log_level = LevelFilter::Info;
//
//         let mut builder = Builder::new();
//         builder
//             .target(env_logger::Target::Pipe(target))
//             .format(|buf, record| {
//                 writeln!(
//                     buf,
//                     "{}[{}] - {:<5} - {}",
//                     chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
//                     record.level(),
//                     record.target(),
//                     record.args(),
//                 )
//             });
//         builder.filter(None, log_level);
//         builder.parse_filters(&std::env::var("RUST_LOG").unwrap_or_else(|_| "my_app=info".into()));
//         builder.init();
//
//         let platform_id = PlatformId::new();
//         cfg_if! {
//             if #[cfg(target_os = "linux")] {
//                 assert_eq!(platform_id.name, "Linux");
//                 assert_eq!(platform_id.vendor, "Ubuntu");
//                 assert_eq!(platform_id.version.major, 23);
//                 assert_eq!(platform_id.version.minor, 10);
//                 assert_eq!(platform_id.version.patch, 0);
//                 assert_eq!(platform_id.number_of_processors, 4);
//                 assert_eq!(platform_id.processor_architecture, ProcessorArchitecture::X86_64);
//             }
//         }
//     }
// }
