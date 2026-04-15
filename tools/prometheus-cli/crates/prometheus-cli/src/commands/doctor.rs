use anyhow::Result;
use colored::Colorize;
use prometheus_agents::detect_installed_agents;
use prometheus_learn::memory::SurrealMemoryClient;

pub async fn run() -> Result<()> {
    println!("{}", "🩺 Health Check".bold());

    let mut issues = 0u32;

    // Check skills directory
    print!("  Skills directory... ");
    if std::path::Path::new("skills").exists() {
        println!("{}", "✅".green());
    } else {
        println!("{} not found", "❌".red());
        issues += 1;
    }

    // Check installed agents
    let agents = detect_installed_agents();
    println!("  Detected agents: {}", agents.len().to_string().cyan());
    for agent in &agents {
        println!("    {} {}", "▸".dimmed(), agent.kind.display_name());
    }
    if agents.is_empty() {
        println!("    {} No agents found", "⚠️".yellow());
        issues += 1;
    }

    // Check surreal-memory
    print!("  Surreal-memory... ");
    if let Some(client) = SurrealMemoryClient::from_env() {
        match client.ping().await {
            Ok(true) => println!("{} ({})", "✅".green(), client.base_url()),
            Ok(false) => { println!("{} unreachable", "⚠️".yellow()); }
            Err(_) => { println!("{} unreachable", "⚠️".yellow()); }
        }
    } else {
        println!("{} not configured", "⚠️".yellow());
    }

    // Check KBD state
    print!("  KBD orchestrator... ");
    if std::path::Path::new(".kbd-orchestrator").exists() {
        println!("{}", "✅".green());
    } else {
        println!("{}", "not initialized".dimmed());
    }

    // Check evolver state
    print!("  Evolver state... ");
    if std::path::Path::new(".evolver").exists() {
        println!("{}", "✅".green());
    } else {
        println!("{}", "not initialized".dimmed());
    }

    // Check trace store
    print!("  Trace store... ");
    let trace_dir = std::path::Path::new(".prometheus/traces");
    if trace_dir.exists() {
        let store = prometheus_learn::TraceStore::default_for_project(std::path::Path::new("."));
        let count = store.count_all().unwrap_or(0);
        println!("{} ({} traces)", "✅".green(), count);
    } else {
        println!("{}", "no traces yet".dimmed());
    }

    println!("\n{}", "─".repeat(40));
    if issues == 0 {
        println!("{}", "✨ All checks passed".green().bold());
    } else {
        println!("{}", format!("⚠️  {} issue(s) found", issues).yellow().bold());
    }

    Ok(())
}
