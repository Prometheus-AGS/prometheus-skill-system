use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{DateTime, Utc};
use ed25519_dalek::SigningKey;
use hyper::Method;
use prometheus_exec_contracts::{
    sign_request_ed25519, CapabilityManifest, CodeIdentity, CodeKind, Digest, ExecutionLimits,
    ExecutionProvenance, ExecutionReceipt, NamedInput, RequestedTier, RunState, RuntimeKind,
    SignatureAlgorithm, SignedExecRequest, VerificationKey, SCHEMA_VERSION,
};
use prometheus_exec_core::ArtifactStore;
use prometheus_exec_service::{LocalExecutionFacade, RunRecord, DEFAULT_INLINE_ARTIFACT_BYTES};
use prometheus_exec_tier_w::ComponentAuthorizer;
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{Implementation, ServerCapabilities, ServerInfo},
    serve_server, tool_handler, tool_router,
    transport::io::stdio,
    ServerHandler,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{daemon, identity, uds_client, BoxError};

const MAX_CODE_BYTES: usize = 8 * 1024 * 1024;
const MAX_INPUT_BYTES: usize = 16 * 1024 * 1024;

pub fn tool_contracts() -> Value {
    let tools = [
        ("exec-run", schemars::schema_for!(ExecRunParams)),
        ("exec-status", schemars::schema_for!(ExecStatusParams)),
        ("exec-events", schemars::schema_for!(ExecEventsParams)),
        ("exec-receipt", schemars::schema_for!(ExecStatusParams)),
        ("exec-artifact", schemars::schema_for!(ExecArtifactParams)),
        ("exec-verify", schemars::schema_for!(ExecVerifyParams)),
    ]
    .into_iter()
    .map(|(name, input_schema)| {
        serde_json::json!({
            "name": name,
            "inputSchema": input_schema,
            "outputEnvelope": {
                "success": {"ok": true, "result": "tool-specific JSON value"},
                "failure": {"error": {"code": "string", "message": "string"}, "ok": false}
            }
        })
    })
    .collect::<Vec<_>>();
    serde_json::json!({
        "schemaVersion": "1",
        "service": "prometheus-exec",
        "version": env!("CARGO_PKG_VERSION"),
        "maximumInlineArtifactBytes": DEFAULT_INLINE_ARTIFACT_BYTES,
        "tools": tools,
    })
}

#[derive(Clone, Debug)]
pub struct McpConfig {
    pub state_dir: PathBuf,
    pub identity: PathBuf,
    pub plugin_root: PathBuf,
    pub artifact_budget_bytes: u64,
}

