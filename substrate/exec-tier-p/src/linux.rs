use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Component, Path, PathBuf},
};

use prometheus_exec_contracts::{hash_serializable, Digest, RequestedTier};
use prometheus_exec_core::ValidatedExecutionJob;
use serde::{Deserialize, Serialize};
use thiserror::Error;

const BWRAP_PROFILE_VERSION: &str = "prometheus-bwrap-v1";
const SANDBOX_ROOT: &str = "/work";
const SANDBOX_INPUTS: &str = "/work/inputs";
const SANDBOX_OUTPUTS: &str = "/work/outputs";

/// Immutable bubblewrap installation identity and read-only system roots.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BwrapConfig {
    executable: PathBuf,
    version: String,
    system_read_roots: BTreeSet<PathBuf>,
}

impl BwrapConfig {
    /// Creates a deterministic configuration from an already verified binary.
    pub fn new(
        executable: impl Into<PathBuf>,
        version: impl Into<String>,
        system_read_roots: impl IntoIterator<Item = PathBuf>,
    ) -> Result<Self, LinuxSandboxError> {
        let executable = validate_absolute_path(executable.into(), "bwrap executable")?;
        let version = version.into();
        if version.trim().is_empty() || version.chars().any(char::is_control) {
            return Err(LinuxSandboxError::InvalidVersion(version));
        }
        let system_read_roots = system_read_roots
            .into_iter()
            .map(|root| validate_absolute_path(root, "system read root"))
            .collect::<Result<_, _>>()?;
        Ok(Self {
            executable,
            version,
            system_read_roots,
        })
    }

    /// Finds bubblewrap and records its reported version on Linux.
    #[cfg(target_os = "linux")]
    pub fn detect() -> Result<Self, LinuxSandboxError> {
        let executable = discover_bwrap().ok_or_else(|| LinuxSandboxError::TierUnavailable {
            reason: "bubblewrap was not found; Landlock fallback is not runtime-certified".into(),
        })?;
        let output = std::process::Command::new(&executable)
            .arg("--version")
            .output()
            .map_err(|error| LinuxSandboxError::BwrapProbe(error.to_string()))?;
        if !output.status.success() {
            return Err(LinuxSandboxError::BwrapProbe(format!(
                "{} --version exited with {}",
                executable.display(),
                output.status
            )));
        }
        let version = std::str::from_utf8(&output.stdout)
            .map_err(|error| LinuxSandboxError::BwrapProbe(error.to_string()))?
            .trim()
            .to_owned();
        Self::new(executable, version, default_linux_read_roots())
    }

    /// Non-Linux hosts cannot select a Tier P Linux backend.
    #[cfg(not(target_os = "linux"))]
    pub fn detect() -> Result<Self, LinuxSandboxError> {
        Err(LinuxSandboxError::UnsupportedPlatform)
    }

