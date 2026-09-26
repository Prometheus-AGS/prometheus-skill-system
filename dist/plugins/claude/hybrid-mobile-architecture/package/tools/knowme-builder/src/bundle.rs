use std::path::{Component, Path};

use anyhow::{Context, Result, bail};
use include_dir::{Dir, DirEntry, include_dir};

use crate::{
    cli::Profile,
    model::{BUILDER_VERSION, BuilderManifest, ProfileManifest},
};

pub static TEMPLATES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../assets/templates");
pub static SKILLS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../skills");
const MANIFEST_JSON: &str = include_str!("../../../builder.manifest.json");
const PROMETHEUS_CONTRACT_JSON: &str =
    include_str!("../../../compatibility/prometheus-control-plane.json");
const UAR_CONTRACT_JSON: &str = include_str!("../../../compatibility/uar-runtime.json");
const ACTIVATION_MANIFEST_JSON: &str = include_str!("../../../templates/activation-manifest.json");

pub fn manifest() -> Result<BuilderManifest> {
    validate_embedded_source_tree(&TEMPLATES)?;
    let manifest: BuilderManifest =
        serde_json::from_str(MANIFEST_JSON).context("embedded Builder manifest is invalid")?;
    if manifest.schema_version != 1 {
        bail!(
            "unsupported embedded Builder manifest schema {}",
            manifest.schema_version
        );
    }
    if manifest.package.version != BUILDER_VERSION {
        bail!(
            "crate version {} differs from manifest version {}",
            BUILDER_VERSION,
            manifest.package.version
        );
    }
    Ok(manifest)
}

fn validate_embedded_source_tree(root: &Dir<'_>) -> Result<()> {
    fn visit(dir: &Dir<'_>) -> Result<()> {
        for entry in dir.entries() {
            if let DirEntry::Dir(child) = entry {
                let name = child
                    .path()
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or_default();
                if is_generated_output_directory(name) {
                    bail!(
                        "embedded template source contains generated dependency/build output: {}",
                        child.path().display()
                    );
                }
                visit(child)?;
            }
        }
        Ok(())
    }

    visit(root)
}

fn is_generated_output_directory(name: &str) -> bool {
    matches!(
        name,
        "node_modules" | "target" | "dist" | "build" | ".dart_tool" | ".gradle" | "Pods"
    )
}

pub fn profile(manifest: &BuilderManifest, profile: Profile) -> Result<&ProfileManifest> {
    manifest
        .profiles
        .get(profile.as_str())
        .with_context(|| format!("profile {} is not packaged", profile.as_str()))
}

pub fn prometheus_contract() -> Result<serde_json::Value> {
    serde_json::from_str(PROMETHEUS_CONTRACT_JSON)
        .context("embedded Prometheus compatibility contract is invalid")
}

pub fn uar_contract() -> Result<serde_json::Value> {
    serde_json::from_str(UAR_CONTRACT_JSON)
        .context("embedded UAR compatibility contract is invalid")
}

pub fn activation_manifest() -> &'static [u8] {
    ACTIVATION_MANIFEST_JSON.as_bytes()
}

pub fn validate_relative_path(path: &Path) -> Result<()> {
    if path.as_os_str().is_empty() || path.is_absolute() {
        bail!(
            "generated path must be non-empty and relative: {}",
            path.display()
        );
    }
    for component in path.components() {
        if matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        ) {
            bail!("generated path escapes the destination: {}", path.display());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{is_generated_output_directory, manifest, validate_relative_path};

    #[test]
    fn embedded_manifest_matches_crate_version() {
        let manifest = manifest().expect("manifest");
        assert_eq!(manifest.skills.len(), 35);
        assert_eq!(manifest.supported_harnesses.len(), 6);
    }

    #[test]
    fn rejects_escaping_paths() {
        assert!(validate_relative_path(Path::new("../secret")).is_err());
        assert!(validate_relative_path(Path::new("/tmp/file")).is_err());
        assert!(validate_relative_path(Path::new("src/main.rs")).is_ok());
    }

    #[test]
    fn classifies_generated_output_directories() {
        for name in [
            "node_modules",
            "target",
            "dist",
            "build",
            ".dart_tool",
            ".gradle",
            "Pods",
        ] {
            assert!(is_generated_output_directory(name), "{name}");
        }
        assert!(!is_generated_output_directory("src"));
    }
}