#[derive(Clone)]
pub struct ExecMcpServer {
    tool_router: ToolRouter<Self>,
    facade: LocalExecutionFacade,
    signing_key: Arc<SigningKey>,
    plugin_root: PathBuf,
    runner_socket: PathBuf,
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum McpRuntime {
    WasmComponent,
    Python3,
    Node,
    Bash,
}

impl From<McpRuntime> for RuntimeKind {
    fn from(value: McpRuntime) -> Self {
        match value {
            McpRuntime::WasmComponent => Self::WasmComponent,
            McpRuntime::Python3 => Self::Python3,
            McpRuntime::Node => Self::Node,
            McpRuntime::Bash => Self::Bash,
        }
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExecRunParams {
    /// Stable client request identity. Resubmit it with the same `issuedAt` and
    /// payload to recover the original run after response loss.
    #[serde(default)]
    pub request_id: Option<Uuid>,
    /// Stable client issue time used in the signed canonical payload. Omit both
    /// this field and `requestId` for a one-shot server-identified submission.
    #[serde(default)]
    pub issued_at: Option<DateTime<Utc>>,
    pub runtime: McpRuntime,
    /// Unpadded base64url code or component bytes.
    pub code_base64: String,
    /// Named unpadded base64url input bytes.
    #[serde(default)]
    pub inputs: BTreeMap<String, String>,
    #[serde(default = "default_timeout_ms")]
    #[schemars(range(min = 1))]
    pub timeout_ms: u64,
    #[serde(default = "default_output_mb")]
    #[schemars(range(min = 1))]
    pub output_mb: u64,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExecStatusParams {
    pub run_id: Uuid,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExecEventsParams {
    pub run_id: Uuid,
    #[serde(default)]
    pub after: u64,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExecArtifactParams {
    pub digest: String,
    #[serde(default = "default_inline_ceiling")]
    #[schemars(range(min = 1))]
    pub inline_ceiling_bytes: usize,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExecVerifyParams {
    pub receipt: Value,
    pub public_key: String,
    #[serde(default)]
    pub request: Option<Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RunSummary {
    run_id: Uuid,
    request_id: Uuid,
    request_hash: Digest,
    state: RunState,
    replayed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    receipt: Option<ExecutionReceipt>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ArtifactRetrievalGuidance {
    transport: &'static str,
    method: &'static str,
    socket_path: PathBuf,
    path: String,
}

impl RunSummary {
    fn from_record(record: RunRecord, replayed: bool) -> Self {
        Self {
            run_id: record.run_id,
            request_id: record.request_id,
            request_hash: record.request_hash,
            state: record.state,
            replayed,
            receipt: record.terminal.map(|terminal| terminal.receipt),
        }
    }
}

impl ExecMcpServer {
    fn new(
        facade: LocalExecutionFacade,
        signing_key: SigningKey,
        plugin_root: PathBuf,
        runner_socket: PathBuf,
    ) -> Self {
        Self {
            tool_router: Self::tool_router(),
            facade,
            signing_key: Arc::new(signing_key),
            plugin_root,
            runner_socket,
        }
    }

    async fn serve_stdio(self) -> Result<(), BoxError> {
        let running = serve_server(self, stdio()).await?;
        running.waiting().await?;
        Ok(())
    }

    fn build_request(&self, params: ExecRunParams) -> Result<SignedExecRequest, String> {
        if params.timeout_ms == 0 || params.output_mb == 0 {
            return Err("timeoutMs and outputMb must be non-zero".into());
        }
        if params.request_id.is_some() != params.issued_at.is_some() {
            return Err("requestId and issuedAt must be supplied together".into());
        }
        let code = decode_bounded("codeBase64", &params.code_base64, MAX_CODE_BYTES)?;
        let request_id = params.request_id.unwrap_or_else(Uuid::new_v4);
        let runtime: RuntimeKind = params.runtime.into();
        let component_authorization = if runtime == RuntimeKind::WasmComponent {
            Some(
                ComponentAuthorizer::estate(&self.plugin_root)
                    .authorization_for_bytes(&code)
                    .map_err(|error| error.to_string())?,
            )
        } else {
            None
        };
        let mut decoded_inputs = Vec::with_capacity(params.inputs.len());
        let mut total_input_bytes = 0usize;
        for (name, encoded) in params.inputs {
            if name.is_empty() {
                return Err("input names must not be empty".into());
            }
            let remaining = MAX_INPUT_BYTES.saturating_sub(total_input_bytes);
            let bytes = decode_bounded(&format!("inputs.{name}"), &encoded, remaining)?;
            total_input_bytes = total_input_bytes.saturating_add(bytes.len());
            decoded_inputs.push((name, bytes));
        }
        let upload_pin = format!("upload:{request_id}");
        let stored_code = self
            .facade
            .artifacts()
            .put_pinned(&code, &upload_pin)
            .map_err(|error| error.to_string())?;
        let mut pinned_hashes = vec![stored_code.hash.clone()];
        let mut inputs = Vec::with_capacity(decoded_inputs.len());
        for (name, bytes) in decoded_inputs {
            let stored = match self.facade.artifacts().put_pinned(&bytes, &upload_pin) {
                Ok(stored) => stored,
                Err(error) => {
                    return Err(rollback_upload_pins(
                        self.facade.artifacts(),
                        &pinned_hashes,
                        &upload_pin,
                        error.to_string(),
                    ));
                }
            };
            if !pinned_hashes.contains(&stored.hash) {
                pinned_hashes.push(stored.hash.clone());
            }
            inputs.push(NamedInput {
                name,
                hash: stored.hash,
            });
        }
        inputs.sort_by(|left, right| left.name.cmp(&right.name));
        let now = params.issued_at.unwrap_or_else(Utc::now);
        let mut request = SignedExecRequest {
            schema_version: SCHEMA_VERSION.into(),
            request_id,
            issued_at: now,
            queued_at: Some(now),
            validity_window_secs: params.timeout_ms.saturating_add(60_000).div_ceil(1000),
            tier: if runtime == RuntimeKind::WasmComponent {
                RequestedTier::W
            } else {
                RequestedTier::P
            },
            code: CodeIdentity {
                kind: if runtime == RuntimeKind::WasmComponent {
                    CodeKind::Component
                } else {
                    CodeKind::File
                },
                hash: stored_code.hash,
                runtime,
                toolchain_pin: None,
            },
            inputs,
            capabilities: CapabilityManifest::default(),
            limits: ExecutionLimits {
                wall_clock_ms: params.timeout_ms,
                output_mb: params.output_mb,
                ..ExecutionLimits::default()
            },
            targets: Vec::new(),
            provenance: ExecutionProvenance {
                harness: Some("prometheus-exec-mcp".into()),
                component_authorization,
                ..ExecutionProvenance::default()
            },
            signer_key_id: None,
            sig_alg: SignatureAlgorithm::Ed25519,
            signature: None,
        };
        if let Err(error) = sign_request_ed25519(&mut request, &self.signing_key) {
            return Err(rollback_upload_pins(
                self.facade.artifacts(),
                &pinned_hashes,
                &upload_pin,
                error.to_string(),
            ));
        }
        Ok(request)
    }
}

fn rollback_upload_pins(
    artifacts: &ArtifactStore,
    hashes: &[Digest],
    reason: &str,
    primary: String,
) -> String {
    let failures = hashes
        .iter()
        .filter_map(|hash| {
            artifacts
                .unpin(hash, reason)
                .err()
                .map(|error| format!("{hash}: {error}"))
        })
        .collect::<Vec<_>>();
    if failures.is_empty() {
        primary
    } else {
        format!(
            "{primary}; upload-pin rollback failed: {}",
            failures.join("; ")
        )
    }
}

#[tool_router(router = tool_router)]
impl ExecMcpServer {
    #[rmcp::tool(
        name = "exec-run",
        description = "Submit code or a Wasm component to the local evidence-producing execution service."
    )]
    pub async fn exec_run(&self, params: Parameters<ExecRunParams>) -> String {
        let request = match self.build_request(params.0) {
            Ok(request) => request,
            Err(error) => return failure("invalid_request", error),
        };
        let facade = self.facade.clone();
        match tokio::task::spawn_blocking(move || facade.submit(request)).await {
            Ok(Ok(result)) => success(RunSummary::from_record(result.record, result.replayed)),
            Ok(Err(error)) => failure("submit_failed", error.to_string()),
            Err(error) => failure("submit_task_failed", error.to_string()),
        }
    }

    #[rmcp::tool(
        name = "exec-status",
        description = "Read durable execution status by run ID."
    )]
    pub async fn exec_status(&self, params: Parameters<ExecStatusParams>) -> String {
        let run_id = params.0.run_id;
        let facade = self.facade.clone();
        match tokio::task::spawn_blocking(move || facade.run(run_id)).await {
            Ok(Ok(Some(record))) => success(RunSummary::from_record(record, false)),
            Ok(Ok(None)) => failure("run_not_found", format!("run {run_id} was not found")),
            Ok(Err(error)) => failure("status_failed", error.to_string()),
            Err(error) => failure("status_task_failed", error.to_string()),
        }
    }

    #[rmcp::tool(
        name = "exec-events",
        description = "Read ordered execution events after an exclusive sequence cursor."
    )]
    pub async fn exec_events(&self, params: Parameters<ExecEventsParams>) -> String {
        let params = params.0;
        let facade = self.facade.clone();
        match tokio::task::spawn_blocking(move || facade.events_after(params.run_id, params.after))
            .await
        {
            Ok(Ok(events)) => success(events),
            Ok(Err(error)) => failure("events_failed", error.to_string()),
            Err(error) => failure("events_task_failed", error.to_string()),
        }
    }

    #[rmcp::tool(
        name = "exec-receipt",
        description = "Read a terminal signed receipt by run ID."
    )]
    pub async fn exec_receipt(&self, params: Parameters<ExecStatusParams>) -> String {
        let run_id = params.0.run_id;
        let facade = self.facade.clone();
        match tokio::task::spawn_blocking(move || facade.receipt(run_id)).await {
            Ok(Ok(Some(receipt))) => success(receipt),
            Ok(Ok(None)) => failure(
                "receipt_not_found",
                format!("run {run_id} has no terminal receipt"),
            ),
            Ok(Err(error)) => failure("receipt_failed", error.to_string()),
            Err(error) => failure("receipt_task_failed", error.to_string()),
        }
    }

