use anyhow::Result;
use colored::Colorize;
use prometheus_agents::{detect_installed_agents, platforms};

pub fn run(name: &str, agent_filter: Option<&str>) -> Result<()> {
    println!("{} {}", "🗑️  Uninstalling".bold(), name);

    let agents = if let Some(filter) = agent_filter {
        let names: Vec<&str> = filter.split(',').collect();
        detect_installed_agents()
            .into_iter()
            .filter(|a| names.contains(&a.kind.name()))
            .collect::<Vec<_>>()
    } else {
        detect_installed_agents()
    };

    for agent in &agents {
        print!("  {} {}... ", "→".dimmed(), agent.kind.display_name());
        match platforms::uninstall_from_agent(agent, name) {
            Ok(()) => println!("{}", "✅".green()),
            Err(e) => println!("{} {}", "❌".red(), e),
        }
    }

    println!("\n{}", "✨ Uninstallation complete".green().bold());
    Ok(())
}
