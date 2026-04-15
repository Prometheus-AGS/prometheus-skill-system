use anyhow::{Context, Result};
use colored::Colorize;
use prometheus_learn::memory::SurrealMemoryClient;
use std::path::Path;
use std::process::Command;

pub async fn ping() -> Result<()> {
    println!("{}", "🧠 Checking surreal-memory...".bold());

    let client = SurrealMemoryClient::from_env()
        .ok_or_else(|| anyhow::anyhow!("SURREAL_MEMORY_URL not set"))?;

    match client.ping().await {
        Ok(true) => println!("  {} Server healthy at {}", "✅".green(), client.base_url()),
        Ok(false) => println!("  {} Server responded but unhealthy", "⚠️".yellow()),
        Err(e) => println!("  {} Cannot reach server: {}", "❌".red(), e),
    }

    Ok(())
}

pub async fn stats() -> Result<()> {
    println!("{}", "🧠 Surreal-Memory Stats".bold());

    let client = SurrealMemoryClient::from_env()
        .ok_or_else(|| anyhow::anyhow!("SURREAL_MEMORY_URL not set"))?;

    let entities = client.search("*", None).await?;
    println!("  Entities: {}", entities.len().to_string().cyan());

    Ok(())
}

pub async fn search(query: &str, entity_type: Option<&str>) -> Result<()> {
    println!("{} {}", "🔍 Searching:".bold(), query);

    let client = SurrealMemoryClient::from_env()
        .ok_or_else(|| anyhow::anyhow!("SURREAL_MEMORY_URL not set"))?;

    let results = client.search(query, entity_type).await?;
    println!("  Found {} result(s):\n", results.len());

    for entity in &results {
        println!("  {} [{}] {}", "▸".cyan(), entity.entity_type.dimmed(), entity.name.bold());
        for obs in entity.observations.iter().take(3) {
            println!("    {}", obs.dimmed());
        }
    }

    Ok(())
}

