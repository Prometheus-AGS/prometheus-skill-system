//! forge-mcp — Axum SSE MCP server exposing forge-rs as MCP tools.
//!
//! Exposes the following MCP tools:
//! - `forge_enrich`   — enrich an OpenSpec task folder with skills + Karpathy context
//! - `forge_reflect`  — process a completed iteration into the Karpathy loop
//! - `forge_drift`    — report skill drift across recent iterations
//! - `forge_validate` — check a task or file against the active constitution
//!
//! Transport: JSON-RPC 2.0 over HTTP POST /mcp (+ GET /events SSE stream)
//! Default port: 8943

use axum::body::Body;
use axum::http::Request;
use axum::response::Response;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use subtle::ConstantTimeEq;
use tower_http::validate_request::{ValidateRequest, ValidateRequestHeaderLayer};

pub struct ForgeServer {
    port: u16,
    bind_addr: String,
    skills_root: std::path::PathBuf,
    project_root: std::path::PathBuf,
    pk_mcp_url: Option<String>,
    /// Opt-in: serve `/mcp` without bearer auth. Only honoured on a loopback
    /// bind; see [`ForgeServer::run`].
    no_auth: bool,
}

impl ForgeServer {
    pub fn new(
        port: u16,
        skills_root: impl Into<std::path::PathBuf>,
        project_root: impl Into<std::path::PathBuf>,
        pk_mcp_url: Option<String>,
    ) -> Self {
        Self::with_bind_addr(port, "127.0.0.1", skills_root, project_root, pk_mcp_url)
    }

    pub fn with_bind_addr(
        port: u16,
        bind_addr: impl Into<String>,
        skills_root: impl Into<std::path::PathBuf>,
        project_root: impl Into<std::path::PathBuf>,
        pk_mcp_url: Option<String>,
    ) -> Self {
        let bind_addr = bind_addr.into();
        if bind_addr == "0.0.0.0" {
            eprintln!(
                "WARNING: forge-mcp is binding to 0.0.0.0 which exposes the MCP endpoint on \
                 all network interfaces. This is not recommended for local development. \
                 Use 127.0.0.1 (the default) to restrict to localhost only."
            );
        }
        Self {
            port,
            bind_addr,
            skills_root: skills_root.into(),
            project_root: project_root.into(),
            pk_mcp_url,
            no_auth: false,
        }
    }

    /// Serve `/mcp` without bearer auth.
    ///
    /// Opt-in and loopback-only: [`ForgeServer::run`] REFUSES to start if this
    /// is set while bound to anything other than a loopback address. Without
    /// that refusal the flag would be a foot-gun — `--bind 0.0.0.0 --no-auth`
    /// publishes an unauthenticated API that reads and writes project files to
    /// the whole network, and a warning is not enough for that.
    ///
    /// The trade-off this accepts on loopback: any LOCAL process can reach the
    /// endpoint. That is the same posture as the other loopback services in
    /// this stack, and it is the caller's decision to make — hence opt-in
    /// rather than default.
    #[must_use]
    pub fn with_no_auth(mut self, no_auth: bool) -> Self {
        self.no_auth = no_auth;
        self
    }

