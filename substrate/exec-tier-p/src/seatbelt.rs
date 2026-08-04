use std::{
    collections::{BTreeMap, BTreeSet},
    io,
    os::unix::{fs::PermissionsExt, process::CommandExt as _},
    path::{Component, Path, PathBuf},
    process::ExitStatus,
    sync::Arc,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use chrono::Utc;
use nix::{
    errno::Errno,
    sys::signal::{killpg, Signal},
    unistd::Pid,
};
use prometheus_exec_contracts::{
    hash_bytes, Digest, EvidenceClass, ExecutionBackend, ExecutionExit, ExecutionTier,
    RequestedTier, ResourceUsage, RunState, RuntimeKind,
};
use prometheus_exec_core::{
    BackendExecution, ExecutionPort, ProducedArtifact, ValidatedExecutionJob,
};
use sha2::{Digest as _, Sha256};
use tempfile::TempDir;
use thiserror::Error;
use tokio::{
    fs,
    io::{AsyncRead, AsyncReadExt},
    process::{Child, Command},
    sync::{mpsc, Mutex},
    task::JoinHandle,
};
use walkdir::WalkDir;

const STREAM_CHUNK_BYTES: usize = 16 * 1024;

/// Host paths and explicitly trusted environment values used by Seatbelt.
#[derive(Clone, Debug)]
pub struct SeatbeltConfig {
    sandbox_exec: PathBuf,
    python3: PathBuf,
    node: Option<PathBuf>,
    bash: PathBuf,
    system_read_roots: BTreeSet<PathBuf>,
    allowed_environment: BTreeMap<String, String>,
    work_root: Option<PathBuf>,
}

impl SeatbeltConfig {
    /// Discovers the fixed macOS sandbox tool and installed native runtimes.
    pub fn detect() -> Result<Self, SeatbeltError> {
        let sandbox_exec = PathBuf::from("/usr/bin/sandbox-exec");
        require_executable(&sandbox_exec, "Seatbelt launcher")?;

        let system_read_roots = [
            "/System",
            "/usr",
            "/bin",
            "/sbin",
            "/Library",
            "/private/var/db",
            "/dev",
        ]
        .into_iter()
        .map(PathBuf::from)
        .map(|path| path.canonicalize().unwrap_or(path))
        .collect();

        Ok(Self {
            sandbox_exec,
            python3: PathBuf::from("/usr/bin/python3"),
            node: discover_node_runtime(),
            bash: PathBuf::from("/bin/bash"),
            system_read_roots,
            allowed_environment: BTreeMap::new(),
            work_root: None,
        })
    }

    /// Overrides one runtime with an absolute executable path.
    pub fn with_runtime(
        mut self,
        runtime: RuntimeKind,
        path: impl Into<PathBuf>,
    ) -> Result<Self, SeatbeltError> {
        let path = canonical_executable(path.into(), "runtime")?;
        match runtime {
            RuntimeKind::Python3 => self.python3 = path,
            RuntimeKind::Node => self.node = Some(path),
            RuntimeKind::Bash => self.bash = path,
            RuntimeKind::WasmComponent => return Err(SeatbeltError::UnsupportedRuntime(runtime)),
        }
        Ok(self)
    }

    /// Makes one trusted host value eligible for an explicit request.
    pub fn allow_environment(
        mut self,
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self, SeatbeltError> {
        let name = name.into();
        validate_environment_name(&name)?;
        self.allowed_environment.insert(name, value.into());
        Ok(self)
    }

    /// Places private run directories beneath an operator-selected root.
    pub fn with_work_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.work_root = Some(root.into());
        self
    }
}

/// Exact Seatbelt source and the digest recorded in an attested receipt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeatbeltProfile {
    source: String,
    hash: Digest,
}

impl SeatbeltProfile {
    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn hash(&self) -> &Digest {
        &self.hash
    }

