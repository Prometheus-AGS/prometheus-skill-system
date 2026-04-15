use anyhow::{Context, Result};
use colored::Colorize;
use std::path::Path;
use std::process::Command;

pub fn run(gitops_path: &str, service: &str, overlay: &str) -> Result<()> {
    let overlay_path = Path::new(gitops_path)
        .join("services")
        .join(service)
        .join("overlays")
        .join(overlay);

    println!("{}", "🏗️  Building Kustomize overlay...".bold());
    println!("  Service: {}", service.cyan());
    println!("  Overlay: {}", overlay.cyan());

    if !overlay_path.exists() {
        anyhow::bail!("Overlay not found: {}", overlay_path.display());
    }

    let output = Command::new("kustomize")
        .args(["build", &overlay_path.to_string_lossy()])
        .output()
        .context("kustomize not found — install it first")?;

    if output.status.success() {
        let manifest = String::from_utf8_lossy(&output.stdout);
        println!("\n{}", manifest);
        println!("{}", "✅ Build successful".green().bold());
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("\n{}", stderr.red());
        std::process::exit(1);
    }

    Ok(())
}
