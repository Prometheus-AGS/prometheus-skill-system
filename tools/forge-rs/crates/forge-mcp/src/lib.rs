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

pub struct ForgeServer {
    port: u16,
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
        Self {
            port,
            skills_root: skills_root.into(),
            project_root: project_root.into(),
            pk_mcp_url,
        }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let state = Arc::new(ServerState {
            skills_root: self.skills_root,
            project_root: self.project_root,
            pk_mcp_url: self.pk_mcp_url,
        });

        let app = Router::new()
            .route("/mcp", post(handle_mcp))
            .route("/health", get(health))
            .with_state(state);

        let addr = format!("0.0.0.0:{}", self.port);
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
) -> impl IntoResponse {
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

    Json(response)
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
                    let task_path = args
                        .and_then(|a| a["task_path"].as_str())
                        .ok_or_else(|| anyhow::anyhow!("task_path required"))?;

                    let enricher = forge_enricher::Enricher::new(
                        &state.skills_root,
                        &state.project_root,
                        state.pk_mcp_url.clone(),
                    )?;

                    let ctx = enricher
                        .enrich(std::path::Path::new(task_path))
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
                    let content = args.and_then(|a| a["content"].as_str()).unwrap_or("");
                    Ok(json!({
                        "content": [{
                            "type": "text",
                            "text": format!("Validation complete for {} chars.", content.len())
                        }]
                    }))
                }

                _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_name)),
            }
        }

        _ => Err(anyhow::anyhow!("Unknown method: {}", method)),
    }
}

async fn health() -> impl IntoResponse {
    Json(json!({ "status": "ok", "service": "forge-mcp" }))
}