    pub async fn run(self) -> anyhow::Result<()> {
        // `--no-auth` is loopback-only, enforced by REFUSING to start rather
        // than by warning. A warning is the wrong tool here: the failure mode
        // is an unauthenticated file-reading/writing API published to the
        // network, and warnings scroll past. Fail closed instead.
        if self.no_auth && !is_loopback_bind(&self.bind_addr) {
            anyhow::bail!(
                "refusing to start: --no-auth is only permitted on a loopback bind, but \
                 --bind is {}. Without auth this would expose forge_enrich / forge_reflect / \
                 forge_drift / forge_validate — which read and write project files — to every \
                 host that can reach this address. Drop --no-auth, or bind 127.0.0.1.",
                self.bind_addr
            );
        }

        // An empty or whitespace-only FORGE_MCP_TOKEN would otherwise yield the
        // near-guessable secret `Bearer ` (env var set-but-empty does NOT hit
        // the unset fallback), so treat blank as absent and mint a random token.
        //
        // Skipped entirely under --no-auth: minting a token nobody checks would
        // print a misleading "here is your token" line for an endpoint that
        // ignores it.
        let token = if self.no_auth {
            None
        } else {
            Some(
                std::env::var("FORGE_MCP_TOKEN")
                    .ok()
                    .filter(|t| !t.trim().is_empty())
                    .unwrap_or_else(|| {
                        let generated = uuid::Uuid::new_v4().to_string();
                        eprintln!(
                            "FORGE_MCP_TOKEN not set — generated a random dev token for this \
                             session: {}",
                            generated
                        );
                        eprintln!(
                            "This value is written to stderr for convenience; set \
                             FORGE_MCP_TOKEN in your environment (non-empty) to use a stable \
                             token and avoid logging it."
                        );
                        generated
                    }),
            )
        };

        let state = Arc::new(ServerState {
            skills_root: self.skills_root,
            project_root: self.project_root,
            pk_mcp_url: self.pk_mcp_url,
        });

        // Bearer auth with a constant-time token comparison (replaces the
        // deprecated ValidateRequestHeaderLayer::bearer, whose check is not
        // constant-time — a timing side channel on the token).
        //
        // Two routers rather than an Option<Layer>: axum's layer types differ
        // between the guarded and unguarded routes, so branching on the route
        // is simpler than trying to make one layer conditionally inert.
        let app = match &token {
            Some(token) => {
                let auth_layer = ValidateRequestHeaderLayer::custom(BearerAuth::new(token));
                Router::new().route("/mcp", post(handle_mcp).layer(auth_layer))
            }
            None => {
                tracing::warn!(
                    "forge-mcp running WITHOUT authentication on {} — any local process can \
                     call forge_enrich / forge_reflect / forge_drift / forge_validate",
                    self.bind_addr
                );
                Router::new().route("/mcp", post(handle_mcp))
            }
        }
        .route("/health", get(health))
        .with_state(state);

        let addr = format!("{}:{}", self.bind_addr, self.port);
        tracing::info!("forge-mcp listening on {}", addr);

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;
        Ok(())
    }
}

/// True when `bind_addr` names a loopback interface.
///
/// Parsed as an IP rather than string-compared, so the whole 127.0.0.0/8 range
/// and IPv6 `::1` are recognised — a `--bind 127.0.0.2 --no-auth` is genuinely
/// loopback and a naive `== "127.0.0.1"` check would reject it. The literal
/// hostname `localhost` is accepted because it is what callers actually type;
/// it is not resolved, since a DNS lookup here would let a hosts-file entry
/// decide whether auth is required.
fn is_loopback_bind(bind_addr: &str) -> bool {
    if bind_addr.eq_ignore_ascii_case("localhost") {
        return true;
    }
    // Accept a bracketed IPv6 literal ("[::1]") as well as a bare one.
    let candidate = bind_addr
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
        .unwrap_or(bind_addr);
    candidate
        .parse::<std::net::IpAddr>()
        .map(|ip| ip.is_loopback())
        .unwrap_or(false)
}

struct ServerState {
    skills_root: std::path::PathBuf,
    project_root: std::path::PathBuf,
    pk_mcp_url: Option<String>,
}

/// Bearer-token request validator with a constant-time comparison.
///
/// Rejects any request whose `Authorization` header is missing, is not a
/// `Bearer <token>`, or whose token does not match — returning `401
/// Unauthorized`. The token comparison uses `subtle::ConstantTimeEq` so the
/// time taken does not depend on how many leading bytes match, closing the
/// timing side channel that `ValidateRequestHeaderLayer::bearer` (a plain `==`)
/// left open.
#[derive(Clone)]
struct BearerAuth {
    // The full expected header value ("Bearer <token>") as bytes, so the whole
    // comparison is constant-time in one shot.
    expected: Arc<[u8]>,
}

impl BearerAuth {
    fn new(token: &str) -> Self {
        Self {
            expected: Arc::from(format!("Bearer {token}").into_bytes().into_boxed_slice()),
        }
    }

    fn unauthorized() -> Response<Body> {
        // Infallible construction (no `expect`): all inputs are constants.
        let mut resp = Response::new(Body::empty());
        *resp.status_mut() = StatusCode::UNAUTHORIZED;
        resp
    }
}

impl<B> ValidateRequest<B> for BearerAuth {
    type ResponseBody = Body;

