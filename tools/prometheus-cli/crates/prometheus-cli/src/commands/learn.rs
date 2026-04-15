use anyhow::Result;
use colored::Colorize;
use prometheus_learn::trace::TraceStore;
use std::path::Path;

pub async fn run(capture_session: bool, compile: bool, lint: bool, dry_run: bool) -> Result<()> {
    println!("{}", "🧠 Self-Learning Pipeline".bold());

    if dry_run {
        println!("  {}", "(dry run — no changes will be written)".dimmed());
    }

    let project_root = Path::new(".");
    let store = TraceStore::default_for_project(project_root);

    if capture_session {
        println!("\n  {} Capturing session traces...", "→".cyan());
        // In a real implementation, this reads the Claude Code session log
        // and extracts skill invocations into ExecutionTrace objects.
        println!("  {} Session trace capture requires integration with the AI agent's session API", "ℹ️".dimmed());
        println!("  {} Traces are written to .prometheus/traces/{{skill}}/{{timestamp}}.json", "ℹ️".dimmed());
    }

    if compile || (!capture_session && !lint) {
        println!("\n  {} Compiling traces into knowledge...", "→".cyan());
        let trace_count = store.count_all().unwrap_or(0);
        println!("  Total traces available: {}", trace_count.to_string().cyan());

        if trace_count == 0 {
            println!("  {} No traces found. Run skills first, then capture traces.", "⚠️".yellow());
            return Ok(());
        }

        println!("  {} Knowledge compilation requires prometheus-knowledge crate", "ℹ️".dimmed());
        println!("  {} Add git dependency to enable: prometheus-knowledge from Prometheus-AGS/prometheus-knowledge-rs", "ℹ️".dimmed());
    }

    if lint {
        println!("\n  {} Running knowledge lint...", "→".cyan());
        let wiki_dir = project_root.join(".prometheus").join("wiki");
        if wiki_dir.exists() {
            let count = std::fs::read_dir(&wiki_dir)?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
                .count();
            println!("  Wiki entries: {}", count.to_string().cyan());
        } else {
            println!("  {} No wiki entries found. Run compile first.", "⚠️".yellow());
        }
    }

    println!("\n{}", "✨ Pipeline complete".green().bold());
    Ok(())
}
