use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchConfig {
    pub server: ServerConfig,
    pub jobs: JobsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub surface_bridge_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobsConfig {
    pub job_dir: PathBuf,
}

impl Default for ResearchConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                port: 7891,
                surface_bridge_url: "http://127.0.0.1:7890".to_string(),
            },
            jobs: JobsConfig {
                job_dir: dirs_next::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".research-jobs"),
            },
        }
    }
}

impl ResearchConfig {
    pub fn default_path() -> PathBuf {
        dirs_next::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("prometheus-research")
            .join("config.toml")
    }

    pub fn load(path: &PathBuf) -> anyhow::Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)?;
        let cfg: Self = toml::from_str(&content)?;
        Ok(cfg)
    }

    pub fn job_dir(&self) -> &PathBuf {
        &self.jobs.job_dir
    }

    pub fn surface_bridge_url(&self) -> &str {
        &self.server.surface_bridge_url
    }
}
