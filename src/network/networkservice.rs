//! The `networkservice` module provides the `NetworkService` trait that defines the interface for
//! all systems on a machine that provide network services.

use crate::error::FoundationError;
use crate::network::networkconfiguration::NetworkConfiguration;
use std::collections::HashMap;
use std::path::PathBuf;

pub trait NetworkService {
    fn load_configuration(
        &mut self,
        config_map: &mut HashMap<String, NetworkConfiguration>,
    ) -> Result<(), FoundationError>;

    fn write_configuration(
        &self,
        configurations: &HashMap<String, NetworkConfiguration>,
    ) -> Result<(), FoundationError>;

    fn get_configuration_file(&self) -> PathBuf;

    fn remove_config_file(&self) -> Result<(), FoundationError> {
        match std::fs::remove_file(&self.get_configuration_file()) {
            Ok(_) => Ok(()),
            Err(e) => Err(FoundationError::IO(e)),
        }
    }

    fn start(&self) -> Result<(), FoundationError>;

    fn stop(&self) -> Result<(), FoundationError>;

    fn restart(&self) -> Result<(), FoundationError> {
        self.stop()?;
        self.start()
    }
}
