#[cfg(target_os = "macos")]
use std::process::Command;
use std::{collections::BTreeMap, fs, path::PathBuf};

use hyper::Method;
use prometheus_exec_contracts::{hash_bytes, ReceiptLogSegment};
#[cfg(feature = "estate")]
use prometheus_exec_remote::DispatchQueue;
use prometheus_exec_service::{RunRecord, SpawnStatus};
use prometheus_exec_tier_w::{ComponentAuthorizer, EngineProfile, TierWEngine};
use serde::Serialize;

use crate::{identity, uds_client};

#[derive(Clone, Debug)]
pub struct DoctorConfig {
    pub socket: PathBuf,
    pub state_dir: PathBuf,
    pub identity: PathBuf,
    pub plugin_root: PathBuf,
    pub service_definition: Option<PathBuf>,
    pub mcp_schema: Option<PathBuf>,
    #[cfg(feature = "estate")]
    pub remote_queue: Option<PathBuf>,
    pub exclusions: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    Pass,
    Fail,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DoctorCheck {
    pub name: String,
    pub status: CheckStatus,
    pub required: bool,
    pub detail: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DoctorReport {
    pub healthy: bool,
    pub version: String,
    pub excluded: Vec<String>,
    pub checks: Vec<DoctorCheck>,
}

impl DoctorReport {
    fn finish(checks: Vec<DoctorCheck>, excluded: Vec<String>) -> Self {
        let healthy = !checks
            .iter()
            .any(|check| check.required && check.status == CheckStatus::Fail);
        Self {
            healthy,
            version: env!("CARGO_PKG_VERSION").into(),
            excluded,
            checks,
        }
    }
}

pub async fn inspect(config: DoctorConfig) -> DoctorReport {
    let selection = DoctorSelection::new(config.exclusions.clone());
    let mut checks = Vec::new();
    inspect_binary(&mut checks);
    inspect_identity(&config, &mut checks);
    if let Some(definition) = config.service_definition.as_ref() {
        if let Some(label) = inspect_service_definition(definition, &mut checks) {
            inspect_service_loaded(&label, &mut checks);
        } else {
            fail(
                &mut checks,
                "service-loaded-state",
                "loaded state cannot be checked until the service definition is valid".into(),
            );
        }
    }
    let socket_healthy = inspect_socket(&config, &mut checks).await;
    inspect_tier_p(&mut checks);
    inspect_tier_w(&config, &mut checks);
    if let Some(records) = inspect_state(&config, socket_healthy, &mut checks) {
        inspect_receipt_reconciliation(&config, &records, &mut checks);
        inspect_cas(&config, &records, &mut checks);
    }
    if let Some(schema) = config.mcp_schema.as_ref() {
        inspect_mcp_schema(schema, &mut checks);
    }
    #[cfg(feature = "estate")]
    if selection.selected_remote_queue() {
        if let Some(queue) = config.remote_queue.as_ref() {
            inspect_remote_queue(queue, &mut checks);
        }
    }
    DoctorReport::finish(checks, selection.excluded)
}

struct DoctorSelection {
    excluded: Vec<String>,
}

impl DoctorSelection {
    fn new(excluded: Vec<String>) -> Self {
        Self { excluded }
    }

    #[cfg(feature = "estate")]
    fn selected_remote_queue(&self) -> bool {
        !self.excluded.iter().any(|scope| {
            matches!(
                scope.as_str(),
                "remote" | "remote-queue" | "service:sovereign-sync"
            )
        })
    }
}

fn inspect_binary(checks: &mut Vec<DoctorCheck>) {
    match std::env::current_exe().and_then(fs::read) {
        Ok(bytes) => pass(
            checks,
            "binary-identity",
            format!(
                "prometheus-exec {} {}",
                env!("CARGO_PKG_VERSION"),
                hash_bytes(&bytes)
            ),
        ),
        Err(error) => fail(checks, "binary-identity", error.to_string()),
    }
}

fn inspect_identity(config: &DoctorConfig, checks: &mut Vec<DoctorCheck>) {
    match identity::load(&config.identity) {
        Ok(identity) => pass(
            checks,
            "receipt-identity",
            format!("{} is internally consistent", identity.file.key_id),
        ),
        Err(error) => fail(checks, "receipt-identity", error.to_string()),
    }
}

fn inspect_service_definition(
    path: &std::path::Path,
    checks: &mut Vec<DoctorCheck>,
) -> Option<String> {
    let result = fs::symlink_metadata(path)
        .map_err(|error| error.to_string())
        .and_then(|metadata| {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err("service definition must be a regular non-symlink file".into());
            }
            let bytes = fs::read(path).map_err(|error| error.to_string())?;
            let (label, arguments) = parse_launchagent(path)?;
            let executable = arguments
                .first()
                .and_then(|argument| std::path::Path::new(argument).file_name())
                .and_then(|name| name.to_str());
            if executable != Some("prometheus-exec")
                || arguments.get(1).map(String::as_str) != Some("daemon")
            {
                return Err("service ProgramArguments must launch prometheus-exec daemon".into());
            }
            Ok((hash_bytes(&bytes), label))
        });
    match result {
        Ok((hash, label)) => {
            pass(
                checks,
                "service-definition",
                format!("{} verifies as {hash}", path.display()),
            );
            Some(label)
        }
        Err(error) => {
            fail(checks, "service-definition", error);
            None
        }
    }
}

#[cfg(target_os = "macos")]
fn parse_launchagent(path: &std::path::Path) -> Result<(String, Vec<String>), String> {
    let output = Command::new("/usr/bin/plutil")
        .args(["-convert", "json", "-o", "-"])
        .arg(path)
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "service definition is not a valid plist: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).map_err(|error| error.to_string())?;
    let label = value
        .get("Label")
        .and_then(serde_json::Value::as_str)
        .filter(|label| !label.is_empty())
        .ok_or_else(|| "service definition has no LaunchAgent Label".to_string())?
        .to_string();
    let arguments = value
        .get("ProgramArguments")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "service definition has no ProgramArguments array".to_string())?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| "service ProgramArguments must contain only strings".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok((label, arguments))
}

