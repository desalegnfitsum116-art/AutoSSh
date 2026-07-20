use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub device_name: String,
    pub remote_host: String,
    pub username: String,
    pub port: u16,
    pub ssh_key_path: String,
    pub auto_connect: bool,
    pub start_on_boot: bool,
    pub poll_interval_seconds: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            device_name: String::from("My Laptop"),
            remote_host: String::from("192.168.1.100"),
            username: String::from("user"),
            port: 22,
            ssh_key_path: String::from("~/.ssh/id_ed25519"),
            auto_connect: true,
            start_on_boot: false,
            poll_interval_seconds: 5,
        }
    }
}

impl Config {
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|mut p| {
            p.push("auto-ssh");
            p.push("config.toml");
            p
        })
    }

    pub fn load() -> Self {
        let path = match Self::config_path() {
            Some(p) => p,
            None => {
                log::warn!("Could not determine config directory, using defaults");
                return Self::default();
            }
        };

        match fs::read_to_string(&path) {
            Ok(contents) => match toml::from_str(&contents) {
                Ok(config) => {
                    log::info!("Loaded config from {}", path.display());
                    config
                }
                Err(e) => {
                    log::warn!(
                        "Failed to parse config at {}: {}. Using defaults.",
                        path.display(),
                        e
                    );
                    Self::default()
                }
            },
            Err(_) => {
                log::info!(
                    "No config found at {}. Using defaults.",
                    path.display()
                );
                Self::default()
            }
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let path = match Self::config_path() {
            Some(p) => p,
            None => return Err(String::from("Could not determine config directory")),
        };

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        let contents =
            toml::to_string_pretty(self).map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(&path, &contents)
            .map_err(|e| format!("Failed to write config: {}", e))?;

        log::info!("Saved config to {}", path.display());
        Ok(())
    }
}