    fn validate(&mut self, request: &mut Request<B>) -> Result<(), Response<Self::ResponseBody>> {
        // Only the first Authorization header is read (`HeaderMap::get`), which
        // is safe here: the server is loopback-bound by default, so there is no
        // trusted fronting proxy that could smuggle a second header.
        let header = request
            .headers()
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok());

        match header {
            // Compare the full "Bearer <token>" header against the expected
            // bytes with subtle::ct_eq. Among equal-length inputs this is
            // constant-time, closing the per-byte short-circuit that a plain
            // `==` (the old deprecated bearer layer) leaked. subtle DOES return
            // early on a length mismatch, so the only thing that can leak by
            // timing is the total header length = 7 ("Bearer ") + token length;
            // the token length is not secret-bearing here, so that is
            // acceptable. Non-Bearer schemes / wrong tokens fail closed (401).
            Some(value) if value.as_bytes().ct_eq(&self.expected).into() => Ok(()),
            _ => Err(Self::unauthorized()),
        }
    }
}

#[derive(Deserialize)]
struct McpRequest {
    /// JSON-RPC 2.0 version tag ("2.0"). Deserialized to document the wire
    /// contract (and reject payloads missing it); not used in dispatch logic.
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

async fn handle_mcp(
    State(state): State<Arc<ServerState>>,
    Json(req): Json<McpRequest>,
) -> axum::response::Response {
    // JSON-RPC notifications (no `id`, e.g. notifications/initialized) expect no
    // response body — acknowledge with 202 so the client proceeds.
    if req.id.is_none() {
        return StatusCode::ACCEPTED.into_response();
    }

    let result = dispatch_method(&state, &req.method, req.params.as_ref()).await;

    let response = match result {
        Ok(value) => json!({
            "jsonrpc": "2.0",
            "id": req.id,
            "result": value
        }),
        Err(e) => json!({
            "jsonrpc": "2.0",
            "id": req.id,
            "error": { "code": -32603, "message": e.to_string() }
        }),
    };

    Json(response).into_response()
}

async fn dispatch_method(
    state: &ServerState,
    method: &str,
    params: Option<&Value>,
) -> anyhow::Result<Value> {
    match method {
        "tools/list" => Ok(json!({
            "tools": [
                {
                    "name": "forge_enrich",
                    "description": "Enrich an OpenSpec task with language skills, constitution, and Karpathy context. Returns the path to the enriched context document.",
                    "inputSchema": {
                        "type": "object",
                        "required": ["task_path"],
                        "properties": {
                            "task_path": { "type": "string", "description": "Path to OpenSpec task folder or tasks.md file" }
                        }
                    }
                },
                {
                    "name": "forge_reflect",
                    "description": "Process a completed implementation iteration into the Karpathy learning loop via pk ingest.",
                    "inputSchema": {
                        "type": "object",
                        "required": ["iteration_id"],
                        "properties": {
                            "iteration_id": { "type": "string", "description": "Task ID or iteration ID to process" }
                        }
                    }
                },
                {
                    "name": "forge_drift",
                    "description": "Report skill drift — which skills are being overridden most often.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "language": { "type": "string", "description": "Filter by language (optional)" }
                        }
                    }
                },
                {
                    "name": "forge_validate",
                    "description": "Check content against the active language constitution.",
                    "inputSchema": {
                        "type": "object",
                        "required": ["content", "language"],
                        "properties": {
                            "content":  { "type": "string" },
                            "language": { "type": "string" }
                        }
                    }
                }
            ]
        })),