    #[rmcp::tool(
        name = "exec-artifact",
        description = "Read a content-addressed artifact inline when it fits the bounded response ceiling."
    )]
    pub async fn exec_artifact(&self, params: Parameters<ExecArtifactParams>) -> String {
        let params = params.0;
        let digest = match Digest::parse(params.digest) {
            Ok(digest) => digest,
            Err(error) => return failure("invalid_digest", error.to_string()),
        };
        if params.inline_ceiling_bytes == 0 {
            return failure("invalid_ceiling", "inlineCeilingBytes must be non-zero");
        }
        let ceiling = params
            .inline_ceiling_bytes
            .min(DEFAULT_INLINE_ARTIFACT_BYTES);
        let facade = self.facade.clone();
        match tokio::task::spawn_blocking(move || facade.artifact(&digest, ceiling)).await {
            Ok(Ok(payload)) => {
                let retrieval = (!payload.is_inline()).then(|| ArtifactRetrievalGuidance {
                    transport: "unix-domain-http",
                    method: "GET",
                    socket_path: self.runner_socket.clone(),
                    path: format!("/api/v2/exec/artifacts/{}", payload.digest),
                });
                success(serde_json::json!({
                    "digest": payload.digest,
                    "sizeBytes": payload.size_bytes,
                    "inline": payload.is_inline(),
                    "bytesBase64": payload.bytes.map(|bytes| URL_SAFE_NO_PAD.encode(bytes)),
                    "retrieval": retrieval,
                }))
            }
            Ok(Err(error)) => failure("artifact_failed", error.to_string()),
            Err(error) => failure("artifact_task_failed", error.to_string()),
        }
    }

    #[rmcp::tool(
        name = "exec-verify",
        description = "Verify a signed execution receipt offline using a public key and optional request."
    )]
    pub async fn exec_verify(&self, params: Parameters<ExecVerifyParams>) -> String {
        let params = params.0;
        let receipt: ExecutionReceipt = match serde_json::from_value(params.receipt) {
            Ok(receipt) => receipt,
            Err(error) => return failure("invalid_receipt", error.to_string()),
        };
        let request: Option<SignedExecRequest> = match params.request.map(serde_json::from_value) {
            Some(Ok(request)) => Some(request),
            Some(Err(error)) => return failure("invalid_request", error.to_string()),
            None => None,
        };
        let key = match VerificationKey::from_base64url(
            receipt.executing_device.sig_alg,
            &params.public_key,
        ) {
            Ok(key) => key,
            Err(error) => return failure("invalid_public_key", error.to_string()),
        };
        success(self.facade.verify(&receipt, &key, request.as_ref(), None))
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for ExecMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build()).with_server_info(
            Implementation::new("prometheus-exec", env!("CARGO_PKG_VERSION")),
        )
    }
}

