use anyhow::{Context, Result};
use colored::Colorize;
use std::process::Command;

pub fn run(path: Option<&str>) -> Result<()> {
    println!("{}", "🔍 Validating skills...".bold());

    let mut cmd = Command::new("node");
    cmd.arg("scripts/validate-skills.js");

    if let Some(p) = path {
        cmd.arg(p);
    }

    let output = cmd
        .output()
        .context("Failed to run validator. Is Node.js installed?")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    print!("{}", stdout);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.is_empty() {
            eprint!("{}", stderr);
        }
        std::process::exit(1);
    }

    Ok(())
}
