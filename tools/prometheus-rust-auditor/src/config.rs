use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const DEFAULT_TOML: &str = include_str!("../default-config.toml");

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuditorConfig {
    #[serde(default)]
    pub workspace: WorkspaceConfig,
    #[serde(default)]
    pub invariants: InvariantsConfig,
    #[serde(default)]
    pub clippy: ClippyConfig,
}

impl Default for AuditorConfig {
    fn default() -> Self {
        toml::from_str(DEFAULT_TOML).expect("default-config.toml is valid TOML")
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkspaceConfig {
    #[serde(default = "default_dot")]
    pub path: PathBuf,
    #[serde(default)]
    pub partitions: Vec<PartitionDef>,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self { path: PathBuf::from("."), partitions: vec![] }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PartitionDef {
    pub name: String,
    pub crates: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct InvariantsConfig {
    #[serde(default)]
    pub actor_no_shared_mutable_state: bool,
    #[serde(default)]
    pub wasm_unsafe_confined: bool,
    #[serde(default)]
    pub async_cancellation_safe: bool,
    #[serde(default)]
    pub zero_copy_preference: bool,
    #[serde(default)]
    pub no_platform_coupling: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClippyConfig {
    #[serde(default = "default_true")]
    pub workspace_lints: bool,
    #[serde(default = "default_warn")]
    pub pedantic: String,
    #[serde(default = "default_warn")]
    pub nursery: String,
    #[serde(default = "default_warn")]
    pub cargo: String,
    #[serde(default = "default_deny")]
    pub unwrap_used: String,
    #[serde(default = "default_warn")]
    pub expect_used: String,
    #[serde(default = "default_deny")]
    pub panic: String,
    #[serde(default = "default_deny")]
    pub redundant_clone: String,
    #[serde(default = "default_deny")]
    pub await_holding_lock: String,
}

impl Default for ClippyConfig {
    fn default() -> Self {
        Self {
            workspace_lints: true,
            pedantic: "warn".into(),
            nursery: "warn".into(),
            cargo: "warn".into(),
            unwrap_used: "deny".into(),
            expect_used: "warn".into(),
            panic: "deny".into(),
            redundant_clone: "deny".into(),
            await_holding_lock: "deny".into(),
        }
    }
}

fn default_dot() -> PathBuf { PathBuf::from(".") }
fn default_true() -> bool { true }
fn default_warn() -> String { "warn".into() }
fn default_deny() -> String { "deny".into() }

pub fn load(path: Option<&Path>) -> Result<AuditorConfig> {
    let candidate = path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("prometheus-auditor.toml"));

    if candidate.exists() {
        let raw = std::fs::read_to_string(&candidate)
            .with_context(|| format!("reading {}", candidate.display()))?;
        toml::from_str(&raw)
            .with_context(|| format!("parsing {}", candidate.display()))
    } else {
        toml::from_str(DEFAULT_TOML).context("parsing built-in default config")
    }
}