pub async fn run(config: McpConfig) -> Result<(), BoxError> {
    let loaded = identity::load(&config.identity)?;
    let service = Arc::new(prometheus_exec_service::ExecutionService::open(
        config.state_dir.join("service"),
    )?);
    let artifacts = Arc::new(ArtifactStore::open(
        config.state_dir.join("artifacts"),
        config.artifact_budget_bytes,
    )?);
    let facade = LocalExecutionFacade::new(service, artifacts);
    let socket = config.state_dir.join(".mcp-runner.sock");
    let daemon_config = daemon::DaemonConfig {
        socket: socket.clone(),
        state_dir: config.state_dir,
        identity: config.identity,
        plugin_root: config.plugin_root.clone(),
        artifact_budget_bytes: config.artifact_budget_bytes,
    };
    let daemon = tokio::spawn(daemon::run(daemon_config));
    wait_for_runner_ready(&socket, &daemon, Duration::from_secs(5)).await?;
    let server = ExecMcpServer::new(facade, loaded.signing_key, config.plugin_root, socket);
    let result = server.serve_stdio().await;
    daemon.abort();
    let _ = daemon.await;
    result
}

async fn wait_for_runner_ready(
    socket: &Path,
    daemon: &tokio::task::JoinHandle<Result<(), BoxError>>,
    timeout: Duration,
) -> Result<(), BoxError> {
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        if daemon.is_finished() {
            return Err("MCP execution runner exited before binding its private socket".into());
        }
        if socket.exists() {
            let readiness = tokio::time::timeout(
                Duration::from_millis(250),
                uds_client::request(socket, Method::GET, "/ready", Vec::new()),
            )
            .await;
            if matches!(readiness, Ok(Ok(response)) if response.status == 200) {
                return Ok(());
            }
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(format!(
                "MCP execution runner did not become ready within {} milliseconds",
                timeout.as_millis()
            )
            .into());
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

fn decode_bounded(name: &str, encoded: &str, limit: usize) -> Result<Vec<u8>, String> {
    let bytes = URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|error| format!("{name} is not unpadded base64url: {error}"))?;
    if bytes.len() > limit {
        return Err(format!("{name} exceeds the {limit}-byte limit"));
    }
    Ok(bytes)
}

fn success(value: impl Serialize) -> String {
    serde_json::to_string(&serde_json::json!({"ok": true, "result": value}))
        .unwrap_or_else(|error| failure("serialization_failed", error.to_string()))
}

fn failure(code: impl Into<String>, message: impl Into<String>) -> String {
    serde_json::to_string(&serde_json::json!({
        "ok": false,
        "error": {"code": code.into(), "message": message.into()}
    }))
    .unwrap_or_else(|_| {
        "{\"error\":{\"code\":\"serialization_failed\",\"message\":\"unable to encode error\"},\"ok\":false}".into()
    })
}

const fn default_timeout_ms() -> u64 {
    120_000
}

const fn default_output_mb() -> u64 {
    10
}

const fn default_inline_ceiling() -> usize {
    256 * 1024
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, future, sync::Arc, time::Duration};

    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    use chrono::Utc;
    use ed25519_dalek::SigningKey;
    use prometheus_exec_contracts::{
        sign_request_ed25519, CapabilityManifest, CodeIdentity, CodeKind, ExecutionLimits,
        ExecutionProvenance, NamedInput, RequestedTier, RuntimeKind, SignatureAlgorithm,
        SignedExecRequest, SCHEMA_VERSION,
    };
    use prometheus_exec_core::ArtifactStore;
    use prometheus_exec_service::{ExecutionService, LocalExecutionFacade, RunEventData};
    use rmcp::handler::server::wrapper::Parameters;
    use serde_json::Value;
    use tempfile::tempdir;
    use uuid::Uuid;

    use super::{
        wait_for_runner_ready, ExecArtifactParams, ExecMcpServer, ExecRunParams, ExecVerifyParams,
        McpRuntime,
    };

    fn fixture() -> (ExecMcpServer, Arc<ExecutionService>) {
        let directory = tempdir().expect("temporary directory").keep();
        let runner_socket = directory.join(".mcp-runner.sock");
        let service = Arc::new(
            ExecutionService::open(directory.join("service")).expect("execution service opens"),
        );
        let artifacts = Arc::new(
            ArtifactStore::open(directory.join("artifacts"), 8 * 1024 * 1024)
                .expect("artifact store opens"),
        );
        let facade = LocalExecutionFacade::new(service.clone(), artifacts);
        (
            ExecMcpServer::new(
                facade,
                SigningKey::from_bytes(&[7; 32]),
                directory,
                runner_socket,
            ),
            service,
        )
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn runner_socket_path_is_not_readiness_evidence() {
        let directory = tempdir().expect("temporary directory");
        let socket = directory.path().join("stalled.sock");
        let _listener = tokio::net::UnixListener::bind(&socket).expect("socket binds");
        let daemon = tokio::spawn(async { future::pending::<Result<(), super::BoxError>>().await });

        let error = wait_for_runner_ready(&socket, &daemon, Duration::from_millis(50))
            .await
            .expect_err("a bound socket without a live ready response must fail");
        assert!(error.to_string().contains("did not become ready"));
        daemon.abort();
        let _ = daemon.await;
    }

    fn signed_request(server: &ExecMcpServer) -> SignedExecRequest {
        let request_id = Uuid::new_v4();
        let upload_pin = format!("upload:{request_id}");
        let code = server
            .facade
            .artifacts()
            .put_pinned(b"print(42)\n", &upload_pin)
            .expect("code stored");
        let input = server
            .facade
            .artifacts()
            .put_pinned(b"{}", &upload_pin)
            .expect("input stored");
        let now = Utc::now();
        let mut request = SignedExecRequest {
            schema_version: SCHEMA_VERSION.into(),
            request_id,
            issued_at: now,
            queued_at: Some(now),
            validity_window_secs: 60,
            tier: RequestedTier::P,
            code: CodeIdentity {
                kind: CodeKind::File,
                hash: code.hash,
                runtime: RuntimeKind::Python3,
                toolchain_pin: None,
            },
            inputs: vec![NamedInput {
                name: "payload".into(),
                hash: input.hash,
            }],
            capabilities: CapabilityManifest::default(),
            limits: ExecutionLimits::default(),
            targets: Vec::new(),
            provenance: ExecutionProvenance {
                harness: Some("mcp-test".into()),
                ..ExecutionProvenance::default()
            },
            signer_key_id: None,
            sig_alg: SignatureAlgorithm::Ed25519,
            signature: None,
        };
        sign_request_ed25519(&mut request, &server.signing_key).expect("request signed");
        request
    }

    #[test]
    fn run_schema_rejects_private_key_arguments() {
        let parsed = serde_json::from_value::<ExecRunParams>(serde_json::json!({
            "runtime": "python3",
            "codeBase64": "cHJpbnQoNDIp",
            "privateKey": "forbidden"
        }));
        assert!(parsed.is_err());
        let valid = ExecRunParams {
            request_id: None,
            issued_at: None,
            runtime: McpRuntime::Python3,
            code_base64: "cHJpbnQoNDIp".into(),
            inputs: BTreeMap::new(),
            timeout_ms: 1,
            output_mb: 1,
        };
        let contract = super::tool_contracts();
        let run = contract["tools"]
            .as_array()
            .expect("tool list")
            .iter()
            .find(|tool| tool["name"] == "exec-run")
            .expect("exec-run contract");
        assert_eq!(run["inputSchema"]["properties"]["timeoutMs"]["minimum"], 1);
        assert_eq!(run["inputSchema"]["properties"]["outputMb"]["minimum"], 1);
        assert_eq!(valid.timeout_ms, 1);
    }

    #[tokio::test]
    async fn run_tool_replays_a_client_identified_canonical_request() {
        let (server, _) = fixture();
        let request_id = Uuid::new_v4();
        let issued_at = Utc::now();
        let params = ExecRunParams {
            request_id: Some(request_id),
            issued_at: Some(issued_at),
            runtime: McpRuntime::Python3,
            code_base64: "cHJpbnQoNDIp".into(),
            inputs: BTreeMap::new(),
            timeout_ms: 1_000,
            output_mb: 1,
        };

        let first: Value = serde_json::from_str(&server.exec_run(Parameters(params.clone())).await)
            .expect("first result JSON");
        let replay: Value = serde_json::from_str(&server.exec_run(Parameters(params)).await)
            .expect("replay result JSON");

        assert_eq!(first["ok"], true);
        assert_eq!(replay["ok"], true);
        assert_eq!(first["result"]["requestId"], request_id.to_string());
        assert_eq!(first["result"]["runId"], replay["result"]["runId"]);
        assert_eq!(
            first["result"]["requestHash"],
            replay["result"]["requestHash"]
        );
        assert_eq!(first["result"]["replayed"], false);
        assert_eq!(replay["result"]["replayed"], true);
    }

    #[tokio::test]
    async fn run_tool_rejects_same_id_with_a_different_canonical_payload() {
        let (server, _) = fixture();
        let request_id = Uuid::new_v4();
        let issued_at = Utc::now();
        let params = ExecRunParams {
            request_id: Some(request_id),
            issued_at: Some(issued_at),
            runtime: McpRuntime::Python3,
            code_base64: "cHJpbnQoNDIp".into(),
            inputs: BTreeMap::new(),
            timeout_ms: 1_000,
            output_mb: 1,
        };
        let first: Value = serde_json::from_str(&server.exec_run(Parameters(params.clone())).await)
            .expect("first result JSON");
        assert_eq!(first["ok"], true);

        let mut conflict = params;
        conflict.code_base64 = "cHJpbnQoNDMp".into();
        let response: Value = serde_json::from_str(&server.exec_run(Parameters(conflict)).await)
            .expect("conflict result JSON");
        assert_eq!(response["ok"], false);
        assert_eq!(response["error"]["code"], "submit_failed");
        assert!(response["error"]["message"]
            .as_str()
            .expect("error message")
            .contains("already exists with hash"));
    }

    #[test]
    fn rejected_wasm_authorization_does_not_materialize_or_pin_code() {
        let (server, _) = fixture();
        let code = b"not-an-authorized-component";
        let code_hash = prometheus_exec_contracts::hash_bytes(code);
        let result = server.build_request(ExecRunParams {
            request_id: None,
            issued_at: None,
            runtime: McpRuntime::WasmComponent,
            code_base64: URL_SAFE_NO_PAD.encode(code),
            inputs: BTreeMap::new(),
            timeout_ms: 1_000,
            output_mb: 1,
        });

        assert!(result.is_err());
        assert!(server.facade.artifacts().get(&code_hash).is_err());
    }

    #[test]
    fn shared_facade_preserves_replay_and_event_cursor_semantics() {
        let (server, service) = fixture();
        let request = signed_request(&server);
        let first = server.facade.submit(request.clone()).expect("first submit");
        let replay = server
            .facade
            .submit(request.clone())
            .expect("replay submit");
        assert!(!first.replayed);
        assert!(replay.replayed);
        assert_eq!(first.record.run_id, replay.record.run_id);
        service
            .mark_spawned(request.request_id, &first.record.request_hash)
            .expect("spawn boundary");
        service
            .append_runtime_event(
                first.record.run_id,
                "stdout.1",
                Utc::now(),
                RunEventData::Stdout {
                    chunk: "42\n".into(),
                },
            )
            .expect("runtime event");
        let events = server
            .facade
            .events_after(first.record.run_id, 2)
            .expect("events after cursor");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].sequence, 3);
    }

    #[tokio::test]
    async fn artifact_tool_never_truncates_an_oversized_inline_result() {
        let (server, _) = fixture();
        let stored = server
            .facade
            .artifacts()
            .put(b"four")
            .expect("artifact stored");
        let response = server
            .exec_artifact(Parameters(ExecArtifactParams {
                digest: stored.hash.to_string(),
                inline_ceiling_bytes: 2,
            }))
            .await;
        let value: Value = serde_json::from_str(&response).expect("valid result JSON");
        assert_eq!(value["ok"], true);
        assert_eq!(value["result"]["inline"], false);
        assert_eq!(value["result"]["sizeBytes"], 4);
        assert!(value["result"]["bytesBase64"].is_null());
        assert_eq!(
            value["result"]["retrieval"]["transport"],
            "unix-domain-http"
        );
        assert_eq!(value["result"]["retrieval"]["method"], "GET");
        assert_eq!(
            value["result"]["retrieval"]["path"],
            format!("/api/v2/exec/artifacts/{}", stored.hash)
        );
        assert!(value["result"]["retrieval"]["socketPath"]
            .as_str()
            .expect("socket path")
            .ends_with("/.mcp-runner.sock"));
    }

    #[tokio::test]
    async fn verify_tool_accepts_the_archived_independent_receipt_fixture() {
        let (server, _) = fixture();
        let receipt: Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../.kbd-orchestrator/phases/prometheus-exec-code-execution-engine/evidence/change-exec-003-tier-w-mobile/receipt.json"
        )))
        .expect("receipt fixture");
        let request: Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../.kbd-orchestrator/phases/prometheus-exec-code-execution-engine/evidence/change-exec-003-tier-w-mobile/request.json"
        )))
        .expect("request fixture");
        let public: Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../.kbd-orchestrator/phases/prometheus-exec-code-execution-engine/evidence/change-exec-003-tier-w-mobile/public-key.json"
        )))
        .expect("public key fixture");
        let response = server
            .exec_verify(Parameters(ExecVerifyParams {
                receipt,
                public_key: public["publicKey"]
                    .as_str()
                    .expect("fixture public key")
                    .into(),
                request: Some(request),
            }))
            .await;
        let value: Value = serde_json::from_str(&response).expect("valid result JSON");
        assert_eq!(value["ok"], true);
        assert_eq!(value["result"]["valid"], true);
    }
}
