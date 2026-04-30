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