pub async fn install(dry_run: bool) -> Result<()> {
    println!("{}", "🧠 surreal-memory Server Installation".bold());
    println!("{}", "=".repeat(45));

    if dry_run {
        println!("  {}", "(dry run — no changes will be made)".dimmed());
    }

    // ── Step 1: Detect if already installed/running ─────────────────────
    println!("\n  {} Detecting environment...", "→".cyan());

    // Check if server is running (highest priority — may be Docker, binary, or remote)
    let client = SurrealMemoryClient::from_env();
    let server_running = if let Some(ref c) = client {
        c.ping().await.unwrap_or(false)
    } else {
        false
    };

    if server_running {
        let url = client.as_ref().map(|c| c.base_url().to_string()).unwrap_or_default();
        println!("  {} Server already running at {}", "✅".green(), url);

        // Detect if it's running in Docker
        let docker_container = detect_docker_container("surreal-memory");
        if let Some(container_name) = &docker_container {
            println!("  {} Running in Docker container: {}", "🐳".cyan(), container_name);
        }

        println!("\n{}", "✨ surreal-memory is ready — no installation needed".green().bold());
        return Ok(());
    }

    // Check if Docker containers exist but are stopped
    let docker_container = detect_docker_container("surreal-memory");
    if let Some(container_name) = &docker_container {
        println!("  {} Docker container found (stopped): {}", "🐳".yellow(), container_name);
        if !dry_run {
            println!("  Starting container...");
            let _ = Command::new("docker").args(["start", container_name]).output();
            // Also start the surrealdb container if it exists
            if let Some(db_name) = &detect_docker_container("surrealdb") {
                let _ = Command::new("docker").args(["start", db_name]).output();
            }
            println!("  {} Containers started. Verify with: prometheus memory ping", "✅".green());
        } else {
            println!("  Would run: docker start {}", container_name);
        }
        return Ok(());
    }

    // Check if binary exists on PATH
    let binary_installed = which_binary("surreal-memory-server");
    if binary_installed {
        println!("  {} Binary found at: {}", "✅".green(),
            Command::new("which").arg("surreal-memory-server").output()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_default()
        );
    }

    if binary_installed && !server_running {
        println!("  {} Binary installed but server not running", "ℹ️".yellow());
        println!("  Start with: surreal-memory-server");
        if !dry_run {
            println!("\n  Would you like to start it now? Run:");
            println!("    surreal-memory-server &");
        }
        return Ok(());
    }

    // ── Step 2: Detect available installation methods ───────────────────
    println!("\n  {} Checking installation methods...", "→".cyan());

    let has_docker = Command::new("docker").arg("--version").output().is_ok();
    let has_rust = Command::new("cargo").arg("--version").output().is_ok();
    let submodule_path = find_submodule_path();

    println!("  Docker: {}", if has_docker { "✅ available".green() } else { "❌ not found".red() });
    println!("  Rust:   {}", if has_rust { "✅ available".green() } else { "❌ not found".red() });
    println!("  Source: {}", if submodule_path.is_some() {
        "✅ submodule found".green()
    } else {
        "⚠️ submodule not found".yellow()
    });

    // ── Step 3: Select and execute installation method ──────────────────
    if let Some(source_path) = &submodule_path {
        if has_rust {
            println!("\n  {} Recommended: Build from source (embedded SurrealDB)", "→".cyan());
            println!("  This creates a single binary with embedded RocksDB — no external database needed.");

            if dry_run {
                println!("\n  Would run:");
                println!("    cd {} && cargo build --release", source_path.display());
                println!("    cp target/release/surreal-memory-server /usr/local/bin/");
                return Ok(());
            }

            println!("\n  {} Building from source (this may take a few minutes)...", "🔨".yellow());

            let build_output = Command::new("cargo")
                .args(["build", "--release"])
                .current_dir(source_path)
                .output()
                .context("Failed to build surreal-memory-server")?;

            if !build_output.status.success() {
                let stderr = String::from_utf8_lossy(&build_output.stderr);
                anyhow::bail!("Build failed:\n{}", stderr);
            }

            println!("  {} Build complete", "✅".green());

            // Install binary
            let binary = source_path.join("target/release/surreal-memory-server");
            let target = Path::new("/usr/local/bin/surreal-memory-server");

            match std::fs::copy(&binary, target) {
                Ok(_) => println!("  {} Installed to {}", "✅".green(), target.display()),
                Err(e) => {
                    // Try ~/.local/bin as fallback
                    let home_bin = dirs::home_dir().unwrap_or_default().join(".local/bin");
                    std::fs::create_dir_all(&home_bin)?;
                    let fallback = home_bin.join("surreal-memory-server");
                    std::fs::copy(&binary, &fallback)
                        .context(format!("Failed to install: {} (try sudo)", e))?;
                    println!("  {} Installed to {}", "✅".green(), fallback.display());
                    println!("  {} Ensure ~/.local/bin is on your PATH", "ℹ️".dimmed());
                }
            }

            println!("\n  Start the server with:");
            println!("    surreal-memory-server &");
            println!("\n{}", "✨ Installation complete".green().bold());
            return Ok(());
        }
    }

    if has_docker {
        if let Some(source_path) = &submodule_path {
            println!("\n  {} Alternative: Docker Compose (production mode)", "→".cyan());
            println!("  Runs SurrealDB + memory server as containers.");

            if dry_run {
                println!("\n  Would run:");
                println!("    cd {} && docker compose up -d", source_path.display());
                return Ok(());
            }

            let compose_output = Command::new("docker")
                .args(["compose", "up", "-d"])
                .current_dir(source_path)
                .output()
                .context("Failed to run docker compose")?;

            if compose_output.status.success() {
                println!("  {} Docker containers started", "✅".green());
            } else {
                let stderr = String::from_utf8_lossy(&compose_output.stderr);
                println!("  {} Docker compose failed: {}", "❌".red(), stderr.lines().next().unwrap_or("unknown error"));
            }

            println!("\n{}", "✨ Installation complete".green().bold());
            return Ok(());
        }
    }

    // Fallback
    println!("\n  {} Manual installation required:", "ℹ️".yellow());
    println!("    1. Clone: git clone https://github.com/Prometheus-AGS/surreal-memory-server.git");
    println!("    2. Build: cd surreal-memory-server && cargo build --release");
    println!("    3. Install: cp target/release/surreal-memory-server /usr/local/bin/");
    println!("    4. Run: surreal-memory-server &");

    Ok(())
}

fn which_binary(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn find_submodule_path() -> Option<std::path::PathBuf> {
    // Search up from current directory for tools/surreal-memory-server
    let mut dir = std::env::current_dir().ok()?;
    for _ in 0..5 {
        let candidate = dir.join("tools/surreal-memory-server");
        if candidate.join("Cargo.toml").exists() {
            return Some(candidate);
        }
        dir = dir.parent()?.to_path_buf();
    }
    None
}

/// Detect a Docker container by name substring (running or stopped).
fn detect_docker_container(name_pattern: &str) -> Option<String> {
    // Check running containers first
    let running = Command::new("docker")
        .args(["ps", "--format", "{{.Names}}"])
        .output()
        .ok()?;

    if running.status.success() {
        let names = String::from_utf8_lossy(&running.stdout);
        for name in names.lines() {
            if name.contains(name_pattern) {
                return Some(name.to_string());
            }
        }
    }

    // Check stopped containers
    let all = Command::new("docker")
        .args(["ps", "-a", "--format", "{{.Names}}"])
        .output()
        .ok()?;

    if all.status.success() {
        let names = String::from_utf8_lossy(&all.stdout);
        for name in names.lines() {
            if name.contains(name_pattern) {
                return Some(name.to_string());
            }
        }
    }

    None
}