#[cfg(not(target_os = "macos"))]
fn parse_launchagent(_path: &std::path::Path) -> Result<(String, Vec<String>), String> {
    Err("LaunchAgent definition diagnosis is available only on macOS".into())
}

#[cfg(target_os = "macos")]
fn inspect_service_loaded(label: &str, checks: &mut Vec<DoctorCheck>) {
    let uid = Command::new("/usr/bin/id")
        .arg("-u")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let Some(uid) = uid else {
        fail(
            checks,
            "service-loaded-state",
            "unable to resolve the current GUI user ID".into(),
        );
        return;
    };
    let domain = format!("gui/{uid}/{label}");
    match Command::new("/bin/launchctl")
        .args(["print", &domain])
        .output()
    {
        Ok(output) if output.status.success() => pass(
            checks,
            "service-loaded-state",
            format!("{label} is loaded in gui/{uid}"),
        ),
        Ok(output) => fail(
            checks,
            "service-loaded-state",
            format!(
                "{label} is not loaded in gui/{uid}: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        ),
        Err(error) => fail(checks, "service-loaded-state", error.to_string()),
    }
}

#[cfg(not(target_os = "macos"))]
fn inspect_service_loaded(_label: &str, checks: &mut Vec<DoctorCheck>) {
    fail(
        checks,
        "service-loaded-state",
        "LaunchAgent loaded-state diagnosis is available only on macOS".into(),
    );
}

fn inspect_mcp_schema(path: &std::path::Path, checks: &mut Vec<DoctorCheck>) {
    let result = fs::symlink_metadata(path)
        .map_err(|error| error.to_string())
        .and_then(|metadata| {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err("MCP schema must be a regular non-symlink file".into());
            }
            let checked: serde_json::Value =
                serde_json::from_slice(&fs::read(path).map_err(|error| error.to_string())?)
                    .map_err(|error| error.to_string())?;
            let compiled = serde_json::to_value(crate::mcp::tool_contracts())
                .map_err(|error| error.to_string())?;
            if checked != compiled {
                return Err("checked MCP schema differs from compiled tool contracts".into());
            }
            Ok(hash_bytes(
                &serde_json::to_vec(&checked).map_err(|error| error.to_string())?,
            ))
        });
    match result {
        Ok(hash) => pass(
            checks,
            "mcp-schema",
            format!("compiled and checked tool contracts agree at {hash}"),
        ),
        Err(error) => fail(checks, "mcp-schema", error),
    }
}

#[cfg(feature = "estate")]
fn inspect_remote_queue(path: &std::path::Path, checks: &mut Vec<DoctorCheck>) {
    match DispatchQueue::inspect_read_only(path) {
        Ok(records) => pass(
            checks,
            "remote-queue",
            format!(
                "{} immutable dispatch record(s) verify without mutation",
                records.len()
            ),
        ),
        Err(error) => fail(checks, "remote-queue", error.to_string()),
    }
}

async fn inspect_socket(config: &DoctorConfig, checks: &mut Vec<DoctorCheck>) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::{FileTypeExt as _, PermissionsExt as _};

        let metadata = match fs::symlink_metadata(&config.socket) {
            Ok(metadata)
                if !metadata.file_type().is_symlink()
                    && metadata.file_type().is_socket()
                    && metadata.permissions().mode() & 0o777 == 0o600 =>
            {
                pass(
                    checks,
                    "socket-permissions",
                    format!("{} is a mode-0600 Unix socket", config.socket.display()),
                );
                true
            }
            Ok(_) => {
                fail(
                    checks,
                    "socket-permissions",
                    format!("{} is not a mode-0600 Unix socket", config.socket.display()),
                );
                false
            }
            Err(error) => {
                fail(checks, "socket-permissions", error.to_string());
                false
            }
        };
        if !metadata {
            return false;
        }
    }
    #[cfg(not(unix))]
    {
        fail(
            checks,
            "socket-permissions",
            "Unix sockets are unavailable on this platform".into(),
        );
        return false;
    }

    let health = uds_client::request(&config.socket, Method::GET, "/health", vec![]).await;
    match health {
        Ok(response) if response.status == 200 => pass(
            checks,
            "socket-peer-health",
            "same-UID /health request succeeded".into(),
        ),
        Ok(response) => {
            fail(
                checks,
                "socket-peer-health",
                format!("/health returned HTTP {}", response.status),
            );
            return false;
        }
        Err(error) => {
            fail(checks, "socket-peer-health", error.to_string());
            return false;
        }
    }
    match uds_client::request(&config.socket, Method::GET, "/ready", vec![]).await {
        Ok(response) if response.status == 200 => {
            pass(
                checks,
                "readiness",
                "all required subsystems are ready".into(),
            );
            true
        }
        Ok(response) => {
            fail(
                checks,
                "readiness",
                format!(
                    "/ready returned HTTP {}: {}",
                    response.status,
                    String::from_utf8_lossy(&response.body)
                ),
            );
            false
        }
        Err(error) => {
            fail(checks, "readiness", error.to_string());
            false
        }
    }
}

