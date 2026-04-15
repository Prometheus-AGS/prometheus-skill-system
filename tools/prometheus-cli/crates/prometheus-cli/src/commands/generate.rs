use anyhow::Result;
use colored::Colorize;

pub fn run(path: &str, language: Option<&str>) -> Result<()> {
    println!("{}", "🔧 Generating skills from source...".bold());
    println!("  Source: {}", path.cyan());
    if let Some(lang) = language {
        println!("  Language filter: {}", lang);
    }

    // Detect project type
    let source = std::path::Path::new(path);
    if !source.exists() {
        anyhow::bail!("Source path not found: {}", path);
    }

    let mut detected = Vec::new();
    if source.join("Cargo.toml").exists() { detected.push("Rust"); }
    if source.join("package.json").exists() { detected.push("TypeScript/JavaScript"); }
    if source.join("pyproject.toml").exists() || source.join("setup.py").exists() { detected.push("Python"); }
    if source.join("go.mod").exists() { detected.push("Go"); }

    println!("  Detected: {}", detected.join(", ").green());
    println!("\n  {} Skill generation from source creates SKILL.md files from code documentation", "ℹ️".dimmed());
    println!("  {} Full implementation generates llms.txt and module-level skills", "ℹ️".dimmed());

    Ok(())
}
