//! forge-enricher — Core enrichment pipeline.
//!
//! Takes an OpenSpec task folder path, detects the language, resolves applicable
//! skills via `forge-skills`, calls `pk focus` via the prometheus-knowledge MCP
//! or CLI for Karpathy context, renders all skill templates, and writes the
//! enriched context document to `.forge/enriched/<task-id>.context.md`.
//!
//! Architecture:
//! ```
//! OpenSpec task dir
//!   → TaskReader (parse task description, language detection)
//!   → SkillRegistry.resolve(language, description, path)
//!   → ConstitutionChecker.check(task, constitution)
//!   → KarpathyFocus.focus(description) → pk focus output
//!   → SkillRegistry.render_templates(skills, context)
//!   → ContextWriter.write(enrichment_context) → .forge/enriched/*.context.md
//! ```

use anyhow::{Context, Result};
use chrono::Utc;
use forge_core::{
    Constitution, ConstitutionWarning, EnrichmentContext, Language, Severity, SkillManifest,
};
use forge_skills::SkillRegistry;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{info, warn};
use uuid::Uuid;

// ─── Enricher ────────────────────────────────────────────────────────────────

pub struct Enricher {
    skill_registry: SkillRegistry,
    constitutions: HashMap<Language, Constitution>,
    forge_dir: PathBuf,        // `.forge/` in the project
    pk_mcp_url: Option<String>, // Optional prometheus-knowledge MCP endpoint
}

impl Enricher {
    pub fn new(
        skills_root: &Path,
        project_root: &Path,
        pk_mcp_url: Option<String>,
    ) -> Result<Self> {
        let forge_dir = project_root.join(".forge");
        let project_skills = forge_dir.join("skills");
        let constitution_dir = forge_dir.join("constitution");

        let skill_registry = SkillRegistry::load(
            skills_root,
            Some(&project_skills),
        )?;

        let constitutions = load_constitutions(&constitution_dir)?;

        Ok(Self {
            skill_registry,
            constitutions,
            forge_dir,
            pk_mcp_url,
        })
    }

    /// Enrich a task from an OpenSpec task folder.
    pub async fn enrich(&self, task_path: &Path) -> Result<EnrichmentContext> {
        // 1. Read the task
        let task = read_task(task_path)?;
        let language = detect_language(task_path, &task.description)?;

        info!(
            "Enriching task: {} (language: {:?})",
            task.id,
            language
        );

        // 2. Resolve applicable skills
        let skills = self
            .skill_registry
            .resolve(&language, &task.description, task.path_str());

        info!("Resolved {} skill(s)", skills.len());

        // 3. Check constitution
        let constitution = self.constitutions.get(&language);
        let constitution_summary = constitution
            .map(|c| summarize_constitution(c))
            .unwrap_or_default();
        let constitution_warnings = constitution
            .map(|c| check_constitution(c, &task.description))
            .unwrap_or_default();

        // 4. Call pk focus for Karpathy context
        let karpathy_focus = self.focus_karpathy(&task.description).await;

        // 5. Render templates
        let mut task_context = task.as_template_context();
        task_context.insert(
            "constitution_summary".to_string(),
            constitution_summary.clone(),
        );
        if let Some(focus) = &karpathy_focus {
            task_context.insert("karpathy_focus".to_string(), focus.clone());
        }

        let rendered_templates = self
            .skill_registry
            .render_templates(&skills, &task_context)
            .context("rendering skill templates")?;

        // 6. Build enrichment context
        let ctx = EnrichmentContext {
            id: Uuid::new_v4(),
            task_path: task.path_str().to_string(),
            task_description: task.description.clone(),
            language,
            applied_skills: skills.iter().map(|s| s.name.clone()).collect(),
            constitution_summary,
            karpathy_focus,
            rendered_templates,
            constitution_warnings,
            created_at: Utc::now(),
        };

        // 7. Write context document
        self.write_context(&ctx, &task.id)?;

        Ok(ctx)
    }