fn inspect_tier_p(checks: &mut Vec<DoctorCheck>) {
    #[cfg(target_os = "macos")]
    match prometheus_exec_tier_p::SeatbeltConfig::detect() {
        Ok(_) => pass(
            checks,
            "tier-p-backend",
            "macOS Seatbelt launcher and runtimes are available".into(),
        ),
        Err(error) => fail(checks, "tier-p-backend", error.to_string()),
    }
    #[cfg(target_os = "linux")]
    match prometheus_exec_tier_p::BwrapConfig::detect() {
        Ok(config) => pass(
            checks,
            "tier-p-backend",
            format!("bubblewrap {} is available", config.version()),
        ),
        Err(error) => fail(checks, "tier-p-backend", error.to_string()),
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    fail(
        checks,
        "tier-p-backend",
        "Tier P is unavailable on this platform".into(),
    );
}

fn inspect_tier_w(config: &DoctorConfig, checks: &mut Vec<DoctorCheck>) {
    let profile = EngineProfile::for_current_target();
    let availability = TierWEngine::probe(profile);
    if availability.available {
        pass(
            checks,
            "tier-w-backend",
            format!(
                "Wasmtime 46 {} backend is available for {} without compiling guest bytes",
                availability.backend.name(),
                availability.target.name()
            ),
        );
    } else {
        fail(
            checks,
            "tier-w-backend",
            availability
                .reason
                .unwrap_or_else(|| "Tier W backend probe failed".into()),
        );
    }

    match ComponentAuthorizer::estate(&config.plugin_root).inspect() {
        Ok(inspection) => pass(
            checks,
            "tier-w-trust",
            format!(
                "active signed generation {} verifies {} component(s)",
                inspection.generation_id.as_deref().unwrap_or("exact-pins"),
                inspection.component_count
            ),
        ),
        Err(error) => fail(checks, "tier-w-trust", error.to_string()),
    }
}

fn inspect_state(
    config: &DoctorConfig,
    daemon_healthy: bool,
    checks: &mut Vec<DoctorCheck>,
) -> Option<Vec<RunRecord>> {
    let runs = config.state_dir.join("service/ledger/runs");
    let entries = match fs::read_dir(&runs) {
        Ok(entries) => entries,
        Err(error) => {
            fail(checks, "state-reconciliation", error.to_string());
            return None;
        }
    };
    let mut total = 0usize;
    let mut in_flight = 0usize;
    let mut records = Vec::new();
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                fail(checks, "state-reconciliation", error.to_string());
                return None;
            }
        };
        let path = entry.path();
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) if !metadata.file_type().is_symlink() && metadata.is_file() => metadata,
            Ok(_) => {
                fail(
                    checks,
                    "state-reconciliation",
                    format!("unsafe run record: {}", path.display()),
                );
                return None;
            }
            Err(error) => {
                fail(checks, "state-reconciliation", error.to_string());
                return None;
            }
        };
        if metadata.len() > 8 * 1024 * 1024 {
            fail(
                checks,
                "state-reconciliation",
                format!("oversized run record: {}", path.display()),
            );
            return None;
        }
        let record: RunRecord = match fs::read(&path)
            .map_err(|error| error.to_string())
            .and_then(|bytes| serde_json::from_slice(&bytes).map_err(|error| error.to_string()))
        {
            Ok(record) => record,
            Err(error) => {
                fail(checks, "state-reconciliation", error);
                return None;
            }
        };
        if record.validate().is_err() {
            fail(
                checks,
                "state-reconciliation",
                format!("invalid run record: {}", path.display()),
            );
            return None;
        }
        if !record.state.is_terminal() && matches!(record.spawn, SpawnStatus::Spawned { .. }) {
            in_flight += 1;
        }
        total += 1;
        records.push(record);
    }
    if in_flight > 0 && !daemon_healthy {
        fail(
            checks,
            "state-reconciliation",
            format!("{in_flight} spawned runs require daemon restart reconciliation"),
        );
    } else {
        pass(
            checks,
            "state-reconciliation",
            format!("{total} records are structurally valid; {in_flight} are in flight"),
        );
    }
    Some(records)
}