        "tools/call" => {
            let tool_name = params.and_then(|p| p["name"].as_str()).unwrap_or("");
            let args = params.and_then(|p| p.get("arguments"));

            match tool_name {
                "forge_enrich" => {
                    let task_path_str = args
                        .and_then(|a| a["task_path"].as_str())
                        .ok_or_else(|| anyhow::anyhow!("task_path required"))?;

                    // Path confinement: prevent traversal outside project root.
                    let raw_path = std::path::Path::new(task_path_str);
                    let canonical = raw_path.canonicalize().map_err(|e| {
                        anyhow::anyhow!("invalid task_path '{}': {}", task_path_str, e)
                    })?;
                    let project_root_canonical = state
                        .project_root
                        .canonicalize()
                        .map_err(|e| anyhow::anyhow!("cannot canonicalize project root: {}", e))?;
                    if !canonical.starts_with(&project_root_canonical) {
                        return Err(anyhow::anyhow!(
                            "task_path '{}' is outside the project root '{}' — access denied",
                            task_path_str,
                            state.project_root.display()
                        ));
                    }

                    let enricher = forge_enricher::Enricher::new(
                        &state.skills_root,
                        &state.project_root,
                        state.pk_mcp_url.clone(),
                    )?;

                    let ctx = enricher.enrich(&canonical).await?;

                    let context_path = state
                        .project_root
                        .join(".forge")
                        .join("enriched")
                        .join(format!("{}.context.md", ctx.task_path));

                    Ok(json!({
                        "content": [{
                            "type": "text",
                            "text": format!(
                                "Enriched {} with {} skill(s). Context at: {}\nApplied: {}",
                                ctx.task_description,
                                ctx.applied_skills.len(),
                                context_path.display(),
                                ctx.applied_skills.join(", ")
                            )
                        }]
                    }))
                }

                "forge_reflect" => {
                    let iteration_id = args
                        .and_then(|a| a["iteration_id"].as_str())
                        .ok_or_else(|| anyhow::anyhow!("iteration_id required"))?;

                    let reflector = forge_reflect::Reflector::new(&state.project_root);
                    let record = reflector.reflect(iteration_id).await?;

                    Ok(json!({
                        "content": [{
                            "type": "text",
                            "text": format!(
                                "Reflected iteration {} ({:?}). {} skill(s) drift recorded. Ingested to prometheus-knowledge.",
                                record.task_id,
                                record.language,
                                record.skill_drift.len()
                            )
                        }]
                    }))
                }

                "forge_drift" => {
                    let drift_dir = state
                        .project_root
                        .join(".forge")
                        .join("memory")
                        .join("drift");
                    let summary = if drift_dir.exists() {
                        format!("Drift reports available at: {}", drift_dir.display())
                    } else {
                        "No drift data yet. Run forge reflect after completing iterations."
                            .to_string()
                    };
                    Ok(json!({ "content": [{ "type": "text", "text": summary }] }))
                }

                "forge_validate" => {
                    use std::str::FromStr as _;
                    let content = args
                        .and_then(|a| a["content"].as_str())
                        .ok_or_else(|| anyhow::anyhow!("content required"))?;
                    let lang_str = args
                        .and_then(|a| a["language"].as_str())
                        .ok_or_else(|| anyhow::anyhow!("language required"))?;
                    let lang = forge_core::Language::from_str(lang_str)
                        .map_err(|e| anyhow::anyhow!("{}", e))?;
                    let constitution_dir = state.project_root.join(".forge").join("constitution");
                    let constitutions = forge_enricher::load_constitutions(&constitution_dir)?;

                    let text = match constitutions.get(&lang) {
                        None => format!(
                            "No constitution found for {} at {}. Run `forge init` to scaffold one.",
                            lang_str,
                            constitution_dir.display()
                        ),
                        Some(constitution) => {
                            let warnings =
                                forge_enricher::check_constitution(constitution, content);
                            if warnings.is_empty() {
                                format!(
                                    "✅ No violations found for {} ({} chars).",
                                    lang_str,
                                    content.len()
                                )
                            } else {
                                let lines: Vec<String> = warnings
                                    .iter()
                                    .map(|w| {
                                        format!("[{:?}] {} — {}", w.severity, w.rule, w.violation)
                                    })
                                    .collect();
                                format!(
                                    "{} violation(s) found:\n{}",
                                    warnings.len(),
                                    lines.join("\n")
                                )
                            }
                        }
                    };

                    Ok(json!({ "content": [{ "type": "text", "text": text }] }))
                }

                _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_name)),
            }
        }

        // MCP lifecycle — required so spec-compliant clients (Claude Code, etc.)
        // complete the Streamable HTTP handshake before listing/calling tools.
        "initialize" => Ok(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "forge-mcp", "version": env!("CARGO_PKG_VERSION") }
        })),
        "ping" => Ok(json!({})),

        _ => Err(anyhow::anyhow!("Unknown method: {}", method)),
    }
}

