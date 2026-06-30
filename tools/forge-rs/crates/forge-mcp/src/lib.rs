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

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::validate_request::ValidateRequestHeaderLayer;

pub struct ForgeServer {
    port: u16,
    bind_addr: String,
    skills_root: std::path::PathBuf,
    project_root: std::path::PathBuf,
    pk_mcp_url: Option<String>,
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
        }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let token = std::env::var("FORGE_MCP_TOKEN").unwrap_or_else(|_| {
            let generated = uuid::Uuid::new_v4().to_string();
            eprintln!(
                "FORGE_MCP_TOKEN not set — generated token for this session: {}",
                generated
            );
            eprintln!(
                "Set FORGE_MCP_TOKEN in your environment to use a stable token."
            );
            generated
        });

        let state = Arc::new(ServerState {
            skills_root: self.skills_root,
            project_root: self.project_root,
            pk_mcp_url: self.pk_mcp_url,
        });

        let app = Router::new()
            .route(
                "/mcp",
                post(handle_mcp).layer(ValidateRequestHeaderLayer::bearer(&token)),
            )
            .route("/health", get(health))
            .with_state(state);

        let addr = format!("{}:{}", self.bind_addr, self.port);
        tracing::info!("forge-mcp listening on {}", addr);

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;
        Ok(())
    }
}

struct ServerState {
    skills_root: std::path::PathBuf,
    project_root: std::path::PathBuf,
    pk_mcp_url: Option<String>,
}

#[derive(Deserialize)]
struct McpRequest {
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
            let tool_name = params
                .and_then(|p| p["name"].as_str())
                .unwrap_or("");
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
                    let project_root_canonical = state.project_root.canonicalize().map_err(|e| {
                        anyhow::anyhow!("cannot canonicalize project root: {}", e)
                    })?;
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

                    let ctx = enricher
                        .enrich(&canonical)
                        .await?;

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
                    let drift_dir = state.project_root.join(".forge").join("memory").join("drift");
                    let summary = if drift_dir.exists() {
                        format!("Drift reports available at: {}", drift_dir.display())
                    } else {
                        "No drift data yet. Run forge reflect after completing iterations.".to_string()
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
                            let warnings = forge_enricher::check_constitution(constitution, content);
                            if warnings.is_empty() {
                                format!("✅ No violations found for {} ({} chars).", lang_str, content.len())
                            } else {
                                let lines: Vec<String> = warnings.iter().map(|w| {
                                    format!("[{:?}] {} — {}", w.severity, w.rule, w.violation)
                                }).collect();
                                format!("{} violation(s) found:\n{}", warnings.len(), lines.join("\n"))
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
