use anyhow::Result;
use colored::Colorize;

pub fn run(name: &str, domain: &str, phase: Option<&str>) -> Result<()> {
    let cmd = match phase {
        Some(p) => format!("/evolve-{} \"{}\"", p, name),
        None => format!("/evolve \"{}\"", name),
    };

    println!("{}", "🔄 Iterative Evolution".bold());
    println!("  Name:   {}", name.cyan());
    println!("  Domain: {}", domain);
    println!("  Command: {}", cmd.green().bold());
    println!();
    println!("  Run this command in your AI coding agent to start the evolution cycle.");
    println!("  The evolution will use the {} domain adapter.", domain);

    // Check for existing state
    let state_dir = std::path::Path::new(".evolver")
        .join("evolutions")
        .join(name);

    if state_dir.exists() {
        println!("\n  {} Existing state found — will resume from last checkpoint", "📎".yellow());
    } else {
        println!("\n  {} New evolution — will start fresh", "🆕".green());
    }

    Ok(())
}
