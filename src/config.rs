use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::read_to_string;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub global: Option<ConfigRecord>,
    pub ip: Option<ConfigRecord>,
    pub route: Option<HashMap<String, ConfigRecord>>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Config> {
        let file = read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&file)?;
        Ok(config)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigRecord {
    pub max_requests: usize,
    pub time_window: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_config() {
        let config = read_to_string("./tests/config.yaml").unwrap();
        let _: Config = serde_yaml::from_str(&config).unwrap();
    }
}