async fn health() -> impl IntoResponse {
    Json(json!({ "status": "ok", "service": "forge-mcp" }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_state(project_root: impl Into<PathBuf>) -> ServerState {
        ServerState {
            skills_root: PathBuf::from("/nonexistent/skills"),
            project_root: project_root.into(),
            pk_mcp_url: None,
        }
    }

    // ─── dispatch_method: initialize ────────────────────────────────────────

    #[tokio::test]
    async fn initialize_returns_protocol_version() {
        let state = make_state("/tmp/forge-test-mcp-init");
        let result = dispatch_method(&state, "initialize", None).await.unwrap();
        assert_eq!(result["protocolVersion"], "2024-11-05");
        assert_eq!(result["serverInfo"]["name"], "forge-mcp");
    }

    // ─── dispatch_method: ping ───────────────────────────────────────────────

    #[tokio::test]
    async fn ping_returns_empty_object() {
        let state = make_state("/tmp/forge-test-mcp-ping");
        let result = dispatch_method(&state, "ping", None).await.unwrap();
        assert!(result.as_object().map_or(false, |o| o.is_empty()));
    }

    // ─── dispatch_method: unknown method ────────────────────────────────────

    #[tokio::test]
    async fn unknown_method_returns_error() {
        let state = make_state("/tmp/forge-test-mcp-unknown");
        let err = dispatch_method(&state, "tools/nonexistent", None).await;
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("Unknown method"));
    }

    // ─── dispatch_method: tools/list ────────────────────────────────────────

    #[tokio::test]
    async fn tools_list_returns_four_tools() {
        let state = make_state("/tmp/forge-test-mcp-list");
        let result = dispatch_method(&state, "tools/list", None).await.unwrap();
        let tools = result["tools"].as_array().expect("tools must be an array");
        assert_eq!(
            tools.len(),
            4,
            "expected forge_enrich, forge_reflect, forge_drift, forge_validate"
        );

        let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
        assert!(names.contains(&"forge_enrich"));
        assert!(names.contains(&"forge_reflect"));
        assert!(names.contains(&"forge_drift"));
        assert!(names.contains(&"forge_validate"));
    }

    // ─── dispatch_method: tools/call — unknown tool ──────────────────────────

    #[tokio::test]
    async fn tools_call_unknown_tool_returns_error() {
        let state = make_state("/tmp/forge-test-mcp-call-unknown");
        let params = json!({ "name": "no_such_tool", "arguments": {} });
        let err = dispatch_method(&state, "tools/call", Some(&params)).await;
        assert!(err.is_err());
    }

    // ─── dispatch_method: tools/call — forge_drift (no drift dir) ───────────

    #[tokio::test]
    async fn forge_drift_returns_text_when_no_drift_data() {
        let state = make_state("/tmp/forge-test-mcp-drift-nodata");
        let params = json!({ "name": "forge_drift", "arguments": {} });
        let result = dispatch_method(&state, "tools/call", Some(&params))
            .await
            .unwrap();
        let text = result["content"][0]["text"].as_str().unwrap_or("");
        assert!(
            text.contains("No drift data") || text.contains("Drift reports"),
            "unexpected drift response: {}",
            text
        );
    }

    // ─── dispatch_method: tools/call — forge_enrich path confinement ────────

    #[tokio::test]
    async fn forge_enrich_rejects_path_outside_project_root() {
        let state = make_state("/tmp/forge-test-mcp-path-confinement");
        let params = json!({
            "name": "forge_enrich",
            "arguments": { "task_path": "/etc/passwd" }
        });
        let err = dispatch_method(&state, "tools/call", Some(&params)).await;
        assert!(
            err.is_err(),
            "path traversal outside project root must be rejected"
        );
    }

    // ─── ForgeServer::new — bind_addr default ────────────────────────────────

    #[test]
    fn forge_server_new_defaults_to_loopback() {
        let server = ForgeServer::new(8943, "/tmp/skills", "/tmp/project", None);
        assert_eq!(server.bind_addr, "127.0.0.1");
        assert_eq!(server.port, 8943);
    }

    #[test]
    fn forge_server_with_bind_addr_stores_address() {
        let server = ForgeServer::with_bind_addr(8943, "127.0.0.1", "/tmp/s", "/tmp/p", None);
        assert_eq!(server.bind_addr, "127.0.0.1");
    }

    // ─── BearerAuth: constant-time bearer-token validation ──────────────────

    /// Build a bare request with an optional Authorization header value.
    fn req_with_auth(auth: Option<&str>) -> Request<Body> {
        let mut builder = Request::builder().uri("/mcp");
        if let Some(v) = auth {
            builder = builder.header(axum::http::header::AUTHORIZATION, v);
        }
        builder.body(Body::empty()).unwrap()
    }

    #[test]
    fn bearer_auth_accepts_matching_token() {
        let mut auth = BearerAuth::new("s3cr3t-token");
        let mut req = req_with_auth(Some("Bearer s3cr3t-token"));
        assert!(auth.validate(&mut req).is_ok());
    }

    #[test]
    fn bearer_auth_rejects_wrong_token() {
        let mut auth = BearerAuth::new("s3cr3t-token");
        let mut req = req_with_auth(Some("Bearer wrong-token"));
        let err = auth.validate(&mut req).unwrap_err();
        assert_eq!(err.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn bearer_auth_rejects_missing_header() {
        let mut auth = BearerAuth::new("s3cr3t-token");
        let mut req = req_with_auth(None);
        let err = auth.validate(&mut req).unwrap_err();
        assert_eq!(err.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn bearer_auth_rejects_non_bearer_scheme() {
        let mut auth = BearerAuth::new("s3cr3t-token");
        // Correct token, wrong scheme — must still be rejected.
        let mut req = req_with_auth(Some("Basic s3cr3t-token"));
        let err = auth.validate(&mut req).unwrap_err();
        assert_eq!(err.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn bearer_auth_rejects_token_prefix() {
        // A token that is a prefix of the real one must not pass (length differs).
        let mut auth = BearerAuth::new("s3cr3t-token");
        let mut req = req_with_auth(Some("Bearer s3cr3t"));
        let err = auth.validate(&mut req).unwrap_err();
        assert_eq!(err.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn bearer_auth_rejects_empty_bearer_against_real_token() {
        // A real token is configured; a client sending `Bearer ` (empty token)
        // must be rejected. (The run() path separately refuses to build a
        // BearerAuth from an empty/blank FORGE_MCP_TOKEN.)
        let mut auth = BearerAuth::new("s3cr3t-token");
        let mut req = req_with_auth(Some("Bearer "));
        let err = auth.validate(&mut req).unwrap_err();
        assert_eq!(err.status(), StatusCode::UNAUTHORIZED);
    }
}

#[cfg(test)]
mod no_auth_tests {
    use super::*;

    #[test]
    fn loopback_forms_are_recognised() {
        // Not just the literal default: the whole 127.0.0.0/8 range and IPv6
        // loopback are genuinely local, and a string compare would reject them.
        for addr in [
            "127.0.0.1",
            "127.0.0.2",
            "127.1.2.3",
            "::1",
            "[::1]",
            "localhost",
            "LOCALHOST",
        ] {
            assert!(is_loopback_bind(addr), "{addr} should be loopback");
        }
    }

    #[test]
    fn non_loopback_forms_are_rejected() {
        // 0.0.0.0 is the dangerous one --no-auth must never be allowed with.
        for addr in [
            "0.0.0.0",
            "192.168.1.10",
            "10.0.0.1",
            "::",
            "example.com",
            "",
        ] {
            assert!(!is_loopback_bind(addr), "{addr} must NOT count as loopback");
        }
    }

    #[tokio::test]
    async fn no_auth_refuses_to_start_on_non_loopback_bind() {
        // The guard must FAIL CLOSED, not warn: --bind 0.0.0.0 --no-auth would
        // otherwise publish an unauthenticated file-writing API to the network.
        let server = ForgeServer::with_bind_addr(0, "0.0.0.0", ".", ".", None).with_no_auth(true);
        let err = server.run().await.expect_err("must refuse to start");
        let msg = err.to_string();
        assert!(
            msg.contains("--no-auth"),
            "error should name the flag: {msg}"
        );
        assert!(
            msg.contains("loopback"),
            "error should explain the rule: {msg}"
        );
    }

    #[test]
    fn no_auth_defaults_off() {
        // Auth stays the default; --no-auth is strictly opt-in.
        let server = ForgeServer::with_bind_addr(0, "127.0.0.1", ".", ".", None);
        assert!(!server.no_auth);
    }
}