    fn generate(
        run_root: &Path,
        output_root: &Path,
        runtime: &Path,
        system_read_roots: &BTreeSet<PathBuf>,
    ) -> Result<Self, SeatbeltError> {
        let mut read_roots = system_read_roots.clone();
        read_roots.insert(run_root.to_path_buf());
        let read_ancestors: BTreeSet<_> = read_roots
            .iter()
            .flat_map(|root| root.ancestors().skip(1).map(Path::to_path_buf))
            .collect();

        let mut source = String::from(
            "(version 1)\n(deny default)\n(allow process*)\n(allow sysctl-read)\n(allow mach-lookup)\n",
        );
        for ancestor in read_ancestors {
            source.push_str(&format!(
                "(allow file-read* (literal \"{}\"))\n",
                seatbelt_literal(&ancestor)?
            ));
        }
        for root in read_roots {
            source.push_str(&format!(
                "(allow file-read* (subpath \"{}\"))\n",
                seatbelt_literal(&root)?
            ));
        }
        source.push_str(&format!(
            "(allow file-read* (literal \"{}\"))\n",
            seatbelt_literal(runtime)?
        ));
        source.push_str(&format!(
            "(allow file-write* (subpath \"{}\"))\n",
            seatbelt_literal(output_root)?
        ));
        source.push_str("(allow file-write-data (literal \"/dev/null\"))\n");

        let hash = hash_bytes(source.as_bytes());
        Ok(Self { source, hash })
    }
}

/// Fail-closed macOS native execution adapter.
#[derive(Clone, Debug)]
pub struct SeatbeltExecutor {
    config: SeatbeltConfig,
}

impl SeatbeltExecutor {
    pub fn new(config: SeatbeltConfig) -> Self {
        Self { config }
    }

    fn runtime(&self, runtime: RuntimeKind) -> Result<PathBuf, SeatbeltError> {
        if runtime == RuntimeKind::WasmComponent {
            return Err(SeatbeltError::UnsupportedRuntime(runtime));
        }
        let configured = match runtime {
            RuntimeKind::Python3 => &self.config.python3,
            RuntimeKind::Node => self
                .config
                .node
                .as_ref()
                .ok_or(SeatbeltError::RuntimeUnavailable(runtime))?,
            RuntimeKind::Bash => &self.config.bash,
            RuntimeKind::WasmComponent => unreachable!("Wasm is rejected above"),
        };
        canonical_executable(configured.to_path_buf(), "runtime")
    }

    fn requested_environment(
        &self,
        job: &ValidatedExecutionJob,
    ) -> Result<BTreeMap<String, String>, SeatbeltError> {
        let mut environment = BTreeMap::new();
        for name in &job.request().capabilities.env.read {
            validate_environment_name(name)?;
            let value = self
                .config
                .allowed_environment
                .get(name)
                .ok_or_else(|| SeatbeltError::EnvironmentUnavailable(name.clone()))?;
            environment.insert(name.clone(), value.clone());
        }
        Ok(environment)
    }

    fn validate_capabilities(&self, job: &ValidatedExecutionJob) -> Result<(), SeatbeltError> {
        let request = job.request();
        if request.tier == RequestedTier::W {
            return Err(SeatbeltError::TierMismatch);
        }
        if !request.capabilities.net.egress.is_empty() {
            return Err(SeatbeltError::CapabilityUnavailable(
                "Seatbelt Tier P currently supports deny-all networking only".into(),
            ));
        }
        if !request.capabilities.clock || !request.capabilities.random {
            return Err(SeatbeltError::CapabilityUnavailable(
                "Seatbelt cannot attest denial of clock or kernel randomness".into(),
            ));
        }

        for path in &request.capabilities.fs.read_write {
            if !is_output_scoped(path) {
                return Err(SeatbeltError::CapabilityUnavailable(format!(
                    "write path is not output-scoped: {path}"
                )));
            }
        }
        for path in &request.capabilities.fs.read_only {
            if path != "." && !job.inputs().contains_key(path) {
                return Err(SeatbeltError::CapabilityUnavailable(format!(
                    "read path is not a declared input: {path}"
                )));
            }
        }
        self.requested_environment(job)?;
        Ok(())
    }

