//! forge-reflect — Iteration reflection and Karpathy ingestion.
//!
//! Processes a completed iteration record, computes skill drift, records
//! constitution violations, and pipes the result to `pk ingest` to feed
//! the Karpathy learning loop in prometheus-knowledge.

use anyhow::{Context, Result};
use chrono::Utc;
use forge_core::{DriftReport, DriftType, IterationRecord, Language, SkillDrift, SkillDriftSummary};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::info;
use uuid::Uuid;

// ─── Reflector ───────────────────────────────────────────────────────────────

pub struct Reflector {
    forge_dir: PathBuf,
}

impl Reflector {
    pub fn new(project_root: &Path) -> Self {
        Self {
            forge_dir: project_root.join(".forge"),
        }
    }

    /// Process a completed iteration: compute drift, update memory, ingest to pk.
    pub async fn reflect(&self, iteration_id: &str) -> Result<IterationRecord> {
        // Load the iteration record
        let record = self.load_iteration(iteration_id)?;

        // Compute drift summary
        let drift = compute_drift(&record);

        // Persist drift to .forge/memory/drift/
        self.persist_drift(&drift)?;

        // Persist the iteration record to .forge/memory/iterations/
        self.persist_iteration(&record)?;

        // Ingest to prometheus-knowledge (Karpathy loop)
        self.ingest_to_pk(&record, &drift).await?;

        Ok(record)
    }

    /// Load an iteration record from `.forge/memory/iterations/<id>.json`
    /// or from `.forge/enriched/<id>.context.md` (fallback: create from context).
    fn load_iteration(&self, iteration_id: &str) -> Result<IterationRecord> {
        let iterations_dir = self.forge_dir.join("memory").join("iterations");
        let record_path = iterations_dir.join(format!("{}.json", iteration_id));

        if record_path.exists() {
            let raw = std::fs::read_to_string(&record_path)?;
            return Ok(serde_json::from_str(&raw)?);
        }

        // No iteration record yet — create a stub from the enrichment context
        let context_path = self
            .forge_dir
            .join("enriched")
            .join(format!("{}.context.md", iteration_id));

        let task_description = if context_path.exists() {
            std::fs::read_to_string(&context_path)?
        } else {
            format!("Iteration {}", iteration_id)
        };

        Ok(IterationRecord {
            id: Uuid::new_v4(),
            task_id: iteration_id.to_string(),
            language: Language::Rust, // default — will be overridden when user provides data
            applied_skills: vec![],
            agent_produced: String::new(),
            user_accepted: None,
            diff_summary: None,
            skill_drift: vec![],
            constitution_violations: vec![],
            completed_at: Utc::now(),
        })
    }

    fn persist_drift(&self, drift: &DriftReport) -> Result<()> {
        let drift_dir = self.forge_dir.join("memory").join("drift");
        std::fs::create_dir_all(&drift_dir)?;

        let path = drift_dir.join(format!(
            "{}-{}.json",
            drift.language.as_str(),
            drift.generated_at.format("%Y%m%d")
        ));
        std::fs::write(path, serde_json::to_string_pretty(drift)?)?;
        Ok(())
    }

    fn persist_iteration(&self, record: &IterationRecord) -> Result<()> {
        let iterations_dir = self.forge_dir.join("memory").join("iterations");
        std::fs::create_dir_all(&iterations_dir)?;

        let path = iterations_dir.join(format!("{}.json", record.task_id));
        std::fs::write(path, serde_json::to_string_pretty(record)?)?;
        Ok(())
    }

    async fn ingest_to_pk(&self, record: &IterationRecord, drift: &DriftReport) -> Result<()> {
        let summary = format_ingestion_summary(record, drift);
        let source = format!("forge:reflect:{}", record.task_id);

        ingest_to_pk_cli(&summary, &source).await
    }
}

// ─── Drift computation ────────────────────────────────────────────────────────

