use crate::state::PeerIdType;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeConfig {
    pub peer_id: PeerIdType,
    pub tcp_port: u16,
    pub udp_discovery_port: u16,
}

pub fn load_or_create(file_path: &str) -> Result<NodeConfig, Box<dyn std::error::Error>> {
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
