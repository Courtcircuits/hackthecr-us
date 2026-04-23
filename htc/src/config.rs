use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{cert::generate_ed25519_pem};

use clap::ValueEnum;

const DEFAULT_API: &str = "https://api.hackthecrous.com";

#[derive(Debug, Clone, PartialEq, Eq, Hash, ValueEnum)]
pub enum OutputFormat {
    Yaml,
    KubernetesSecret,
}



#[derive(Serialize, Deserialize)]
pub struct Config {
    pub server: String,
    pub client_key_data: String,
    pub public_key_data: String,
    pub user: String,
    pub schedule: Option<CronConfig>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CronConfig {
    pub restaurants: Option<EntityScheduleConfig>,
    pub schools: Option<EntityScheduleConfig>,
    pub meals: Option<EntityScheduleConfig>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EntityScheduleConfig {
    pub schedule: String,
    pub target: Vec<String>,
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
    #[error("Unknown region: {0}")]
    UnknownRegion(String),
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

    pub fn generate<'a>(user: &'a str, server_name: Option<&str>, schedule: Option<CronConfig>) -> Result<Self, ConfigError<'a>> {
        let certificates =
            generate_ed25519_pem().map_err(|e| ConfigError::CertificateGenFailed(e.to_string()))?;

        Ok(Config {
            server: server_name.unwrap_or(DEFAULT_API).to_string(),
            client_key_data: certificates.private_key,
            public_key_data: certificates.certificate,
            user: user.to_string(),
            schedule,
        })
    }
    
    pub fn format(&self, output_format: OutputFormat, secret_name: Option<&str>) -> Result<String, ConfigError<'_>> {
        match output_format {
            OutputFormat::Yaml => self.as_yaml(),
            OutputFormat::KubernetesSecret => {
                let secret_name = secret_name.ok_or_else(|| ConfigError::UnknownRegion("Secret name is required for Kubernetes Secret format".to_string()))?;
                self.as_kubernetes_secret(secret_name)
            }
        }
    }

    pub fn write(&self, path: &PathBuf, output_format: OutputFormat, secret_name: Option<&str>) -> Result<(), ConfigError<'_>> {
        let content = self.format(output_format, secret_name)?;
        std::fs::write(path, content).map_err(|e| ConfigError::WriteUnable(path.to_str().unwrap_or("unknown path").to_string(), e.to_string()))
    }
    
    pub fn print(&self, output_format: OutputFormat, secret_name: Option<&str>) -> Result<(), ConfigError<'_>> {
        let content = self.format(output_format, secret_name)?;
        println!("{}", content);
        Ok(())
    }

    pub fn as_yaml(&self) -> Result<String, ConfigError<'_>> {
        let yaml =
            serde_yaml::to_string(self).map_err(|e| ConfigError::InvalidYAML(e.to_string()))?;
        Ok(yaml)
    }
    
    pub fn as_kubernetes_secret(&self, secret_name: &str) -> Result<String, ConfigError<'_>> {
        let config = serde_yaml::to_string(self).map_err(|e| ConfigError::InvalidYAML(e.to_string()))?;
        let indented_config = config
            .lines()
            .map(|line| format!("    {line}"))
            .collect::<Vec<_>>()
            .join("\n");
        let secret = include_str!("k8s_secret_template.yaml")
            .replace("{{config}}", &indented_config)
            .replace("{{secret_name}}", secret_name);

        Ok(secret)
    }
}
