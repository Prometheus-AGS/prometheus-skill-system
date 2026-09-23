use anyhow::{Context, Result};
use colored::Colorize;
use prometheus_agents::{detect_installed_agents, platforms};
use std::path::PathBuf;
use std::process::Command;

pub async fn run(
    source: &str,
    agent_filter: Option<&str>,
    local: bool,
    no_symlink: bool,
    _plugin: bool,
) -> Result<()> {
    println!("{}", "🔥 Installing skill pack...".bold());

    let source_path = if source.contains('/') && !std::path::Path::new(source).exists() {
        // GitHub repo — clone to cache
        let cache_dir = dirs::home_dir()
            .unwrap_or_default()
            .join(".prometheus")
            .join("repos");
        std::fs::create_dir_all(&cache_dir)?;

        let repo_name = source.split('/').next_back().unwrap_or(source);
        let repo_dir = cache_dir.join(repo_name);

        if repo_dir.exists() {
            println!("  Updating cached repo...");
            let output = Command::new("git")
                .args(["pull", "--ff-only"])
                .current_dir(&repo_dir)
                .output()
                .context("git pull failed")?;
            anyhow::ensure!(output.status.success(), "git pull failed: {}", String::from_utf8_lossy(&output.stderr));
        } else {
            println!("  Cloning {}...", source);
            let url = format!("https://github.com/{}.git", source);
            let output = Command::new("git")
                .args(["clone", "--depth", "1", &url, &repo_dir.to_string_lossy()])
                .output()
                .context("git clone failed")?;
            anyhow::ensure!(output.status.success(), "git clone failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        repo_dir
    } else {
        PathBuf::from(source).canonicalize()?
    };

    let skill_name = source_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // Determine target agents
    let agents = if let Some(filter) = agent_filter {
        let names: Vec<&str> = filter.split(',').collect();
        detect_installed_agents()
            .into_iter()
            .filter(|a| names.contains(&a.kind.name()))
            .collect::<Vec<_>>()
    } else {
        detect_installed_agents()
    };

    if agents.is_empty() {
        println!(
            "  {} No AI coding agents detected on this system",
            "⚠️".yellow()
        );
        return Ok(());
    }

    let scope = if local { "project" } else { "global" };
    println!(
        "  Installing to {} agent(s) (scope: {})\n",
        agents.len(),
        scope
    );

    let mut failures = Vec::new();
    for agent in &agents {
        let mut agent = agent.clone();
        if local {
            agent.global_skills_dir = std::env::current_dir()?.join(agent.project_skills_dir);
            if agent.tools_dir.is_some() { agent.tools_dir = Some(std::env::current_dir()?.join(".opencode/tools")); }
        }
        print!("  {} {}... ", "→".dimmed(), agent.kind.display_name());
        match platforms::install_to_agent_with_copy(&agent, &source_path, &skill_name, no_symlink || crate::host::is_managed()) {
            Ok(()) => println!("{}", "✅".green()),
            Err(e) => { println!("{} {}", "❌".red(), e); failures.push(agent.kind.name()); }
        }
    }
    anyhow::ensure!(failures.is_empty(), "Installation failed for: {}", failures.join(", "));

    println!("\n{}", "✨ Installation complete".green().bold());
    Ok(())
}
