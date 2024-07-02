#[macro_use]
extern crate cfg_if;

#[macro_use]
extern crate lazy_static;

extern crate num_cpus;

pub mod constants;
pub mod defer;
pub mod delayed_handler;
pub mod error;
pub mod filesystem;
pub mod filesystem_monitor;
pub mod hash;
pub mod keyvalueconfigfile;
pub mod multiqueue;
pub mod network;
pub mod partition;
pub mod platformid;
pub mod progressmeter;
pub mod protected;
pub mod result;
pub mod shell;
pub mod substring;
pub mod sync;
pub mod threadcontroller;
pub mod threadpool;

cfg_if! {
    if #[cfg(target_os = "linux")] {
        pub mod systemctlservice;
    }
}
