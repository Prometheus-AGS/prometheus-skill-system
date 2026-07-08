use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ResearchStartParams {
    /// Research query — the topic or question to investigate.
    pub query: String,
    /// Research depth: "shallow", "moderate", or "exhaustive".
    #[serde(default = "default_depth")]
    pub depth: String,
    /// Maximum number of sources to gather.
    #[serde(default = "default_max_sources")]
    pub max_sources: u32,
    /// Citation style: "apa", "mla", "chicago", or "none".
    #[serde(default = "default_citation_style")]
    pub citation_style: String,
}

fn default_depth() -> String {
    "moderate".into()
}

fn default_max_sources() -> u32 {
    10
}

fn default_citation_style() -> String {
    "apa".into()
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ResearchStatusParams {
    /// Job ID returned by research_start.
    pub job_id: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ResearchCancelParams {
    /// Job ID to cancel.
    pub job_id: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ResearchExportParams {
    /// Job ID to export results from.
    pub job_id: String,
    /// Export format: "markdown", "json", or "html".
    #[serde(default = "default_export_format")]
    pub format: String,
}

fn default_export_format() -> String {
    "markdown".into()
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RenderComponentParams {
    /// A2UI component name (e.g. "graph_view", "source_list", "progress_ring").
    pub name: String,
    /// Component props as a JSON object.
    #[serde(default)]
    pub props: serde_json::Value,
}
