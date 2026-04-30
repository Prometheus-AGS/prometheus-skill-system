//! forge-skills — Skill manifest discovery, loading, resolution, and template rendering.
//!
//! Scans `skills/<language>/` directories in the skill pack root (and the
//! project-local `.forge/skills/` directory for overrides) for `skill.toml`
//! manifests, loads them into a registry, resolves applicable skills for a
//! given task, and renders Tera templates with task context.

use anyhow::{Context, Result};
use forge_core::{EnrichmentContext, Language, RenderedTemplate, SkillManifest, SkillTrigger};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tera::Tera;
use tracing::{debug, info, warn};
use walkdir::WalkDir;

// ─── Registry ────────────────────────────────────────────────────────────────

/// Loaded and indexed skill manifests.
pub struct SkillRegistry {
    /// skill_name → (manifest, template_dir)
    skills: HashMap<String, (SkillManifest, PathBuf)>,
    /// Tera template engine — pre-loaded with all templates
    tera: Tera,
}

impl SkillRegistry {
    /// Load all skills from `skills_root` (the skill pack `skills/` directory)
    /// and optionally from `project_override` (`.forge/skills/`).
    pub fn load(skills_root: &Path, project_override: Option<&Path>) -> Result<Self> {
        let mut skills = HashMap::new();
        let mut tera = Tera::default();

        // Load from skill pack root
        load_from_dir(skills_root, &mut skills, &mut tera)
            .with_context(|| format!("loading skills from {}", skills_root.display()))?;

        // Load from project override (may override skill-pack skills)
        if let Some(override_dir) = project_override {
            if override_dir.exists() {
                load_from_dir(override_dir, &mut skills, &mut tera)
                    .with_context(|| format!("loading skills from {}", override_dir.display()))?;
            }
        }

        info!("Loaded {} skill(s) into registry", skills.len());
        Ok(Self { skills, tera })
    }

    /// Resolve which skills apply to a given task, in dependency order.
    pub fn resolve(
        &self,
        language: &Language,
        task_description: &str,
        task_path: &str,
    ) -> Vec<&SkillManifest> {
        let mut applicable: Vec<&SkillManifest> = self
            .skills
            .values()
            .filter_map(|(manifest, _)| {
                if skill_applies(manifest, language, task_description, task_path) {
                    Some(manifest)
                } else {
                    None
                }
            })
            .collect();

        // Sort: always-for-language skills first, then keyword/path-triggered
        applicable.sort_by_key(|m| {
            let priority = m.triggers.iter().any(|t| {
                matches!(t, SkillTrigger::AlwaysForLanguage { .. })
            });
            if priority { 0u8 } else { 1u8 }
        });

        // Topological sort by depends_on
        topological_sort(applicable, &self.skills)
    }

    /// Render all templates for a resolved skill set into the enrichment context.
    pub fn render_templates(
        &self,
        skills: &[&SkillManifest],
        task_context: &HashMap<String, String>,
    ) -> Result<Vec<RenderedTemplate>> {
        let mut rendered = Vec::new();

        for manifest in skills {
            let (_, template_dir) = match self.skills.get(&manifest.name) {
                Some(entry) => entry,
                None => {
                    warn!("Skill {} not found in registry for rendering", manifest.name);
                    continue;
                }
            };

            for template_ref in &manifest.templates {
                let template_path = template_dir.join(&template_ref.path);
                let template_name = format!("{}/{}", manifest.name, template_ref.path);

                match self.tera.render(&template_name, &tera::Context::from_serialize(task_context)?) {
                    Ok(content) => {
                        rendered.push(RenderedTemplate {
                            skill_name: manifest.name.clone(),
                            template_path: template_ref.path.clone(),
                            content,
                        });
                    }
                    Err(e) => {
                        warn!("Template render failed for {}: {}", template_name, e);
                    }
                }
            }
        }

        Ok(rendered)
    }
}

// ─── Internal helpers ────────────────────────────────────────────────────────

fn load_from_dir(
    root: &Path,
    skills: &mut HashMap<String, (SkillManifest, PathBuf)>,
    tera: &mut Tera,
) -> Result<()> {
    for entry in WalkDir::new(root)
        .min_depth(2)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name() == "skill.toml")
    {
        let manifest_path = entry.path();
        let skill_dir = manifest_path.parent().unwrap().to_owned();

        let raw = std::fs::read_to_string(manifest_path)
            .with_context(|| format!("reading {}", manifest_path.display()))?;
        let manifest: SkillManifest = toml::from_str(&raw)
            .with_context(|| format!("parsing {}", manifest_path.display()))?;

        // Register Tera templates
        let templates_dir = skill_dir.join("templates");
        if templates_dir.exists() {
            let glob = format!("{}/**/*.tera", templates_dir.display());
            tera.add_template_files(
                WalkDir::new(&templates_dir)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().map_or(false, |ext| ext == "tera"))
                    .map(|e| {
                        let path = e.path().to_owned();
                        let name = format!(
                            "{}/{}",
                            manifest.name,
                            path.strip_prefix(&templates_dir).unwrap().display()
                        );
                        (path, Some(name))
                    })
                    .collect::<Vec<_>>(),
            )?;
        }

        debug!("Loaded skill: {}", manifest.name);
        skills.insert(manifest.name.clone(), (manifest, skill_dir));
    }
    Ok(())
}

fn skill_applies(
    manifest: &SkillManifest,
    language: &Language,
    task_description: &str,
    task_path: &str,
) -> bool {
    if &manifest.language != language {
        return false;
    }
    manifest.triggers.iter().any(|trigger| match trigger {
        SkillTrigger::AlwaysForLanguage { language: l } => l == language,
        SkillTrigger::Keywords { keywords } => {
            let desc_lower = task_description.to_lowercase();
            keywords.iter().any(|k| desc_lower.contains(&k.to_lowercase()))
        }
        SkillTrigger::PathGlob { glob } => {
            // Simple glob: just check if the path contains the pattern
            task_path.contains(glob.trim_matches('*'))
        }
        SkillTrigger::DependsOnPackage { .. } => false, // Resolved externally
    })
}

fn topological_sort<'a>(
    mut skills: Vec<&'a SkillManifest>,
    registry: &HashMap<String, (SkillManifest, PathBuf)>,
) -> Vec<&'a SkillManifest> {
    // Simple insertion-sort by dependency order
    // Full topological sort would be warranted with complex dep graphs
    let mut result: Vec<&SkillManifest> = Vec::new();
    let mut placed: std::collections::HashSet<String> = std::collections::HashSet::new();

    let max_iterations = skills.len() * skills.len() + 1;
    let mut iterations = 0;

    while !skills.is_empty() && iterations < max_iterations {
        iterations += 1;
        let mut deferred = Vec::new();
        for skill in skills.drain(..) {
            let deps_satisfied = skill.depends_on.iter().all(|dep| placed.contains(dep));
            if deps_satisfied {
                placed.insert(skill.name.clone());
                result.push(skill);
            } else {
                deferred.push(skill);
            }
        }
        skills = deferred;
    }

    // Any remaining skills with unresolvable deps go at the end
    result.extend(skills);
    result
}