    async fn execute_inner(
        &self,
        job: &ValidatedExecutionJob,
    ) -> Result<BackendExecution, SeatbeltError> {
        self.validate_capabilities(job)?;
        require_executable(&self.config.sandbox_exec, "Seatbelt launcher")?;
        let runtime = self.runtime(job.request().code.runtime)?;
        let toolchain_hash = hash_file(&runtime).await?;
        if let Some(expected) = &job.request().code.toolchain_pin {
            if expected != &toolchain_hash {
                return Err(SeatbeltError::ToolchainMismatch {
                    expected: expected.clone(),
                    observed: toolchain_hash,
                });
            }
        }

        let work = create_work_dir(self.config.work_root.as_deref())?;
        // Seatbelt aborts instead of returning a parse error for some profile
        // filters containing symlinked path prefixes (notably `/var` and
        // `/tmp` on macOS). Profile and command paths therefore use the same
        // canonical identity.
        let run_root = work.path().canonicalize()?;
        let input_root = run_root.join("inputs");
        let output_root = run_root.join("outputs");
        fs::create_dir_all(&input_root).await?;
        fs::create_dir_all(&output_root).await?;
        fs::create_dir_all(output_root.join("tmp")).await?;
        fs::set_permissions(&output_root, std::fs::Permissions::from_mode(0o700)).await?;

        for (name, bytes) in job.inputs() {
            let relative = validate_relative_path(name)?;
            let destination = input_root.join(relative);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent).await?;
            }
            write_read_only(&destination, bytes).await?;
        }

        let code_path = run_root.join(code_filename(job.request().code.runtime));
        write_read_only(&code_path, job.code()).await?;
        let profile = SeatbeltProfile::generate(
            &run_root,
            &output_root,
            &runtime,
            &self.config.system_read_roots,
        )?;
        let environment = self.requested_environment(job)?;
        let output_limit = mebibytes_to_usize(job.request().limits.output_mb);

        let mut command = Command::new(&self.config.sandbox_exec);
        command
            .arg("-p")
            .arg(profile.source())
            .arg(&runtime)
            .arg(&code_path)
            .current_dir(&run_root)
            .env_clear()
            .env("HOME", &run_root)
            .env("TMPDIR", output_root.join("tmp"))
            .env("PROMETHEUS_INPUT_DIR", &input_root)
            .env("PROMETHEUS_OUTPUT_DIR", &output_root)
            .env("PYTHONDONTWRITEBYTECODE", "1")
            .envs(&environment)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);
        command.as_std_mut().process_group(0);

        let started_at = Utc::now();
        let started = Instant::now();
        let mut child = command.spawn().map_err(SeatbeltError::Spawn)?;
        let process_group = process_group_id(&child)?;
        let stdout = child.stdout.take().ok_or(SeatbeltError::MissingPipe)?;
        let stderr = child.stderr.take().ok_or(SeatbeltError::MissingPipe)?;
        let (overflow_tx, mut overflow_rx) = mpsc::channel(1);
        let output_counter = Arc::new(Mutex::new(0usize));
        let stdout_task = tokio::spawn(read_bounded(
            stdout,
            output_limit,
            Arc::clone(&output_counter),
            overflow_tx.clone(),
        ));
        let stderr_task = tokio::spawn(read_bounded(
            stderr,
            output_limit,
            output_counter,
            overflow_tx,
        ));

        let timeout = Duration::from_millis(job.request().limits.wall_clock_ms);
        let termination = tokio::select! {
            status = child.wait() => Termination::Exited(status.map_err(SeatbeltError::Wait)?),
            _ = tokio::time::sleep(timeout) => Termination::TimedOut,
            Some(()) = overflow_rx.recv() => Termination::OutputExceeded,
        };
        let (status, forced_trap) =
            finish_process_group(&mut child, process_group, termination).await?;
        let stdout = join_output(stdout_task).await?;
        let stderr = join_output(stderr_task).await?;

        let stream_bytes = stdout.len().saturating_add(stderr.len());
        let artifact_budget = output_limit.saturating_sub(stream_bytes);
        let (artifacts, artifact_trap) =
            match collect_artifacts(&output_root, artifact_budget).await {
                Ok(artifacts) => (artifacts, None),
                Err(error) => (Vec::new(), Some(error.to_string())),
            };
        let trap = forced_trap.or(artifact_trap);
        let state = if status.success() && trap.is_none() {
            RunState::Succeeded
        } else {
            RunState::Failed
        };
        let platform = format!("macos-{}", std::env::consts::ARCH);
        let finished_at = Utc::now();
        let wall_clock_ms = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;

        Ok(BackendExecution {
            state,
            evidence_class: EvidenceClass::Attested,
            tier: ExecutionTier::P,
            sandbox_profile_hash: profile.hash().clone(),
            backend: ExecutionBackend::Seatbelt,
            exit: ExecutionExit {
                status: exit_status_code(&status),
                signal_or_trap: trap.or_else(|| exit_signal(&status)),
            },
            stdout,
            stderr,
            artifacts,
            usage: ResourceUsage {
                wall_clock_ms,
                ..ResourceUsage::default()
            },
            started_at,
            finished_at,
            toolchain_hash: Some(toolchain_hash),
            environment,
            platform,
        })
    }
}

