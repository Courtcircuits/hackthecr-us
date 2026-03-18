use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::actions::config_gen::generate_ed25519_pem;

const DEFAULT_API: &str = "https://api.hackthecrous.com";

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub server: String,
    pub client_key_data: String,
    pub public_key_data: String,
    pub user: String,
    pub schedule: Option<CronConfig>,
}

#[derive(Serialize, Deserialize)]
pub struct CronConfig {
    pub restaurants: Option<String>,
    pub schools: Option<String>,
    pub meals: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError<'a> {
    #[error("Couldn't find the config file at {0}")]
    NotFound(&'a PathBuf),
    #[error("Couldn't parse the config file, invalid YAML : {0}")]
    InvalidYAML(String),
    #[error("Empty file")]
    EmptyFile,
    #[error("Couldn't write file {0} : {1}")]
    WriteUnable(String, String),
    #[error("Certificate generation failed : {0}")]
    CertificateGenFailed(String),
}

impl Config {
    pub fn from(path: &PathBuf) -> Result<Self, ConfigError<'_>> {
        let config = std::fs::read_to_string(path).map_err(|_| ConfigError::NotFound(path))?;
        if config.is_empty() {
            return Err(ConfigError::EmptyFile);
        }
        let deserialized_config: Config =
            serde_yaml::from_str(&config).map_err(|e| ConfigError::InvalidYAML(e.to_string()))?;
        Ok(Config {
            server: deserialized_config.server,
            client_key_data: deserialized_config.client_key_data,
            public_key_data: deserialized_config.public_key_data,
            user: deserialized_config.user,
            schedule: deserialized_config.schedule,
        })
    }

    pub fn generate<'a>(user: &'a str) -> Result<Self, ConfigError<'a>> {
        let certificates =
            generate_ed25519_pem().map_err(|e| ConfigError::CertificateGenFailed(e.to_string()))?;

        Ok(Config {
            server: DEFAULT_API.to_string(),
            client_key_data: certificates.private_key,
            public_key_data: certificates.certificate,
            user: user.to_string(),
            schedule: None,
        })
    }

    pub fn write(&self, path: &PathBuf) -> Result<(), ConfigError<'_>> {
        std::fs::write(path, self.as_yaml()?).map_err(|e| {
            ConfigError::WriteUnable(path.to_string_lossy().to_string(), e.to_string())
        })
    }

    pub fn as_yaml(&self) -> Result<String, ConfigError<'_>> {
        let yaml =
            serde_yaml::to_string(self).map_err(|e| ConfigError::InvalidYAML(e.to_string()))?;

        Ok(yaml)
    }
}
