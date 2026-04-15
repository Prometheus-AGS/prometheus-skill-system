use anyhow::Result;
use colored::Colorize;
use std::path::Path;

pub fn run(path: &str) -> Result<()> {
    let root = Path::new(path);
    println!("{}", "📊 Project Status".bold());

    // Skills.toml
    let skills_toml = root.join("Skills.toml");
    if skills_toml.exists() {
        let content = std::fs::read_to_string(&skills_toml)?;
        println!("\n  {} Skills.toml", "▸".cyan());
        for line in content.lines().take(10) {
            println!("    {}", line.dimmed());
        }
    }

    // KBD waypoint
    let waypoint = root.join(".kbd-orchestrator/current-waypoint.json");
    if waypoint.exists() {
        let content = std::fs::read_to_string(&waypoint)?;
        let wp: serde_json::Value = serde_json::from_str(&content)?;
        println!("\n  {} KBD Waypoint", "▸".cyan());
        println!("    Phase: {}", wp["active_phase"].as_str().unwrap_or("—").bold());
        println!("    Next:  {}", wp["exact_next_command"].as_str().unwrap_or("—").green());
    }

    // Evolver state
    let evolver_registry = root.join(".evolver/registry.json");
    if evolver_registry.exists() {
        let content = std::fs::read_to_string(&evolver_registry)?;
        let reg: serde_json::Value = serde_json::from_str(&content)?;
        println!("\n  {} Evolver", "▸".cyan());
        if let Some(obj) = reg.as_object() {
            for (name, _) in obj {
                println!("    📎 {}", name.cyan());
            }
        }
    }

    // Trace count
    let trace_dir = root.join(".prometheus/traces");
    if trace_dir.exists() {
        let store = prometheus_learn::TraceStore::default_for_project(root);
        let count = store.count_all().unwrap_or(0);
        println!("\n  {} Traces: {}", "▸".cyan(), count);
    }

    Ok(())
}