#[async_trait]
impl ExecutionPort for SeatbeltExecutor {
    type Error = SeatbeltError;

    fn tier(&self) -> ExecutionTier {
        ExecutionTier::P
    }

    async fn execute(&self, job: &ValidatedExecutionJob) -> Result<BackendExecution, Self::Error> {
        self.execute_inner(job).await
    }
}

#[derive(Debug, Error)]
pub enum SeatbeltError {
    #[error("Seatbelt launcher is unavailable: {0}")]
    SandboxUnavailable(String),
    #[error("runtime is unsupported by Tier P: {0:?}")]
    UnsupportedRuntime(RuntimeKind),
    #[error("runtime is unavailable: {0:?}")]
    RuntimeUnavailable(RuntimeKind),
    #[error("requested tier does not permit Tier P execution")]
    TierMismatch,
    #[error("requested capability cannot be enforced: {0}")]
    CapabilityUnavailable(String),
    #[error("requested environment value is not configured: {0}")]
    EnvironmentUnavailable(String),
    #[error("invalid environment name: {0}")]
    InvalidEnvironmentName(String),
    #[error("invalid relative input path: {0}")]
    InvalidInputPath(String),
    #[error("invalid path for Seatbelt profile: {0}")]
    InvalidProfilePath(String),
    #[error("toolchain pin mismatch: expected {expected}, observed {observed}")]
    ToolchainMismatch { expected: Digest, observed: Digest },
    #[error("digest construction failed: {0}")]
    Digest(String),
    #[error("failed to spawn sandboxed process: {0}")]
    Spawn(io::Error),
    #[error("failed waiting for sandboxed process: {0}")]
    Wait(io::Error),
    #[error("sandboxed process did not expose its output pipes")]
    MissingPipe,
    #[error("sandboxed process has no process ID")]
    MissingProcessId,
    #[error("failed to terminate sandbox process group: {0}")]
    ProcessGroup(Errno),
    #[error("output reader task failed: {0}")]
    OutputTask(String),
    #[error("filesystem operation failed: {0}")]
    Io(#[from] io::Error),
}

#[derive(Debug)]
enum Termination {
    Exited(ExitStatus),
    TimedOut,
    OutputExceeded,
}

fn create_work_dir(root: Option<&Path>) -> Result<TempDir, SeatbeltError> {
    let mut builder = tempfile::Builder::new();
    builder.prefix("prometheus-exec-");
    let work = match root {
        Some(root) => {
            std::fs::create_dir_all(root)?;
            builder.tempdir_in(root)?
        }
        None => builder.tempdir()?,
    };
    std::fs::set_permissions(work.path(), std::fs::Permissions::from_mode(0o700))?;
    Ok(work)
}

fn canonical_executable(path: PathBuf, label: &str) -> Result<PathBuf, SeatbeltError> {
    require_executable(&path, label)?;
    path.canonicalize().map_err(SeatbeltError::Io)
}

fn require_executable(path: &Path, label: &str) -> Result<(), SeatbeltError> {
    let metadata = std::fs::metadata(path).map_err(|error| {
        SeatbeltError::SandboxUnavailable(format!("{label} {}: {error}", path.display()))
    })?;
    if !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
        return Err(SeatbeltError::SandboxUnavailable(format!(
            "{label} {} is not executable",
            path.display()
        )));
    }
    Ok(())
}

