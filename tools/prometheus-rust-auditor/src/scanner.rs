use anyhow::{Context as _, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct CrateInfo {
    pub name: String,
    pub manifest_path: PathBuf,
    pub root_path: PathBuf,
}

#[derive(Debug, Deserialize)]
struct MetadataPackage {
    name: String,
    manifest_path: String,
}

#[derive(Debug, Deserialize)]
struct CargoMetadata {
    packages: Vec<MetadataPackage>,
    #[allow(dead_code)]
    workspace_root: String,
}

pub fn discover_workspace_crates(workspace_root: &Path) -> Result<Vec<CrateInfo>> {
    let output = Command::new("cargo")
        .args(["metadata", "--format-version=1", "--no-deps"])
        .current_dir(workspace_root)
        .output()
        .context("failed to run `cargo metadata`")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("cargo metadata failed: {stderr}");
    }

    let meta: CargoMetadata =
        serde_json::from_slice(&output.stdout).context("parsing cargo metadata output")?;

    let crates = meta
        .packages
        .into_iter()
        .map(|p| {
            let manifest = PathBuf::from(&p.manifest_path);
            let root = manifest.parent().unwrap_or(Path::new(".")).to_path_buf();
            CrateInfo { name: p.name, manifest_path: manifest, root_path: root }
        })
        .collect();

    Ok(crates)
}

pub fn match_partition_glob(crate_name: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if let Some(suffix) = pattern.strip_prefix('*') {
        return crate_name.ends_with(suffix);
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        return crate_name.starts_with(prefix);
    }
    crate_name == pattern
}
