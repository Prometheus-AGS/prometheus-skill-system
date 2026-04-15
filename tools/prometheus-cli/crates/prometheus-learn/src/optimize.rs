//! Skill prompt optimization via dspy-rs.
//!
//! Converts SKILL.md prompts into dspy-rs Signatures, builds Datasets from
//! execution traces, and runs BootstrapFewShot + MIPRO to optimize prompts.
//!
//! Requires the `optimize` feature flag. Without it, provides SKILL.md parsing
//! and analysis but not actual optimization.

use crate::trace::ExecutionTrace;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A parsed skill definition ready for optimization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSignature {
    pub name: String,
    pub description: String,
    pub instruction: String,
    pub input_fields: Vec<SignatureField>,
    pub output_fields: Vec<SignatureField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureField {
    pub name: String,
    pub description: Option<String>,
}

/// Report from a skill optimization run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationReport {
    pub skill_name: String,
    pub traces_used: usize,
    pub successful_traces: usize,
    pub demos_collected: usize,
    pub instruction_optimized: bool,
    pub before_score: f64,
    pub after_score: f64,
    pub improvement_pct: f64,
}

/// Parse a SKILL.md file into a SkillSignature.
pub fn parse_skill_to_signature(skill_path: &Path) -> anyhow::Result<SkillSignature> {
    let content = std::fs::read_to_string(skill_path.join("SKILL.md"))
        .map_err(|e| anyhow::anyhow!("Failed to read SKILL.md: {}", e))?;

    let (frontmatter, body) = parse_frontmatter(&content)?;

    Ok(SkillSignature {
        name: frontmatter.name,
        description: frontmatter.description,
        instruction: body.trim().to_string(),
        input_fields: vec![
            SignatureField { name: "task".into(), description: Some("What to accomplish".into()) },
            SignatureField { name: "context".into(), description: Some("Project context and constraints".into()) },
        ],
        output_fields: vec![
            SignatureField { name: "result".into(), description: Some("The skill's output".into()) },
            SignatureField { name: "reasoning".into(), description: Some("Why this approach was chosen".into()) },
        ],
    })
}

// ─── Feature-gated dspy-rs optimization ─────────────────────────────────────

#[cfg(feature = "optimize")]
mod inner {
    use super::*;
    use dspy::{
        Signature as DspySignature, Predict, BootstrapFewShot,
        Example, Dataset, Record, Value, Metric,
        LanguageModel, OpenAIModel,
    };
    use std::sync::Arc;

    pub struct SkillOptimizer {
        model: Arc<dyn LanguageModel>,
    }

    impl SkillOptimizer {
        /// Create an optimizer using the configured model backend.
        ///
        /// Routes through local model by default (sovereignty-safe — no data egress).
        /// Uses the same routing pattern as pk-librarian's ModelRouter:
        /// - `$OPTIMIZER_LLM_URL` → local endpoint (default: http://localhost:1234/v1)
        /// - `$OPTIMIZER_LLM_MODEL` → model name (default: qwen2.5-14b-instruct)
        /// - Set `$OPTIMIZER_USE_CLOUD=true` to route through cloud API instead
        pub fn new() -> anyhow::Result<Self> {
            let use_cloud = std::env::var("OPTIMIZER_USE_CLOUD")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false);

            let base_url = if use_cloud {
                std::env::var("CLOUD_LLM_URL")
                    .unwrap_or_else(|_| "https://api.anthropic.com/v1".into())
            } else {
                std::env::var("OPTIMIZER_LLM_URL")
                    .unwrap_or_else(|_| "http://localhost:1234/v1".into())
            };

            let model_name = std::env::var("OPTIMIZER_LLM_MODEL")
                .unwrap_or_else(|_| if use_cloud {
                    "claude-sonnet-4-6".into()
                } else {
                    "qwen2.5-14b-instruct-q4_k_m".into()
                });