    /// Write the enriched context to `.forge/enriched/<task-id>.context.md`
    fn write_context(&self, ctx: &EnrichmentContext, task_id: &str) -> Result<()> {
        let enriched_dir = self.forge_dir.join("enriched");
        std::fs::create_dir_all(&enriched_dir)?;

        let output_path = enriched_dir.join(format!("{}.context.md", task_id));
        let content = render_context_document(ctx);
        std::fs::write(&output_path, content)?;

        info!("Context written to {}", output_path.display());
        Ok(())
    }

    /// Call `pk focus` via CLI subprocess or MCP endpoint.
    async fn focus_karpathy(&self, description: &str) -> Option<String> {
        // Try MCP first if configured
        if let Some(url) = &self.pk_mcp_url {
            match call_pk_focus_mcp(url, description).await {
                Ok(focus) => return Some(focus),
                Err(e) => warn!("pk focus MCP call failed: {e}, falling back to CLI"),
            }
        }

        // Fall back to CLI subprocess
        match call_pk_focus_cli(description).await {
            Ok(focus) => Some(focus),
            Err(e) => {
                warn!("pk focus CLI call failed: {e}");
                None
            }
        }
    }
}

// ─── Task reading ─────────────────────────────────────────────────────────────

struct Task {
    id: String,
    description: String,
    path: PathBuf,
    acceptance_criteria: Option<String>,
}

impl Task {
    fn path_str(&self) -> &str {
        self.path.to_str().unwrap_or("")
    }

    fn as_template_context(&self) -> HashMap<String, String> {
        let mut ctx = HashMap::new();
        ctx.insert("task_id".into(), self.id.clone());
        ctx.insert("task_description".into(), self.description.clone());
        ctx.insert("task_path".into(), self.path_str().to_string());
        if let Some(ac) = &self.acceptance_criteria {
            ctx.insert("acceptance_criteria".into(), ac.clone());
        }
        ctx
    }
}

fn read_task(task_path: &Path) -> Result<Task> {
    // Try reading a tasks.md file (OpenSpec standard)
    let tasks_file = if task_path.is_dir() {
        task_path.join("tasks.md")
    } else {
        task_path.to_owned()
    };

    let content = std::fs::read_to_string(&tasks_file)
        .with_context(|| format!("reading task file {}", tasks_file.display()))?;

    let id = task_path
        .parent()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Extract acceptance criteria if present (GIVEN/WHEN/THEN blocks)
    let acceptance_criteria = extract_acceptance_criteria(&content);

    Ok(Task {
        id,
        description: content.clone(),
        path: task_path.to_owned(),
        acceptance_criteria,
    })
}

