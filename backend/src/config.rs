use crate::types::PeerId;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeConfig {
    pub peer_id: PeerId,
    pub tcp_port: u16,
    pub udp_discovery_port: u16,
}

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Parse(toml::de::Error),
    Serialize(toml::ser::Error),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Io(err) => write!(f, "IO error: {}", err),
            ConfigError::Parse(err) => write!(f, "Parse error: {}", err),
            ConfigError::Serialize(err) => write!(f, "Serialize error: {}", err),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::Io(err) => Some(err),
            ConfigError::Parse(err) => Some(err),
            ConfigError::Serialize(err) => Some(err),
        }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::Io(err)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(err: toml::de::Error) -> Self {
        ConfigError::Parse(err)
    }
}

impl From<toml::ser::Error> for ConfigError {
    fn from(err: toml::ser::Error) -> Self {
        ConfigError::Serialize(err)
    }
}

pub fn load_or_create(file_path: &str) -> Result<NodeConfig, ConfigError> {
    let path = Path::new(file_path);

    if path.exists() {
        let content = fs::read_to_string(path)?;
        let config: NodeConfig = toml::from_str(&content)?;
        Ok(config)
    } else {
        let mut rng = rand::rng();

        let config = NodeConfig {
            peer_id: rng.random(),
            tcp_port: 2137,
            udp_discovery_port: 9000,
        };

        let toml_string = toml::to_string_pretty(&config)?;

        fs::write(path, toml_string)?;

        eprintln!("Generated new config at: {}", file_path);
        Ok(config)
    }
}
