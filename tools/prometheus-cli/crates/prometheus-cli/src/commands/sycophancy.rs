use anyhow::Result;
use colored::Colorize;
use sycophancy_core::{
    skill::detector::Detector,
    config::SkillConfig,
    CorrectionMode, Strictness, TargetType,
};
use std::path::Path;

pub fn detect(file: &str, strictness: &str) -> Result<()> {
    let content = read_content(file)?;
    let strictness = parse_strictness(strictness);
    let config = SkillConfig::default();
    let detector = Detector::new(config);

    let result = detector.detect(&content, &[], &strictness);

    println!("{}", "🔍 Sycophancy Detection".bold());
    println!("  File: {}", file.cyan());
    println!("  Score: {}", format_score(result.sycophancy_score));
    println!("  Patterns: {}", result.classifications.len());
    println!("  Critical: {}", if result.has_critical { "YES".red().bold() } else { "no".green() });
    println!("  Correction needed: {}", if result.correction_mandatory { "YES".yellow().bold() } else { "no".green() });

    if !result.classifications.is_empty() {
        println!("\n  {}", "Detected Patterns:".bold());
        for class in &result.classifications {
            let severity_color = match class.severity.as_str() {
                "critical" => class.severity.as_str().red().bold(),
                "high" => class.severity.as_str().red(),
                "medium" => class.severity.as_str().yellow(),
                _ => class.severity.as_str().dimmed(),
            };
            println!("  {} [{}] {} — {}",
                severity_color,
                class.pattern_id.cyan(),
                class.location.as_deref().unwrap_or(""),
                class.rationale.dimmed(),
            );
        }
    }

    Ok(())
}

pub fn score(file: &str) -> Result<()> {
    let content = read_content(file)?;
    let config = SkillConfig::default();
    let detector = Detector::new(config);
    let result = detector.detect(&content, &[], &Strictness::Standard);

    // Machine-readable output: just the score
    println!("{:.2}", result.sycophancy_score);

    // Exit with non-zero if correction is mandatory
    if result.correction_mandatory {
        std::process::exit(1);
    }

    Ok(())
}

pub fn correct(file: &str, strictness: &str) -> Result<()> {
    let content = read_content(file)?;
    let strictness_val = parse_strictness(strictness);
    let config = SkillConfig::default();
    let detector = Detector::new(config);

    let result = detector.detect(&content, &[], &strictness_val);

    println!("{}", "⚡ Sycophancy Correction".bold());
    println!("  File: {}", file.cyan());
    println!("  Score: {}", format_score(result.sycophancy_score));

    if result.classifications.is_empty() {
        println!("  {} No sycophancy patterns detected — no correction needed", "✅".green());
        return Ok(());
    }

    println!("  Patterns found: {}", result.classifications.len());
    for class in &result.classifications {
        println!("    [{}] {} — {}", class.pattern_id, class.severity.as_str(), class.rationale);
    }

    // Note: Full correction requires an LLM client (PmpoExecutor).
    // Without LLM access, we provide detection + manual guidance.
    println!("\n  {} Correction guidance:", "→".cyan());
    for class in &result.classifications {
        match class.pattern_id.as_str() {
            "S-01" => println!("    Remove unprompted affirmation language. Lead with analysis, not praise."),
            "S-02" => println!("    Add independent reasoning chain before agreement. Show your work."),
            "S-03" => println!("    Add at least one trade-off, risk, or alternative. Zero friction is suspicious."),
            "S-04" => println!("    Remove self-congratulatory language. Let the output speak for itself."),
            "S-05" => println!("    If reversing a prior position, surface new evidence that justifies the change."),
            "S-06" => println!("    Replace 'obviously'/'clearly' with explicit reasoning. Derive, don't assert."),
            "S-07" => println!("    Reduce scope. Does all this content deliver analytical value or visible effort?"),
            "S-08" => println!("    Restructure: Delta → Root Cause → Corrective Actions. Lead with what diverged."),
            _ => println!("    Review and address pattern {}", class.pattern_id),
        }
    }

    Ok(())
}

fn read_content(file: &str) -> Result<String> {
    if file == "-" {
        use std::io::Read;
        let mut content = String::new();
        std::io::stdin().read_to_string(&mut content)?;
        Ok(content)
    } else {
        std::fs::read_to_string(file)
            .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", file, e))
    }
}

fn parse_strictness(s: &str) -> Strictness {
    match s.to_lowercase().as_str() {
        "permissive" => Strictness::Permissive,
        "strict" => Strictness::Strict,
        "adversarial" => Strictness::Adversarial,
        _ => Strictness::Standard,
    }
}

fn format_score(score: f32) -> colored::ColoredString {
    if score < 0.3 {
        format!("{:.2}", score).green()
    } else if score < 0.5 {
        format!("{:.2}", score).yellow()
    } else if score < 0.7 {
        format!("{:.2}", score).red()
    } else {
        format!("{:.2}", score).red().bold()
    }
}