fn discover_node_runtime() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(path) = std::env::var_os("PATH") {
        candidates.extend(std::env::split_paths(&path).map(|directory| directory.join("node")));
    }
    if let Some(volta_home) = std::env::var_os("VOLTA_HOME") {
        let image_root = PathBuf::from(volta_home).join("tools/image/node");
        if let Ok(versions) = std::fs::read_dir(image_root) {
            let mut installed: Vec<_> = versions
                .filter_map(Result::ok)
                .map(|entry| entry.path().join("bin/node"))
                .collect();
            installed.sort();
            installed.reverse();
            candidates.extend(installed);
        }
    }

    candidates.into_iter().find_map(|candidate| {
        let candidate = canonical_executable(candidate, "Node runtime").ok()?;
        let output = std::process::Command::new(&candidate)
            .args(["-p", "process.execPath"])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let path = std::str::from_utf8(&output.stdout).ok()?.trim();
        if path.is_empty() {
            return None;
        }
        canonical_executable(PathBuf::from(path), "Node runtime").ok()
    })
}

fn validate_environment_name(name: &str) -> Result<(), SeatbeltError> {
    let mut bytes = name.bytes();
    let Some(first) = bytes.next() else {
        return Err(SeatbeltError::InvalidEnvironmentName(name.into()));
    };
    if !(first == b'_' || first.is_ascii_alphabetic())
        || !bytes.all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
    {
        return Err(SeatbeltError::InvalidEnvironmentName(name.into()));
    }
    Ok(())
}

fn validate_relative_path(path: &str) -> Result<PathBuf, SeatbeltError> {
    let path = Path::new(path);
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(SeatbeltError::InvalidInputPath(
            path.to_string_lossy().into_owned(),
        ));
    }
    Ok(path.to_path_buf())
}

fn is_output_scoped(path: &str) -> bool {
    let path = path.replace('\\', "/");
    let path = path.trim_end_matches('/');
    let mut components = path.split('/');
    matches!(components.next(), Some("outputs"))
        && components
            .all(|component| !component.is_empty() && component != "." && component != "..")
}