fn extract_acceptance_criteria(content: &str) -> Option<String> {
    let lines: Vec<&str> = content
        .lines()
        .filter(|l| {
            l.trim_start().starts_with("GIVEN")
                || l.trim_start().starts_with("WHEN")
                || l.trim_start().starts_with("THEN")
        })
        .collect();
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

// ─── Language detection ───────────────────────────────────────────────────────

fn detect_language(task_path: &Path, description: &str) -> Result<Language> {
    // Check path for hints
    let path_str = task_path.to_string_lossy().to_lowercase();
    if path_str.contains("/rust/") || path_str.contains("cargo") {
        return Ok(Language::Rust);
    }
    if path_str.contains("/flutter/") || path_str.contains("dart") {
        return Ok(Language::Flutter);
    }
    if path_str.contains("/tauri/") {
        return Ok(Language::Tauri);
    }
    if path_str.contains("/go/") || path_str.contains("go.mod") {
        return Ok(Language::Go);
    }
    if path_str.contains("/python/") || path_str.contains("pyo3") {
        return Ok(Language::Python);
    }

    // Check description for hints
    let desc_lower = description.to_lowercase();
    if desc_lower.contains("axum") || desc_lower.contains("tokio") || desc_lower.contains("cargo") {
        return Ok(Language::Rust);
    }
    if desc_lower.contains("react") || desc_lower.contains("vite") || desc_lower.contains("tsx") {
        return Ok(Language::React);
    }
    if desc_lower.contains("flutter") || desc_lower.contains("dart") || desc_lower.contains("riverpod") {
        return Ok(Language::Flutter);
    }
    if desc_lower.contains("tauri") {
        return Ok(Language::Tauri);
    }
    if desc_lower.contains("golang") || desc_lower.contains("func main()") {
        return Ok(Language::Go);
    }
    if desc_lower.contains("pyo3") || desc_lower.contains("python") {
        return Ok(Language::Python);
    }
    if desc_lower.contains("typescript") || desc_lower.contains("eslint") {
        return Ok(Language::TypeScript);
    }

    // Default to Rust (primary language)
    Ok(Language::Rust)
}

// ─── Constitution helpers ─────────────────────────────────────────────────────

fn load_constitutions(constitution_dir: &Path) -> Result<HashMap<Language, Constitution>> {
    let mut map = HashMap::new();
    if !constitution_dir.exists() {
        return Ok(map);
    }
    for entry in std::fs::read_dir(constitution_dir)? {
        let path = entry?.path();
        if path.extension().map_or(false, |e| e == "toml") {
            let raw = std::fs::read_to_string(&path)?;
            let constitution: Constitution = toml::from_str(&raw)
                .with_context(|| format!("parsing constitution {}", path.display()))?;
            map.insert(constitution.language.clone(), constitution);
        }
    }
    Ok(map)
}

fn summarize_constitution(c: &Constitution) -> String {
    let mut lines = vec![
        format!("Language: {:?}", c.language),
        "## Standards".to_string(),
    ];
    for (k, v) in &c.standards {
        lines.push(format!("- **{}**: {}", k, v));
    }
    lines.push("## Forbidden Patterns".to_string());
    for f in &c.forbidden_patterns {
        lines.push(format!("- `{}` — {} ({:?})", f.pattern, f.reason, f.severity));
    }
    lines.join("\n")
}

fn check_constitution(c: &Constitution, description: &str) -> Vec<ConstitutionWarning> {
    let desc_lower = description.to_lowercase();
    c.forbidden_patterns
        .iter()
        .filter(|f| desc_lower.contains(&f.pattern.to_lowercase()))
        .map(|f| ConstitutionWarning {
            rule: f.pattern.clone(),
            violation: format!("Task description references forbidden pattern: {}", f.pattern),
            severity: f.severity.clone(),
        })
        .collect()
}

// ─── pk focus integration ─────────────────────────────────────────────────────

async fn call_pk_focus_mcp(url: &str, description: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "knowledge_focus",
            "arguments": { "topic": description }
        }
    });
    let resp: serde_json::Value = client
        .post(url)
        .json(&payload)
        .send()
        .await?
        .json()
        .await?;

    Ok(resp["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or("")
        .to_string())
}

async fn call_pk_focus_cli(description: &str) -> Result<String> {
    let output = tokio::process::Command::new("pk")
        .args(["focus", description])
        .output()
        .await
        .context("running pk focus")?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

// ─── Context document renderer ────────────────────────────────────────────────

fn render_context_document(ctx: &EnrichmentContext) -> String {
    let mut doc = format!(
        "# Forge Enrichment Context\n\n\
         **Task**: {}\n\
         **Language**: {:?}\n\
         **Generated**: {}\n\
         **Applied Skills**: {}\n\n",
        ctx.task_description,
        ctx.language,
        ctx.created_at.format("%Y-%m-%d %H:%M:%S UTC"),
        ctx.applied_skills.join(", ")
    );

    if !ctx.constitution_summary.is_empty() {
        doc.push_str("## Constitution\n\n");
        doc.push_str(&ctx.constitution_summary);
        doc.push_str("\n\n");
    }

    if !ctx.constitution_warnings.is_empty() {
        doc.push_str("## ⚠️ Constitution Warnings\n\n");
        for w in &ctx.constitution_warnings {
            doc.push_str(&format!("- **{}**: {} ({:?})\n", w.rule, w.violation, w.severity));
        }
        doc.push('\n');
    }

    if let Some(focus) = &ctx.karpathy_focus {
        doc.push_str("## Prior Knowledge (Karpathy Focus)\n\n");
        doc.push_str(focus);
        doc.push_str("\n\n");
    }

    for template in &ctx.rendered_templates {
        doc.push_str(&format!("## Skill: {} — {}\n\n", template.skill_name, template.template_path));
        doc.push_str(&template.content);
        doc.push_str("\n\n");
    }

    doc
}