fn compute_drift(record: &IterationRecord) -> DriftReport {
    let mut skill_map: HashMap<String, Vec<DriftType>> = HashMap::new();

    for drift in &record.skill_drift {
        skill_map
            .entry(drift.skill_name.clone())
            .or_default()
            .push(drift.drift_type.clone());
    }

    let skills = skill_map
        .into_iter()
        .map(|(name, types)| {
            let total = types.len() as u32;
            let accepted = types.iter().filter(|t| matches!(t, DriftType::Accepted)).count() as u32;
            let modified = types.iter().filter(|t| matches!(t, DriftType::Modified)).count() as u32;
            let replaced = types.iter().filter(|t| matches!(t, DriftType::Replaced)).count() as u32;
            let deleted  = types.iter().filter(|t| matches!(t, DriftType::Deleted)).count()  as u32;
            let acceptance_rate = if total > 0 { accepted as f32 / total as f32 } else { 1.0 };

            SkillDriftSummary {
                skill_name: name,
                total_applications: total,
                accepted_count: accepted,
                modified_count: modified,
                replaced_count: replaced,
                deleted_count: deleted,
                acceptance_rate,
                stale_candidate: acceptance_rate < 0.5,
            }
        })
        .collect();

    DriftReport {
        generated_at: Utc::now(),
        language: record.language.clone(),
        skills,
    }
}

// ─── Ingestion summary ───────────────────────────────────────────────────────

fn format_ingestion_summary(record: &IterationRecord, drift: &DriftReport) -> String {
    let mut lines = vec![
        format!("# Forge Reflect: {}", record.task_id),
        format!("Language: {:?}", record.language),
        format!("Completed: {}", record.completed_at.format("%Y-%m-%d %H:%M UTC")),
        String::new(),
        "## Applied Skills".to_string(),
    ];

    for skill in &record.applied_skills {
        lines.push(format!("- {}", skill));
    }

    if let Some(diff) = &record.diff_summary {
        lines.push(String::new());
        lines.push("## What Changed (user edits)".to_string());
        lines.push(diff.clone());
    }

    // Stale skills
    let stale: Vec<&SkillDriftSummary> = drift
        .skills
        .iter()
        .filter(|s| s.stale_candidate)
        .collect();

    if !stale.is_empty() {
        lines.push(String::new());
        lines.push("## Stale Skill Candidates (acceptance < 50%)".to_string());
        for s in stale {
            lines.push(format!(
                "- {} (accepted {}/{}, rate: {:.0}%)",
                s.skill_name,
                s.accepted_count,
                s.total_applications,
                s.acceptance_rate * 100.0
            ));
        }
    }

    if !record.constitution_violations.is_empty() {
        lines.push(String::new());
        lines.push("## Recurring Constitution Violations".to_string());
        for v in &record.constitution_violations {
            lines.push(format!("- `{}`: {} (recurrence: {})", v.rule, v.occurrence, v.recurrence_count));
        }
    }

    lines.join("\n")
}

// ─── pk ingest integration ────────────────────────────────────────────────────

