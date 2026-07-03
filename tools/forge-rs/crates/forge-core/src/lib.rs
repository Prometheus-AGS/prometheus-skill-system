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
            Language::Rust => "rust",
            Language::TypeScript => "typescript",
            Language::React => "react",
            Language::Flutter => "flutter",
            Language::Go => "go",
            Language::Python => "python",
            Language::Tauri => "tauri",
        }
    }
}

impl std::str::FromStr for Language {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "rust"       => Ok(Language::Rust),
            "typescript" | "ts" => Ok(Language::TypeScript),
            "react"      => Ok(Language::React),
            "flutter"    => Ok(Language::Flutter),
            "go" | "golang" => Ok(Language::Go),
            "python"     => Ok(Language::Python),
            "tauri"      => Ok(Language::Tauri),
            other => Err(anyhow::anyhow!("unknown language: '{}' (expected: rust, typescript, react, flutter, go, python, tauri)", other)),
        }
    }
}

// ─── Constitution ─────────────────────────────────────────────────────────────

/// Per-language coding standards loaded from `.forge/constitution/<lang>.toml`.
///
/// Only `language` is required. The collection fields default to empty when
/// absent so a partial or minimally-populated constitution still parses — a
/// config parser should not hard-fail on a missing optional section (and TOML
/// key ordering can legitimately leave a top-level array unset).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constitution {
    pub language: Language,
    #[serde(default)]
    pub standards: HashMap<String, String>,
    #[serde(default)]
    pub forbidden_patterns: Vec<ForbiddenPattern>,
    #[serde(default)]
    pub required_skills: Vec<String>,
    #[serde(default)]
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
    pub karpathy_focus: Option<String>, // Output of `pk focus` for this task
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
    pub agent_produced: String,        // What the agent wrote
    pub user_accepted: Option<String>, // What the user kept (if they changed it)
    pub diff_summary: Option<String>,  // Summary of what changed between produced and accepted
    pub skill_drift: Vec<SkillDrift>,  // Which skills were overridden and how
    pub constitution_violations: Vec<ConstitutionViolation>,
    pub completed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDrift {
    pub skill_name: String,
    pub override_description: String, // What the user changed that diverged from the skill
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
    pub recurrence_count: u32, // How many times this violation has appeared across iterations
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    // ─── Language::from_str ─────────────────────────────────────────────────

    #[test]
    fn language_from_str_rust() {
        assert_eq!(Language::from_str("rust").unwrap(), Language::Rust);
    }

    #[test]
    fn language_from_str_typescript_alias() {
        assert_eq!(Language::from_str("ts").unwrap(), Language::TypeScript);
    }

    #[test]
    fn language_from_str_case_insensitive() {
        assert_eq!(Language::from_str("RUST").unwrap(), Language::Rust);
        assert_eq!(Language::from_str("Go").unwrap(), Language::Go);
    }

    #[test]
    fn language_from_str_unknown_returns_err() {
        assert!(Language::from_str("cobol").is_err());
    }

    #[test]
    fn language_from_str_golang_alias() {
        assert_eq!(Language::from_str("golang").unwrap(), Language::Go);
    }

    // ─── Language::as_str ───────────────────────────────────────────────────

    #[test]
    fn language_as_str_roundtrip() {
        for lang in [
            Language::Rust,
            Language::TypeScript,
            Language::React,
            Language::Flutter,
            Language::Go,
            Language::Python,
            Language::Tauri,
        ] {
            let s = lang.as_str();
            let parsed = Language::from_str(s).expect("as_str produced invalid slug");
            assert_eq!(lang, parsed, "roundtrip failed for {s}");
        }
    }

    // ─── Severity ordering / equality ───────────────────────────────────────

    #[test]
    fn severity_variants_are_distinct() {
        assert_ne!(Severity::Error, Severity::Warning);
        assert_ne!(Severity::Warning, Severity::Info);
        assert_ne!(Severity::Error, Severity::Info);
    }

    // ─── SkillDriftSummary stale threshold ──────────────────────────────────

    #[test]
    fn drift_summary_stale_candidate_below_threshold() {
        let s = SkillDriftSummary {
            skill_name: "axum-patterns".into(),
            total_applications: 10,
            accepted_count: 3,
            modified_count: 5,
            replaced_count: 2,
            deleted_count: 0,
            acceptance_rate: 0.3,
            stale_candidate: true,
        };
        assert!(s.stale_candidate);
        assert!(s.acceptance_rate < 0.5);
    }

    #[test]
    fn drift_summary_not_stale_above_threshold() {
        let s = SkillDriftSummary {
            skill_name: "error-handling".into(),
            total_applications: 20,
            accepted_count: 18,
            modified_count: 2,
            replaced_count: 0,
            deleted_count: 0,
            acceptance_rate: 0.9,
            stale_candidate: false,
        };
        assert!(!s.stale_candidate);
        assert!(s.acceptance_rate >= 0.5);
    }
}
