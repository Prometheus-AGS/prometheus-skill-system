use anyhow::Result;
use colored::Colorize;
use prometheus_agents::{detect_installed_agents, platforms};

pub fn run(_all: bool, _global: bool, _project: bool, verbose: bool) -> Result<()> {
    println!("{}", "📋 Installed Skills".bold());

    let agents = detect_installed_agents();
    if agents.is_empty() {
        println!("  No AI coding agents detected.");
        return Ok(());
    }

    for agent in &agents {
        let skills = platforms::list_skills(agent)?;
        println!("\n  {} {} ({} skill{})",
            "▸".cyan(),
            agent.kind.display_name().bold(),
            skills.len(),
            if skills.len() == 1 { "" } else { "s" },
        );

        for skill in &skills {
            if verbose {
                let link_target = std::fs::read_link(agent.global_skills_dir.join(skill));
                match link_target {
                    Ok(target) => println!("    {} → {}", skill, target.display().to_string().dimmed()),
                    Err(_) => println!("    {} (copied)", skill),
                }
            } else {
                println!("    {}", skill);
            }
        }
    }

    Ok(())
}
