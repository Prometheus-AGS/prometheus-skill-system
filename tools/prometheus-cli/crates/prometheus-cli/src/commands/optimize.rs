use anyhow::Result;
use colored::Colorize;
use prometheus_learn::optimize::parse_skill_to_signature;
use prometheus_learn::trace::TraceStore;
use std::path::Path;

pub async fn run(skill: &str, min_traces: usize, dry_run: bool) -> Result<()> {
    println!("{}", "⚡ Skill Prompt Optimization".bold());

    if dry_run {
        println!("  {}", "(dry run — no changes will be written)".dimmed());
    }

    let skill_path = Path::new(skill);
    if !skill_path.join("SKILL.md").exists() {
        anyhow::bail!("No SKILL.md found at {}", skill_path.display());
    }

    // Parse skill into signature
    let signature = parse_skill_to_signature(skill_path)?;
    println!("\n  Skill: {}", signature.name.cyan().bold());
    println!("  Description: {}", signature.description.dimmed());
    println!("  Instruction length: {} chars", signature.instruction.len());

    // Load traces
    let store = TraceStore::default_for_project(Path::new("."));
    let traces = store.load_for_skill(&signature.name)?;
    println!("  Available traces: {}", traces.len().to_string().cyan());

    if traces.len() < min_traces {
        println!(
            "\n  {} Need at least {} traces, have {}. Run the skill more first.",
            "⚠️".yellow(),
            min_traces,
            traces.len()
        );
        return Ok(());
    }

    let successful = traces.iter().filter(|t| t.score.unwrap_or(0.0) >= 0.7).count();
    let avg_score = traces
        .iter()
        .filter_map(|t| t.score)
        .sum::<f64>()
        / traces.len().max(1) as f64;

    println!("  Successful traces: {}", successful.to_string().green());
    println!("  Average score: {:.2}", avg_score);

    println!("\n  {} Optimization pipeline:", "→".cyan());
    println!("    1. Convert SKILL.md → dspy-rs Signature");
    println!("    2. Build Dataset from {} successful traces", successful);
    println!("    3. Run BootstrapFewShot to collect best demos");
    println!("    4. Run MIPRO to optimize instruction");
    println!("    5. Write optimized prompt back to SKILL.md");

    println!("\n  {} Full optimization requires dspy crate dependency", "ℹ️".dimmed());
    println!("  {} Add git dependency to enable: dspy from GQAdonis/dspy-rs", "ℹ️".dimmed());

    if !dry_run {
        println!("\n  {} Would write optimized SKILL.md (skipped — deps not yet linked)", "⏭️".dimmed());
    }

    println!("\n{}", "✨ Optimization analysis complete".green().bold());
    Ok(())
}
