use std::fs;
use std::path::PathBuf;
use anyhow::Error;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub token: String,
    pub server_id: u64,
    pub log_channel: u64
}

impl Default for Config {
    fn default() -> Self {
        Self {
            token: "PLEASE FILL APP TOKEN".to_string(),
            server_id: 0,
            log_channel: 0,
        }
    }
}

impl Config {
    pub fn from_file(path: PathBuf) -> Result<Self, Error> {
        if path.exists() {
            Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
        }
        else {
            fs::write(path.clone(), serde_json::to_string_pretty(&Config::default())?)?;
            Err(Error::msg(format!("Created a new config file at {}. Please fill in information first", path.to_str().unwrap())))
        }
    }
}