fn seatbelt_literal(path: &Path) -> Result<String, SeatbeltError> {
    let value = path
        .to_str()
        .ok_or_else(|| SeatbeltError::InvalidProfilePath(path.display().to_string()))?;
    if value.chars().any(char::is_control) {
        return Err(SeatbeltError::InvalidProfilePath(value.into()));
    }
    Ok(value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn code_filename(runtime: RuntimeKind) -> &'static str {
    match runtime {
        RuntimeKind::Python3 => "program.py",
        RuntimeKind::Node => "program.js",
        RuntimeKind::Bash => "program.sh",
        RuntimeKind::WasmComponent => "program.wasm",
    }
}

async fn write_read_only(path: &Path, bytes: &[u8]) -> Result<(), SeatbeltError> {
    fs::write(path, bytes).await?;
    fs::set_permissions(path, std::fs::Permissions::from_mode(0o400)).await?;
    Ok(())
}

async fn hash_file(path: &Path) -> Result<Digest, SeatbeltError> {
    let mut file = fs::File::open(path).await?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; STREAM_CHUNK_BYTES];
    loop {
        let read = file.read(&mut buffer).await?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Digest::parse(format!("sha256:{:x}", hasher.finalize()))
        .map_err(|error| SeatbeltError::Digest(error.to_string()))
}

fn mebibytes_to_usize(mebibytes: u64) -> usize {
    let bytes = mebibytes.saturating_mul(1024 * 1024);
    usize::try_from(bytes).unwrap_or(usize::MAX)
}

async fn read_bounded<R>(
    mut reader: R,
    limit: usize,
    counter: Arc<Mutex<usize>>,
    overflow: mpsc::Sender<()>,
) -> io::Result<Vec<u8>>
where
    R: AsyncRead + Unpin,
{
    let mut captured = Vec::new();
    let mut buffer = vec![0u8; STREAM_CHUNK_BYTES];
    let mut reported_overflow = false;
    loop {
        let read = reader.read(&mut buffer).await?;
        if read == 0 {
            break;
        }
        let accepted = {
            let mut total = counter.lock().await;
            let remaining = limit.saturating_sub(*total);
            let accepted = read.min(remaining);
            *total = (*total).saturating_add(read);
            accepted
        };
        captured.extend_from_slice(&buffer[..accepted]);
        if accepted < read && !reported_overflow {
            reported_overflow = true;
            let _ = overflow.try_send(());
        }
    }
    Ok(captured)
}

fn process_group_id(child: &Child) -> Result<Pid, SeatbeltError> {
    child
        .id()
        .and_then(|id| i32::try_from(id).ok())
        .map(Pid::from_raw)
        .ok_or(SeatbeltError::MissingProcessId)
}

async fn finish_process_group(
    child: &mut Child,
    process_group: Pid,
    termination: Termination,
) -> Result<(ExitStatus, Option<String>), SeatbeltError> {
    let (status, trap) = match termination {
        Termination::Exited(status) => {
            terminate_process_group(process_group)?;
            (status, None)
        }
        Termination::TimedOut => {
            terminate_process_group(process_group)?;
            (
                child.wait().await.map_err(SeatbeltError::Wait)?,
                Some("wall_clock_timeout".into()),
            )
        }
        Termination::OutputExceeded => {
            terminate_process_group(process_group)?;
            (
                child.wait().await.map_err(SeatbeltError::Wait)?,
                Some("output_limit_exceeded".into()),
            )
        }
    };
    Ok((status, trap))
}

fn terminate_process_group(process_group: Pid) -> Result<(), SeatbeltError> {
    match killpg(process_group, Signal::SIGKILL) {
        Ok(()) | Err(Errno::ESRCH) => Ok(()),
        Err(error) => Err(SeatbeltError::ProcessGroup(error)),
    }
}

async fn join_output(task: JoinHandle<io::Result<Vec<u8>>>) -> Result<Vec<u8>, SeatbeltError> {
    task.await
        .map_err(|error| SeatbeltError::OutputTask(error.to_string()))?
        .map_err(SeatbeltError::Io)
}

async fn collect_artifacts(
    output_root: &Path,
    limit: usize,
) -> Result<Vec<ProducedArtifact>, ArtifactViolation> {
    let root = output_root.to_path_buf();
    tokio::task::spawn_blocking(move || collect_artifacts_sync(&root, limit))
        .await
        .map_err(|error| ArtifactViolation::Walk(error.to_string()))?
}

fn collect_artifacts_sync(
    output_root: &Path,
    limit: usize,
) -> Result<Vec<ProducedArtifact>, ArtifactViolation> {
    let mut paths = Vec::new();
    for entry in WalkDir::new(output_root).follow_links(false) {
        let entry = entry.map_err(|error| ArtifactViolation::Walk(error.to_string()))?;
        if entry.path() == output_root || entry.file_type().is_dir() {
            continue;
        }
        if entry.file_type().is_symlink() || !entry.file_type().is_file() {
            return Err(ArtifactViolation::UnsafeType(
                entry.path().display().to_string(),
            ));
        }
        paths.push(entry.into_path());
    }
    paths.sort();

    let mut total = 0usize;
    let mut artifacts = Vec::new();
    for path in paths {
        let relative = path
            .strip_prefix(output_root)
            .map_err(|_| ArtifactViolation::Escaped(path.display().to_string()))?;
        let relative = relative
            .to_str()
            .ok_or_else(|| ArtifactViolation::NonUtf8(path.display().to_string()))?
            .replace(std::path::MAIN_SEPARATOR, "/");
        let metadata =
            std::fs::metadata(&path).map_err(|error| ArtifactViolation::Walk(error.to_string()))?;
        let size = usize::try_from(metadata.len()).unwrap_or(usize::MAX);
        total = total
            .checked_add(size)
            .ok_or(ArtifactViolation::OutputLimitExceeded)?;
        if total > limit {
            return Err(ArtifactViolation::OutputLimitExceeded);
        }
        let bytes =
            std::fs::read(&path).map_err(|error| ArtifactViolation::Walk(error.to_string()))?;
        artifacts.push(ProducedArtifact {
            path: format!("outputs/{relative}"),
            bytes,
        });
    }
    Ok(artifacts)
}

#[derive(Debug, Error)]
enum ArtifactViolation {
    #[error("artifact traversal failed: {0}")]
    Walk(String),
    #[error("artifact has an unsafe file type: {0}")]
    UnsafeType(String),
    #[error("artifact escaped the output directory: {0}")]
    Escaped(String),
    #[error("artifact path is not UTF-8: {0}")]
    NonUtf8(String),
    #[error("artifact output limit exceeded")]
    OutputLimitExceeded,
}

#[cfg(unix)]
fn exit_signal(status: &ExitStatus) -> Option<String> {
    use std::os::unix::process::ExitStatusExt as _;
    status.signal().map(|signal| format!("signal:{signal}"))
}

fn exit_status_code(status: &ExitStatus) -> i32 {
    status.code().unwrap_or(-1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_is_deterministic_and_hashes_exact_source() {
        let roots = [PathBuf::from("/System"), PathBuf::from("/usr")]
            .into_iter()
            .collect();
        let first = SeatbeltProfile::generate(
            Path::new("/private/tmp/run"),
            Path::new("/private/tmp/run/outputs"),
            Path::new("/bin/bash"),
            &roots,
        )
        .unwrap();
        let second = SeatbeltProfile::generate(
            Path::new("/private/tmp/run"),
            Path::new("/private/tmp/run/outputs"),
            Path::new("/bin/bash"),
            &roots,
        )
        .unwrap();

        assert_eq!(first, second);
        assert_eq!(first.hash(), &hash_bytes(first.source().as_bytes()));
        assert!(first.source().starts_with("(version 1)\n(deny default)"));
        assert!(!first.source().contains("(allow network"));
    }

    #[test]
    fn profile_literals_escape_quotes_and_backslashes() {
        let escaped = seatbelt_literal(Path::new("/tmp/a\\b\"c")).unwrap();
        assert_eq!(escaped, "/tmp/a\\\\b\\\"c");
    }

    #[test]
    fn relative_input_paths_reject_traversal() {
        assert!(validate_relative_path("nested/data.json").is_ok());
        assert!(validate_relative_path("../secret").is_err());
        assert!(validate_relative_path("/absolute").is_err());
        assert!(validate_relative_path("./relative").is_err());
    }
}
