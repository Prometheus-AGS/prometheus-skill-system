//! `prometheus contract` — read and validate the pack's integration contract.
//!
//! Seams and rationale: `docs/integration-contract.md`.
//!
//! Two invariants this module exists to hold:
//!
//! 1. **Silence when absent.** `show` reports `endpoint: null` with source
//!    `absent`, exits 0, and writes nothing to stderr when no control endpoint
//!    is discovered. The pack must never warn about a missing extension.
//! 2. **Discovery order is the CLI's own.** The order below mirrors
//!    `control_transport::default_target` exactly. If that chain changes, this
//!    reporting changes with it or the contract is a lie.

use anyhow::{Context, Result};
use serde::Serialize;
use std::path::{Path, PathBuf};

/// The contract version this build implements. Bumped only with a documented
/// change to `docs/integration-contract.md`.
pub const CONTRACT_VERSION: &str = "1.0.0";

#[derive(Serialize)]
pub struct ContractReport {
    pub contract_version: &'static str,
    /// `None` when nothing was discovered. Never an error, never a warning.
    pub endpoint: Option<String>,
    /// One of: `env:PROMETHEUS_CONTROL_ENDPOINT`, `env:SOVEREIGN_SYNC_SOCKET`,
    /// `default:socket`, `default:tcp`, `absent`.
    pub endpoint_source: &'static str,
    pub service_manifest: Option<String>,
}

/// Resolve the control endpoint using the same order as the KBD transport.
///
/// A socket path is reported when it exists on disk; a configured-but-absent
/// socket reports `absent` rather than a path that nothing is serving, because
/// the contract promises callers a discovered endpoint, not a guess.
fn discover(root: &Path) -> (Option<String>, &'static str) {
    if let Ok(endpoint) = std::env::var("PROMETHEUS_CONTROL_ENDPOINT") {
        let trimmed = endpoint.trim_end_matches('/').to_string();
        if !trimmed.is_empty() {
            return (Some(trimmed), "env:PROMETHEUS_CONTROL_ENDPOINT");
        }
    }

    #[cfg(unix)]
    {
        if let Some(socket) = std::env::var_os("SOVEREIGN_SYNC_SOCKET") {
            let path = PathBuf::from(socket);
            if path.exists() {
                return (
                    Some(path.to_string_lossy().into_owned()),
                    "env:SOVEREIGN_SYNC_SOCKET",
                );
            }
            return (None, "absent");
        }
        if let Some(path) = dirs::data_local_dir()
            .map(|base| base.join("prometheus/run/sovereign-sync.sock"))
        {
            if path.exists() {
                return (Some(path.to_string_lossy().into_owned()), "default:socket");
            }
        }
        let _ = root;
        return (None, "absent");
    }

    #[cfg(not(unix))]
    {
        let _ = root;
        (Some("http://127.0.0.1:7892".to_string()), "default:tcp")
    }
}

pub fn report(path: &str) -> ContractReport {
    let root = Path::new(path);
    let (endpoint, endpoint_source) = discover(root);
    let manifest = root.join("shared/services.manifest.json");
    ContractReport {
        contract_version: CONTRACT_VERSION,
        endpoint,
        endpoint_source,
        service_manifest: manifest
            .exists()
            .then(|| "shared/services.manifest.json".to_string()),
    }
}

/// `prometheus contract show [--json]`
pub fn show(path: &str, json: bool) -> Result<()> {
    let report = report(path);
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("contract version : {}", report.contract_version);
        println!(
            "control endpoint : {} ({})",
            report.endpoint.as_deref().unwrap_or("absent"),
            report.endpoint_source
        );
        println!(
            "service manifest : {}",
            report.service_manifest.as_deref().unwrap_or("absent")
        );
    }
    Ok(())
}

fn parse_semver(value: &str) -> Option<(u64, u64, u64)> {
    let core = value.split(['-', '+']).next()?;
    let mut parts = core.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

fn is_kebab(value: &str) -> bool {
    !value.is_empty()
        && value
            .split('-')
            .all(|seg| !seg.is_empty() && seg.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()))
}

/// `prometheus contract validate <skill-package.json>`
///
/// Validates the declaration against the rules in
/// `shared/schemas/skill-package.schema.json`. The schema file stays
/// authoritative for external tooling; these checks mirror it so the CLI needs
/// no JSON Schema dependency.
pub fn validate(file: &str) -> Result<()> {
    let text = std::fs::read_to_string(file)
        .with_context(|| format!("cannot read declaration: {file}"))?;
    let doc: serde_json::Value = serde_json::from_str(&text)
        .with_context(|| format!("declaration is not valid JSON: {file}"))?;

    let mut errors: Vec<String> = Vec::new();

    let obj = match doc.as_object() {
        Some(obj) => obj,
        None => {
            anyhow::bail!("declaration must be a JSON object: {file}");
        }
    };

    for key in ["name", "version", "minimumContractVersion"] {
        if !obj.contains_key(key) {
            errors.push(format!("missing required field `{key}`"));
        }
    }

    if let Some(name) = obj.get("name").and_then(|v| v.as_str()) {
        if !is_kebab(name) || name.len() > 64 {
            errors.push(format!(
                "`name` must be kebab-case and at most 64 characters, got `{name}`"
            ));
        }
    }

    if let Some(version) = obj.get("version").and_then(|v| v.as_str()) {
        if parse_semver(version).is_none() {
            errors.push(format!("`version` is not a semantic version: `{version}`"));
        }
    }

    // The contract-version gate: refuse a requirement this build cannot meet,
    // and name both numbers so the operator knows what to upgrade.
    if let Some(required) = obj.get("minimumContractVersion").and_then(|v| v.as_str()) {
        match (parse_semver(required), parse_semver(CONTRACT_VERSION)) {
            (Some(req), Some(have)) => {
                if req > have {
                    errors.push(format!(
                        "declaration requires integration contract {required}, but this pack implements {CONTRACT_VERSION}"
                    ));
                }
            }
            (None, _) => errors.push(format!(
                "`minimumContractVersion` is not a semantic version: `{required}`"
            )),
            _ => {}
        }
    }

    if let Some(bundles) = obj
        .get("hooks")
        .and_then(|h| h.get("bundles"))
        .and_then(|b| b.as_array())
    {
        for bundle in bundles {
            let name = bundle.as_str().unwrap_or_default();
            let mut parts = name.split('/');
            let ns = parts.next().unwrap_or_default();
            let leaf = parts.next().unwrap_or_default();
            if parts.next().is_some() || !is_kebab(ns) || !is_kebab(leaf) {
                errors.push(format!(
                    "hook bundle `{name}` must be namespaced as `<package>/<bundle>` in kebab-case"
                ));
            }
        }
    }

    if let Some(servers) = obj.get("mcpServers").and_then(|v| v.as_object()) {
        for (key, server) in servers {
            if server.get("command").and_then(|c| c.as_str()).is_none() {
                errors.push(format!("mcpServers.{key} is missing `command`"));
            }
        }
    }

    if errors.is_empty() {
        let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("?");
        let version = obj.get("version").and_then(|v| v.as_str()).unwrap_or("?");
        println!("valid: {name} {version} (contract {CONTRACT_VERSION})");
        return Ok(());
    }

    for error in &errors {
        eprintln!("invalid: {error}");
    }
    anyhow::bail!("{} validation error(s) in {file}", errors.len())
}