            let api_key = if use_cloud {
                std::env::var("CLOUD_LLM_API_KEY").ok()
            } else {
                None // Local models don't need API keys
            };

            // OpenAIModel uses the OpenAI-compatible API format, which works
            // with local servers (LM Studio, Ollama, vLLM) that expose /v1/chat/completions
            let mut model = OpenAIModel::new(&model_name);
            model.base_url = Some(base_url);
            model.api_key = api_key;

            tracing::info!(
                model = %model_name,
                cloud = use_cloud,
                "Optimizer model configured"
            );

            Ok(Self { model: Arc::new(model) })
        }

        pub fn to_dspy_signature(sig: &SkillSignature) -> DspySignature {
            let mut dsig = DspySignature::new(&sig.name)
                .with_instruction(&sig.instruction);
            for f in &sig.input_fields {
                dsig = dsig.input(&f.name, f.description.as_deref());
            }
            for f in &sig.output_fields {
                dsig = dsig.output(&f.name, f.description.as_deref());
            }
            dsig
        }

        pub fn traces_to_dataset(traces: &[ExecutionTrace]) -> Dataset {
            let examples: Vec<Example> = traces.iter()
                .filter(|t| t.score.unwrap_or(0.0) >= 0.7)
                .map(|t| {
                    let mut record = Record::new();
                    record.insert("task", Value::String(t.input_summary.clone()));
                    record.insert("context", Value::String(
                        t.project_path.clone().unwrap_or_default()
                    ));
                    record.insert("result", Value::String(t.output_summary.clone()));
                    record.insert("reasoning", Value::String(
                        t.reflection.clone().unwrap_or_default()
                    ));
                    Example::new(record)
                })
                .collect();
            Dataset::new(examples)
        }

        pub async fn optimize(
            &self,
            signature: &SkillSignature,
            traces: &[ExecutionTrace],
        ) -> anyhow::Result<OptimizationReport> {
            let dsig = Self::to_dspy_signature(signature);
            let dataset = Self::traces_to_dataset(traces);
            let successful = dataset.examples.len();

            if successful < 3 {
                anyhow::bail!(
                    "Need at least 3 successful traces for optimization, have {}",
                    successful
                );
            }

            let student = Predict::new(dsig.clone(), self.model.clone());
            let metric: Metric = skill_success_metric;

            // Run BootstrapFewShot to collect best demos
            let optimizer = BootstrapFewShot::new(
                Arc::new(student.clone()),
                metric,
            );
            let optimized = optimizer.compile(student, &dataset).await;
            let demos_collected = optimized.demos.len();

            // Evaluate before/after
            let evaluator = dspy::Evaluate::new(metric, 4);
            let after_score = evaluator.run(&optimized, &dataset).await
                .unwrap_or(0.0);

            Ok(OptimizationReport {
                skill_name: signature.name.clone(),
                traces_used: traces.len(),
                successful_traces: successful,
                demos_collected,
                instruction_optimized: false, // MIPRO not run in this pass
                before_score: 0.0, // Would need baseline evaluation
                after_score,
                improvement_pct: 0.0,
            })
        }
    }

    /// Skill success metric using term-overlap Jaccard similarity.
    ///
    /// This is a semantic comparator, not a length heuristic. It measures the
    /// proportion of significant terms (4+ chars) shared between the expected
    /// and actual outputs using Jaccard index. This prevents the optimizer from
    /// converging toward verbosity.
    ///
    /// For production use, replace with embedding cosine similarity via
    /// surreal-memory's HNSW index or a local embedding model (candle + BERT).
    fn skill_success_metric(example: &Example, prediction: &dspy::Prediction) -> f64 {
        let expected = example.record.get("result").and_then(|v| v.as_str()).unwrap_or("");
        let actual = prediction.record.get("result").and_then(|v| v.as_str()).unwrap_or("");

        if actual.is_empty() { return 0.0; }
        if expected.is_empty() { return 0.5; } // No ground truth — neutral score

        // Extract significant terms (4+ chars, lowercased, deduplicated)
        let expected_terms: std::collections::HashSet<String> = expected
            .split_whitespace()
            .filter(|w| w.len() >= 4)
            .map(|w| w.to_lowercase().trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|w| !w.is_empty())
            .collect();

        let actual_terms: std::collections::HashSet<String> = actual
            .split_whitespace()
            .filter(|w| w.len() >= 4)
            .map(|w| w.to_lowercase().trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|w| !w.is_empty())
            .collect();

        if expected_terms.is_empty() || actual_terms.is_empty() {
            return 0.3; // Insufficient signal
        }

        // Jaccard similarity: |intersection| / |union|
        let intersection = expected_terms.intersection(&actual_terms).count() as f64;
        let union = expected_terms.union(&actual_terms).count() as f64;

        intersection / union
    }
}

