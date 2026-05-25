use anyhow::Result;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentStatus {
    Ok,
    Missing,
    SkippedDocker,
    SkippedLaunchd,
    Installed,
    NotInstalled,
}

impl ComponentStatus {
    fn icon(&self) -> colored::ColoredString {
        match self {
            Self::Ok | Self::SkippedDocker | Self::SkippedLaunchd | Self::Installed => {
                "✅".green()
            }
            Self::Missing | Self::NotInstalled => "❌".red(),
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Missing => "missing",
            Self::SkippedDocker => "running (Docker)",
            Self::SkippedLaunchd => "running (launchd)",
            Self::Installed => "installed",
            Self::NotInstalled => "not installed",
        }
    }

    fn needs_action(&self) -> bool {
        matches!(self, Self::Missing | Self::NotInstalled)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentState {
    pub status: ComponentStatus,
    pub last_checked: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetupState {
    pub last_run: String,
    pub components: std::collections::HashMap<String, ComponentState>,
}

struct Component {
    id: &'static str,
    description: &'static str,
    detect: fn() -> ComponentStatus,
    install: Option<fn() -> Result<()>>,
}

fn detect_port(port: u16) -> bool {
    std::net::TcpStream::connect_timeout(
        &std::net::SocketAddr::from(([127, 0, 0, 1], port)),
        std::time::Duration::from_millis(200),
    )
    .is_ok()
}

fn detect_docker_container(name: &str) -> bool {
    std::process::Command::new("docker")
        .args(["ps", "--filter", &format!("name={name}"), "--format", "{{.Names}}"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains(name))
        .unwrap_or(false)
}

fn detect_launchd(label: &str) -> bool {
    std::process::Command::new("launchctl")
        .args(["list", label])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn detect_binary(name: &str) -> bool {
    std::process::Command::new("which")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn detect_surreal_memory() -> ComponentStatus {
    if detect_docker_container("surreal-memory") || detect_port(23001) {
        ComponentStatus::SkippedDocker
    } else {
        ComponentStatus::Missing
    }
}

fn detect_openai_proxy() -> ComponentStatus {
    if detect_launchd("dev.prometheusags.openai-proxy") || detect_port(8181) {
        ComponentStatus::SkippedLaunchd
    } else {
        ComponentStatus::Missing
    }
}

fn detect_forge_mcp() -> ComponentStatus {
    if detect_launchd("dev.prometheusags.forge-mcp") || detect_port(8943) {
        ComponentStatus::Ok
    } else {
        ComponentStatus::Missing
    }
}

fn detect_pk_mcp() -> ComponentStatus {
    if detect_launchd("dev.prometheusags.pk-mcp") || detect_port(8942) {
        ComponentStatus::Ok
    } else {
        ComponentStatus::Missing
    }
}

fn detect_liter_llm() -> ComponentStatus {
    if detect_binary("liter-llm") {
        ComponentStatus::Installed
    } else {
        ComponentStatus::NotInstalled
    }
}

fn detect_prometheus_cli() -> ComponentStatus {
    if detect_binary("prometheus") {
        ComponentStatus::Installed
    } else {
        ComponentStatus::NotInstalled
    }
}

fn detect_forge_bin() -> ComponentStatus {
    if detect_binary("forge") {
        ComponentStatus::Installed
    } else {
        ComponentStatus::NotInstalled
    }
}

fn detect_pk_cherry() -> ComponentStatus {
    if detect_binary("pk-cherry") {
        ComponentStatus::Installed
    } else {
        ComponentStatus::NotInstalled
    }
}

fn detect_sycophancy_correction() -> ComponentStatus {
    if detect_binary("sycophancy-correction") {
        ComponentStatus::Installed
    } else {
        ComponentStatus::NotInstalled
    }
}

fn detect_template_forge() -> ComponentStatus {
    if detect_binary("template-forge") {
        ComponentStatus::Installed
    } else {
        ComponentStatus::NotInstalled
    }
}

fn detect_template_forge_mcp() -> ComponentStatus {
    if detect_binary("template-forge-mcp") {
        ComponentStatus::Installed
    } else {
        ComponentStatus::NotInstalled
    }
}

fn install_template_forge_binaries() -> Result<()> {
    let repo_root = std::env::var("PROMETHEUS_SKILL_PACK_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            // Heuristic: walk up from the prometheus binary location
            std::env::current_exe()
                .ok()
                .and_then(|p| {
                    // target/release/prometheus → look for skills/imported/artifact-refiner
                    p.ancestors()
                        .find(|a| a.join("skills/imported/artifact-refiner").exists())
                        .map(|p| p.to_path_buf())
                })
                .unwrap_or_default()
        });

    let forge_dir =
        repo_root.join("skills/imported/artifact-refiner/tools/template-forge-rs");
    anyhow::ensure!(
        forge_dir.exists(),
        "template-forge-rs not found at {}; run: git submodule update --init --recursive",
        forge_dir.display()
    );

    let status = std::process::Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(&forge_dir)
        .env("RUSTUP_TOOLCHAIN", "stable")
        .status()?;
    anyhow::ensure!(status.success(), "cargo build failed for template-forge-rs");

    let bin_dir = dirs::home_dir().unwrap_or_default().join(".local/bin");
    std::fs::create_dir_all(&bin_dir)?;
    for name in &["template-forge", "template-forge-mcp"] {
        std::fs::copy(forge_dir.join("target/release").join(name), bin_dir.join(name))?;
    }
    Ok(())
}

fn load_launchd_forge_mcp() -> Result<()> {
    let plist = dirs::home_dir()
        .unwrap_or_default()
        .join("Library/LaunchAgents/dev.prometheusags.forge-mcp.plist");
    let status = std::process::Command::new("launchctl")
        .args(["load", &plist.to_string_lossy()])
        .status()?;
    anyhow::ensure!(status.success(), "launchctl load failed for forge-mcp");
    Ok(())
}

fn load_launchd_pk_mcp() -> Result<()> {
    let plist = dirs::home_dir()
        .unwrap_or_default()
        .join("Library/LaunchAgents/dev.prometheusags.pk-mcp.plist");
    let status = std::process::Command::new("launchctl")
        .args(["load", &plist.to_string_lossy()])
        .status()?;
    anyhow::ensure!(status.success(), "launchctl load failed for pk-mcp");
    Ok(())
}

fn components() -> Vec<Component> {
    vec![
        Component {
            id: "surreal-memory-server",
            description: "surreal-memory-server (Docker, port 23001)",
            detect: detect_surreal_memory,
            install: None,
        },
        Component {
            id: "openai-proxy",
            description: "openai-proxy (launchd, port 8181)",
            detect: detect_openai_proxy,
            install: None,
        },
        Component {
            id: "forge-mcp",
            description: "forge-mcp SSE server (launchd, port 8943)",
            detect: detect_forge_mcp,
            install: Some(load_launchd_forge_mcp),
        },
        Component {
            id: "pk-mcp",
            description: "prometheus-knowledge MCP server (launchd, port 8942)",
            detect: detect_pk_mcp,
            install: Some(load_launchd_pk_mcp),
        },
        Component {
            id: "liter-llm",
            description: "liter-llm stdio MCP proxy (~/.local/bin/liter-llm)",
            detect: detect_liter_llm,
            install: None,
        },
        Component {
            id: "prometheus",
            description: "prometheus CLI (~/.local/bin/prometheus)",
            detect: detect_prometheus_cli,
            install: None,
        },
        Component {
            id: "forge",
            description: "forge code enrichment CLI (~/.local/bin/forge)",
            detect: detect_forge_bin,
            install: None,
        },
        Component {
            id: "pk-cherry",
            description: "pk-cherry knowledge MCP binary (~/.local/bin/pk-cherry)",
            detect: detect_pk_cherry,
            install: None,
        },
        Component {
            id: "sycophancy-correction",
            description: "sycophancy-correction binary (/usr/local/bin/)",
            detect: detect_sycophancy_correction,
            install: None,
        },
        Component {
            id: "template-forge",
            description: "template-forge artifact renderer (~/.local/bin/template-forge)",
            detect: detect_template_forge,
            install: Some(install_template_forge_binaries),
        },
        Component {
            id: "template-forge-mcp",
            description: "template-forge-mcp stdio MCP server (~/.local/bin/template-forge-mcp)",
            detect: detect_template_forge_mcp,
            install: None,
        },
    ]
}

fn setup_state_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".prometheus")
        .join("setup-state.json")
}

fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn write_setup_state(states: &[(String, ComponentStatus)]) -> Result<()> {
    let path = setup_state_path();
    std::fs::create_dir_all(path.parent().unwrap())?;

    let mut map = std::collections::HashMap::new();
    for (id, status) in states {
        map.insert(
            id.clone(),
            ComponentState { status: *status, last_checked: now_rfc3339() },
        );
    }

    let state = SetupState { last_run: now_rfc3339(), components: map };
    std::fs::write(&path, serde_json::to_string_pretty(&state)?)?;
    Ok(())
}

fn prompt_yes(label: &str) -> bool {
    use std::io::{BufRead, Write};
    print!("  Install {}? [y/N] ", label);
    std::io::stdout().flush().ok();
    let mut line = String::new();
    std::io::stdin().lock().read_line(&mut line).ok();
    matches!(line.trim().to_lowercase().as_str(), "y" | "yes")
}

pub fn run(non_interactive: bool, dry_run: bool, check: bool) -> Result<()> {
    println!("{}", "🚀 Prometheus Setup".bold());

    if dry_run {
        println!("  {}", "(dry run — no changes will be made)".dimmed());
    }
    println!();

    let comps = components();
    let statuses: Vec<(&Component, ComponentStatus)> =
        comps.iter().map(|c| (c, (c.detect)())).collect();

    // Print status table
    println!("{}", "Component Status".bold().underline());
    let mut gap_count = 0u32;
    for (comp, status) in &statuses {
        println!("  {} {} — {}", status.icon(), comp.description, status.label().dimmed());
        if status.needs_action() {
            gap_count += 1;
        }
    }
    println!();

    if gap_count == 0 {
        println!("{}", "✨ All components healthy — nothing to do.".green().bold());
    } else {
        println!(
            "  {} gap(s) detected.",
            gap_count.to_string().yellow().bold()
        );
    }

    if check || gap_count == 0 {
        // Save state and exit
        let pairs: Vec<_> = statuses.iter().map(|(c, s)| (c.id.to_string(), *s)).collect();
        write_setup_state(&pairs)?;
        return Ok(());
    }

    println!();

    // Interactive or automatic install
    let mut final_states: Vec<(String, ComponentStatus)> = statuses
        .iter()
        .map(|(c, s)| (c.id.to_string(), *s))
        .collect();

    for (i, (comp, status)) in statuses.iter().enumerate() {
        if !status.needs_action() || comp.install.is_none() {
            continue;
        }
        let installer = comp.install.unwrap();

        let should_install = if dry_run {
            println!("  {} would install: {}", "▸".dimmed(), comp.description);
            false
        } else if non_interactive {
            true
        } else {
            prompt_yes(comp.description)
        };

        if should_install {
            print!("  Installing {}... ", comp.description);
            match installer() {
                Ok(()) => {
                    println!("{}", "done".green());
                    final_states[i].1 = ComponentStatus::Installed;
                }
                Err(e) => {
                    println!("{} {}", "failed:".red(), e);
                }
            }
        }
    }

    println!();
    write_setup_state(&final_states)?;
    println!(
        "  State written to {}",
        setup_state_path().display().to_string().dimmed()
    );

    if !dry_run {
        println!("{}", "\n✨ Setup complete.".green().bold());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn component_status_needs_action_only_for_gap_states() {
        assert!(ComponentStatus::Missing.needs_action());
        assert!(ComponentStatus::NotInstalled.needs_action());
        assert!(!ComponentStatus::Ok.needs_action());
        assert!(!ComponentStatus::SkippedDocker.needs_action());
        assert!(!ComponentStatus::SkippedLaunchd.needs_action());
        assert!(!ComponentStatus::Installed.needs_action());
    }

    #[test]
    fn setup_state_path_ends_with_expected_filename() {
        let path = setup_state_path();
        assert_eq!(path.file_name().unwrap().to_str().unwrap(), "setup-state.json");
    }

    #[test]
    fn component_status_labels_are_non_empty() {
        let statuses = [
            ComponentStatus::Ok,
            ComponentStatus::Missing,
            ComponentStatus::SkippedDocker,
            ComponentStatus::SkippedLaunchd,
            ComponentStatus::Installed,
            ComponentStatus::NotInstalled,
        ];
        for status in statuses {
            assert!(!status.label().is_empty());
        }
    }
}
