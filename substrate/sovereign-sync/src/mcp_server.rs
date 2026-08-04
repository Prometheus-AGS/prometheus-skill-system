use kbd_runtime::{Actor, ActorKind, Checkpoint, CommandEnvelope, CommandKind, KbdStateV2};
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{Implementation, ServerCapabilities, ServerInfo},
    serve_server, tool, tool_handler, tool_router,
    transport::io::stdio,
    ServerHandler,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

use crate::kbd_control::KbdControlPlane;
use crate::kbd_control::KbdProjectRouter;
use crate::rest_api::{execute_signed_sync_push, sync_peers_value, sync_status_value, AppState};

// ---------------------------------------------------------------------------
// SkillIndex — keyword-only loader (no embeddings, no external calls)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SkillEntry {
    pub name: String,
    pub description: String,
    pub path: PathBuf,
    pub keywords: Vec<String>,
    pub search_text: String,
}

#[derive(Debug)]
pub struct SkillIndex {
    entries: Vec<SkillEntry>,
    /// Entries merged in from a synced peer's skill-index domain (see
    /// `domains::SkillIndexAdapter`). Kept separate from `entries` so a
    /// local reload never clobbers synced state and vice versa.
    remote: std::sync::RwLock<Vec<SkillEntry>>,
}

impl SkillIndex {
    pub fn load_from_dir(skills_dir: &Path) -> Self {
        #[cfg(test)]
        if skills_dir.join(".slow-scan-test").is_file() {
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
        let generation_index = skills_dir
            .parent()
            .map(|parent| parent.join("indexes/skills.json"));
        let mut entries: Vec<SkillEntry> = generation_index
            .filter(|candidate| candidate.is_file())
            .and_then(|candidate| prometheus_skill_index::SkillIndex::from_path(&candidate).ok())
            .map(|index| {
                index
                    .entries
                    .into_iter()
                    .map(|entry| SkillEntry {
                        name: entry.id,
                        description: entry.description,
                        path: skills_dir.join(entry.relative_path),
                        keywords: Vec::new(),
                        search_text: entry.search_text,
                    })
                    .collect()
            })
            .unwrap_or_default();
        if entries.is_empty() {
            if let Ok(iter) = std::fs::read_dir(skills_dir) {
                for entry in iter.flatten() {
                    let path = entry.path();
                    let skill_md = path.join("SKILL.md");
                    if skill_md.exists() {
                        if let Some((name, description, keywords)) = Self::parse_skill_md(&skill_md)
                        {
                            let search_text = format!("{name} {description}").to_lowercase();
                            entries.push(SkillEntry {
                                name,
                                description,
                                path,
                                keywords,
                                search_text,
                            });
                        }
                    }
                }
            }
        }
        entries.sort_by(|left, right| left.name.cmp(&right.name));
        info!(
            "SkillIndex loaded {} skills from {:?}",
            entries.len(),
            skills_dir
        );
        Self {
            entries,
            remote: std::sync::RwLock::new(Vec::new()),
        }
    }

    /// Local (non-synced) entries, for the skill-index sync adapter's export path.
    pub fn local_entries(&self) -> &[SkillEntry] {
        &self.entries
    }

    /// Replace the full set of remote (synced-from-peer) entries, for the
    /// skill-index sync adapter's import path.
    pub fn replace_remote(&self, remote: Vec<SkillEntry>) {
        if let Ok(mut guard) = self.remote.write() {
            *guard = remote;
        }
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

    pub fn search(&self, query: &str) -> Vec<SkillEntry> {
        let remote_guard = self.remote.read().ok();
        let remote_entries: &[SkillEntry] =
            remote_guard.as_deref().map(|v| v.as_slice()).unwrap_or(&[]);
        let combined: Vec<SkillEntry> = self
            .entries
            .iter()
            .chain(remote_entries.iter())
            .cloned()
            .collect();
        let canonical = prometheus_skill_index::SkillIndex {
            schema_version: prometheus_skill_index::SCHEMA_VERSION.to_string(),
            entries: combined
                .iter()
                .map(|entry| prometheus_skill_index::SkillIndexEntry {
                    id: entry.name.clone(),
                    description: entry.description.clone(),
                    relative_path: entry.path.to_string_lossy().into_owned(),
                    search_text: entry.search_text.clone(),
                })
                .collect(),
            sha256: String::new(),
        };
        canonical
            .search(query, usize::MAX)
            .into_iter()
            .filter_map(|selected| {
                combined
                    .iter()
                    .find(|entry| entry.name == selected.id)
                    .cloned()
            })
            .collect()
    }
}

#[cfg(test)]
mod skill_index_tests {
    use super::*;

    #[test]
    fn generation_index_is_the_host_search_authority() {
        let generation = tempfile::tempdir().unwrap();
        let skills = generation.path().join("skills");
        let indexes = generation.path().join("indexes");
        std::fs::create_dir_all(skills.join("backend/api-design")).unwrap();
        std::fs::create_dir_all(&indexes).unwrap();
        std::fs::write(
            skills.join("backend/api-design/SKILL.md"),
            "---\nname: wrong-file-name\ndescription: should not be rescanned\n---\n",
        )
        .unwrap();
        std::fs::write(
            indexes.join("skills.json"),
            r#"{
  "entries": [
    {
      "description": "REST contracts",
      "id": "api-design",
      "relativePath": "backend/api-design",
      "searchText": "api-design rest contracts"
    },
    {
      "description": "Rust correctness",
      "id": "rust-review",
      "relativePath": "rust/rust-review",
      "searchText": "rust-review rust correctness"
    }
  ],
  "schemaVersion": "prometheus-skill-index-v1",
  "sha256": "08cd6d1575faa7966b5aef855f3091a0e4e1a53d8f5fba2806519934a16a9888"
}
"#,
        )
        .unwrap();

        let index = SkillIndex::load_from_dir(&skills);
        let selected = index.search("rest contracts");
        assert_eq!(selected[0].name, "api-design");
        assert!(index.search("wrong-file-name").is_empty());
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

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct KbdReasonParams {
    pub project_id: Option<String>,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct KbdResumeParams {
    pub project_id: Option<String>,
    pub plan_revision: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct KbdReviseParams {
    pub project_id: Option<String>,
    pub reason: String,
    pub exact_next_work: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct KbdEventsParams {
    pub project_id: Option<String>,
    pub since_revision: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct KbdProjectParams {
    pub project_id: Option<String>,
}

// ---------------------------------------------------------------------------
// SovereignMcpServer
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct SovereignMcpServer {
    tool_router: ToolRouter<Self>,
    skill_index: Arc<SkillIndex>,
    kbd_projects: Arc<KbdProjectRouter>,
    sync_state: AppState,
    _prefix_tools: bool,
    _uar_passthrough: bool,
}

impl SovereignMcpServer {
    pub async fn new(skills_dir: &Path, prefix_tools: bool, uar_passthrough: bool) -> Self {
        let skill_index = Arc::new(SkillIndex::load_from_dir(skills_dir));
        let kbd_projects = Arc::new(
            KbdProjectRouter::open_registered()
                .await
                .expect("cannot open registered KBD control planes"),
        );
        if let Some(project_root) = std::env::current_dir().ok().and_then(|cwd| {
            cwd.ancestors()
                .find(|candidate| candidate.join(".prometheus/project.json").is_file())
                .map(Path::to_path_buf)
        }) {
            kbd_projects
                .ensure_registered_path(&project_root)
                .await
                .expect("cannot ensure current KBD project registration");
        }
        let learner_model_dir = dirs_next::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".prometheus")
            .join("learn")
            .join("learner-model");
        let sync_state = AppState::from_mcp_components(
            kbd_projects.clone(),
            skill_index.clone(),
            learner_model_dir,
        )
        .expect("cannot open sync receipt service");
        Self {
            tool_router: Self::tool_router(),
            skill_index,
            kbd_projects,
            sync_state,
            _prefix_tools: prefix_tools,
            _uar_passthrough: uar_passthrough,
        }
    }

    /// Start the MCP server on stdio (blocks until client disconnects).
    pub async fn serve_stdio(self) -> anyhow::Result<()> {
        let transport = stdio();
        let running = serve_server(self, transport).await?;
        running.waiting().await?;
        Ok(())
    }

    async fn submit_fresh(
        &self,
        control: Arc<KbdControlPlane>,
        state: KbdStateV2,
        actor_kind: ActorKind,
        command: CommandKind,
    ) -> String {
        let envelope = CommandEnvelope {
            schema_version: "2".into(),
            project_id: state.project_id.clone(),
            run_id: state.run_id.clone(),
            command_id: Uuid::new_v4().to_string(),
            frontier: Some(state.frontier.clone()),
            expected_revision: state.revision,
            actor: mcp_actor(actor_kind),
            command,
        };
        match control.submit(envelope).await {
            Ok(committed) => {
                serde_json::to_string_pretty(&committed).unwrap_or_else(|error| error.to_string())
            }
            Err(error) => format!("KBD control error: {error}"),
        }
    }

    fn control(&self, project_id: Option<&str>) -> Result<Arc<KbdControlPlane>, String> {
        if let Some(project_id) = project_id {
            return self
                .kbd_projects
                .control(project_id)
                .map_err(|error| error.to_string());
        }
        let project_ids = self.kbd_projects.project_ids();
        if project_ids.len() != 1 {
            return Err(format!(
                "project_id is required when {} projects are registered: {}",
                project_ids.len(),
                project_ids.join(", ")
            ));
        }
        self.kbd_projects
            .control(&project_ids[0])
            .map_err(|error| error.to_string())
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
        let mut status = sync_status_value(&self.sync_state).await;
        if let Some(domain) = params.0.domain {
            status["requestedDomain"] = serde_json::Value::String(domain);
        }
        serde_json::to_string_pretty(&status).unwrap_or_else(|error| error.to_string())
    }

    /// Push local CRDT state for a domain to connected peers.
    #[tool(
        name = "sync-push",
        description = "Push local CRDT state for a sync domain to connected P2P peers."
    )]
    pub async fn sync_push(&self, params: Parameters<SyncPushParams>) -> String {
        let request = match self.sync_state.signed_local_push_request(params.0.domain) {
            Ok(request) => request,
            Err(error) => return format!("sync-push signing error: {error}"),
        };
        match execute_signed_sync_push(&self.sync_state, request).await {
            Ok((_, receipt)) => {
                serde_json::to_string_pretty(&receipt).unwrap_or_else(|error| error.to_string())
            }
            Err(response) => format!("sync-push failed with HTTP status {}", response.status()),
        }
    }

    /// List known P2P peers.
    #[tool(
        name = "sync-peers",
        description = "List known P2P peers in the current sync group."
    )]
    pub async fn sync_peers(&self) -> String {
        serde_json::to_string_pretty(&sync_peers_value(&self.sync_state).await)
            .unwrap_or_else(|error| error.to_string())
    }

    #[tool(
        name = "kbd_projects",
        description = "List registered KBD projects, replicas, and route readiness."
    )]
    pub async fn kbd_projects(&self) -> String {
        match self.kbd_projects.routes() {
            Ok(routes) => {
                serde_json::to_string_pretty(&routes).unwrap_or_else(|error| error.to_string())
            }
            Err(error) => format!("KBD registry error: {error}"),
        }
    }

