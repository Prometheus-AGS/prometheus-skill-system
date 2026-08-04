mod daemon;
mod doctor;
mod identity;
mod uds_client;

use std::{
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
    contract_schemas, openapi_components, sign_request_ed25519, verify_receipt, CapabilityManifest,
    CodeIdentity, CodeKind, ExecutionLimits, ExecutionProvenance, ExecutionReceipt, NamedInput,
    RequestedTier, RunState, RuntimeKind, SignatureAlgorithm, SignedExecRequest, VerificationKey,
    SCHEMA_VERSION,
};
use prometheus_exec_core::ArtifactStore;
use prometheus_exec_service::RunResponse;
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
    /// Run the health-first local sidecar and Tier P worker.
    Daemon {
        #[arg(long)]
        socket: PathBuf,
        #[arg(long)]
        state_dir: PathBuf,
        #[arg(long)]
        identity: PathBuf,
        #[arg(long, default_value_t = 2048)]
        artifact_budget_mb: u64,
    },
    /// Submit one signed Tier P execution and wait for its durable receipt.
    Run {
        #[arg(long)]
        socket: PathBuf,
        #[arg(long)]
        state_dir: PathBuf,
        #[arg(long)]
        identity: PathBuf,
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
        #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,
    },
    /// Generate deterministic JSON Schema and OpenAPI components.
    Contracts {
        #[arg(long)]
        output_dir: PathBuf,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OutputFormat {
    Human,
    Json,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum RuntimeArg {
    Python3,
    Node,
    Bash,
}

impl From<RuntimeArg> for RuntimeKind {
    fn from(value: RuntimeArg) -> Self {
        match value {
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
    match run(Cli::parse()).await {
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
            artifact_budget_mb,
        } => {
            daemon::run(daemon::DaemonConfig {
                socket,
                state_dir,
                identity,
                artifact_budget_bytes: mebibytes(artifact_budget_mb)?,
            })
            .await?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Run {
            socket,
            state_dir,
            identity,
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
            format,
        } => doctor_command(socket, state_dir, identity, format).await,
        Command::Verify {
            receipt,
            public_key,
            request,
            artifacts,
            format,
        } => verify_command(
            &receipt,
            &public_key,
            request.as_deref(),
            artifacts.as_deref(),
            format,
        ),
        Command::Contracts { output_dir } => generate_contracts(&output_dir),
    }
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
    let code = fs::read(&command.code)?;
    let stored_code = artifacts.put(&code)?;
    let mut named_inputs = Vec::new();
    for input in command.inputs {
        let (name, path) = input
            .split_once('=')
            .ok_or("input must use NAME=PATH syntax")?;
        if name.is_empty() {
            return Err("input name cannot be empty".into());
        }
        let stored = artifacts.put(&fs::read(path)?)?;
        named_inputs.push(NamedInput {
            name: name.into(),
            hash: stored.hash,
        });
    }
    named_inputs.sort_by(|left, right| left.name.cmp(&right.name));
    let now = Utc::now();
    let mut request = SignedExecRequest {
        schema_version: SCHEMA_VERSION.into(),
        request_id: Uuid::new_v4(),
        issued_at: now,
        queued_at: Some(now),
        validity_window_secs: command.timeout_ms.saturating_add(60_000).div_ceil(1000),
        tier: RequestedTier::P,
        code: CodeIdentity {
            kind: CodeKind::File,
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
            ..ExecutionProvenance::default()
        },
        signer_key_id: None,
        sig_alg: SignatureAlgorithm::Ed25519,
        signature: None,
    };
    sign_request_ed25519(&mut request, &identity.signing_key)?;
    let response = uds_client::request(
        &command.socket,
        Method::POST,
        "/api/v2/exec/runs",
        serde_json::to_vec(&request)?,
    )
    .await?;
    if !matches!(response.status, 200 | 202) {
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
    socket: PathBuf,
    state_dir: PathBuf,
    identity: PathBuf,
    format: OutputFormat,
) -> Result<ExitCode, BoxError> {
    let report = doctor::inspect(doctor::DoctorConfig {
        socket,
        state_dir,
        identity,
    })
    .await;
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

fn verify_command(
    receipt_path: &Path,
    public_key: &str,
    request_path: Option<&Path>,
    artifact_root: Option<&Path>,
    format: OutputFormat,
) -> Result<ExitCode, BoxError> {
    let receipt: ExecutionReceipt = read_json(receipt_path)?;
    let key = VerificationKey::from_base64url(receipt.executing_device.sig_alg, public_key)?;
    let request = request_path
        .map(read_json::<SignedExecRequest>)
        .transpose()?;
    let result = verify_receipt(&receipt, &key, request.as_ref(), artifact_root);
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
        }
        OutputFormat::Human => {
            println!("INVALID");
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
