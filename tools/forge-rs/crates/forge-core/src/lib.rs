//! forge-core — Domain types for the forge-rs code enrichment engine.
//!
//! All types are `Serialize + Deserialize` and owned-value (no references).
//! This crate has no I/O — it is a pure domain definition.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ─── Language ────────────────────────────────────────────────────────────────

/// A language for which forge-rs has skill manifests and constitution rules.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Rust,
    TypeScript,
    React,
    Flutter,
    Go,
    Python,
    Tauri,
}

impl Language {
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::Rust       => "rust",
            Language::TypeScript => "typescript",
            Language::React      => "react",
            Language::Flutter    => "flutter",
            Language::Go         => "go",
            Language::Python     => "python",
            Language::Tauri      => "tauri",
        }
    }
}

// ─── Constitution ─────────────────────────────────────────────────────────────

/// Per-language coding standards loaded from `.forge/constitution/<lang>.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constitution {
    pub language: Language,
    pub standards: HashMap<String, String>,
    pub forbidden_patterns: Vec<ForbiddenPattern>,
    pub required_skills: Vec<String>,
    pub framework_versions: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForbiddenPattern {
    pub pattern: String,
    pub reason: String,
    pub severity: Severity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

// ─── Skill Manifest ───────────────────────────────────────────────────────────

/// A forge skill manifest loaded from `skills/<lang>/<name>/skill.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillManifest {
    pub name: String,
    pub language: Language,
    pub description: String,
    pub version: String,
    /// Template files in `templates/` rendered via Tera.
    pub templates: Vec<TemplateRef>,
    /// Trigger conditions — when should this skill be injected?
    pub triggers: Vec<SkillTrigger>,
    /// Skills this skill depends on (must be resolved first).
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateRef {
    pub path: String,
    pub output_description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SkillTrigger {
    /// Applies to all tasks in this language.
    AlwaysForLanguage { language: Language },
    /// Applies when the task description contains any of these keywords.
    Keywords { keywords: Vec<String> },
    /// Applies when the task path matches this glob pattern.
    PathGlob { glob: String },
    /// Applies when a specific crate/package is in scope.
    DependsOnPackage { package: String },
}

// ─── Enrichment Context ───────────────────────────────────────────────────────

/// The output of `forge enrich` — consumed by the AI agent as implementation context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichmentContext {
    pub id: Uuid,
    pub task_path: String,
    pub task_description: String,
    pub language: Language,
    pub applied_skills: Vec<String>,
    pub constitution_summary: String,
    pub karpathy_focus: Option<String>,  // Output of `pk focus` for this task
    pub rendered_templates: Vec<RenderedTemplate>,
    pub constitution_warnings: Vec<ConstitutionWarning>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderedTemplate {
    pub skill_name: String,
    pub template_path: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstitutionWarning {
    pub rule: String,
    pub violation: String,
    pub severity: Severity,
}

// ─── Iteration Record ─────────────────────────────────────────────────────────

/// Produced by `forge reflect` — feeds the Karpathy loop via `pk ingest`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationRecord {
    pub id: Uuid,
    pub task_id: String,
    pub language: Language,
    pub applied_skills: Vec<String>,
    pub agent_produced: String,          // What the agent wrote
    pub user_accepted: Option<String>,   // What the user kept (if they changed it)
    pub diff_summary: Option<String>,    // Summary of what changed between produced and accepted
    pub skill_drift: Vec<SkillDrift>,    // Which skills were overridden and how
    pub constitution_violations: Vec<ConstitutionViolation>,
    pub completed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDrift {
    pub skill_name: String,
    pub override_description: String,  // What the user changed that diverged from the skill
    pub drift_type: DriftType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DriftType {
    /// User accepted the skill's suggestion unchanged.
    Accepted,
    /// User modified the skill's suggestion.
    Modified,
    /// User replaced the skill's suggestion entirely.
    Replaced,
    /// User deleted the skill's contribution.
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstitutionViolation {
    pub rule: String,
    pub occurrence: String,
    pub recurrence_count: u32,  // How many times this violation has appeared across iterations
}

// ─── Drift Report ────────────────────────────────────────────────────────────

/// Summary of skill drift across all iterations for a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftReport {
    pub generated_at: DateTime<Utc>,
    pub language: Language,
    pub skills: Vec<SkillDriftSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDriftSummary {
    pub skill_name: String,
    pub total_applications: u32,
    pub accepted_count: u32,
    pub modified_count: u32,
    pub replaced_count: u32,
    pub deleted_count: u32,
    /// 0.0 (always overridden) to 1.0 (always accepted unchanged)
    pub acceptance_rate: f32,
    /// If acceptance_rate < 0.5, this skill may be stale.
    pub stale_candidate: bool,
}
