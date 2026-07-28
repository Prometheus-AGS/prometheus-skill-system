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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KbdVoterConfig {
    pub id: u64,
    #[serde(default)]
    pub endpoint: String,
    #[serde(default)]
    pub witness: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbdConfig {
    #[serde(default = "default_kbd_node_id")]
    pub node_id: u64,
    #[serde(default = "default_kbd_voters")]
    pub voters: Vec<KbdVoterConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SovereignConfig {
    #[serde(default)]
    pub node: NodeConfig,
    #[serde(default)]
    pub peers: PeersConfig,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub kbd: KbdConfig,
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

impl Default for KbdConfig {
    fn default() -> Self {
        Self {
            node_id: default_kbd_node_id(),
            voters: default_kbd_voters(),
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

fn default_kbd_node_id() -> u64 {
    1
}

fn default_kbd_voters() -> Vec<KbdVoterConfig> {
    vec![KbdVoterConfig {
        id: default_kbd_node_id(),
        endpoint: "local".into(),
        witness: false,
    }]
}

impl KbdConfig {
    pub fn quorum_policy(&self) -> anyhow::Result<crate::kbd_raft::QuorumPolicy> {
        let policy = crate::kbd_raft::QuorumPolicy::new(
            self.node_id,
            self.voters.iter().map(|voter| voter.id),
        )?;
        if policy.voters().len() != self.voters.len() {
            anyhow::bail!("kbd.voters contains duplicate voter ids");
        }
        Ok(policy)
    }
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