fn inspect_receipt_reconciliation(
    config: &DoctorConfig,
    records: &[RunRecord],
    checks: &mut Vec<DoctorCheck>,
) {
    let identity = match identity::load(&config.identity) {
        Ok(identity) => identity,
        Err(error) => {
            fail(checks, "receipt-reconciliation", error.to_string());
            return;
        }
    };
    let root = config.state_dir.join("service/ledger/receipts/segments");
    let mut files = match fs::read_dir(&root) {
        Ok(entries) => entries
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<std::io::Result<Vec<_>>>(),
        Err(error) => {
            fail(checks, "receipt-reconciliation", error.to_string());
            return;
        }
    };
    let Ok(ref mut files) = files else {
        fail(
            checks,
            "receipt-reconciliation",
            files.unwrap_err().to_string(),
        );
        return;
    };
    files.sort();
    let mut previous = None;
    let mut logged = BTreeMap::new();
    for (sequence, path) in files.iter().enumerate() {
        let metadata = match fs::symlink_metadata(path) {
            Ok(metadata)
                if !metadata.file_type().is_symlink()
                    && metadata.is_file()
                    && metadata.len() <= 16 * 1024 * 1024 =>
            {
                metadata
            }
            Ok(_) => {
                fail(
                    checks,
                    "receipt-reconciliation",
                    format!("unsafe receipt segment: {}", path.display()),
                );
                return;
            }
            Err(error) => {
                fail(checks, "receipt-reconciliation", error.to_string());
                return;
            }
        };
        let _ = metadata;
        let segment: ReceiptLogSegment = match fs::read(path)
            .map_err(|error| error.to_string())
            .and_then(|bytes| serde_json::from_slice(&bytes).map_err(|error| error.to_string()))
        {
            Ok(segment) => segment,
            Err(error) => {
                fail(checks, "receipt-reconciliation", error);
                return;
            }
        };
        if segment.header.sequence != sequence as u64 {
            fail(
                checks,
                "receipt-reconciliation",
                format!("receipt sequence mismatch at {}", path.display()),
            );
            return;
        }
        if let Err(error) = segment.verify(previous.as_ref(), |key_id, algorithm| {
            (key_id == identity.file.key_id && algorithm == identity.file.sig_alg)
                .then(|| identity.verification_key.clone())
        }) {
            fail(checks, "receipt-reconciliation", error.to_string());
            return;
        }
        for entry in &segment.entries {
            logged.insert(entry.receipt.run_id, entry.receipt.clone());
        }
        previous = segment.segment_hash.clone();
    }
    for record in records {
        if let Some(terminal) = &record.terminal {
            if logged.get(&record.run_id) != Some(&terminal.receipt) {
                fail(
                    checks,
                    "receipt-reconciliation",
                    format!("terminal run {} is absent or mismatched", record.run_id),
                );
                return;
            }
        }
    }
    pass(
        checks,
        "receipt-reconciliation",
        format!(
            "{} signed receipt(s) reconcile with {} run record(s)",
            logged.len(),
            records.len()
        ),
    );
}

