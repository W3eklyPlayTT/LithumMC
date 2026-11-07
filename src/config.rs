use serde::{Deserialize, Serialize};
use std::{error::Error, fs, path::Path};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub metrics: MetricsConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
    pub motd: String,
    pub max_players: u32,
    pub max_connections: usize,
    pub worker_threads: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MetricsConfig {
    pub refresh_interval_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                address: "0.0.0.0".into(),
                port: 25565,
                motd: "A Lithum server".into(),
                max_players: 20,
                max_connections: 256,
                worker_threads: 0,
            },
            metrics: MetricsConfig {
                refresh_interval_ms: 1000,
            },
        }
    }
}

impl Config {
    pub fn load_or_create() -> Result<Self, Box<dyn Error>> {
        let path = Path::new("lithum.toml");
        if path.exists() {
            return Ok(toml::from_str(&fs::read_to_string(path)?)?);
        }
        let config = Self::default();
        fs::write(path, toml::to_string_pretty(&config)?)?;
        println!("Created {} with default settings", path.display());
        Ok(config)
    }
}