    #[tool(
        name = "kbd_status",
        description = "Read canonical KBD lifecycle, revision, checkpoint, and workflow state."
    )]
    pub async fn kbd_status(&self, params: Parameters<KbdProjectParams>) -> String {
        let control = match self.control(params.0.project_id.as_deref()) {
            Ok(control) => control,
            Err(error) => return format!("KBD project error: {error}"),
        };
        match control.status() {
            Ok(state) => {
                serde_json::to_string_pretty(&state).unwrap_or_else(|error| error.to_string())
            }
            Err(error) => format!("KBD status error: {error}"),
        }
    }

    #[tool(
        name = "kbd_pause",
        description = "Checkpoint and pause the KBD run. Operator pause overrides writer steering."
    )]
    pub async fn kbd_pause(&self, params: Parameters<KbdReasonParams>) -> String {
        let control = match self.control(params.0.project_id.as_deref()) {
            Ok(control) => control,
            Err(error) => return format!("KBD project error: {error}"),
        };
        match control.status() {
            Ok(state) => {
                let command = CommandKind::Pause {
                    checkpoint: Checkpoint {
                        reason: params.0.reason,
                        previous_state: state.lifecycle.clone(),
                        last_completed: None,
                        exact_next_work: state.exact_next_work.clone(),
                        decisions: Vec::new(),
                        blockers: Vec::new(),
                        dirty_work_summary: None,
                        plan_revision: state.plan_revision,
                    },
                };
                self.submit_fresh(control, state, ActorKind::Operator, command)
                    .await
            }
            Err(error) => format!("KBD status error: {error}"),
        }
    }

    #[tool(
        name = "kbd_revise",
        description = "Create an immutable N+1 plan revision that supersedes the previous next work."
    )]
    pub async fn kbd_revise(&self, params: Parameters<KbdReviseParams>) -> String {
        let control = match self.control(params.0.project_id.as_deref()) {
            Ok(control) => control,
            Err(error) => return format!("KBD project error: {error}"),
        };
        match control.status() {
            Ok(state) => {
                self.submit_fresh(
                    control,
                    state,
                    ActorKind::Harness,
                    CommandKind::PlanRevise {
                        reason: params.0.reason,
                        exact_next_work: params.0.exact_next_work,
                    },
                )
                .await
            }
            Err(error) => format!("KBD status error: {error}"),
        }
    }

    #[tool(
        name = "kbd_resume",
        description = "Resume a paused KBD run at the validated plan revision."
    )]
    pub async fn kbd_resume(&self, params: Parameters<KbdResumeParams>) -> String {
        let control = match self.control(params.0.project_id.as_deref()) {
            Ok(control) => control,
            Err(error) => return format!("KBD project error: {error}"),
        };
        match control.status() {
            Ok(state) => {
                let plan_revision = params.0.plan_revision.unwrap_or(state.plan_revision);
                self.submit_fresh(
                    control,
                    state,
                    ActorKind::Operator,
                    CommandKind::Resume { plan_revision },
                )
                .await
            }
            Err(error) => format!("KBD status error: {error}"),
        }
    }

    #[tool(
        name = "kbd_cancel",
        description = "Gracefully cancel the KBD run while preserving its audit history."
    )]
    pub async fn kbd_cancel(&self, params: Parameters<KbdReasonParams>) -> String {
        let control = match self.control(params.0.project_id.as_deref()) {
            Ok(control) => control,
            Err(error) => return format!("KBD project error: {error}"),
        };
        match control.status() {
            Ok(state) => {
                self.submit_fresh(
                    control,
                    state,
                    ActorKind::Operator,
                    CommandKind::Cancel {
                        reason: params.0.reason,
                    },
                )
                .await
            }
            Err(error) => format!("KBD status error: {error}"),
        }
    }

    #[tool(
        name = "kbd_events",
        description = "Read immutable KBD events from an optional starting revision."
    )]
    pub async fn kbd_events(&self, params: Parameters<KbdEventsParams>) -> String {
        let control = match self.control(params.0.project_id.as_deref()) {
            Ok(control) => control,
            Err(error) => return format!("KBD project error: {error}"),
        };
        match control.events(params.0.since_revision.unwrap_or(1)) {
            Ok(events) => {
                serde_json::to_string_pretty(&events).unwrap_or_else(|error| error.to_string())
            }
            Err(error) => format!("KBD events error: {error}"),
        }
    }
}

fn mcp_actor(kind: ActorKind) -> Actor {
    let mut actor = Actor::operator("sovereign-sync", "mcp");
    actor.kind = kind;
    actor
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for SovereignMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("sovereign-sync", "0.1.0"))
    }
}
