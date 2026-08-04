use std::{
    fs,
    io::{self, Write as _},
    path::{Path, PathBuf},
    process::ExitCode,
};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use clap::{Parser, Subcommand, ValueEnum};
use ed25519_dalek::SigningKey;
use prometheus_exec_contracts::{
    contract_schemas, openapi_components, verify_receipt, ExecutionReceipt, SignatureAlgorithm,
    SignedExecRequest, VerificationKey,
};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IdentityFile {
    schema_version: String,
    sig_alg: SignatureAlgorithm,
    key_id: String,
    public_key: String,
    private_key: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PublicIdentity<'a> {
    sig_alg: SignatureAlgorithm,
    key_id: &'a str,
    public_key: &'a str,
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("prometheus-exec: {error}");
            ExitCode::from(2)
        }
    }
}

fn run(cli: Cli) -> Result<ExitCode, Box<dyn std::error::Error>> {
    match cli.command {
        Command::Init { identity } => init_identity(&identity),
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

fn init_identity(path: &Path) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;

    let key = SigningKey::generate(&mut OsRng);
    let public = VerificationKey::ed25519(key.verifying_key().to_bytes());
    let public_key = public.to_base64url();
    let identity = IdentityFile {
        schema_version: "1".into(),
        sig_alg: SignatureAlgorithm::Ed25519,
        key_id: public.key_id(),
        public_key,
        private_key: URL_SAFE_NO_PAD.encode(key.to_bytes()),
    };
    let mut bytes = serde_json::to_vec_pretty(&identity)?;
    bytes.push(b'\n');

    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    set_private_permissions(temporary.as_file())?;
    temporary.write_all(&bytes)?;
    temporary.as_file().sync_all()?;
    temporary.persist_noclobber(path)?;

    let public = PublicIdentity {
        sig_alg: identity.sig_alg,
        key_id: &identity.key_id,
        public_key: &identity.public_key,
    };
    serde_json::to_writer_pretty(io::stdout().lock(), &public)?;
    println!();
    Ok(ExitCode::SUCCESS)
}

#[cfg(unix)]
fn set_private_permissions(file: &fs::File) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt as _;
    file.set_permissions(fs::Permissions::from_mode(0o600))
}

#[cfg(not(unix))]
fn set_private_permissions(_file: &fs::File) -> io::Result<()> {
    Ok(())
}

fn verify_command(
    receipt_path: &Path,
    public_key: &str,
    request_path: Option<&Path>,
    artifact_root: Option<&Path>,
    format: OutputFormat,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
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

fn generate_contracts(output_dir: &Path) -> Result<ExitCode, Box<dyn std::error::Error>> {
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

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, Box<dyn std::error::Error>> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn write_json(path: PathBuf, value: &impl Serialize) -> Result<(), Box<dyn std::error::Error>> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    fs::write(path, bytes)?;
    Ok(())
}
