use std::{fmt, str::FromStr};

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{de, Deserialize, Deserializer, Serialize};
use uuid::Uuid;

use crate::{canonical_bytes_without, hash_bytes, ContractError, Result, SCHEMA_VERSION};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct Digest(String);

impl Digest {
    pub fn parse(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        let Some(hex) = value.strip_prefix("sha256:") else {
            return Err(ContractError::InvalidDigest(value));
        };
        if hex.len() != 64
            || !hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(ContractError::InvalidDigest(value));
        }
        Ok(Self(value))
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self(format!(
            "sha256:{}",
            hex::encode(crate::canonical::sha256_raw(bytes))
        ))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Digest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl FromStr for Digest {
    type Err = ContractError;

    fn from_str(value: &str) -> Result<Self> {
        Self::parse(value)
    }
}

impl<'de> Deserialize<'de> for Digest {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::parse(value).map_err(de::Error::custom)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SignatureAlgorithm {
    #[default]
    Ed25519,
    P256,
}

impl fmt::Display for SignatureAlgorithm {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ed25519 => formatter.write_str("ed25519"),
            Self::P256 => formatter.write_str("p256"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum RequestedTier {
    W,
    P,
    Auto,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionTier {
    W,
    P,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum EvidenceClass {
    Verified,
    Attested,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum CodeKind {
    Inline,
    Component,
    File,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeKind {
    WasmComponent,
    Python3,
    Node,
    Bash,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionBackend {
    Cranelift,
    Pulley,
    Seatbelt,
    Bwrap,
    Landlock,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ComponentAuthorizationMode {
    SignedGeneration,
    HashPin,
    Bundled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ComponentAuthorization {
    pub mode: ComponentAuthorizationMode,
    pub world: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manifest_hash: Option<Digest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation_id: Option<String>,
}

impl ComponentAuthorization {
    pub fn validate(&self) -> Result<()> {
        if self.world != "prometheus:component@0.1.0" {
            return Err(ContractError::ReceiptInvariant(format!(
                "unsupported component world: {}",
                self.world
            )));
        }
        if self.mode == ComponentAuthorizationMode::SignedGeneration
            && (self.manifest_hash.is_none()
                || self.generation_id.as_deref().unwrap_or("").is_empty())
        {
            return Err(ContractError::ReceiptInvariant(
                "signed-generation authorization requires manifestHash and generationId".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ComponentProvenance {
    pub authorization: ComponentAuthorization,
    pub engine_version: String,
    pub backend_profile_hash: Digest,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionFailureKind {
    Trap,
    FuelExhausted,
    EpochDeadline,
    MemoryLimit,
    CapabilityDenied,
    ComponentUnauthorized,
    BackendUnavailable,
    Interrupted,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionFailure {
    pub kind: ExecutionFailureKind,
    pub code: String,
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum RunState {
    Queued,
    GrantPending,
    Running,
    Succeeded,
    Failed,
    Rejected,
    Interrupted,
}

impl RunState {
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Succeeded | Self::Failed | Self::Rejected | Self::Interrupted
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CodeIdentity {
    pub kind: CodeKind,
    pub hash: Digest,
    pub runtime: RuntimeKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub toolchain_pin: Option<Digest>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct NamedInput {
    pub name: String,
    pub hash: Digest,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FilesystemCapabilities {
    #[serde(default)]
    pub read_only: Vec<String>,
    #[serde(default)]
    pub read_write: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct NetworkCapabilities {
    #[serde(default)]
    pub egress: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentCapabilities {
    #[serde(default)]
    pub read: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityManifest {
    #[serde(default)]
    pub fs: FilesystemCapabilities,
    #[serde(default)]
    pub net: NetworkCapabilities,
    #[serde(default)]
    pub env: EnvironmentCapabilities,
    #[serde(default)]
    pub clock: bool,
    #[serde(default)]
    pub random: bool,
}

impl Default for CapabilityManifest {
    fn default() -> Self {
        Self {
            fs: FilesystemCapabilities {
                read_only: vec![".".into()],
                read_write: vec!["outputs/".into()],
            },
            net: NetworkCapabilities::default(),
            env: EnvironmentCapabilities::default(),
            clock: true,
            random: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionLimits {
    pub memory_mb: u64,
    pub fuel: u64,
    pub wall_clock_ms: u64,
    pub output_mb: u64,
    pub stack_kb: u64,
}

impl Default for ExecutionLimits {
    fn default() -> Self {
        Self {
            memory_mb: 256,
            fuel: 500_000_000,
            wall_clock_ms: 120_000,
            output_mb: 10,
            stack_kb: 512,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionProvenance {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harness: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component_authorization: Option<ComponentAuthorization>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SignedExecRequest {
    pub schema_version: String,
    pub request_id: Uuid,
    pub issued_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queued_at: Option<DateTime<Utc>>,
    pub validity_window_secs: u64,
    pub tier: RequestedTier,
    pub code: CodeIdentity,
    #[serde(default)]
    pub inputs: Vec<NamedInput>,
    pub capabilities: CapabilityManifest,
    pub limits: ExecutionLimits,
    #[serde(default)]
    pub targets: Vec<String>,
    #[serde(default)]
    pub provenance: ExecutionProvenance,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signer_key_id: Option<String>,
    pub sig_alg: SignatureAlgorithm,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

impl SignedExecRequest {
    pub fn canonical_unsigned(&self) -> Result<Vec<u8>> {
        canonical_bytes_without(self, "signature")
    }

    pub fn request_hash(&self) -> Result<Digest> {
        Ok(hash_bytes(&self.canonical_unsigned()?))
    }

    pub fn validate(&self) -> Result<()> {
        ensure_schema(&self.schema_version)?;
        if self.validity_window_secs == 0
            || self.limits.memory_mb == 0
            || self.limits.wall_clock_ms == 0
            || self.limits.output_mb == 0
            || self.limits.stack_kb == 0
        {
            return Err(ContractError::ReceiptInvariant(
                "validity window, memory, wall-clock, output, and stack limits must be non-zero"
                    .into(),
            ));
        }
        let component_request = self.code.kind == CodeKind::Component
            || self.code.runtime == RuntimeKind::WasmComponent
            || self.tier == RequestedTier::W;
        if component_request
            && (self.code.kind != CodeKind::Component
                || self.code.runtime != RuntimeKind::WasmComponent
                || self.provenance.component_authorization.is_none())
        {
            return Err(ContractError::ReceiptInvariant(
                "Tier W requires component code, the wasm-component runtime, and component authorization"
                    .into(),
            ));
        }
        if let Some(authorization) = &self.provenance.component_authorization {
            authorization.validate()?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum GrantKind {
    SshManifest,
    Interactive,
    CedarAuto,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionGrant {
    pub kind: GrantKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#ref: Option<Digest>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactReference {
    pub path: String,
    pub hash: Digest,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionOutputs {
    pub stdout: Digest,
    pub stderr: Digest,
    #[serde(default)]
    pub artifacts: Vec<ArtifactReference>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionExit {
    pub status: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signal_or_trap: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResourceUsage {
    pub wall_clock_ms: u64,
    pub cpu_ms: u64,
    pub peak_mem_mb: u64,
    pub fuel_consumed: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExecutingDevice {
    pub key_id: String,
    pub sig_alg: SignatureAlgorithm,
    pub platform: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionReceipt {
    pub schema_version: String,
    pub run_id: Uuid,
    pub request_hash: Digest,
    pub state: RunState,
    pub evidence_class: EvidenceClass,
    pub tier: ExecutionTier,
    pub code_hash: Digest,
    pub input_set_hash: Digest,
    pub env_hash: Digest,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub toolchain_hash: Option<Digest>,
    pub sandbox_profile_hash: Digest,
    pub backend: ExecutionBackend,
    pub exit: ExecutionExit,
    pub outputs: ExecutionOutputs,
    pub usage: ResourceUsage,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub executing_device: ExecutingDevice,
    #[serde(default)]
    pub grants: Vec<ExecutionGrant>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component: Option<ComponentProvenance>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure: Option<ExecutionFailure>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

impl ExecutionReceipt {
    pub fn canonical_unsigned(&self) -> Result<Vec<u8>> {
        canonical_bytes_without(self, "signature")
    }

    pub fn receipt_hash(&self) -> Result<Digest> {
        Ok(hash_bytes(&crate::canonical_bytes(self)?))
    }

    pub fn validate(&self) -> Result<()> {
        ensure_schema(&self.schema_version)?;
        if !self.state.is_terminal() {
            return Err(ContractError::ReceiptInvariant(
                "a signed receipt must describe a terminal run".into(),
            ));
        }
        match (self.tier, self.evidence_class) {
            (ExecutionTier::W, EvidenceClass::Verified)
            | (ExecutionTier::P, EvidenceClass::Attested) => {}
            _ => {
                return Err(ContractError::ReceiptInvariant(
                    "tier W must be verified and tier P must be attested".into(),
                ))
            }
        }
        match self.tier {
            ExecutionTier::W
                if !matches!(
                    self.backend,
                    ExecutionBackend::Cranelift | ExecutionBackend::Pulley
                ) || self.component.is_none() =>
            {
                return Err(ContractError::ReceiptInvariant(
                    "tier W requires a Wasmtime backend and component provenance".into(),
                ));
            }
            ExecutionTier::P if self.component.is_some() => {
                return Err(ContractError::ReceiptInvariant(
                    "tier P cannot claim component provenance".into(),
                ));
            }
            _ => {}
        }
        if let Some(component) = &self.component {
            component.authorization.validate()?;
            if component.engine_version.trim().is_empty() {
                return Err(ContractError::ReceiptInvariant(
                    "component engineVersion must not be empty".into(),
                ));
            }
        }
        if self.state == RunState::Succeeded && self.failure.is_some() {
            return Err(ContractError::ReceiptInvariant(
                "a succeeded receipt cannot contain failure details".into(),
            ));
        }
        if self.finished_at < self.started_at {
            return Err(ContractError::ReceiptInvariant(
                "finishedAt precedes startedAt".into(),
            ));
        }
        let mut paths = std::collections::HashSet::new();
        for artifact in &self.outputs.artifacts {
            validate_artifact_path(&artifact.path)?;
            if !paths.insert(&artifact.path) {
                return Err(ContractError::ReceiptInvariant(format!(
                    "duplicate artifact path: {}",
                    artifact.path
                )));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TierWReplayRequest {
    pub receipt: ExecutionReceipt,
    pub component_hash: Digest,
    #[serde(default)]
    pub inputs: Vec<NamedInput>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TierWReplayResult {
    pub valid: bool,
    #[serde(default)]
    pub checks: Vec<String>,
    #[serde(default)]
    pub mismatches: Vec<String>,
}

pub fn validate_artifact_path(path: &str) -> Result<()> {
    let candidate = std::path::Path::new(path);
    let mut components = candidate.components();
    if candidate.is_absolute()
        || !matches!(components.next(), Some(std::path::Component::Normal(first)) if first == "outputs")
        || components.any(|part| !matches!(part, std::path::Component::Normal(_)))
    {
        return Err(ContractError::UnsafeArtifactPath(path.into()));
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionEventKind {
    Accepted,
    Stdout,
    Stderr,
    Progress,
    GrantPending,
    Completed,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionEvent {
    pub schema_version: String,
    pub run_id: Uuid,
    pub sequence: u64,
    pub occurred_at: DateTime<Utc>,
    pub kind: ExecutionEventKind,
    #[serde(default)]
    pub data: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ErrorEnvelope {
    pub schema_version: String,
    pub error: ErrorDetail,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<Uuid>,
}

pub fn ensure_schema(version: &str) -> Result<()> {
    if version == SCHEMA_VERSION {
        Ok(())
    } else {
        Err(ContractError::UnsupportedSchemaVersion(version.into()))
    }
}
