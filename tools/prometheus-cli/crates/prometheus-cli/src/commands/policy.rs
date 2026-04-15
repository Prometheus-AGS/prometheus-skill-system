use anyhow::Result;
use colored::Colorize;
use prometheus_cedar::{MutationContext, SkillMutationPep, SkillOperation};

pub fn show() -> Result<()> {
    println!("{}", "📜 Cedar Policies".bold());

    let pep = SkillMutationPep::from_default_dir()?;
    println!("  Policies loaded: {}", pep.policy_count().to_string().cyan());
    println!("\n{}", pep.display_policies());

    Ok(())
}

pub fn validate() -> Result<()> {
    println!("{}", "🔍 Validating Cedar policies...".bold());

    match SkillMutationPep::from_default_dir() {
        Ok(pep) => {
            println!("  {} {} policies loaded and valid", "✅".green(), pep.policy_count());
        }
        Err(e) => {
            println!("  {} Policy validation failed: {}", "❌".red(), e);
            std::process::exit(1);
        }
    }

    // Also check entities.json
    let entities_path = std::path::Path::new("policies/entities.json");
    if entities_path.exists() {
        let content = std::fs::read_to_string(entities_path)?;
        match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(_) => println!("  {} entities.json is valid JSON", "✅".green()),
            Err(e) => {
                println!("  {} entities.json parse error: {}", "❌".red(), e);
                std::process::exit(1);
            }
        }
    }

    println!("\n{}", "✨ All policies valid".green().bold());
    Ok(())
}

pub fn check(agent: &str, operation: &str, skill: &str, environment: &str) -> Result<()> {
    println!("{}", "🔐 Cedar Policy Check".bold());

    let pep = SkillMutationPep::from_default_dir()?;

    let op = match operation {
        "skill.mutate" => SkillOperation::Mutate,
        "skill.generate" => SkillOperation::Generate,
        "skill.promote" => SkillOperation::Promote,
        "trace.capture" => SkillOperation::TraceCapture,
        _ => anyhow::bail!("Unknown operation: {}. Use: skill.mutate, skill.generate, skill.promote, trace.capture", operation),
    };

    let context = MutationContext {
        environment: environment.to_string(),
        governance_consent: Some(true),
        ..Default::default()
    };

    let decision = pep.check(agent, op, skill, &context);

    println!("  Agent:       {}", agent.cyan());
    println!("  Operation:   {}", operation);
    println!("  Skill:       {}", skill);
    println!("  Environment: {}", environment);
    println!();

    if decision.allowed {
        println!("  {} {}", "PERMIT".green().bold(), "— operation allowed by Cedar policy");
    } else {
        println!("  {} {}", "DENY".red().bold(), "— operation blocked by Cedar policy");
    }

    if !decision.reasons.is_empty() {
        println!("  Reasons: {}", decision.reasons.join(", ").dimmed());
    }

    Ok(())
}