fn inspect_cas(config: &DoctorConfig, records: &[RunRecord], checks: &mut Vec<DoctorCheck>) {
    let root = config.state_dir.join("artifacts/blobs/sha256");
    let mut files = Vec::new();
    if let Err(error) = collect_files(&root, &mut files) {
        fail(checks, "artifact-cas", error);
        return;
    }
    for path in &files {
        let relative = match path.strip_prefix(&root) {
            Ok(relative) => relative,
            Err(_) => {
                fail(checks, "artifact-cas", "CAS path escaped its root".into());
                return;
            }
        };
        let expected = relative
            .components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<String>();
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) => {
                fail(checks, "artifact-cas", error.to_string());
                return;
            }
        };
        let observed = hash_bytes(&bytes);
        if observed.as_str().trim_start_matches("sha256:") != expected {
            fail(
                checks,
                "artifact-cas",
                format!("CAS hash mismatch: {}", path.display()),
            );
            return;
        }
    }
    for record in records {
        let mut referenced = vec![record.request.code.hash.clone()];
        referenced.extend(record.request.inputs.iter().map(|input| input.hash.clone()));
        if let Some(terminal) = &record.terminal {
            referenced.push(terminal.receipt.outputs.stdout.clone());
            referenced.push(terminal.receipt.outputs.stderr.clone());
            referenced.extend(
                terminal
                    .receipt
                    .outputs
                    .artifacts
                    .iter()
                    .map(|artifact| artifact.hash.clone()),
            );
        }
        for hash in referenced {
            let hex = hash.as_str().trim_start_matches("sha256:");
            let path = root.join(&hex[..2]).join(&hex[2..]);
            if !path.is_file() {
                fail(
                    checks,
                    "artifact-cas",
                    format!("run {} references missing CAS blob {hash}", record.run_id),
                );
                return;
            }
        }
    }
    pass(
        checks,
        "artifact-cas",
        format!(
            "{} content-addressed blobs verify and reconcile with {} run record(s)",
            files.len(),
            records.len()
        ),
    );
}

fn collect_files(root: &std::path::Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let metadata = fs::symlink_metadata(root).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!("unsafe CAS directory: {}", root.display()));
    }
    for entry in fs::read_dir(root).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink() {
            return Err(format!("CAS symlink is forbidden: {}", path.display()));
        }
        if metadata.is_dir() {
            collect_files(&path, files)?;
        } else if metadata.is_file() {
            files.push(path);
        } else {
            return Err(format!("unsafe CAS entry: {}", path.display()));
        }
    }
    files.sort();
    Ok(())
}

fn pass(checks: &mut Vec<DoctorCheck>, name: &str, detail: String) {
    checks.push(DoctorCheck {
        name: name.into(),
        status: CheckStatus::Pass,
        required: true,
        detail,
    });
}

