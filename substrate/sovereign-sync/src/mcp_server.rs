use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{Implementation, ServerCapabilities, ServerInfo},
    serve_server, tool, tool_handler, tool_router,
    transport::io::stdio,
    ServerHandler, ServiceExt,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::info;

use crate::error::SyncError;

// ---------------------------------------------------------------------------
// SkillIndex — keyword-only loader (no embeddings, no external calls)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SkillEntry {
    pub name: String,
    pub description: String,
    pub path: PathBuf,
    pub keywords: Vec<String>,
}

#[derive(Debug)]
pub struct SkillIndex {
    entries: Vec<SkillEntry>,
}

impl SkillIndex {
    pub fn load_from_dir(skills_dir: &Path) -> Self {
        let mut entries = Vec::new();
        if let Ok(iter) = std::fs::read_dir(skills_dir) {
            for entry in iter.flatten() {
                let path = entry.path();
                let skill_md = path.join("SKILL.md");
                if skill_md.exists() {
                    if let Some((name, description, keywords)) = Self::parse_skill_md(&skill_md) {
                        entries.push(SkillEntry {
                            name,
                            description,
                            path,
                            keywords,
                        });
                    }
                }
            }
        }
        info!(
            "SkillIndex loaded {} skills from {:?}",
            entries.len(),
            skills_dir
        );
        Self { entries }
    }

    fn parse_skill_md(path: &Path) -> Option<(String, String, Vec<String>)> {
        let content = std::fs::read_to_string(path).ok()?;
        let mut name = String::new();
        let mut description = String::new();
        let mut in_frontmatter = false;
        let mut frontmatter_done = false;
        let mut line_count = 0;

        for line in content.lines() {
            if line == "---" {
                if !in_frontmatter && line_count == 0 {
                    in_frontmatter = true;
                    line_count += 1;
                    continue;
                } else if in_frontmatter {
                    frontmatter_done = true;
                    break;
                }
            }
            line_count += 1;
            if in_frontmatter {
                if let Some(rest) = line.strip_prefix("name:") {
                    name = rest.trim().to_string();
                } else if let Some(rest) = line.strip_prefix("description:") {
                    description = rest.trim().to_string();
                }
            }
        }
        if !frontmatter_done || name.is_empty() {
            return None;
        }

        // Keywords: name tokens + description tokens (lowercased, deduplicated).
        let mut keywords: Vec<String> = name
            .split('-')
            .chain(description.split_whitespace())
            .map(|t| {
                t.to_lowercase()
                    .trim_matches(|c: char| !c.is_alphanumeric())
                    .to_string()
            })
            .filter(|t| t.len() > 2)
            .collect();
        keywords.dedup();

        Some((name, description, keywords))
    }

    pub fn search(&self, query: &str) -> Vec<&SkillEntry> {
        let tokens: Vec<String> = query.split_whitespace().map(|t| t.to_lowercase()).collect();
        let mut results: Vec<(&SkillEntry, usize)> = self
            .entries
            .iter()
            .filter_map(|e| {
                let score = tokens
                    .iter()
                    .filter(|t| {
                        e.keywords.iter().any(|k| k.contains(t.as_str()))
                            || e.name.contains(t.as_str())
                            || e.description.to_lowercase().contains(t.as_str())
                    })
                    .count();
                if score > 0 {
                    Some((e, score))
                } else {
                    None
                }
            })
            .collect();
        results.sort_by(|a, b| b.1.cmp(&a.1));
        results.into_iter().map(|(e, _)| e).collect()
    }
}

// ---------------------------------------------------------------------------
// Tool parameter schemas
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SearchSkillsParams {
    /// Natural language query to find matching skills.
    pub query: String,
    /// Maximum results to return (default: 5).
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    5
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SyncStatusParams {
    /// Domain to check status for (optional — all domains if empty).
    #[serde(default)]
    pub domain: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SyncPushParams {
    /// Domain to push (e.g. "kbd-orchestrator", "learner-model").
    pub domain: String,
}

// ---------------------------------------------------------------------------
// SovereignMcpServer
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct SovereignMcpServer {
    tool_router: ToolRouter<Self>,
    skill_index: Arc<SkillIndex>,
    prefix_tools: bool,
    uar_passthrough: bool,
}

impl SovereignMcpServer {
    pub fn new(skills_dir: &Path, prefix_tools: bool, uar_passthrough: bool) -> Self {
        let skill_index = Arc::new(SkillIndex::load_from_dir(skills_dir));
        Self {
            tool_router: Self::tool_router(),
            skill_index,
            prefix_tools,
            uar_passthrough,
        }
    }

    /// Start the MCP server on stdio (blocks until client disconnects).
    pub async fn serve_stdio(self) -> anyhow::Result<()> {
        let transport = stdio();
        let running = serve_server(self, transport).await?;
        running.waiting().await?;
        Ok(())
    }
}

#[tool_router(router = tool_router)]
impl SovereignMcpServer {
    /// Search the local skill index for skills matching a query.
    /// Uses keyword matching — no external calls, privacy-safe.
    #[tool(
        name = "search-skills",
        description = "Search the local skill index for skills matching a natural language query."
    )]
    pub async fn search_skills(&self, params: Parameters<SearchSkillsParams>) -> String {
        let p = params.0;
        let results = self.skill_index.search(&p.query);
        let limited: Vec<_> = results.into_iter().take(p.limit).collect();
        if limited.is_empty() {
            return format!("No skills found matching: {}", p.query);
        }
        let mut out = format!("Found {} skill(s) for \"{}\":\n\n", limited.len(), p.query);
        for entry in limited {
            out.push_str(&format!("- **{}**: {}\n", entry.name, entry.description));
        }
        out
    }

    /// Get the current sync status (node state, connected peers, domain versions).
    #[tool(
        name = "sync-status",
        description = "Get the current P2P sync status including node state and connected peers."
    )]
    pub async fn sync_status(&self, params: Parameters<SyncStatusParams>) -> String {
        let domain = params.0.domain.unwrap_or_else(|| "all".into());
        // Detailed implementation wired in change-sync-010 (REST API).
        // Stub response for MCP server scaffolding.
        format!(
            "sovereign-sync node status:\n  domain: {}\n  state: idle\n  peers: 0\n  (full status available via REST on :7892)",
            domain
        )
    }

    /// Push local CRDT state for a domain to connected peers.
    #[tool(
        name = "sync-push",
        description = "Push local CRDT state for a sync domain to connected P2P peers."
    )]
    pub async fn sync_push(&self, params: Parameters<SyncPushParams>) -> String {
        let domain = &params.0.domain;
        // Full implementation wired in change-sync-015.
        format!(
            "sync-push for domain '{}' queued — peer broadcast will occur when P2P node is active.",
            domain
        )
    }

    /// List known P2P peers.
    #[tool(
        name = "sync-peers",
        description = "List known P2P peers in the current sync group."
    )]
    pub async fn sync_peers(&self) -> String {
        // Full implementation in change-sync-014.
        "No peers connected yet. Start the daemon with --mode daemon to enable P2P.".into()
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for SovereignMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("sovereign-sync", "0.1.0"))
    }
}
