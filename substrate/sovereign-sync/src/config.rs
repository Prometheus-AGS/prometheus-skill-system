use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    #[serde(default = "default_skills_dir")]
    pub skills_dir: String,
    #[serde(default)]
    pub operator_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PeersConfig {
    #[serde(default)]
    pub bootstrap: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SovereignConfig {
    #[serde(default)]
    pub node: NodeConfig,
    #[serde(default)]
    pub peers: PeersConfig,
    #[serde(default)]
    pub server: ServerConfig,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            skills_dir: default_skills_dir(),
            operator_id: String::new(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
        }
    }
}

fn default_skills_dir() -> String {
    dirs_next::home_dir()
        .map(|h| {
            h.join(".claude")
                .join("skills")
                .to_string_lossy()
                .into_owned()
        })
        .unwrap_or_else(|| "~/.claude/skills".to_string())
}

fn default_port() -> u16 {
    7892
}

impl SovereignConfig {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn default_path() -> PathBuf {
        dirs_next::home_dir()
            .map(|h| h.join(".config").join("sovereign-sync").join("config.toml"))
            .unwrap_or_else(|| PathBuf::from("config.toml"))
    }

    pub fn write_default(path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let default = Self::default();
        let content = toml::to_string_pretty(&default)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