    pub fn executable(&self) -> &Path {
        &self.executable
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    /// Builds, but does not spawn, the exact fail-closed bubblewrap command.
    pub fn plan(
        &self,
        job: &ValidatedExecutionJob,
        run_root: &Path,
        output_root: &Path,
        runtime: &Path,
        code_path: &Path,
        environment: &BTreeMap<String, String>,
    ) -> Result<BwrapPlan, LinuxSandboxError> {
        validate_capabilities(job, environment)?;
        let run_root = validate_absolute_path(run_root.to_path_buf(), "run root")?;
        let output_root = validate_absolute_path(output_root.to_path_buf(), "output root")?;
        let runtime = validate_absolute_path(runtime.to_path_buf(), "runtime")?;
        let code_path = validate_absolute_path(code_path.to_path_buf(), "code path")?;
        let expected_output = run_root.join("outputs");
        if output_root != expected_output {
            return Err(LinuxSandboxError::InvalidRunLayout(format!(
                "output root must be {}",
                expected_output.display()
            )));
        }
        let code_relative = code_path
            .strip_prefix(&run_root)
            .map_err(|_| {
                LinuxSandboxError::InvalidRunLayout(
                    "code path must remain beneath the run root".into(),
                )
            })?
            .to_path_buf();
        validate_relative_path(&code_relative, "code path")?;
        if code_relative.starts_with("outputs") {
            return Err(LinuxSandboxError::InvalidRunLayout(
                "code cannot be sourced from the writable output tree".into(),
            ));
        }

        let runtime_parent = runtime.parent().ok_or_else(|| {
            LinuxSandboxError::InvalidRunLayout("runtime has no parent directory".into())
        })?;
        let mut read_roots = self.system_read_roots.clone();
        if !read_roots.iter().any(|root| runtime.starts_with(root)) {
            read_roots.insert(runtime_parent.to_path_buf());
        }

        let mut args = vec![
            "--die-with-parent".into(),
            "--new-session".into(),
            "--unshare-user".into(),
            "--unshare-pid".into(),
            "--unshare-ipc".into(),
            "--unshare-uts".into(),
            "--unshare-cgroup-try".into(),
            "--unshare-net".into(),
            "--cap-drop".into(),
            "ALL".into(),
            "--clearenv".into(),
            "--proc".into(),
            "/proc".into(),
            "--dev".into(),
            "/dev".into(),
            "--tmpfs".into(),
            "/tmp".into(),
        ];
        for root in read_roots {
            let root = path_string(&root, "system read root")?;
            args.extend(["--ro-bind".into(), root.clone(), root]);
        }

        let run_root = path_string(&run_root, "run root")?;
        let output_root = path_string(&output_root, "output root")?;
        args.extend([
            "--ro-bind".into(),
            run_root,
            SANDBOX_ROOT.into(),
            "--bind".into(),
            output_root,
            SANDBOX_OUTPUTS.into(),
            "--chdir".into(),
            SANDBOX_ROOT.into(),
            "--setenv".into(),
            "HOME".into(),
            SANDBOX_ROOT.into(),
            "--setenv".into(),
            "TMPDIR".into(),
            format!("{SANDBOX_OUTPUTS}/tmp"),
            "--setenv".into(),
            "PROMETHEUS_INPUT_DIR".into(),
            SANDBOX_INPUTS.into(),
            "--setenv".into(),
            "PROMETHEUS_OUTPUT_DIR".into(),
            SANDBOX_OUTPUTS.into(),
            "--setenv".into(),
            "PYTHONDONTWRITEBYTECODE".into(),
            "1".into(),
        ]);
        for (name, value) in environment {
            validate_environment_name(name)?;
            if value.contains('\0') {
                return Err(LinuxSandboxError::InvalidEnvironmentValue(name.clone()));
            }
            args.extend(["--setenv".into(), name.clone(), value.clone()]);
        }

        let sandbox_code = Path::new(SANDBOX_ROOT).join(code_relative);
        args.extend([
            "--".into(),
            path_string(&runtime, "runtime")?,
            path_string(&sandbox_code, "sandbox code path")?,
        ]);

        BwrapPlan::from_parts(&self.executable, &self.version, args)
    }
}

/// Exact bubblewrap process invocation and its receipt identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BwrapPlan {
    program: PathBuf,
    args: Vec<String>,
    profile_hash: Digest,
}

impl BwrapPlan {
    fn from_parts(
        program: &Path,
        bwrap_version: &str,
        args: Vec<String>,
    ) -> Result<Self, LinuxSandboxError> {
        let identity = BwrapProfileIdentity {
            schema_version: BWRAP_PROFILE_VERSION,
            bwrap_program: path_string(program, "bwrap executable")?,
            bwrap_version,
            args: &args,
        };
        let profile_hash = hash_serializable(&identity)
            .map_err(|error| LinuxSandboxError::ProfileHash(error.to_string()))?;
        Ok(Self {
            program: program.to_path_buf(),
            args,
            profile_hash,
        })
    }

    pub fn program(&self) -> &Path {
        &self.program
    }

    pub fn args(&self) -> &[String] {
        &self.args
    }

    pub fn profile_hash(&self) -> &Digest {
        &self.profile_hash
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BwrapProfileIdentity<'a> {
    schema_version: &'static str,
    bwrap_program: String,
    bwrap_version: &'a str,
    args: &'a [String],
}

/// Compatibility policy requested from the rust-landlock builder.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LandlockCompatibility {
    BestEffort,
}

