//! forge-skills — Skill manifest discovery, loading, resolution, and template rendering.
//!
//! Scans `skills/<language>/` directories in the skill pack root (and the
//! project-local `.forge/skills/` directory for overrides) for `skill.toml`
//! manifests, loads them into a registry, resolves applicable skills for a
//! given task, and renders Tera templates with task context.

use anyhow::{Context, Result};
use forge_core::{Language, RenderedTemplate, SkillManifest, SkillTrigger};
use std::collections::{HashMap, HashSet};
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
    ///
    /// `stale_skills` — names of skills whose drift `acceptance_rate < 0.5`.
    /// Stale skills are sorted to the end of the resolved list so agents see
    /// them but prefer non-stale alternatives. No applicable skill is ever
    /// silently dropped.
    pub fn resolve(
        &self,
        language: &Language,
        task_description: &str,
        task_path: &str,
        stale_skills: &HashSet<String>,
    ) -> Vec<&SkillManifest> {
        let mut fresh: Vec<&SkillManifest> = Vec::new();
        let mut stale: Vec<&SkillManifest> = Vec::new();

        for (manifest, _) in self.skills.values() {
            if skill_applies(manifest, language, task_description, task_path) {
                if stale_skills.contains(&manifest.name) {
                    stale.push(manifest);
                } else {
                    fresh.push(manifest);
                }
            }
        }

        // Within each bucket: always-for-language skills first, then keyword/path-triggered
        let priority_key = |m: &&SkillManifest| -> u8 {
            if m.triggers
                .iter()
                .any(|t| matches!(t, SkillTrigger::AlwaysForLanguage { .. }))
            {
                0
            } else {
                1
            }
        };
        fresh.sort_by_key(priority_key);
        stale.sort_by_key(priority_key);

        if !stale.is_empty() {
            let names: Vec<_> = stale.iter().map(|m| m.name.as_str()).collect();
            warn!(
                stale_count = stale.len(),
                "Deprioritizing {} stale skill(s) (acceptance_rate < 0.5): {}",
                stale.len(),
                names.join(", ")
            );
        }

        // Fresh skills first, stale last
        let mut applicable = fresh;
        applicable.extend(stale);

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
            let (_, _template_dir) = match self.skills.get(&manifest.name) {
                Some(entry) => entry,
                None => {
                    warn!(
                        "Skill {} not found in registry for rendering",
                        manifest.name
                    );
                    continue;
                }
            };

            for template_ref in &manifest.templates {
                let template_name = format!("{}/{}", manifest.name, template_ref.path);

                match self.tera.render(
                    &template_name,
                    &tera::Context::from_serialize(task_context)?,
                ) {
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
        let manifest: SkillManifest =
            toml::from_str(&raw).with_context(|| format!("parsing {}", manifest_path.display()))?;

        // Register Tera templates
        let templates_dir = skill_dir.join("templates");
        if templates_dir.exists() {
            tera.add_template_files(
                WalkDir::new(&templates_dir)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().is_some_and(|ext| ext == "tera"))
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
            keywords
                .iter()
                .any(|k| desc_lower.contains(&k.to_lowercase()))
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
    _registry: &HashMap<String, (SkillManifest, PathBuf)>,
) -> Vec<&'a SkillManifest> {
    let mut result: Vec<&SkillManifest> = Vec::new();
    let mut placed: HashSet<String> = HashSet::new();

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

#[cfg(test)]
mod tests {
    use super::*;
    use forge_core::{Language, SkillManifest, SkillTrigger};
    use std::collections::{HashMap, HashSet};
    use std::path::PathBuf;

    fn make_manifest(name: &str, language: Language, always: bool) -> SkillManifest {
        SkillManifest {
            name: name.to_string(),
            language: language.clone(),
            description: format!("{} skill", name),
            version: "1.0.0".to_string(),
            templates: vec![],
            triggers: if always {
                vec![SkillTrigger::AlwaysForLanguage { language }]
            } else {
                vec![SkillTrigger::Keywords {
                    keywords: vec![name.to_string()],
                }]
            },
            depends_on: vec![],
        }
    }

    fn make_registry(manifests: Vec<SkillManifest>) -> SkillRegistry {
        let mut skills = HashMap::new();
        for m in manifests {
            skills.insert(m.name.clone(), (m, PathBuf::new()));
        }
        SkillRegistry {
            skills,
            tera: Tera::default(),
        }
    }

    // ─── resolve with stale skills ───────────────────────────────────────────

    #[test]
    fn stale_skills_appear_after_fresh_skills() {
        let fresh = make_manifest("error-handling", Language::Rust, true);
        let stale_skill = make_manifest("axum-patterns", Language::Rust, true);
        let registry = make_registry(vec![fresh, stale_skill]);

        let mut stale = HashSet::new();
        stale.insert("axum-patterns".to_string());

        let resolved = registry.resolve(&Language::Rust, "task", "", &stale);
        let names: Vec<&str> = resolved.iter().map(|m| m.name.as_str()).collect();

        let fresh_pos = names.iter().position(|&n| n == "error-handling").unwrap();
        let stale_pos = names.iter().position(|&n| n == "axum-patterns").unwrap();
        assert!(
            fresh_pos < stale_pos,
            "fresh skill must precede stale skill; got: {:?}",
            names
        );
    }

    #[test]
    fn empty_stale_set_returns_all_applicable_in_order() {
        let s1 = make_manifest("error-handling", Language::Rust, true);
        let s2 = make_manifest("axum-patterns", Language::Rust, true);
        let registry = make_registry(vec![s1, s2]);

        let resolved = registry.resolve(&Language::Rust, "task", "", &HashSet::new());
        assert_eq!(resolved.len(), 2);
    }

    #[test]
    fn stale_skill_is_included_not_silently_dropped() {
        let stale_skill = make_manifest("axum-patterns", Language::Rust, true);
        let registry = make_registry(vec![stale_skill]);

        let mut stale = HashSet::new();
        stale.insert("axum-patterns".to_string());

        let resolved = registry.resolve(&Language::Rust, "task", "", &stale);
        assert_eq!(resolved.len(), 1, "stale skill must still be returned");
        assert_eq!(resolved[0].name, "axum-patterns");
    }

    #[test]
    fn stale_set_with_non_existent_name_has_no_effect() {
        let skill = make_manifest("error-handling", Language::Rust, true);
        let registry = make_registry(vec![skill]);

        let mut stale = HashSet::new();
        stale.insert("no-such-skill".to_string());

        let resolved = registry.resolve(&Language::Rust, "task", "", &stale);
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].name, "error-handling");
    }

    #[test]
    fn skill_not_applicable_to_language_is_excluded() {
        let react_skill = make_manifest("react-hooks", Language::React, true);
        let registry = make_registry(vec![react_skill]);

        let resolved = registry.resolve(&Language::Rust, "task", "", &HashSet::new());
        assert!(
            resolved.is_empty(),
            "React skill must not resolve for Rust task"
        );
    }

    // ─── topological_sort ────────────────────────────────────────────────────

    #[test]
    fn skills_with_no_deps_all_appear() {
        let s1 = make_manifest("a", Language::Rust, false);
        let s2 = make_manifest("b", Language::Rust, false);
        let registry = make_registry(vec![s1.clone(), s2.clone()]);

        let skills: Vec<&SkillManifest> = registry.skills.values().map(|(m, _)| m).collect();
        let sorted = topological_sort(skills, &registry.skills);
        assert_eq!(sorted.len(), 2);
    }
}
