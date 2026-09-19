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
                let started_at = crate::job::checkpoint::now_rfc3339();
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

    /// Export a completed job's research package after validating its manifest.
    #[rmcp::tool(
        name = "research_export",
        description = "Export a completed research package: validates manifest.json against the research-manifest schema and returns the package path, verification status and verdict, and the report path. Format: markdown (report.md) or json (manifest.json)."
    )]
    pub async fn research_export(&self, params: Parameters<ResearchExportParams>) -> String {
        let p = params.0;
        let cp = match checkpoint::read(&p.job_id) {
            Ok(cp) => cp,
            Err(e) => return serde_json::json!({ "error": e.to_string() }).to_string(),
        };
        if cp.status != "complete" {
            return serde_json::json!({
                "error": format!("job {} is not complete (status: {}{})", p.job_id, cp.status,
                    cp.error.as_deref().map(|e| format!("; {e}")).unwrap_or_default())
            })
            .to_string();
        }
        let Some(package_dir) = cp.package_dir.as_deref() else {
            return serde_json::json!({
                "error": format!("job {} is complete but records no package directory", p.job_id)
            })
            .to_string();
        };
        match validate_package(std::path::Path::new(package_dir)) {
            Ok(summary) => {
                let output_path = match p.format.as_str() {
                    "json" => summary.manifest_path.clone(),
                    "markdown" | "md" => summary.report_path.clone(),
                    other => {
                        return serde_json::json!({
                            "error": format!("unsupported export format {other:?}; use markdown or json")
                        })
                        .to_string()
                    }
                };
                serde_json::json!({
                    "job_id": p.job_id,
                    "package_id": summary.package_id,
                    "package_dir": summary.package_dir,
                    "manifest_path": summary.manifest_path,
                    "report_path": summary.report_path,
                    "verification_status": summary.verification_status,
                    "verification_verdict": summary.verification_verdict,
                    "output_path": output_path,
                    "format": p.format,
                })
                .to_string()
            }
            Err(e) => serde_json::json!({ "error": format!("{e:#}") }).to_string(),
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
// Package validation (research_export)
// ---------------------------------------------------------------------------

/// The manifest schema, embedded at build time so the daemon binary validates
/// against exactly the contract the skill ships. Same file
/// `scripts/check-research-package.sh` validates against in shell.
pub const MANIFEST_SCHEMA_JSON: &str = include_str!(
    "../../../../skills/research/deep-research/references/schemas/research-manifest.schema.json"
);

static MANIFEST_VALIDATOR: std::sync::OnceLock<Result<jsonschema::Validator, String>> =
    std::sync::OnceLock::new();

fn manifest_validator() -> Result<&'static jsonschema::Validator, anyhow::Error> {
    let built = MANIFEST_VALIDATOR.get_or_init(|| {
        let schema: serde_json::Value = serde_json::from_str(MANIFEST_SCHEMA_JSON)
            .map_err(|e| format!("embedded manifest schema is not valid JSON: {e}"))?;
        jsonschema::validator_for(&schema)
            .map_err(|e| format!("embedded manifest schema does not compile: {e}"))
    });
    built
        .as_ref()
        .map_err(|e| anyhow::anyhow!("{e}"))
}

/// What `research_export` returns for a valid package.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ExportSummary {
    pub package_id: String,
    pub package_dir: String,
    pub manifest_path: String,
    pub report_path: String,
    pub verification_status: String,
    pub verification_verdict: String,
}

/// Validate `<dir>/manifest.json` against the embedded schema. An invalid
/// manifest is an error naming every failing field (first five), so a caller
/// never exports a package whose contract is broken.
pub fn validate_package(dir: &std::path::Path) -> anyhow::Result<ExportSummary> {
    use anyhow::Context as _;
    let manifest_path = dir.join("manifest.json");
    let raw = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("no manifest.json in {}", dir.display()))?;
    let manifest: serde_json::Value = serde_json::from_str(&raw)
        .with_context(|| format!("{} is not valid JSON", manifest_path.display()))?;
    let validator = manifest_validator()?;
    let problems: Vec<String> = validator
        .iter_errors(&manifest)
        .take(5)
        .map(|e| {
            let at = e.instance_path().to_string();
            let at = if at.is_empty() { "<root>".to_string() } else { at };
            format!("{at}: {e}")
        })
        .collect();
    if !problems.is_empty() {
        anyhow::bail!(
            "manifest.json invalid against research-manifest.schema.json: {}",
            problems.join("; ")
        );
    }
    let field = |k: &str| -> String {
        manifest
            .get(k)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };
    let report = manifest
        .get("files")
        .and_then(|f| f.get("report"))
        .and_then(|v| v.as_str())
        .unwrap_or("report.md");
    // The manifest is data the pipeline wrote from fetched content; a report
    // path must stay inside the package, never absolute and never `..`.
    {
        use std::path::Component;
        let escapes = std::path::Path::new(report).components().any(|c| {
            matches!(c, Component::ParentDir | Component::RootDir | Component::Prefix(_))
        });
        if escapes {
            anyhow::bail!("manifest files.report {report:?} points outside the package directory");
        }
    }
    let report_path = dir.join(report);
    if !report_path.is_file() {
        anyhow::bail!(
            "manifest names report {report} but {} does not exist",
            report_path.display()
        );
    }
    Ok(ExportSummary {
        package_id: field("package_id"),
        package_dir: dir.display().to_string(),
        manifest_path: manifest_path.display().to_string(),
        report_path: report_path.display().to_string(),
        verification_status: field("verification_status"),
        verification_verdict: field("verification_verdict"),
    })
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