/// Raw status returned after applying an enumerated-child Landlock ruleset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LandlockRulesetStatus {
    FullyEnforced,
    PartiallyEnforced,
    NotEnforced,
}

/// Probe result supplied by the single-threaded Landlock helper before exec.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LandlockProbe {
    pub compatibility: LandlockCompatibility,
    pub ruleset: LandlockRulesetStatus,
    pub no_new_privs: bool,
    pub effective_abi: Option<u32>,
    pub kernel_abi: Option<i32>,
}

/// Honest execution classification; partial enforcement is never attested.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum LandlockClassification {
    FullyEnforced {
        effective_abi: u32,
        kernel_abi: Option<i32>,
    },
    PartiallyEnforced {
        effective_abi: Option<u32>,
        warning: String,
    },
    NotEnforced {
        reason: String,
    },
}

impl LandlockClassification {
    pub fn classify(probe: &LandlockProbe) -> Self {
        match (probe.ruleset, probe.no_new_privs, probe.effective_abi) {
            (LandlockRulesetStatus::FullyEnforced, true, Some(effective_abi)) => {
                Self::FullyEnforced {
                    effective_abi,
                    kernel_abi: probe.kernel_abi,
                }
            }
            (LandlockRulesetStatus::NotEnforced, _, _) => Self::NotEnforced {
                reason: "Landlock ruleset was not enforced by the running kernel".into(),
            },
            _ => Self::PartiallyEnforced {
                effective_abi: probe.effective_abi,
                warning: if probe.no_new_privs {
                    "Landlock reported partial filesystem enforcement; Tier P remains unavailable"
                        .into()
                } else {
                    "Landlock did not enforce no_new_privs; Tier P remains unavailable".into()
                },
            },
        }
    }

    pub fn is_fully_enforced(&self) -> bool {
        matches!(self, Self::FullyEnforced { .. })
    }
}

/// Linux backend selection never exposes an unsandboxed interpreter fallback.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinuxSandboxSelection {
    Bwrap(BwrapConfig),
    TierUnavailable {
        reason: String,
        landlock: LandlockClassification,
    },
}

impl LinuxSandboxSelection {
    pub fn select(
        bwrap: Option<BwrapConfig>,
        landlock: LandlockClassification,
    ) -> LinuxSandboxSelection {
        match bwrap {
            Some(config) => Self::Bwrap(config),
            None => Self::TierUnavailable {
                reason: match &landlock {
                    LandlockClassification::FullyEnforced { .. } =>
                        "bubblewrap is unavailable; the Landlock execution fallback is not yet runtime-certified".into(),
                    LandlockClassification::PartiallyEnforced { .. } =>
                        "bubblewrap is unavailable and Landlock is only partially enforced".into(),
                    LandlockClassification::NotEnforced { .. } =>
                        "no supported Linux sandbox is available".into(),
                },
                landlock,
            },
        }
    }
}

