//! Platform-specific installation logic.
//!
//! Each platform may have additional installation steps beyond symlink creation
//! (e.g., OpenCode needs .opencode/tools/ linked, Claude Code needs plugin registration).

use crate::config::{AgentConfig, AgentKind};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Install a skill pack to a specific agent platform.
///
/// Creates a symlink from the agent's skills directory to the skill pack source.
/// Uses symlinks by default for single-source-of-truth development.
pub fn install_to_agent(agent: &AgentConfig, source: &Path, name: &str) -> Result<()> {
    let target = agent.global_skills_dir.join(name);

    // Ensure parent directory exists
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    // Remove existing symlink if present
    if target.exists() || target.symlink_metadata().is_ok() {
        fs::remove_file(&target)
            .or_else(|_| fs::remove_dir_all(&target))
            .with_context(|| format!("Failed to remove existing: {}", target.display()))?;
    }

    // Create symlink
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, &target)
        .with_context(|| format!("Failed to create symlink: {} -> {}", target.display(), source.display()))?;

    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(source, &target)
        .with_context(|| format!("Failed to create symlink: {} -> {}", target.display(), source.display()))?;

    // Platform-specific extras
    match agent.kind {
        AgentKind::OpenCode => {
            // Also link .opencode/tools/ if present
            let tools_source = source.join(".opencode").join("tools");
            if tools_source.exists() {
                if let Some(tools_dir) = &agent.tools_dir {
                    let tools_target = tools_dir.join(name);
                    fs::create_dir_all(tools_dir)?;
                    if tools_target.exists() || tools_target.symlink_metadata().is_ok() {
                        let _ = fs::remove_file(&tools_target);
                    }
                    #[cfg(unix)]
                    std::os::unix::fs::symlink(&tools_source, &tools_target)?;
                }
            }
        }
        _ => {}
    }

    Ok(())
}

/// Remove a skill pack from a specific agent platform.
pub fn uninstall_from_agent(agent: &AgentConfig, name: &str) -> Result<()> {
    let target = agent.global_skills_dir.join(name);
    if target.exists() || target.symlink_metadata().is_ok() {
        fs::remove_file(&target)
            .or_else(|_| fs::remove_dir_all(&target))
            .with_context(|| format!("Failed to remove: {}", target.display()))?;
    }

    // Clean up OpenCode tools link
    if agent.kind == AgentKind::OpenCode {
        if let Some(tools_dir) = &agent.tools_dir {
            let tools_target = tools_dir.join(name);
            if tools_target.exists() {
                let _ = fs::remove_file(&tools_target);
            }
        }
    }

    Ok(())
}

/// List installed skills for a specific agent platform.
pub fn list_skills(agent: &AgentConfig) -> Result<Vec<String>> {
    let dir = &agent.global_skills_dir;
    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut skills = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with('.') {
            skills.push(name);
        }
    }

    skills.sort();
    Ok(skills)
}