fn fail(checks: &mut Vec<DoctorCheck>, name: &str, detail: String) {
    checks.push(DoctorCheck {
        name: name.into(),
        status: CheckStatus::Fail,
        required: true,
        detail,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    #[cfg(target_os = "macos")]
    fn service_definition_parses_real_program_arguments_and_rejects_comment_spoofing() {
        let directory = tempdir().unwrap();
        let valid = directory.path().join("valid.plist");
        fs::write(
            &valid,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0"><dict>
  <key>Label</key><string>ai.prometheus.exec</string>
  <key>ProgramArguments</key><array>
    <string>/tmp/prometheus-exec</string><string>daemon</string>
  </array>
</dict></plist>"#,
        )
        .unwrap();
        let mut checks = Vec::new();
        assert_eq!(
            inspect_service_definition(&valid, &mut checks),
            Some("ai.prometheus.exec".into())
        );
        assert_eq!(checks[0].status, CheckStatus::Pass);

        let spoofed = directory.path().join("spoofed.plist");
        fs::write(
            &spoofed,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!-- prometheus-exec daemon -->
<plist version="1.0"><dict>
  <key>Label</key><string>ai.prometheus.exec</string>
  <key>ProgramArguments</key><array><string>/bin/false</string></array>
</dict></plist>"#,
        )
        .unwrap();
        checks.clear();
        assert_eq!(inspect_service_definition(&spoofed, &mut checks), None);
        assert_eq!(checks[0].status, CheckStatus::Fail);
    }

    #[test]
    fn tier_w_doctor_probe_does_not_repair_or_populate_trust_state() {
        let directory = tempdir().unwrap();
        let plugin_root = directory.path().join("plugin");
        fs::create_dir(&plugin_root).unwrap();
        let config = DoctorConfig {
            socket: directory.path().join("exec.sock"),
            state_dir: directory.path().join("state"),
            identity: directory.path().join("identity.json"),
            plugin_root: plugin_root.clone(),
            service_definition: None,
            mcp_schema: None,
            #[cfg(feature = "estate")]
            remote_queue: None,
            exclusions: Vec::new(),
        };
        let before: Vec<_> = fs::read_dir(&plugin_root).unwrap().collect();
        let mut checks = Vec::new();

        inspect_tier_w(&config, &mut checks);

        let after: Vec<_> = fs::read_dir(&plugin_root).unwrap().collect();
        assert!(before.is_empty());
        assert!(after.is_empty());
        assert_eq!(checks.len(), 2);
        assert_eq!(checks[0].name, "tier-w-backend");
        assert_eq!(checks[0].status, CheckStatus::Pass);
        assert_eq!(checks[1].name, "tier-w-trust");
        assert_eq!(checks[1].status, CheckStatus::Fail);
    }

    #[test]
    fn mcp_schema_check_detects_drift_without_rewriting_the_fixture() {
        let directory = tempdir().unwrap();
        let schema = directory.path().join("mcp.json");
        let bytes = serde_json::to_vec_pretty(&crate::mcp::tool_contracts()).unwrap();
        fs::write(&schema, &bytes).unwrap();
        let before = fs::read(&schema).unwrap();
        let mut checks = Vec::new();
        inspect_mcp_schema(&schema, &mut checks);
        assert_eq!(checks[0].status, CheckStatus::Pass);
        assert_eq!(fs::read(&schema).unwrap(), before);

        fs::write(&schema, b"{}\n").unwrap();
        checks.clear();
        inspect_mcp_schema(&schema, &mut checks);
        assert_eq!(checks[0].status, CheckStatus::Fail);
    }

    #[cfg(feature = "estate")]
    #[tokio::test]
    async fn sovereign_exclusion_prevents_remote_queue_construction() {
        let directory = tempdir().unwrap();
        let remote = directory.path().join("must-not-be-created");
        let report = inspect(DoctorConfig {
            socket: directory.path().join("exec.sock"),
            state_dir: directory.path().join("state"),
            identity: directory.path().join("identity.json"),
            plugin_root: directory.path().join("plugin"),
            service_definition: None,
            mcp_schema: None,
            remote_queue: Some(remote.clone()),
            exclusions: vec!["service:sovereign-sync".into()],
        })
        .await;
        assert!(!remote.exists());
        assert!(report
            .checks
            .iter()
            .all(|check| check.name != "remote-queue"));
        assert_eq!(report.excluded, ["service:sovereign-sync"]);
    }
}
