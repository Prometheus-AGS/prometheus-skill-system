//! Skill prompt optimization via dspy-rs integration.
//!
//! Converts SKILL.md prompts into dspy-rs Signatures, builds Datasets from
//! execution traces, and runs BootstrapFewShot + MIPRO to optimize prompts.
//!
//! NOTE: This module defines the optimization types and interfaces. The actual
//! dspy-rs dependency is added when prometheus-knowledge and dspy crates are
//! integrated (requires git dependencies). Until then, this provides the
//! type scaffolding and SKILL.md parsing logic.

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
    pub demos_collected: usize,
    pub instruction_optimized: bool,
    pub before_score: f64,
    pub after_score: f64,
    pub improvement_pct: f64,
}

/// Parse a SKILL.md file into a SkillSignature.
///
/// Extracts frontmatter (name, description) and the body as instruction text.
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

    // Simple YAML parsing for name and description
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
            if !description.is_empty() {
                description.push(' ');
            }
            description.push_str(trimmed);
        }
    }

    Ok((Frontmatter { name, description }, body))
}