async fn ingest_to_pk_cli(content: &str, source: &str) -> Result<()> {
    use tokio::io::AsyncWriteExt;
    use tokio::process::Command;

    let mut child = Command::new("pk")
        .args(["ingest", "--source", source])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .spawn()
        .context("spawning pk ingest")?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(content.as_bytes())
            .await
            .context("writing to pk stdin")?;
    }

    let status = child.wait().await.context("waiting for pk ingest")?;
    if !status.success() {
        tracing::warn!("pk ingest exited with status: {}", status);
    } else {
        info!("Ingested reflection to prometheus-knowledge (source: {})", source);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use forge_core::{DriftType, IterationRecord, Language, SkillDrift};
    use uuid::Uuid;

    fn make_record(drift: Vec<SkillDrift>) -> IterationRecord {
        IterationRecord {
            id: Uuid::new_v4(),
            task_id: "test-task".to_string(),
            language: Language::Rust,
            applied_skills: vec!["rust/error-handling".to_string()],
            agent_produced: String::new(),
            user_accepted: None,
            diff_summary: None,
            skill_drift: drift,
            constitution_violations: vec![],
            completed_at: Utc::now(),
        }
    }

    fn skill_drift(name: &str, dtype: DriftType) -> SkillDrift {
        SkillDrift {
            skill_name: name.to_string(),
            override_description: String::new(),
            drift_type: dtype,
        }
    }

    // ─── compute_drift ────────────────────────────────────────────────────────

    #[test]
    fn compute_drift_empty_record_produces_empty_report() {
        let record = make_record(vec![]);
        let report = compute_drift(&record);
        assert!(report.skills.is_empty());
        assert_eq!(report.language, Language::Rust);
    }

    #[test]
    fn compute_drift_accepted_only_gives_acceptance_rate_one() {
        let record = make_record(vec![
            skill_drift("axum-patterns", DriftType::Accepted),
            skill_drift("axum-patterns", DriftType::Accepted),
        ]);
        let report = compute_drift(&record);
        assert_eq!(report.skills.len(), 1);
        let summary = &report.skills[0];
        assert_eq!(summary.skill_name, "axum-patterns");
        assert_eq!(summary.accepted_count, 2);
        assert_eq!(summary.total_applications, 2);
        assert!((summary.acceptance_rate - 1.0).abs() < f32::EPSILON);
        assert!(!summary.stale_candidate, "all-accepted skill must not be stale");
    }

    #[test]
    fn compute_drift_below_half_acceptance_marks_stale() {
        let record = make_record(vec![
            skill_drift("axum-patterns", DriftType::Accepted),
            skill_drift("axum-patterns", DriftType::Modified),
            skill_drift("axum-patterns", DriftType::Replaced),
        ]);
        let report = compute_drift(&record);
        let summary = &report.skills[0];
        assert_eq!(summary.accepted_count, 1);
        assert_eq!(summary.total_applications, 3);
        assert!(
            summary.stale_candidate,
            "acceptance_rate = 1/3 < 0.5, so stale_candidate must be true"
        );
    }

    #[test]
    fn compute_drift_exactly_half_acceptance_is_not_stale() {
        let record = make_record(vec![
            skill_drift("s", DriftType::Accepted),
            skill_drift("s", DriftType::Deleted),
        ]);
        let report = compute_drift(&record);
        let summary = &report.skills[0];
        // acceptance_rate = 0.5 — NOT < 0.5, so not stale
        assert!(!summary.stale_candidate);
    }

    #[test]
    fn compute_drift_multiple_skills_aggregated_independently() {
        let record = make_record(vec![
            skill_drift("a", DriftType::Accepted),
            skill_drift("b", DriftType::Modified),
            skill_drift("a", DriftType::Replaced),
        ]);
        let report = compute_drift(&record);
        assert_eq!(report.skills.len(), 2);

        let a = report.skills.iter().find(|s| s.skill_name == "a").unwrap();
        assert_eq!(a.total_applications, 2);
        assert_eq!(a.accepted_count, 1);

        let b = report.skills.iter().find(|s| s.skill_name == "b").unwrap();
        assert_eq!(b.total_applications, 1);
        assert_eq!(b.accepted_count, 0);
    }

    // ─── format_ingestion_summary ─────────────────────────────────────────────

    #[test]
    fn format_ingestion_summary_contains_task_id() {
        let record = make_record(vec![]);
        let drift = compute_drift(&record);
        let summary = format_ingestion_summary(&record, &drift);
        assert!(
            summary.contains("test-task"),
            "summary must contain the task ID"
        );
    }

    #[test]
    fn format_ingestion_summary_lists_stale_skills() {
        let record = make_record(vec![
            skill_drift("slow-skill", DriftType::Modified),
            skill_drift("slow-skill", DriftType::Modified),
        ]);
        let drift = compute_drift(&record);
        let summary = format_ingestion_summary(&record, &drift);
        assert!(
            summary.contains("Stale Skill Candidates"),
            "summary must mention stale skills when acceptance < 50%"
        );
        assert!(summary.contains("slow-skill"));
    }

    #[test]
    fn format_ingestion_summary_omits_stale_section_when_all_fresh() {
        let record = make_record(vec![
            skill_drift("good-skill", DriftType::Accepted),
        ]);
        let drift = compute_drift(&record);
        let summary = format_ingestion_summary(&record, &drift);
        assert!(
            !summary.contains("Stale Skill Candidates"),
            "summary must not have stale section when all skills are accepted"
        );
    }

    // ─── Reflector::new ───────────────────────────────────────────────────────

    #[test]
    fn reflector_new_sets_forge_dir() {
        let r = Reflector::new(std::path::Path::new("/tmp/my-project"));
        assert_eq!(r.forge_dir, std::path::PathBuf::from("/tmp/my-project/.forge"));
    }
}
