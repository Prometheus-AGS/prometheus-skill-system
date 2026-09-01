mod daemon;
mod doctor;
mod file_security;
mod identity;
mod mcp;
mod uds_client;

use std::{
    collections::BTreeMap,
    fs,
    io::{self},
    path::{Path, PathBuf},
    process::ExitCode,
    time::Duration,
};

use chrono::Utc;
use clap::{Parser, Subcommand, ValueEnum};
use hyper::Method;
use prometheus_exec_contracts::{
    contract_schemas, openapi_components, sign_request_ed25519, verify_evidence_bundle,
    verify_receipt, CapabilityManifest, CodeIdentity, CodeKind, ExecutionEvidenceIndex,
    ExecutionLimits, ExecutionProvenance, ExecutionReceipt, NamedInput, RequestedTier, RunState,
    RuntimeKind, SignatureAlgorithm, SignedExecRequest, VerificationKey, SCHEMA_VERSION,
};
use prometheus_exec_core::ArtifactStore;
use prometheus_exec_service::RunResponse;
use prometheus_exec_tier_w::{replay_verified_receipt, ComponentAuthorizer};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, Parser)]
#[command(name = "prometheus-exec", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Create a standalone Ed25519 device identity.
    Init {
        #[arg(long)]
        identity: PathBuf,
    },
    /// Run the health-first local sidecar with Tier P and Tier W workers.
    Daemon {
        #[arg(long)]
        socket: PathBuf,
        #[arg(long)]
        state_dir: PathBuf,
        #[arg(long)]
        identity: PathBuf,
        /// Signed plugin estate containing the active component generation.
        #[arg(long)]
        plugin_root: Option<PathBuf>,
        #[arg(long, default_value_t = 2048)]
        artifact_budget_mb: u64,
    },
    /// Serve the execution tools over MCP stdio with a private local runner.
    Mcp {
        #[arg(long)]
        state_dir: PathBuf,
        #[arg(long)]
        identity: PathBuf,
        /// Signed plugin estate containing authorized Wasm components.
        #[arg(long)]
        plugin_root: Option<PathBuf>,
        #[arg(long, default_value_t = 2048)]
        artifact_budget_mb: u64,
    },
    /// Submit one signed native or Wasm-component execution and await its receipt.
    Run {
        #[arg(long)]
        socket: PathBuf,
        #[arg(long)]
        state_dir: PathBuf,
        #[arg(long)]
        identity: PathBuf,
        /// Signed plugin estate used to authorize Wasm component bytes.
        #[arg(long)]
        plugin_root: Option<PathBuf>,
        #[arg(long, value_enum)]
        runtime: RuntimeArg,
        #[arg(long)]
        code: PathBuf,
        /// Named input in NAME=PATH form. May be repeated.
        #[arg(long = "input")]
        inputs: Vec<String>,
        #[arg(long, default_value_t = 120_000)]
        timeout_ms: u64,
        #[arg(long, default_value_t = 10)]
        output_mb: u64,
        #[arg(long, default_value_t = 2048)]
        artifact_budget_mb: u64,
        #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,
    },
    /// Retrieve one durable run state and terminal receipt.
    Status {
        #[arg(long)]
        socket: PathBuf,
        #[arg(long)]
        run_id: Uuid,
        #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,
    },
    /// Inspect the installed sidecar without changing any state.
    Doctor {
        #[arg(long)]
        socket: PathBuf,
        #[arg(long)]
        state_dir: PathBuf,
        #[arg(long)]
        identity: PathBuf,
        /// Signed plugin estate containing the active component generation.
        #[arg(long)]
        plugin_root: Option<PathBuf>,
        /// Installed service definition to verify; omitted when unmanaged.
        #[arg(long)]
        service_definition: Option<PathBuf>,
        /// Checked MCP schema to compare with the compiled tool contract.
        #[arg(long)]
        mcp_schema: Option<PathBuf>,
        /// Optional estate remote queue to inspect without mutation.
        #[cfg(feature = "estate")]
        #[arg(long)]
        remote_queue: Option<PathBuf>,
        /// Exclude a check ID, group, or service scope before construction.
        #[arg(long = "exclude")]
        exclusions: Vec<String>,
        #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,
    },
    /// Verify a signed execution receipt without starting a service.
    Verify {
        #[arg(long)]
        receipt: PathBuf,
        /// Unpadded base64url public key bytes.
        #[arg(long)]
        public_key: String,
        #[arg(long)]
        request: Option<PathBuf>,
        #[arg(long)]
        artifacts: Option<PathBuf>,
        /// Re-execute a verified Tier W receipt with this exact component.
        #[arg(long)]
        component: Option<PathBuf>,
        /// Named replay input in NAME=PATH form. May be repeated.
        #[arg(long = "input")]
        inputs: Vec<String>,
        #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,
    },
    /// Verify a portable execution-evidence bundle without a service or network.
    VerifyBundle {
        #[arg(long)]
        index: PathBuf,
        /// Bundle root; defaults to the index file's parent directory.
        #[arg(long)]
        root: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,
    },
    /// Generate deterministic JSON Schema and OpenAPI components.
    Contracts {
        #[arg(long)]
        output_dir: PathBuf,
    },
    /// Report one file's security descriptor as JSON, changing nothing.
    ///
    /// The installer's owner-only key gate needs a locale-independent read of a
    /// Windows DACL, and Node has no ACL API. This emits owner and trustee
    /// security identifiers as strings so the caller never compares display
    /// names, and it never applies a remediation.
    InspectFileSecurity {
        #[arg(long)]
        path: PathBuf,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OutputFormat {
    Human,
    Json,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum RuntimeArg {
    WasmComponent,
    Python3,
    Node,
    Bash,
}

impl From<RuntimeArg> for RuntimeKind {
    fn from(value: RuntimeArg) -> Self {
        match value {
            RuntimeArg::WasmComponent => Self::WasmComponent,
            RuntimeArg::Python3 => Self::Python3,
            RuntimeArg::Node => Self::Node,
            RuntimeArg::Bash => Self::Bash,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PublicIdentity<'a> {
    sig_alg: SignatureAlgorithm,
    key_id: &'a str,
    public_key: &'a str,
}

#[tokio::main]
async fn main() -> ExitCode {
    match run(parse_cli()).await {
        Ok(code) => code,
        Err(error) => {
            eprintln!("prometheus-exec: {error}");
            ExitCode::from(2)
        }
    }
}

async fn run(cli: Cli) -> Result<ExitCode, BoxError> {
    match cli.command {
        Command::Init { identity } => init_identity(&identity),
        Command::Daemon {
            socket,
            state_dir,
            identity,
            plugin_root,
            artifact_budget_mb,
        } => {
            daemon::run(daemon::DaemonConfig {
                socket,
                state_dir,
                identity,
                plugin_root: plugin_root.unwrap_or(default_plugin_root()?),
                artifact_budget_bytes: mebibytes(artifact_budget_mb)?,
            })
            .await?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Mcp {
            state_dir,
            identity,
            plugin_root,
            artifact_budget_mb,
        } => {
            mcp::run(mcp::McpConfig {
                state_dir,
                identity,
                plugin_root: plugin_root.unwrap_or(default_plugin_root()?),
                artifact_budget_bytes: mebibytes(artifact_budget_mb)?,
            })
            .await?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Run {
            socket,
            state_dir,
            identity,
            plugin_root,
            runtime,
            code,
            inputs,
            timeout_ms,
            output_mb,
            artifact_budget_mb,
            format,
        } => {
            run_command(RunCommand {
                socket,
                state_dir,
                identity,
                plugin_root,
                runtime: runtime.into(),
                code,
                inputs,
                timeout_ms,
                output_mb,
                artifact_budget_bytes: mebibytes(artifact_budget_mb)?,
                format,
            })
            .await
        }
        Command::Status {
            socket,
            run_id,
            format,
        } => status_command(&socket, run_id, format).await,
        Command::Doctor {
            socket,
            state_dir,
            identity,
            plugin_root,
            service_definition,
            mcp_schema,
            #[cfg(feature = "estate")]
            remote_queue,
            exclusions,
            format,
        } => {
            doctor_command(
                doctor::DoctorConfig {
                    socket,
                    state_dir,
                    identity,
                    plugin_root: plugin_root.unwrap_or(default_plugin_root()?),
                    service_definition,
                    mcp_schema,
                    #[cfg(feature = "estate")]
                    remote_queue,
                    exclusions,
                },
                format,
            )
            .await
        }
        Command::Verify {
            receipt,
            public_key,
            request,
            artifacts,
            component,
            inputs,
            format,
        } => verify_command(
            &receipt,
            &public_key,
            request.as_deref(),
            artifacts.as_deref(),
            component.as_deref(),
            &inputs,
            format,
        ),
        Command::VerifyBundle {
            index,
            root,
            format,
        } => verify_bundle_command(&index, root.as_deref(), format),
        Command::Contracts { output_dir } => generate_contracts(&output_dir),
        Command::InspectFileSecurity { path } => {
            println!("{}", file_security::inspect(&path)?);
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn parse_cli() -> Cli {
    let mut arguments: Vec<_> = std::env::args_os().collect();
    if arguments.get(1).is_some_and(|value| value == "--mode")
        && arguments.get(2).is_some_and(|value| value == "mcp")
    {
        arguments.drain(1..=2);
        arguments.insert(1, "mcp".into());
    }
    Cli::parse_from(arguments)
}

fn init_identity(path: &Path) -> Result<ExitCode, BoxError> {
    let identity = identity::create(path)?;
    let public = PublicIdentity {
        sig_alg: identity.sig_alg,
        key_id: &identity.key_id,
        public_key: &identity.public_key,
    };
    serde_json::to_writer_pretty(io::stdout().lock(), &public)?;
    println!();
    Ok(ExitCode::SUCCESS)
}

struct RunCommand {
    socket: PathBuf,
    state_dir: PathBuf,
    identity: PathBuf,
    plugin_root: Option<PathBuf>,
    runtime: RuntimeKind,
    code: PathBuf,
    inputs: Vec<String>,
    timeout_ms: u64,
    output_mb: u64,
    artifact_budget_bytes: u64,
    format: OutputFormat,
}

async fn run_command(command: RunCommand) -> Result<ExitCode, BoxError> {
    if command.timeout_ms == 0 || command.output_mb == 0 {
        return Err("timeout-ms and output-mb must be non-zero".into());
    }
    let identity = identity::load(&command.identity)?;
    let artifacts = ArtifactStore::open(
        command.state_dir.join("artifacts"),
        command.artifact_budget_bytes,
    )?;
    let request_id = Uuid::new_v4();
    let upload_pin = format!("upload:{request_id}");
    let code = fs::read(&command.code)?;
    let (tier, code_kind, component_authorization) =
        if command.runtime == RuntimeKind::WasmComponent {
            let plugin_root = command.plugin_root.unwrap_or(default_plugin_root()?);
            let authorization =
                ComponentAuthorizer::estate(plugin_root).authorization_for_bytes(&code)?;
            (RequestedTier::W, CodeKind::Component, Some(authorization))
        } else {
            (RequestedTier::P, CodeKind::File, None)
        };
    let mut loaded_inputs = Vec::new();
    for input in command.inputs {
        let (name, path) = input
            .split_once('=')
            .ok_or("input must use NAME=PATH syntax")?;
        if name.is_empty() {
            return Err("input name cannot be empty".into());
        }
        loaded_inputs.push((name.to_string(), fs::read(path)?));
    }

    let stored_code = artifacts.put_pinned(&code, &upload_pin)?;
    let mut pinned_hashes = vec![stored_code.hash.clone()];
    let mut named_inputs = Vec::new();
    for (name, bytes) in loaded_inputs {
        let stored = match artifacts.put_pinned(&bytes, &upload_pin) {
            Ok(stored) => stored,
            Err(error) => {
                if let Err(rollback) = release_pins(&artifacts, &pinned_hashes, &upload_pin) {
                    return Err(format!("{error}; upload-pin rollback failed: {rollback}").into());
                }
                return Err(error.into());
            }
        };
        pinned_hashes.push(stored.hash.clone());
        named_inputs.push(NamedInput {
            name,
            hash: stored.hash,
        });
    }
    named_inputs.sort_by(|left, right| left.name.cmp(&right.name));
    let now = Utc::now();
    let mut request = SignedExecRequest {
        schema_version: SCHEMA_VERSION.into(),
        request_id,
        issued_at: now,
        queued_at: Some(now),
        validity_window_secs: command.timeout_ms.saturating_add(60_000).div_ceil(1000),
        tier,
        code: CodeIdentity {
            kind: code_kind,
            hash: stored_code.hash,
            runtime: command.runtime,
            toolchain_pin: None,
        },
        inputs: named_inputs,
        capabilities: CapabilityManifest::default(),
        limits: ExecutionLimits {
            wall_clock_ms: command.timeout_ms,
            output_mb: command.output_mb,
            ..ExecutionLimits::default()
        },
        targets: vec![],
        provenance: ExecutionProvenance {
            harness: Some("prometheus-exec-cli".into()),
            component_authorization,
            ..ExecutionProvenance::default()
        },
        signer_key_id: None,
        sig_alg: SignatureAlgorithm::Ed25519,
        signature: None,
    };
    if let Err(error) = sign_request_ed25519(&mut request, &identity.signing_key) {
        if let Err(rollback) = release_pins(&artifacts, &pinned_hashes, &upload_pin) {
            return Err(format!("{error}; upload-pin rollback failed: {rollback}").into());
        }
        return Err(error.into());
    }
    let request_body = match serde_json::to_vec(&request) {
        Ok(body) => body,
        Err(error) => {
            if let Err(rollback) = release_pins(&artifacts, &pinned_hashes, &upload_pin) {
                return Err(format!("{error}; upload-pin rollback failed: {rollback}").into());
            }
            return Err(error.into());
        }
    };
    let response = match uds_client::request(
        &command.socket,
        Method::POST,
        "/api/v2/exec/runs",
        request_body,
    )
    .await
    {
        Ok(response) => response,
        Err(error) => {
            if let Err(rollback) = release_pins(&artifacts, &pinned_hashes, &upload_pin) {
                return Err(format!("{error}; upload-pin rollback failed: {rollback}").into());
            }
            return Err(error);
        }
    };
    if !matches!(response.status, 200 | 202) {
        if let Err(rollback) = release_pins(&artifacts, &pinned_hashes, &upload_pin) {
            return Err(format!(
                "{}; upload-pin rollback failed: {rollback}",
                http_error(response.status, &response.body)
            )
            .into());
        }
        return Err(http_error(response.status, &response.body).into());
    }
    let mut run: RunResponse = serde_json::from_slice(&response.body)?;
    let deadline = tokio::time::Instant::now()
        + Duration::from_millis(command.timeout_ms.saturating_add(10_000));
    while !run.state.is_terminal() {
        if run.state == RunState::GrantPending {
            print_run(&run, command.format)?;
            return Ok(ExitCode::from(1));
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(format!("timed out waiting for run {}", run.run_id).into());
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
        run = fetch_run(&command.socket, run.run_id).await?;
    }
    print_run(&run, command.format)?;
    Ok(if run.state == RunState::Succeeded {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

fn release_pins(
    artifacts: &ArtifactStore,
    hashes: &[prometheus_exec_contracts::Digest],
    reason: &str,
) -> Result<(), String> {
    let mut failures = Vec::new();
    for hash in hashes {
        if let Err(error) = artifacts.unpin(hash, reason) {
            failures.push(format!("{hash}: {error}"));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("; "))
    }
}

async fn status_command(
    socket: &Path,
    run_id: Uuid,
    format: OutputFormat,
) -> Result<ExitCode, BoxError> {
    let run = fetch_run(socket, run_id).await?;
    print_run(&run, format)?;
    Ok(ExitCode::SUCCESS)
}

async fn fetch_run(socket: &Path, run_id: Uuid) -> Result<RunResponse, BoxError> {
    let response = uds_client::request(
        socket,
        Method::GET,
        &format!("/api/v2/exec/runs/{run_id}"),
        vec![],
    )
    .await?;
    if response.status != 200 {
        return Err(http_error(response.status, &response.body).into());
    }
    Ok(serde_json::from_slice(&response.body)?)
}

fn print_run(run: &RunResponse, format: OutputFormat) -> Result<(), BoxError> {
    match format {
        OutputFormat::Json => {
            serde_json::to_writer_pretty(io::stdout().lock(), run)?;
            println!();
        }
        OutputFormat::Human => {
            println!("{} {:?}", run.run_id, run.state);
            if let Some(receipt) = &run.receipt {
                println!("  receipt {}", receipt.receipt_hash()?);
                println!("  stdout {}", receipt.outputs.stdout);
                println!("  stderr {}", receipt.outputs.stderr);
                for artifact in &receipt.outputs.artifacts {
                    println!("  artifact {} {}", artifact.path, artifact.hash);
                }
            }
        }
    }
    Ok(())
}

async fn doctor_command(
    config: doctor::DoctorConfig,
    format: OutputFormat,
) -> Result<ExitCode, BoxError> {
    let report = doctor::inspect(config).await;
    match format {
        OutputFormat::Json => {
            serde_json::to_writer_pretty(io::stdout().lock(), &report)?;
            println!();
        }
        OutputFormat::Human => {
            println!(
                "prometheus-exec doctor: {}",
                if report.healthy { "PASS" } else { "FAIL" }
            );
            for check in &report.checks {
                println!("  {:?} {}: {}", check.status, check.name, check.detail);
            }
        }
    }
    Ok(if report.healthy {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

fn default_plugin_root() -> Result<PathBuf, BoxError> {
    let home = std::env::var_os("HOME").ok_or("HOME is unavailable; pass --plugin-root")?;
    Ok(PathBuf::from(home).join(".prometheus/plugins/prometheus-skill-pack"))
}

fn verify_command(
    receipt_path: &Path,
    public_key: &str,
    request_path: Option<&Path>,
    artifact_root: Option<&Path>,
    component_path: Option<&Path>,
    input_arguments: &[String],
    format: OutputFormat,
) -> Result<ExitCode, BoxError> {
    let receipt: ExecutionReceipt = read_json(receipt_path)?;
    let key = VerificationKey::from_base64url(receipt.executing_device.sig_alg, public_key)?;
    let request = request_path
        .map(read_json::<SignedExecRequest>)
        .transpose()?;
    let mut result = verify_receipt(&receipt, &key, request.as_ref(), artifact_root);
    if let Some(component_path) = component_path {
        let request = request
            .as_ref()
            .ok_or("--component requires --request for deterministic replay")?;
        let inputs = load_named_inputs(input_arguments)?;
        match replay_verified_receipt(&receipt, request, &key, fs::read(component_path)?, inputs) {
            Ok(replay) => {
                if !replay.valid {
                    result.record_failure(
                        "tier_w.replay_mismatch",
                        format!(
                            "portable replay differs at {}",
                            replay.mismatches.join(", ")
                        ),
                        Some(receipt.run_id.to_string()),
                    );
                }
                result.replay = Some(replay);
            }
            Err(error) => result.record_failure(
                "tier_w.replay_error",
                error.to_string(),
                Some(receipt.run_id.to_string()),
            ),
        }
    } else if !input_arguments.is_empty() {
        return Err("--input requires --component during verification".into());
    }
    match format {
        OutputFormat::Json => {
            serde_json::to_writer_pretty(io::stdout().lock(), &result)?;
            println!();
        }
        OutputFormat::Human if result.valid => {
            println!(
                "VALID {}",
                result
                    .receipt_hash
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "receipt-hash-unavailable".into())
            );
            for check in &result.checks {
                println!("  ok {}: {}", check.code, check.message);
            }
            if let Some(replay) = &result.replay {
                for check in &replay.checks {
                    println!("  replay ok {check}");
                }
            }
        }
        OutputFormat::Human => {
            println!("INVALID");
            for failure in &result.failures {
                println!("  error {}: {}", failure.code, failure.message);
            }
            if let Some(replay) = &result.replay {
                for mismatch in &replay.mismatches {
                    println!("  replay mismatch {mismatch}");
                }
            }
        }
    }
    Ok(if result.valid {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

fn verify_bundle_command(
    index_path: &Path,
    root: Option<&Path>,
    format: OutputFormat,
) -> Result<ExitCode, BoxError> {
    let index: ExecutionEvidenceIndex = read_json(index_path)?;
    let root = root
        .or_else(|| index_path.parent())
        .ok_or("evidence index has no parent; pass --root")?;
    let result = verify_evidence_bundle(&index, root);
    match format {
        OutputFormat::Json => {
            serde_json::to_writer_pretty(io::stdout().lock(), &result)?;
            println!();
        }
        OutputFormat::Human if result.valid => {
            println!(
                "VALID BUNDLE {}",
                result
                    .index_hash
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "index-hash-unavailable".into())
            );
            for check in &result.checks {
                println!("  ok {}: {}", check.code, check.message);
            }
        }
        OutputFormat::Human => {
            println!("INVALID BUNDLE");
            for failure in &result.failures {
                println!("  error {}: {}", failure.code, failure.message);
            }
        }
    }
    Ok(if result.valid {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

fn load_named_inputs(arguments: &[String]) -> Result<BTreeMap<String, Vec<u8>>, BoxError> {
    let mut inputs = BTreeMap::new();
    for argument in arguments {
        let (name, path) = argument
            .split_once('=')
            .ok_or("input must use NAME=PATH syntax")?;
        if name.is_empty() {
            return Err("input name cannot be empty".into());
        }
        if inputs.insert(name.into(), fs::read(path)?).is_some() {
            return Err(format!("duplicate input name: {name}").into());
        }
    }
    Ok(inputs)
}

fn generate_contracts(output_dir: &Path) -> Result<ExitCode, BoxError> {
    fs::create_dir_all(output_dir)?;
    write_json(
        output_dir.join("prometheus-exec.openapi.json"),
        &openapi_components(),
    )?;
    write_json(
        output_dir.join("prometheus-exec.schemas.json"),
        &contract_schemas(),
    )?;
    write_json(
        output_dir.join("prometheus-exec.mcp.json"),
        &mcp::tool_contracts(),
    )?;
    Ok(ExitCode::SUCCESS)
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, BoxError> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn write_json(path: PathBuf, value: &impl Serialize) -> Result<(), BoxError> {
    let mut value = serde_json::to_value(value)?;
    sort_json_keys(&mut value);
    let mut bytes = serde_json::to_vec_pretty(&value)?;
    bytes.push(b'\n');
    fs::write(path, bytes)?;
    Ok(())
}

fn sort_json_keys(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(object) => {
            let mut entries: Vec<_> = std::mem::take(object).into_iter().collect();
            entries.sort_by(|(left, _), (right, _)| left.cmp(right));
            for (key, mut child) in entries {
                sort_json_keys(&mut child);
                object.insert(key, child);
            }
        }
        serde_json::Value::Array(values) => {
            for child in values {
                sort_json_keys(child);
            }
        }
        _ => {}
    }
}

fn mebibytes(value: u64) -> Result<u64, BoxError> {
    value
        .checked_mul(1024 * 1024)
        .ok_or_else(|| "MiB value overflowed u64".into())
}

fn http_error(status: u16, body: &[u8]) -> String {
    format!(
        "sidecar returned HTTP {status}: {}",
        String::from_utf8_lossy(body)
    )
}

#[cfg(test)]
mod pin_tests {
    use super::*;

    #[test]
    fn upload_pin_rollback_releases_every_materialized_hash() {
        let root = tempfile::tempdir().unwrap();
        let artifacts = ArtifactStore::open(root.path(), 1).unwrap();
        let reason = "upload:fixture";
        let first = artifacts.put_pinned(b"first", reason).unwrap();
        let second = artifacts.put_pinned(b"second", reason).unwrap();

        release_pins(
            &artifacts,
            &[first.hash.clone(), second.hash.clone()],
            reason,
        )
        .unwrap();

        assert!(!artifacts.is_pinned(&first.hash).unwrap());
        assert!(!artifacts.is_pinned(&second.hash).unwrap());
    }

    #[test]
    fn portable_bundle_verifier_is_exposed_without_runtime_arguments() {
        let cli = Cli::try_parse_from([
            "prometheus-exec",
            "verify-bundle",
            "--index",
            "bundle/index.json",
            "--format",
            "json",
        ])
        .unwrap();
        assert!(matches!(
            cli.command,
            Command::VerifyBundle {
                index,
                root: None,
                format: OutputFormat::Json,
            } if index == Path::new("bundle/index.json")
        ));
    }
}
