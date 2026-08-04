use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    #[serde(default = "default_skills_dir")]
    pub skills_dir: String,
    #[serde(default = "default_p2p_identity_file")]
    pub p2p_identity_file: String,
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
            p2p_identity_file: default_p2p_identity_file(),
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

fn default_p2p_identity_file() -> String {
    dirs_next::home_dir()
        .map(|home| {
            home.join(".config")
                .join("sovereign-sync")
                .join("p2p-identity.json")
                .to_string_lossy()
                .into_owned()
        })
        .unwrap_or_else(|| "p2p-identity.json".into())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_no_voter_or_quorum_surface() {
        let rendered = toml::to_string_pretty(&SovereignConfig::default()).unwrap();
        assert!(!rendered.contains("[kbd]"));
        assert!(!rendered.contains("voter"));
        assert!(!rendered.contains("quorum"));
    }
}
