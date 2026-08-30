use anyhow::{Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentStatus {
    Ok,
    Missing,
    SkippedDocker,
    SkippedLaunchd,
    Installed,
    Disabled,
    NotInstalled,
    Stale,
}

impl ComponentStatus {
    fn icon(&self) -> colored::ColoredString {
        match self {
            Self::Ok
            | Self::SkippedDocker
            | Self::SkippedLaunchd
            | Self::Installed
            | Self::Disabled => "✅".green(),
            Self::Missing | Self::NotInstalled => "❌".red(),
            Self::Stale => "⚠️ ".yellow(),
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Missing => "missing",
            Self::SkippedDocker => "running (Docker)",
            Self::SkippedLaunchd => "running (launchd)",
            Self::Installed => "installed",
            Self::Disabled => "disabled (optional)",
            Self::NotInstalled => "not installed",
            Self::Stale => "stale (source newer than binary)",
        }
    }

    fn needs_action(&self) -> bool {
        matches!(self, Self::Missing | Self::NotInstalled | Self::Stale)
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

/// Characterises *how* a component is installed.
///
/// The variant determines `--rebuild` eligibility:
/// - `Cargo` components are staleness-tracked binary builds (cargo + cp).
///   They are always targeted by `--rebuild`.
/// - `Custom` components have a bespoke install fn (launchd load, submodule build, etc.)
///   They are only invoked when `status.needs_action()` — never by `--rebuild` alone.
/// - `None` components have no automated installer.
#[derive(Clone, Copy)]
enum ComponentInstaller {
    /// Build from source via cargo and install to ~/.local/bin/.
    Cargo(fn() -> Result<()>),
    /// Any other automated installer (launchctl load, submodule build, etc.).
    Custom(fn() -> Result<()>),
    /// No installer — must be set up manually.
    None,
}

impl ComponentInstaller {
    fn is_some(&self) -> bool {
        !matches!(self, Self::None)
    }

    fn is_cargo(&self) -> bool {
        matches!(self, Self::Cargo(_))
    }

    fn invoke(&self) -> Option<Result<()>> {
        match self {
            Self::Cargo(f) | Self::Custom(f) => Some(f()),
            Self::None => None,
        }
    }
}

struct Component {
    id: &'static str,
    description: &'static str,
    detect: fn() -> ComponentStatus,
    install: ComponentInstaller,
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
        .args([
            "ps",
            "--filter",
            &format!("name={name}"),
            "--format",
            "{{.Names}}",
        ])
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

fn detect_systemd_user(unit: &str) -> bool {
    std::process::Command::new("systemctl")
        .args(["--user", "is-active", "--quiet", unit])
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

// ─── Staleness detection ────────────────────────────────────────────────────
// A binary is "stale" when its source has commits newer than the binary's mtime.
// Source-of-truth mapping:
//   prometheus  → path-scoped: git log -1 --format=%ct -- tools/prometheus-cli/
//   forge       → submodule HEAD time: git -C tools/forge-rs log -1 --format=%ct HEAD
//   pk-cherry   → submodule HEAD time
//   liter-llm   → submodule HEAD time
// If git/source is unavailable, callers fall back to plain `Installed`.

/// Pure comparator — testable without filesystem or git side effects.
fn is_stale(binary_mtime: SystemTime, source_commit_time: SystemTime) -> bool {
    source_commit_time > binary_mtime
}

/// Locate the skill-pack repo root.
/// Prefers PROMETHEUS_SKILL_PACK_ROOT env; falls back to walking up from the current exe.
fn repo_root() -> Option<PathBuf> {
    fn is_repo_root(path: &Path) -> bool {
        path.join("skills/imported/artifact-refiner").exists()
    }

    if let Ok(env_path) = std::env::var("PROMETHEUS_SKILL_PACK_ROOT") {
        let p = PathBuf::from(env_path);
        if is_repo_root(&p) {
            return Some(p);
        }
    }
    if let Ok(current_dir) = std::env::current_dir() {
        if let Some(root) = current_dir.ancestors().find(|path| is_repo_root(path)) {
            return Some(root.to_path_buf());
        }
    }
    std::env::current_exe().ok().and_then(|p| {
        p.ancestors()
            .find(|path| is_repo_root(path))
            .map(|p| p.to_path_buf())
    })
}

/// Resolve the installed binary's mtime.
///
/// `install-binaries.sh` writes the 4 staleness-tracked binaries to `~/.local/bin/`,
/// so we check that path specifically rather than whatever `which` returns first on PATH
/// (which may shadow our install with an unrelated leftover in `/usr/local/bin/`).
fn binary_mtime(name: &str) -> Option<SystemTime> {
    let bin_dir = dirs::home_dir()?.join(".local/bin");
    let path = bin_dir.join(name);
    std::fs::metadata(&path).ok()?.modified().ok()
}

/// Run `git log -1 --format=%ct <args>` in `cwd` and parse the result as a SystemTime.
fn git_commit_time(cwd: &Path, args: &[&str]) -> Option<SystemTime> {
    let out = std::process::Command::new("git")
        .current_dir(cwd)
        .arg("log")
        .arg("-1")
        .arg("--format=%ct")
        .args(args)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let secs: u64 = String::from_utf8_lossy(&out.stdout).trim().parse().ok()?;
    Some(UNIX_EPOCH + Duration::from_secs(secs))
}

/// Dispatch source-commit-time lookup by binary id.
/// Returns None when the binary is not staleness-tracked or git/source is unavailable.
fn source_commit_time_for(binary_id: &str) -> Option<SystemTime> {
    let root = repo_root()?;
    match binary_id {
        "prometheus" => git_commit_time(&root, &["--", "tools/prometheus-cli/"]),
        "forge" => git_commit_time(&root.join("tools/forge-rs"), &["HEAD"]),
        "pk-cherry" => git_commit_time(&root.join("tools/prometheus-knowledge"), &["HEAD"]),
        "liter-llm" => git_commit_time(&root.join("tools/liter-llm"), &["HEAD"]),
        _ => None,
    }
}

/// Best-effort staleness check for a tracked binary id.
/// Returns true only when both mtime and source-time are available AND source is newer.
fn binary_is_stale(binary_id: &str) -> bool {
    match (binary_mtime(binary_id), source_commit_time_for(binary_id)) {
        (Some(mtime), Some(source)) => is_stale(mtime, source),
        _ => false,
    }
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
    if !detect_binary("liter-llm") {
        return ComponentStatus::NotInstalled;
    }
    if binary_is_stale("liter-llm") {
        ComponentStatus::Stale
    } else {
        ComponentStatus::Installed
    }
}

fn detect_prometheus_cli() -> ComponentStatus {
    if !detect_binary("prometheus") {
        return ComponentStatus::NotInstalled;
    }
    if binary_is_stale("prometheus") {
        ComponentStatus::Stale
    } else {
        ComponentStatus::Installed
    }
}

fn detect_forge_bin() -> ComponentStatus {
    if !detect_binary("forge") {
        return ComponentStatus::NotInstalled;
    }
    if binary_is_stale("forge") {
        ComponentStatus::Stale
    } else {
        ComponentStatus::Installed
    }
}

fn detect_pk_cherry() -> ComponentStatus {
    if !detect_binary("pk-cherry") {
        return ComponentStatus::NotInstalled;
    }
    if binary_is_stale("pk-cherry") {
        ComponentStatus::Stale
    } else {
        ComponentStatus::Installed
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

fn detect_control_plane_service() -> ComponentStatus {
    let registered = if cfg!(target_os = "macos") {
        detect_launchd("ai.prometheus.sovereign-sync")
    } else if cfg!(target_os = "linux") {
        detect_systemd_user("ai.prometheus.sovereign-sync.service")
    } else {
        false
    };
    let installed_binary = bin_dir().join("sovereign-sync");
    let binary = if installed_binary.is_file() {
        installed_binary
    } else {
        PathBuf::from("sovereign-sync")
    };
    let healthy = std::process::Command::new(binary)
        .args(["--mode", "status", "--format", "json"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if registered && healthy {
        ComponentStatus::Ok
    } else {
        ComponentStatus::Missing
    }
}

/// Desired ordinary-setup state: the optional sharing service is stopped and
/// explicitly disabled, or it has no managed definition at all.
fn detect_daemon_free_control_plane() -> ComponentStatus {
    if cfg!(target_os = "macos") {
        let labels = [
            "ai.prometheus.sovereign-sync",
            "com.prometheusags.sovereign-sync",
        ];
        if labels.iter().any(|label| detect_launchd(label)) {
            return detect_control_plane_service();
        }

        let launch_agents = dirs::home_dir()
            .unwrap_or_default()
            .join("Library/LaunchAgents");
        if labels
            .iter()
            .all(|label| !launch_agents.join(format!("{label}.plist")).is_file())
        {
            return ComponentStatus::Disabled;
        }

        let uid = std::process::Command::new("id")
            .arg("-u")
            .output()
            .ok()
            .filter(|output| output.status.success())
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
            .unwrap_or_default();
        let disabled = std::process::Command::new("launchctl")
            .args(["print-disabled", &format!("gui/{uid}")])
            .output()
            .ok()
            .filter(|output| output.status.success())
            .map(|output| {
                let registry = String::from_utf8_lossy(&output.stdout);
                labels
                    .iter()
                    .all(|label| registry.contains(&format!("\"{label}\" => disabled")))
            })
            .unwrap_or(false);
        return if disabled {
            ComponentStatus::Disabled
        } else {
            ComponentStatus::Missing
        };
    }

    if cfg!(target_os = "linux") {
        let units = [
            "ai.prometheus.sovereign-sync.service",
            "com.prometheusags.sovereign-sync.service",
        ];
        let installed = units.iter().copied().filter(|unit| {
            std::process::Command::new("systemctl")
                .args(["--user", "cat", unit])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .map(|status| status.success())
                .unwrap_or(false)
        });
        let installed: Vec<_> = installed.collect();
        if installed.is_empty() {
            return ComponentStatus::Disabled;
        }
        if units.iter().any(|unit| detect_systemd_user(unit)) {
            return detect_control_plane_service();
        }
        let all_disabled = installed.iter().all(|unit| {
            std::process::Command::new("systemctl")
                .args(["--user", "is-enabled", unit])
                .output()
                .ok()
                .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
                .is_some_and(|state| matches!(state.as_str(), "disabled" | "masked" | "not-found"))
        });
        return if all_disabled {
            ComponentStatus::Disabled
        } else {
            ComponentStatus::Missing
        };
    }

    ComponentStatus::Disabled
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

    let forge_dir = repo_root.join("skills/imported/artifact-refiner/tools/template-forge-rs");
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
        std::fs::copy(
            forge_dir.join("target/release").join(name),
            bin_dir.join(name),
        )?;
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

// ─── Binary installers (rebuild + install to ~/.local/bin/) ─────────────────
// Each fn shells `cargo build --release -p <pkg>` in the source dir, then
// copies the resulting binary to ~/.local/bin/. Modeled on
// `install_template_forge_binaries`. Package names track the upstream renames
// caught in the machine-refresh phase: forge-cli (not forge), liter-llm-cli
// (not liter-llm).

fn bin_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_default().join(".local/bin")
}

/// Shared helper: build a single cargo package in `crate_dir`, then copy the
/// produced binary `bin_name` from `crate_dir/target/release/` to ~/.local/bin/.
fn cargo_build_and_install(crate_dir: &Path, pkg: &str, bin_name: &str) -> Result<()> {
    anyhow::ensure!(
        crate_dir.exists(),
        "source dir not found: {} — run: git submodule update --init --recursive",
        crate_dir.display()
    );
    let status = std::process::Command::new("cargo")
        .args(["build", "--release", "-p", pkg])
        .current_dir(crate_dir)
        .status()?;
    anyhow::ensure!(status.success(), "cargo build failed for {pkg}");

    let src = crate_dir.join("target/release").join(bin_name);
    let dst_dir = bin_dir();
    std::fs::create_dir_all(&dst_dir)?;
    let dst = dst_dir.join(bin_name);
    std::fs::copy(&src, &dst)
        .with_context(|| format!("failed to copy {} → {}", src.display(), dst.display()))?;
    Ok(())
}

fn install_prometheus_cli() -> Result<()> {
    let root = repo_root().ok_or_else(|| anyhow::anyhow!("could not locate repo root"))?;
    cargo_build_and_install(
        &root.join("tools/prometheus-cli"),
        "prometheus-cli",
        "prometheus",
    )
}

fn install_forge_cli() -> Result<()> {
    let root = repo_root().ok_or_else(|| anyhow::anyhow!("could not locate repo root"))?;
    cargo_build_and_install(&root.join("tools/forge-rs"), "forge-cli", "forge")?;
    kickstart_or_warn("dev.prometheusags.forge-mcp");
    Ok(())
}

fn install_pk_cherry() -> Result<()> {
    let root = repo_root().ok_or_else(|| anyhow::anyhow!("could not locate repo root"))?;
    cargo_build_and_install(
        &root.join("tools/prometheus-knowledge"),
        "pk-cherry",
        "pk-cherry",
    )?;
    kickstart_or_warn("dev.prometheusags.pk-mcp");
    Ok(())
}

fn install_liter_llm() -> Result<()> {
    let root = repo_root().ok_or_else(|| anyhow::anyhow!("could not locate repo root"))?;
    cargo_build_and_install(&root.join("tools/liter-llm"), "liter-llm-cli", "liter-llm")
}

fn managed_service_installer_args(
    dry_run: bool,
    restart: bool,
    sharing: bool,
) -> Vec<&'static str> {
    let mut args = Vec::new();
    if dry_run {
        args.push("--dry-run");
    }
    if restart {
        args.push("--restart");
    }
    if sharing {
        args.push("--sharing");
    }
    args
}

fn managed_service_action(
    full: bool,
    check: bool,
    dry_run: bool,
    rebuild: bool,
    approved: bool,
) -> Option<(bool, bool)> {
    if !full || check {
        None
    } else if dry_run {
        Some((true, rebuild))
    } else if approved {
        Some((false, rebuild))
    } else {
        None
    }
}

fn install_managed_services(dry_run: bool, restart: bool, sharing: bool) -> Result<()> {
    let root = repo_root().ok_or_else(|| anyhow::anyhow!("could not locate repo root"))?;
    let installer = root.join("scripts/install-mcp-services.sh");
    anyhow::ensure!(
        installer.is_file(),
        "managed-service installer not found: {}",
        installer.display()
    );

    let status = std::process::Command::new("bash")
        .arg(&installer)
        .args(managed_service_installer_args(dry_run, restart, sharing))
        .current_dir(&root)
        .status()
        .with_context(|| format!("failed to run {}", installer.display()))?;
    anyhow::ensure!(status.success(), "managed-service installation failed");
    Ok(())
}

/// Best-effort `launchctl kickstart -k gui/<uid>/<label>`.
/// Warns and continues on failure (per locked decision: kickstart is soft).
fn kickstart_or_warn(label: &str) {
    let uid_out = std::process::Command::new("id").arg("-u").output();
    let uid = uid_out
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    if uid.is_empty() {
        eprintln!(
            "  {} kickstart {}: could not resolve current uid",
            "⚠".yellow(),
            label
        );
        return;
    }
    let target = format!("gui/{uid}/{label}");
    let status = std::process::Command::new("launchctl")
        .args(["kickstart", "-k", &target])
        .status();
    match status {
        Ok(s) if s.success() => println!("  {} kickstart {}: ok", "↻".cyan(), label),
        Ok(s) => eprintln!("  {} kickstart {}: exited {}", "⚠".yellow(), label, s),
        Err(e) => eprintln!(
            "  {} kickstart {}: spawn failed: {}",
            "⚠".yellow(),
            label,
            e
        ),
    }
}

fn components() -> Vec<Component> {
    vec![
        Component {
            id: "surreal-memory-server",
            description: "surreal-memory-server (Docker, port 23001)",
            detect: detect_surreal_memory,
            install: ComponentInstaller::None,
        },
        Component {
            id: "openai-proxy",
            description: "openai-proxy (launchd, port 8181)",
            detect: detect_openai_proxy,
            install: ComponentInstaller::None,
        },
        Component {
            id: "forge-mcp",
            description: "forge-mcp SSE server (launchd, port 8943)",
            detect: detect_forge_mcp,
            install: ComponentInstaller::Custom(load_launchd_forge_mcp),
        },
        Component {
            id: "pk-mcp",
            description: "prometheus-knowledge MCP server (launchd, port 8942)",
            detect: detect_pk_mcp,
            install: ComponentInstaller::Custom(load_launchd_pk_mcp),
        },
        Component {
            id: "liter-llm",
            description: "liter-llm stdio MCP proxy (~/.local/bin/liter-llm)",
            detect: detect_liter_llm,
            install: ComponentInstaller::Cargo(install_liter_llm),
        },
        Component {
            id: "prometheus",
            description: "prometheus CLI (~/.local/bin/prometheus)",
            detect: detect_prometheus_cli,
            install: ComponentInstaller::Cargo(install_prometheus_cli),
        },
        Component {
            id: "forge",
            description: "forge code enrichment CLI (~/.local/bin/forge)",
            detect: detect_forge_bin,
            install: ComponentInstaller::Cargo(install_forge_cli),
        },
        Component {
            id: "pk-cherry",
            description: "pk-cherry knowledge MCP binary (~/.local/bin/pk-cherry)",
            detect: detect_pk_cherry,
            install: ComponentInstaller::Cargo(install_pk_cherry),
        },
        Component {
            id: "sycophancy-correction",
            description: "sycophancy-correction binary (/usr/local/bin/)",
            detect: detect_sycophancy_correction,
            install: ComponentInstaller::None,
        },
        Component {
            id: "template-forge",
            description: "template-forge artifact renderer (~/.local/bin/template-forge)",
            detect: detect_template_forge,
            install: ComponentInstaller::Custom(install_template_forge_binaries),
        },
        Component {
            id: "template-forge-mcp",
            description: "template-forge-mcp stdio MCP server (~/.local/bin/template-forge-mcp)",
            detect: detect_template_forge_mcp,
            install: ComponentInstaller::None,
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
            ComponentState {
                status: *status,
                last_checked: now_rfc3339(),
            },
        );
    }

    let state = SetupState {
        last_run: now_rfc3339(),
        components: map,
    };
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

pub fn run(
    full: bool,
    sharing: bool,
    non_interactive: bool,
    dry_run: bool,
    check: bool,
    rebuild: bool,
) -> Result<()> {
    // --rebuild implies --non-interactive (locked decision: rebuild is automation).
    let non_interactive = non_interactive || rebuild;

    println!("{}", "🚀 Prometheus Setup".bold());

    if dry_run {
        println!("  {}", "(dry run — no changes will be made)".dimmed());
    }
    if rebuild {
        println!(
            "  {}",
            "(--rebuild — forcing rebuild of all binary components)".cyan()
        );
    }
    if full {
        println!(
            "  {}",
            "(--full — including managed local services; KBD remains daemon-free)".cyan()
        );
    }
    if sharing {
        println!(
            "  {}",
            "(--sharing — enabling the optional sovereign-sync sharing service)".cyan()
        );
    }
    println!();

    let comps = components();
    let statuses: Vec<(&Component, ComponentStatus)> =
        comps.iter().map(|c| (c, (c.detect)())).collect();

    // Print status table
    println!("{}", "Component Status".bold().underline());
    let mut missing_count = 0u32;
    let mut stale_count = 0u32;
    for (comp, status) in &statuses {
        println!(
            "  {} {} — {}",
            status.icon(),
            comp.description,
            status.label().dimmed()
        );
        match status {
            ComponentStatus::Missing | ComponentStatus::NotInstalled => missing_count += 1,
            ComponentStatus::Stale => stale_count += 1,
            _ => {}
        }
    }
    let control_plane_status = full.then(|| {
        if sharing {
            detect_control_plane_service()
        } else {
            detect_daemon_free_control_plane()
        }
    });
    if let Some(status) = control_plane_status {
        println!(
            "  {} {} — {}",
            status.icon(),
            if sharing {
                "KBD sharing service (sovereign-sync)"
            } else {
                "Optional KBD sharing service (daemon-free target)"
            },
            status.label().dimmed()
        );
        if sharing
            && matches!(
                status,
                ComponentStatus::Missing | ComponentStatus::NotInstalled
            )
        {
            missing_count += 1;
        }
    }
    let gap_count = missing_count + stale_count;
    println!();

    if gap_count == 0 && !rebuild {
        if full {
            println!(
                "{}",
                "✨ All components healthy — managed definitions will be reconciled."
                    .green()
                    .bold()
            );
        } else {
            println!(
                "{}",
                "✨ All components healthy — nothing to do.".green().bold()
            );
        }
    } else if gap_count > 0 {
        println!(
            "  {} gap(s) detected: {} missing, {} stale.",
            gap_count.to_string().yellow().bold(),
            missing_count.to_string().red(),
            stale_count.to_string().yellow(),
        );
    }

    // --check exits before installing. --rebuild bypasses the "all healthy" short-circuit
    // so it can force installs even on a clean system.
    if check || (gap_count == 0 && !rebuild && !full) {
        let mut pairs: Vec<_> = statuses
            .iter()
            .map(|(c, s)| (c.id.to_string(), *s))
            .collect();
        if full {
            pairs.push((
                "control-plane-service".to_string(),
                control_plane_status.expect("full setup has a control-plane status"),
            ));
        }
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
        if !comp.install.is_some() {
            continue;
        }
        // --rebuild forces Cargo (build-from-source) components regardless of status.
        // Custom and None components are only invoked when status.needs_action().
        // This is self-documenting via the ComponentInstaller variant — no allowlist needed.
        let force_rebuild = rebuild && comp.install.is_cargo();
        let must_act = force_rebuild || status.needs_action();
        if !must_act {
            continue;
        }

        let should_install = if dry_run {
            let verb = if force_rebuild {
                "would rebuild"
            } else {
                "would install"
            };
            println!("  {} {}: {}", "▸".dimmed(), verb, comp.description);
            false
        } else if non_interactive {
            true
        } else {
            prompt_yes(comp.description)
        };

        if should_install {
            let verb = if force_rebuild {
                "Rebuilding"
            } else {
                "Installing"
            };
            println!("  {} {}...", verb, comp.description);
            match comp.install.invoke() {
                Some(Ok(())) => {
                    println!("    {}", "done".green());
                    final_states[i].1 = ComponentStatus::Installed;
                }
                Some(Err(e)) => {
                    println!("    {} {}", "failed:".red(), e);
                }
                None => {} // ComponentInstaller::None — unreachable here (filtered above)
            }
        }
    }

    if full {
        let before = control_plane_status.expect("full setup has a control-plane status");
        let approved = if dry_run {
            false
        } else {
            non_interactive
                || prompt_yes(if sharing {
                    "managed services and the optional sovereign-sync sharing service"
                } else {
                    "managed local services (sovereign-sync will be disabled)"
                })
        };
        let action = managed_service_action(full, check, dry_run, rebuild, approved);
        if dry_run {
            println!("  {} managed services: dry-run", "▸".dimmed());
        }
        if let Some((service_dry_run, restart)) = action {
            install_managed_services(service_dry_run, restart, sharing)?;
        }

        let attempted_install = matches!(action, Some((false, _)));
        let after = if dry_run || action.is_none() {
            before
        } else if sharing {
            detect_control_plane_service()
        } else {
            detect_daemon_free_control_plane()
        };
        final_states.push(("control-plane-service".to_string(), after));
        if sharing && attempted_install && !matches!(after, ComponentStatus::Ok) {
            anyhow::bail!(
                "KBD control-plane service is not both registered and healthy after setup"
            );
        }
        if !sharing && attempted_install && !matches!(after, ComponentStatus::Disabled) {
            anyhow::bail!(
                "optional sovereign-sync service is not both stopped and disabled after setup"
            );
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
        assert!(ComponentStatus::Stale.needs_action());
        assert!(!ComponentStatus::Ok.needs_action());
        assert!(!ComponentStatus::SkippedDocker.needs_action());
        assert!(!ComponentStatus::SkippedLaunchd.needs_action());
        assert!(!ComponentStatus::Installed.needs_action());
    }

    #[test]
    fn stale_status_serializes_snake_case() {
        let json = serde_json::to_string(&ComponentStatus::Stale).unwrap();
        assert_eq!(json, "\"stale\"");
        let back: ComponentStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ComponentStatus::Stale);
    }

    #[test]
    fn is_stale_returns_true_when_source_newer() {
        let binary = UNIX_EPOCH + Duration::from_secs(1_000_000);
        let source = UNIX_EPOCH + Duration::from_secs(2_000_000);
        assert!(is_stale(binary, source));
    }

    #[test]
    fn is_stale_returns_false_when_source_older() {
        let binary = UNIX_EPOCH + Duration::from_secs(2_000_000);
        let source = UNIX_EPOCH + Duration::from_secs(1_000_000);
        assert!(!is_stale(binary, source));
    }

    #[test]
    fn is_stale_returns_false_when_equal() {
        let t = UNIX_EPOCH + Duration::from_secs(1_500_000);
        assert!(!is_stale(t, t));
    }

    #[test]
    fn source_commit_time_for_unknown_binary_returns_none() {
        assert!(source_commit_time_for("nonexistent-binary-xyz").is_none());
    }

    #[test]
    fn setup_state_path_ends_with_expected_filename() {
        let path = setup_state_path();
        assert_eq!(
            path.file_name().unwrap().to_str().unwrap(),
            "setup-state.json"
        );
    }

    #[test]
    fn component_status_labels_are_non_empty() {
        let statuses = [
            ComponentStatus::Ok,
            ComponentStatus::Missing,
            ComponentStatus::SkippedDocker,
            ComponentStatus::SkippedLaunchd,
            ComponentStatus::Installed,
            ComponentStatus::Disabled,
            ComponentStatus::NotInstalled,
            ComponentStatus::Stale,
        ];
        for status in statuses {
            assert!(!status.label().is_empty());
        }
    }

    #[test]
    fn managed_service_args_preserve_dry_run_and_restart() {
        assert_eq!(
            managed_service_installer_args(false, false, false),
            Vec::<&str>::new()
        );
        assert_eq!(
            managed_service_installer_args(true, false, false),
            vec!["--dry-run"]
        );
        assert_eq!(
            managed_service_installer_args(false, true, false),
            vec!["--restart"]
        );
        assert_eq!(
            managed_service_installer_args(true, true, false),
            vec!["--dry-run", "--restart"]
        );
        assert_eq!(
            managed_service_installer_args(false, false, true),
            vec!["--sharing"]
        );
    }

    #[test]
    fn managed_services_are_opt_in_and_checks_never_install() {
        assert_eq!(
            managed_service_action(false, false, false, false, true),
            None
        );
        assert_eq!(managed_service_action(true, true, false, false, true), None);
        assert_eq!(
            managed_service_action(true, false, false, false, false),
            None
        );
    }

    #[test]
    fn managed_service_action_covers_preview_install_and_rebuild() {
        assert_eq!(
            managed_service_action(true, false, true, false, false),
            Some((true, false))
        );
        assert_eq!(
            managed_service_action(true, false, false, false, true),
            Some((false, false))
        );
        assert_eq!(
            managed_service_action(true, false, false, true, true),
            Some((false, true))
        );
    }
}