#[cfg(not(feature = "optimize"))]
mod inner {
    use super::*;

    pub struct SkillOptimizer;

    impl SkillOptimizer {
        pub fn new() -> anyhow::Result<Self> {
            anyhow::bail!(
                "Prompt optimization requires the `optimize` feature.\n\
                 Rebuild with: cargo build --features optimize\n\
                 Requires: dspy-rs git dependency"
            )
        }
    }
}

pub use inner::SkillOptimizer;

// ─── Frontmatter parsing (always available) ─────────────────────────────────

#[derive(Debug)]
struct Frontmatter {
    name: String,
    description: String,
}

fn parse_frontmatter(content: &str) -> anyhow::Result<(Frontmatter, &str)> {
    let content = content.trim();
    if !content.starts_with("---") {
        anyhow::bail!("No YAML frontmatter found");
    }

    let after_first = &content[3..];
    let end = after_first.find("\n---")
        .ok_or_else(|| anyhow::anyhow!("Unterminated frontmatter"))?;

    let yaml_str = &after_first[..end];
    let body = &after_first[end + 4..];

    let mut name = String::new();
    let mut description = String::new();
    let mut in_description = false;

    for line in yaml_str.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("name:") {
            name = trimmed.strip_prefix("name:").unwrap_or("").trim().to_string();
            in_description = false;
        } else if trimmed.starts_with("description:") {
            let rest = trimmed.strip_prefix("description:").unwrap_or("").trim();
            if rest == ">" || rest == "|" {
                in_description = true;
            } else {
                description = rest.to_string();
            }
        } else if in_description && (trimmed.is_empty() || !line.starts_with(' ')) {
            in_description = false;
        } else if in_description {
            if !description.is_empty() { description.push(' '); }
            description.push_str(trimmed);
        }
    }

    Ok((Frontmatter { name, description }, body))
}

/// Write optimization results back into a SKILL.md file.
///
/// Appends or updates an "## Optimized Examples" section at the end,
/// preserving all existing content above it.
pub fn write_optimization_to_skill(
    skill_path: &Path,
    report: &OptimizationReport,
    demos_markdown: &str,
) -> anyhow::Result<()> {
    let skill_md_path = skill_path.join("SKILL.md");
    let content = std::fs::read_to_string(&skill_md_path)?;

    // Remove existing optimization section if present
    let base_content = if let Some(idx) = content.find("\n## Optimized Examples") {
        content[..idx].to_string()
    } else {
        content
    };

    let optimization_section = format!(
        "\n\n## Optimized Examples\n\n\
         <!-- Auto-generated by prometheus optimize — {traces} traces, {demo_count} demos, score: {score:.2} -->\n\n\
         {demo_content}\n",
        traces = report.traces_used,
        demo_count = report.demos_collected,
        score = report.after_score,
        demo_content = demos_markdown,
    );

    let new_content = format!("{}{}", base_content.trim_end(), optimization_section);
    std::fs::write(&skill_md_path, new_content)?;
    Ok(())
}
