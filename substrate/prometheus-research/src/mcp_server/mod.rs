pub mod tools;

use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{Implementation, ServerCapabilities, ServerInfo},
    serve_server, tool_handler, tool_router,
    transport::io::stdio,
    ServerHandler,
};

use crate::{
    a2ui::registry::ComponentRegistry,
    job::{cancel::cancel_job, checkpoint, spawn::spawn_job},
};
use tools::{
    RenderComponentParams, ResearchCancelParams, ResearchExportParams, ResearchStartParams,
    ResearchStatusParams,
};

// ---------------------------------------------------------------------------
// ResearchMcpServer
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct ResearchMcpServer {
    tool_router: ToolRouter<Self>,
    registry: ComponentRegistry,
}

impl ResearchMcpServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
            registry: ComponentRegistry::new(),
        }
    }

    pub async fn serve_stdio(self) -> anyhow::Result<()> {
        let transport = stdio();
        let running = serve_server(self, transport).await?;
        running.waiting().await?;
        Ok(())
    }
}

impl Default for ResearchMcpServer {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tool implementations
// ---------------------------------------------------------------------------

#[tool_router(router = tool_router)]
impl ResearchMcpServer {
    /// Start a background deep-research job and return its job ID.
    #[rmcp::tool(
        name = "research_start",
        description = "Start a background deep-research job. Returns job_id and started_at."
    )]
    pub async fn research_start(&self, params: Parameters<ResearchStartParams>) -> String {
        let p = params.0;
        match spawn_job(&p.query, &p.depth, p.max_sources, &p.citation_style) {
            Ok(job_id) => {
                let started_at = chrono_now();
                serde_json::json!({
                    "job_id": job_id,
                    "started_at": started_at,
                    "query": p.query,
                    "depth": p.depth
                })
                .to_string()
            }
            Err(e) => serde_json::json!({ "error": e.to_string() }).to_string(),
        }
    }

    /// Get the current status of a research job.
    #[rmcp::tool(
        name = "research_status",
        description = "Get status of a research job: stage, progress, elapsed, tokens used."
    )]
    pub async fn research_status(&self, params: Parameters<ResearchStatusParams>) -> String {
        let job_id = &params.0.job_id;
        match checkpoint::read(job_id) {
            Ok(cp) => serde_json::json!({
                "job_id": cp.job_id,
                "status": cp.status,
                "stage": cp.stage,
                "stage_name": cp.stage_name,
                "progress": cp.progress,
                "started_at": cp.started_at,
                "last_updated_at": cp.last_updated_at,
                "tokens_used": cp.tokens_used,
                "sources_found": cp.sources_found,
            })
            .to_string(),
            Err(e) => serde_json::json!({ "error": e.to_string() }).to_string(),
        }
    }

    /// Cancel a running research job.
    #[rmcp::tool(
        name = "research_cancel",
        description = "Cancel a running research job by sending SIGTERM to the job process."
    )]
    pub async fn research_cancel(&self, params: Parameters<ResearchCancelParams>) -> String {
        let job_id = &params.0.job_id;
        match cancel_job(job_id) {
            Ok(()) => serde_json::json!({
                "cancelled": true,
                "job_id": job_id,
                "reason": "SIGTERM sent to job process"
            })
            .to_string(),
            Err(e) => serde_json::json!({
                "cancelled": false,
                "job_id": job_id,
                "reason": e.to_string()
            })
            .to_string(),
        }
    }

    /// Export research results to a file.
    #[rmcp::tool(
        name = "research_export",
        description = "Export completed research results. Format: markdown, json, or html."
    )]
    pub async fn research_export(&self, params: Parameters<ResearchExportParams>) -> String {
        let p = params.0;
        match checkpoint::read(&p.job_id) {
            Ok(cp) => {
                if cp.status != "completed" {
                    return serde_json::json!({
                        "error": format!("job {} is not completed (status: {})", p.job_id, cp.status)
                    })
                    .to_string();
                }
                let output_path = format!("{}/report.{}", cp.output_dir, p.format);
                serde_json::json!({
                    "job_id": p.job_id,
                    "output_path": output_path,
                    "format": p.format,
                    "note": "Export generation will be implemented in change-prb-007"
                })
                .to_string()
            }
            Err(e) => serde_json::json!({ "error": e.to_string() }).to_string(),
        }
    }

    /// Render an A2UI component as an HTMX HTML fragment.
    #[rmcp::tool(
        name = "render_component",
        description = "Render an A2UI component as an HTMX HTML fragment for display in MCP App UI."
    )]
    pub async fn render_component(&self, params: Parameters<RenderComponentParams>) -> String {
        let p = params.0;
        self.registry.render(&p.name, p.props)
    }
}

// ---------------------------------------------------------------------------
// ServerHandler impl
// ---------------------------------------------------------------------------

#[tool_handler(router = self.tool_router)]
impl ServerHandler for ResearchMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("prometheus-research", "0.1.0"))
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub async fn run() -> anyhow::Result<()> {
    let server = ResearchMcpServer::new();
    tracing::info!("prometheus-research MCP server starting on stdio");
    server.serve_stdio().await
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn chrono_now() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| {
            let secs = d.as_secs();
            let days = secs / 86400;
            let year = 1970 + days / 365;
            format!("{}-07-08T{:02}:{:02}:{:02}Z", year, (secs % 3600) / 60, (secs % 3600) % 60, secs % 60)
        })
        .unwrap_or_else(|_| "unknown".to_string())
}
