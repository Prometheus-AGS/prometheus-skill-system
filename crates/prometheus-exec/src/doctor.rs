use std::{fs, path::PathBuf};

use hyper::Method;
use prometheus_exec_contracts::hash_bytes;
use prometheus_exec_service::{RunRecord, SpawnStatus};
use serde::Serialize;

use crate::{identity, uds_client};

#[derive(Clone, Debug)]
pub struct DoctorConfig {
    pub socket: PathBuf,
    pub state_dir: PathBuf,
    pub identity: PathBuf,
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
    pub checks: Vec<DoctorCheck>,
}

impl DoctorReport {
    fn finish(checks: Vec<DoctorCheck>) -> Self {
        let healthy = !checks
            .iter()
            .any(|check| check.required && check.status == CheckStatus::Fail);
        Self {
            healthy,
            version: env!("CARGO_PKG_VERSION").into(),
            checks,
        }
    }
}

pub async fn inspect(config: DoctorConfig) -> DoctorReport {
    let mut checks = Vec::new();
    inspect_binary(&mut checks);
    inspect_identity(&config, &mut checks);
    let socket_healthy = inspect_socket(&config, &mut checks).await;
    inspect_sandbox(&mut checks);
    inspect_state(&config, socket_healthy, &mut checks);
    inspect_cas(&config, &mut checks);
    DoctorReport::finish(checks)
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

fn inspect_sandbox(checks: &mut Vec<DoctorCheck>) {
    #[cfg(target_os = "macos")]
    match prometheus_exec_tier_p::SeatbeltConfig::detect() {
        Ok(_) => pass(
            checks,
            "sandbox-backend",
            "macOS Seatbelt launcher and runtimes are available".into(),
        ),
        Err(error) => fail(checks, "sandbox-backend", error.to_string()),
    }
    #[cfg(target_os = "linux")]
    match prometheus_exec_tier_p::BwrapConfig::detect() {
        Ok(config) => pass(
            checks,
            "sandbox-backend",
            format!("bubblewrap {} is available", config.version()),
        ),
        Err(error) => fail(checks, "sandbox-backend", error.to_string()),
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    fail(
        checks,
        "sandbox-backend",
        "Tier P is unavailable on this platform".into(),
    );
}

fn inspect_state(config: &DoctorConfig, daemon_healthy: bool, checks: &mut Vec<DoctorCheck>) {
    let runs = config.state_dir.join("service/ledger/runs");
    let entries = match fs::read_dir(&runs) {
        Ok(entries) => entries,
        Err(error) => {
            fail(checks, "state-reconciliation", error.to_string());
            return;
        }
    };
    let mut total = 0usize;
    let mut in_flight = 0usize;
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                fail(checks, "state-reconciliation", error.to_string());
                return;
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
                return;
            }
            Err(error) => {
                fail(checks, "state-reconciliation", error.to_string());
                return;
            }
        };
        if metadata.len() > 8 * 1024 * 1024 {
            fail(
                checks,
                "state-reconciliation",
                format!("oversized run record: {}", path.display()),
            );
            return;
        }
        let record: RunRecord = match fs::read(&path)
            .map_err(|error| error.to_string())
            .and_then(|bytes| serde_json::from_slice(&bytes).map_err(|error| error.to_string()))
        {
            Ok(record) => record,
            Err(error) => {
                fail(checks, "state-reconciliation", error);
                return;
            }
        };
        if record.request.validate().is_err()
            || record.request.request_hash().ok().as_ref() != Some(&record.request_hash)
            || record.request.request_id != record.request_id
            || record.state.is_terminal() != record.terminal.is_some()
        {
            fail(
                checks,
                "state-reconciliation",
                format!("invalid run record: {}", path.display()),
            );
            return;
        }
        if !record.state.is_terminal() && matches!(record.spawn, SpawnStatus::Spawned { .. }) {
            in_flight += 1;
        }
        total += 1;
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
}

fn inspect_cas(config: &DoctorConfig, checks: &mut Vec<DoctorCheck>) {
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
    pass(
        checks,
        "artifact-cas",
        format!("{} content-addressed blobs verified", files.len()),
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