#[derive(Debug, Error)]
pub enum LinuxSandboxError {
    #[error("Linux Tier P is unavailable on this platform")]
    UnsupportedPlatform,
    #[error("tier_unavailable: {reason}")]
    TierUnavailable { reason: String },
    #[error("bubblewrap probe failed: {0}")]
    BwrapProbe(String),
    #[error("invalid bubblewrap version: {0:?}")]
    InvalidVersion(String),
    #[error("invalid {kind}: {path}")]
    InvalidPath { kind: &'static str, path: String },
    #[error("invalid run layout: {0}")]
    InvalidRunLayout(String),
    #[error("requested capability cannot be enforced by bubblewrap: {0}")]
    CapabilityUnavailable(String),
    #[error("requested environment value was not supplied: {0}")]
    EnvironmentUnavailable(String),
    #[error("unexpected environment value was supplied: {0}")]
    UnexpectedEnvironment(String),
    #[error("invalid environment name: {0}")]
    InvalidEnvironmentName(String),
    #[error("environment value contains NUL: {0}")]
    InvalidEnvironmentValue(String),
    #[error("bubblewrap profile hash failed: {0}")]
    ProfileHash(String),
}

fn validate_capabilities(
    job: &ValidatedExecutionJob,
    environment: &BTreeMap<String, String>,
) -> Result<(), LinuxSandboxError> {
    let request = job.request();
    if request.tier == RequestedTier::W {
        return Err(LinuxSandboxError::CapabilityUnavailable(
            "requested tier does not permit Tier P".into(),
        ));
    }
    if !request.capabilities.net.egress.is_empty() {
        return Err(LinuxSandboxError::CapabilityUnavailable(
            "bwrap Tier P currently supports an isolated network namespace only".into(),
        ));
    }
    if !request.capabilities.clock || !request.capabilities.random {
        return Err(LinuxSandboxError::CapabilityUnavailable(
            "bwrap cannot attest denial of clock or kernel randomness".into(),
        ));
    }
    for path in &request.capabilities.fs.read_write {
        if !is_output_scoped(path) {
            return Err(LinuxSandboxError::CapabilityUnavailable(format!(
                "write path is not output-scoped: {path}"
            )));
        }
    }
    for path in &request.capabilities.fs.read_only {
        if path != "." && !job.inputs().contains_key(path) {
            return Err(LinuxSandboxError::CapabilityUnavailable(format!(
                "read path is not a declared input: {path}"
            )));
        }
    }
    for requested in &request.capabilities.env.read {
        validate_environment_name(requested)?;
        if !environment.contains_key(requested) {
            return Err(LinuxSandboxError::EnvironmentUnavailable(requested.clone()));
        }
    }
    if let Some(unexpected) = environment
        .keys()
        .find(|name| !request.capabilities.env.read.contains(name))
    {
        return Err(LinuxSandboxError::UnexpectedEnvironment(unexpected.clone()));
    }
    Ok(())
}

fn validate_absolute_path(path: PathBuf, kind: &'static str) -> Result<PathBuf, LinuxSandboxError> {
    if !path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
        || path
            .to_str()
            .is_none_or(|value| value.chars().any(char::is_control))
    {
        return Err(LinuxSandboxError::InvalidPath {
            kind,
            path: path.display().to_string(),
        });
    }
    Ok(path)
}

fn validate_relative_path(path: &Path, kind: &'static str) -> Result<(), LinuxSandboxError> {
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(LinuxSandboxError::InvalidPath {
            kind,
            path: path.display().to_string(),
        });
    }
    Ok(())
}

fn path_string(path: &Path, kind: &'static str) -> Result<String, LinuxSandboxError> {
    path.to_str()
        .filter(|value| !value.chars().any(char::is_control))
        .map(str::to_owned)
        .ok_or_else(|| LinuxSandboxError::InvalidPath {
            kind,
            path: path.display().to_string(),
        })
}

fn validate_environment_name(name: &str) -> Result<(), LinuxSandboxError> {
    let mut bytes = name.bytes();
    let Some(first) = bytes.next() else {
        return Err(LinuxSandboxError::InvalidEnvironmentName(name.into()));
    };
    if !(first == b'_' || first.is_ascii_alphabetic())
        || !bytes.all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
    {
        return Err(LinuxSandboxError::InvalidEnvironmentName(name.into()));
    }
    Ok(())
}

fn is_output_scoped(path: &str) -> bool {
    let path = path.replace('\\', "/");
    let path = path.trim_end_matches('/');
    let mut components = path.split('/');
    matches!(components.next(), Some("outputs"))
        && components
            .all(|component| !component.is_empty() && component != "." && component != "..")
}

#[cfg(target_os = "linux")]
fn discover_bwrap() -> Option<PathBuf> {
    use std::os::unix::fs::PermissionsExt as _;

    let mut candidates = vec![PathBuf::from("/usr/bin/bwrap"), PathBuf::from("/bin/bwrap")];
    if let Some(path) = std::env::var_os("PATH") {
        candidates.extend(std::env::split_paths(&path).map(|directory| directory.join("bwrap")));
    }
    candidates.into_iter().find_map(|candidate| {
        let metadata = std::fs::metadata(&candidate).ok()?;
        if !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
            return None;
        }
        candidate.canonicalize().ok()
    })
}

#[cfg(target_os = "linux")]
fn default_linux_read_roots() -> Vec<PathBuf> {
    ["/usr", "/bin", "/sbin", "/lib", "/lib64", "/etc"]
        .into_iter()
        .map(PathBuf::from)
        .filter(|path| path.exists())
        .collect()
}
