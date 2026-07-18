use anyhow::{bail, Result};
use colored::Colorize;
use prometheus_agents::detect_installed_agents;
use prometheus_learn::memory::SurrealMemoryClient;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Default)]
pub struct DoctorOptions {
    pub json: bool,
    pub check: Option<String>,
    pub fix: bool,
    pub refresh: bool,
    pub dry_run: bool,
    pub yes: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Severity {
    Green,
    Yellow,
    Red,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
    Skip,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepairAction {
    pub id: String,
    pub description: String,
    pub safe: bool,
    pub reversible: bool,
    pub dry_run_only: bool,
    pub command_hint: Option<String>,
    pub reason_blocked: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckResult {
    pub id: String,
    pub group: String,
    pub label: String,
    pub severity: Severity,
    pub status: CheckStatus,
    pub summary: String,
    pub details: Vec<String>,
    pub optional: bool,
    pub actions: Vec<RepairAction>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DoctorSummary {
    pub failed: usize,
    pub warned: usize,
    pub passed: usize,
    pub skipped: usize,
    pub exit_code: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct DoctorReport {
    pub schema_version: u32,
    pub mode: String,
    pub summary: DoctorSummary,
    pub checks: Vec<CheckResult>,
    pub repair_plan: Option<RepairPlan>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepairPlan {
    pub backup_dir: Option<String>,
    pub safe_actions: Vec<RepairAction>,
    pub manual_actions: Vec<RepairAction>,
    pub execution: ExecutionPlan,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExecutionPlan {
    pub requested: bool,
    pub dry_run: bool,
    pub approved: bool,
    pub applied_actions: usize,
    pub blocked_actions: usize,
    pub note: String,
}

pub async fn run(options: DoctorOptions) -> Result<()> {
    if (options.fix || options.refresh) && options.yes && !options.dry_run {
        let preflight_report = build_report(&options).await;
        let execution = execute_safe_actions(&options, &preflight_report)?;
        let final_report = build_report(&options).await;
        let manifest_path = write_refresh_manifest(&options, &execution, &final_report)?;

        if options.json {
            println!("{}", serde_json::to_string_pretty(&final_report)?);
        } else {
            println!(
                "{}",
                format!(
                    "Applied {} safe action(s); backup: {}; manifest: {}",
                    execution.applied_actions.len(),
                    execution.backup_dir.display(),
                    manifest_path.display()
                )
                .cyan()
            );
            render_human(&final_report, &options);
        }

        if !execution.failed_actions.is_empty() {
            bail!(
                "doctor execution failed for action(s): {}",
                execution.failed_actions.join(", ")
            );
        }
        if final_report.summary.exit_code != 0 {
            bail!("doctor detected failing required checks");
        }

        return Ok(());
    }

    let report = build_report(&options).await;
    let exit_code = report.summary.exit_code;

    if options.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        render_human(&report, &options);
    }

    if (options.fix || options.refresh)
        && !options.dry_run
        && !options.yes
        && report
            .repair_plan
            .as_ref()
            .is_some_and(|plan| !plan.safe_actions.is_empty())
    {
        bail!("confirmation required before mutating doctor actions can run");
    }

    if exit_code != 0 {
        bail!("doctor detected failing required checks");
    }

    Ok(())
}

async fn build_report(options: &DoctorOptions) -> DoctorReport {
    let mut checks = vec![
        check_skills_directory(),
        check_installed_agents(),
        check_surreal_memory().await,
        check_managed_binaries(),
        check_managed_services(),
        check_managed_mcp(),
        check_managed_hooks(),
        check_kbd_state(),
        check_evolver_state(),
        check_trace_store(),
    ];

    if let Some(filter) = options.check.as_deref() {
        checks.retain(|check| check.id == filter || check.group == filter);
    }

    let failed = checks
        .iter()
        .filter(|check| matches!(check.status, CheckStatus::Fail))
        .count();
    let warned = checks
        .iter()
        .filter(|check| matches!(check.status, CheckStatus::Warn))
        .count();
    let passed = checks
        .iter()
        .filter(|check| matches!(check.status, CheckStatus::Pass))
        .count();
    let skipped = checks
        .iter()
        .filter(|check| matches!(check.status, CheckStatus::Skip))
        .count();

    let mode = if options.refresh {
        "refresh"
    } else if options.fix {
        "fix"
    } else {
        "diagnose"
    };

    let exit_code = if failed > 0 { 1 } else { 0 };

    DoctorReport {
        schema_version: 1,
        mode: mode.to_string(),
        summary: DoctorSummary {
            failed,
            warned,
            passed,
            skipped,
            exit_code,
        },
        repair_plan: build_repair_plan(options, &checks),
        checks,
    }
}

fn render_human(report: &DoctorReport, options: &DoctorOptions) {
    println!("{}", "🩺 Health Check".bold());

    if options.fix || options.refresh {
        let mode = if options.refresh { "refresh" } else { "fix" };
        let preview = if options.dry_run { "dry-run " } else { "" };
        println!(
            "{}",
            format!("  Mode: {preview}{mode}{}", if options.yes { " (non-interactive)" } else { "" }).cyan()
        );
    }

    for check in &report.checks {
        let icon = match check.status {
            CheckStatus::Pass => "✅".green(),
            CheckStatus::Warn => "⚠️".yellow(),
            CheckStatus::Fail => "❌".red(),
            CheckStatus::Skip => "⏭".dimmed(),
        };
        println!("  {} {} {}", check.group, check.label, icon);
        println!("    {}", check.summary);
        for detail in &check.details {
            println!("      {} {}", "▸".dimmed(), detail);
        }
        for action in &check.actions {
            let prefix = if action.safe { "↺" } else { "!" };
            println!("      {} {}", prefix.dimmed(), action.description);
            if let Some(reason) = &action.reason_blocked {
                println!("        {} {}", "manual:".dimmed(), reason);
            }
            if let Some(command_hint) = &action.command_hint {
                println!("        {} {}", "hint:".dimmed(), command_hint);
            }
        }
    }

    if let Some(plan) = &report.repair_plan {
        println!("\n{}", "Repair plan".bold());
        if let Some(backup_dir) = &plan.backup_dir {
            println!("  {} {}", "backup".cyan(), backup_dir);
        }
        println!("  {} {}", "execution".cyan(), plan.execution.note);
        if !plan.safe_actions.is_empty() {
            println!("  {}", "safe actions".cyan());
            for action in &plan.safe_actions {
                println!("    {} {}", "•".dimmed(), action.description);
            }
        }
        if !plan.manual_actions.is_empty() {
            println!("  {}", "manual actions".cyan());
            for action in &plan.manual_actions {
                println!("    {} {}", "•".dimmed(), action.description);
            }
        }
        if (options.fix || options.refresh) && !options.dry_run && !options.yes {
            println!(
                "  {}",
                "Confirmation required: rerun with --yes or use --dry-run for a non-mutating plan.".yellow()
            );
        }
    }

    println!("\n{}", "─".repeat(40));
    if report.summary.failed > 0 {
        println!(
            "{}",
            format!("❌ {} failing required check(s)", report.summary.failed).red().bold()
        );
    } else if report.summary.warned > 0 {
        println!(
            "{}",
            format!("⚠️  {} warning(s), no required failures", report.summary.warned)
                .yellow()
                .bold()
        );
    } else {
        println!("{}", "✨ All required checks passed".green().bold());
    }
}

fn build_repair_plan(options: &DoctorOptions, checks: &[CheckResult]) -> Option<RepairPlan> {
    if !options.fix && !options.refresh {
        return None;
    }

    let mut safe_actions = Vec::new();
    let mut manual_actions = Vec::new();

    for check in checks {
        for action in &check.actions {
            if action.safe {
                safe_actions.push(action.clone());
            } else {
                manual_actions.push(action.clone());
            }
        }
    }

    let backup_dir = planned_backup_dir(options);
    let approved = options.yes || options.dry_run;
    let note = if options.dry_run {
        "dry-run only: no filesystem, service, or config changes were made".to_string()
    } else if !options.yes {
        "mutating execution is blocked until --yes is provided for safe, reversible actions".to_string()
    } else if safe_actions.is_empty() {
        "no safe automated actions are available for the current findings".to_string()
    } else {
        "execution wiring remains deny-by-default here: doctor planned safe actions but did not run installers directly in this pass".to_string()
    };

    Some(RepairPlan {
        backup_dir,
        safe_actions,
        manual_actions: manual_actions.clone(),
        execution: ExecutionPlan {
            requested: true,
            dry_run: options.dry_run,
            approved,
            applied_actions: 0,
            blocked_actions: manual_actions.len(),
            note,
        },
    })
}

fn planned_backup_dir(options: &DoctorOptions) -> Option<String> {
    if !options.fix && !options.refresh {
        return None;
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    let mode = if options.refresh { "refresh" } else { "fix" };
    Some(
        PathBuf::from(".prometheus")
            .join("repair")
            .join("doctor-backups")
            .join(format!("{mode}-{timestamp}"))
            .display()
            .to_string(),
    )
}

#[derive(Debug, Clone)]
struct ExecutionOutcome {
    backup_dir: PathBuf,
    applied_actions: Vec<String>,
    failed_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct RefreshManifest {
    schema_version: u32,
    generated_at: String,
    status: String,
    last_successful_refresh: Option<String>,
    applied_actions: Vec<String>,
    repo_head: Option<String>,
    submodule_heads: Vec<ManifestEntry>,
    build_hashes: Vec<HashedPath>,
    installed_hashes: Vec<HashedPath>,
    service_definition_hashes: Vec<HashedPath>,
    catalog_hash: Option<String>,
    mcp_health_snapshot: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ManifestEntry {
    name: String,
    value: String,
}

#[derive(Debug, Clone, Serialize)]
struct HashedPath {
    path: String,
    sha256: String,
}

fn execute_safe_actions(options: &DoctorOptions, report: &DoctorReport) -> Result<ExecutionOutcome> {
    let Some(plan) = &report.repair_plan else {
        bail!("repair plan missing for mutating doctor mode");
    };
    let backup_dir = PathBuf::from(
        plan.backup_dir
            .clone()
            .unwrap_or_else(|| ".prometheus/repair/doctor-backups/manual".into()),
    );
    fs::create_dir_all(&backup_dir)?;
    fs::write(
        backup_dir.join("preflight.json"),
        serde_json::to_vec_pretty(report)?,
    )?;

    let mut applied_actions = Vec::new();
    let mut failed_actions = Vec::new();
    let mut seen = BTreeSet::new();

    for action in &plan.safe_actions {
        if !seen.insert(action.id.clone()) {
            continue;
        }
        match run_safe_action(options, action) {
            Ok(()) => applied_actions.push(action.id.clone()),
            Err(_) => failed_actions.push(action.id.clone()),
        }
    }

    Ok(ExecutionOutcome {
        backup_dir,
        applied_actions,
        failed_actions,
    })
}

fn run_safe_action(options: &DoctorOptions, action: &RepairAction) -> Result<()> {
    let mut command = Command::new("bash");
    match action.id.as_str() {
        "binaries.install-binaries" => {
            command.arg("scripts/install-binaries.sh");
        }
        "skills.sync-codex-skills" => {
            command.arg("scripts/codex-sync-skills.sh");
        }
        "services.install-mcp-services" => {
            command.arg("scripts/install-mcp-services.sh");
        }
        "mcp.configure-all-tools" => {
            command.arg("scripts/configure-mcp-all-tools.sh");
        }
        other => bail!("no executor wired for safe doctor action {other}"),
    };

    if options.dry_run {
        command.arg("--dry-run");
    }

    let status = command.status()?;
    if !status.success() {
        bail!("doctor action {} failed with status {status}", action.id);
    }

    Ok(())
}

fn write_refresh_manifest(
    options: &DoctorOptions,
    execution: &ExecutionOutcome,
    final_report: &DoctorReport,
) -> Result<PathBuf> {
    let manifest_path = PathBuf::from(".prometheus/repair/install-refresh-manifest.json");
    if let Some(parent) = manifest_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let success = final_report.summary.exit_code == 0 && execution.failed_actions.is_empty();
    let timestamp = chrono::Utc::now().to_rfc3339();
    let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
    let local_bin = home_dir.join(".local/bin");
    let manifest = RefreshManifest {
        schema_version: 1,
        generated_at: timestamp.clone(),
        status: if success { "ok".into() } else { "degraded".into() },
        last_successful_refresh: if options.refresh && success {
            Some(timestamp)
        } else {
            None
        },
        applied_actions: execution.applied_actions.clone(),
        repo_head: command_stdout(&["git", "rev-parse", "HEAD"]),
        submodule_heads: collect_submodule_heads(),
        build_hashes: collect_hashed_paths(&[
            "tools/prometheus-cli/target/release/prometheus",
            "tools/forge-rs/target/release/forge",
            "tools/prometheus-knowledge/target/release/pk",
            "tools/prometheus-knowledge/target/release/pk-cherry",
        ]),
        installed_hashes: collect_hashed_paths_from_paths(&[
            local_bin.join("prometheus"),
            local_bin.join("forge"),
            local_bin.join("pk"),
            local_bin.join("pk-cherry"),
        ]),
        service_definition_hashes: collect_hashed_paths(&[
            "shared/launchagents/ai.prometheus.surreal-memory-native.plist",
            "shared/launchagents/ai.prometheus.pk-cherry.plist",
            "shared/launchagents/ai.prometheus.forge-mcp.plist",
            "shared/systemd/ai.prometheus.surreal-memory-native.service",
            "shared/systemd/ai.prometheus.pk-cherry.service",
            "shared/systemd/ai.prometheus.forge-mcp.service",
        ]),
        catalog_hash: hash_file("config/codex-catalog.txt"),
        mcp_health_snapshot: command_stdout(&["bash", "scripts/check-mcp-health.sh", "--json"]),
    };

    fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)?;
    Ok(manifest_path)
}

fn collect_submodule_heads() -> Vec<ManifestEntry> {
    let Some(output) = command_stdout(&["git", "submodule", "status", "--recursive"]) else {
        return Vec::new();
    };

    output
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let mut parts = trimmed.split_whitespace();
            let sha = parts.next()?.trim_start_matches(['-', '+', 'U', ' ']).to_string();
            let name = parts.next()?.to_string();
            Some(ManifestEntry { name, value: sha })
        })
        .collect()
}

fn collect_hashed_paths(paths: &[&str]) -> Vec<HashedPath> {
    paths.iter()
        .filter_map(|path| {
            hash_file(path).map(|sha256| HashedPath {
                path: (*path).to_string(),
                sha256,
            })
        })
        .collect()
}

fn collect_hashed_paths_from_paths(paths: &[PathBuf]) -> Vec<HashedPath> {
    paths.iter()
        .filter_map(|path| {
            hash_path(path).map(|sha256| HashedPath {
                path: path.display().to_string(),
                sha256,
            })
        })
        .collect()
}

fn hash_file(path: &str) -> Option<String> {
    hash_path(Path::new(path))
}

fn hash_path(path: &Path) -> Option<String> {
    let data = fs::read(path).ok()?;
    let mut hasher = Sha256::new();
    hasher.update(data);
    Some(format!("{:x}", hasher.finalize()))
}

fn command_stdout(command: &[&str]) -> Option<String> {
    let (program, args) = command.split_first()?;
    let output = Command::new(program).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8(output.stdout).ok()?;
    Some(stdout.trim().to_string())
}

fn check_skills_directory() -> CheckResult {
    let exists = Path::new("skills").exists();
    if exists {
        CheckResult {
            id: "skills.directory".into(),
            group: "skills".into(),
            label: "Skills directory".into(),
            severity: Severity::Green,
            status: CheckStatus::Pass,
            summary: "skills/ exists".into(),
            details: vec![],
            optional: false,
            actions: vec![],
        }
    } else {
        CheckResult {
            id: "skills.directory".into(),
            group: "skills".into(),
            label: "Skills directory".into(),
            severity: Severity::Red,
            status: CheckStatus::Fail,
            summary: "skills/ is missing".into(),
            details: vec!["Required runtime skill catalog is absent.".into()],
            optional: false,
            actions: vec![RepairAction {
                id: "manual.restore-checkout".into(),
                description: "Restore the checked-out skills catalog before any automated repair run.".into(),
                safe: false,
                reversible: false,
                dry_run_only: false,
                command_hint: None,
                reason_blocked: Some(
                    "Doctor must not recreate or overwrite an unknown checkout tree automatically."
                        .into(),
                ),
            }],
        }
    }
}

fn check_installed_agents() -> CheckResult {
    let agents = detect_installed_agents();
    if agents.is_empty() {
        CheckResult {
            id: "skills.installed-agents".into(),
            group: "skills".into(),
            label: "Installed agents".into(),
            severity: Severity::Yellow,
            status: CheckStatus::Warn,
            summary: "No supported agent homes detected".into(),
            details: vec!["At least one agent home should exist before broad sync or repair work.".into()],
            optional: true,
            actions: vec![RepairAction {
                id: "manual.install-agent-home".into(),
                description: "Install or initialize the target agent home before syncing runtime skills.".into(),
                safe: false,
                reversible: false,
                dry_run_only: false,
                command_hint: None,
                reason_blocked: Some(
                    "Doctor must not invent or overwrite an unknown user agent directory.".into(),
                ),
            }],
        }
    } else {
        CheckResult {
            id: "skills.installed-agents".into(),
            group: "skills".into(),
            label: "Installed agents".into(),
            severity: Severity::Green,
            status: CheckStatus::Pass,
            summary: format!("Detected {} installed agent(s)", agents.len()),
            details: agents
                .into_iter()
                .map(|agent| agent.kind.display_name().to_string())
                .collect(),
            optional: false,
            actions: vec![RepairAction {
                id: "skills.sync-codex-skills".into(),
                description: "Resync the managed Codex skill catalog into the installed Codex runtime.".into(),
                safe: true,
                reversible: true,
                dry_run_only: false,
                command_hint: Some("bash scripts/codex-sync-skills.sh --dry-run".into()),
                reason_blocked: None,
            }],
        }
    }
}

async fn check_surreal_memory() -> CheckResult {
    let Some(client) = SurrealMemoryClient::from_env() else {
        return CheckResult {
            id: "learning.surreal-memory".into(),
            group: "learning".into(),
            label: "Surreal-memory".into(),
            severity: Severity::Red,
            status: CheckStatus::Fail,
            summary: "Surreal-memory is not configured".into(),
            details: vec!["Set SURREAL_MEMORY_URL or restore the default local service.".into()],
            optional: false,
            actions: vec![RepairAction {
                id: "services.install-mcp-services".into(),
                description: "Re-render and reload the managed MCP service stack for the configured user.".into(),
                safe: true,
                reversible: true,
                dry_run_only: false,
                command_hint: Some("bash scripts/install-mcp-services.sh --dry-run".into()),
                reason_blocked: None,
            }],
        };
    };

    match client.ping().await {
        Ok(true) => CheckResult {
            id: "learning.surreal-memory".into(),
            group: "learning".into(),
            label: "Surreal-memory".into(),
            severity: Severity::Green,
            status: CheckStatus::Pass,
            summary: format!("Reachable at {}", client.base_url()),
            details: vec![],
            optional: false,
            actions: vec![],
        },
        Ok(false) | Err(_) => CheckResult {
            id: "learning.surreal-memory".into(),
            group: "learning".into(),
            label: "Surreal-memory".into(),
            severity: Severity::Red,
            status: CheckStatus::Fail,
            summary: format!("Surreal-memory is unreachable at {}", client.base_url()),
            details: vec![
                "Required memory substrate must be healthy before learning-loop work proceeds.".into(),
            ],
            optional: false,
            actions: vec![RepairAction {
                id: "services.install-mcp-services".into(),
                description: "Re-render and reload the managed MCP service stack, then rescan health.".into(),
                safe: true,
                reversible: true,
                dry_run_only: false,
                command_hint: Some("bash scripts/install-mcp-services.sh --dry-run".into()),
                reason_blocked: None,
            }],
        },
    }
}

fn check_kbd_state() -> CheckResult {
    if Path::new(".kbd-orchestrator").exists() {
        CheckResult {
            id: "state.kbd-orchestrator".into(),
            group: "state".into(),
            label: "KBD orchestrator".into(),
            severity: Severity::Green,
            status: CheckStatus::Pass,
            summary: ".kbd-orchestrator is initialized".into(),
            details: vec![],
            optional: false,
            actions: vec![],
        }
    } else {
        CheckResult {
            id: "state.kbd-orchestrator".into(),
            group: "state".into(),
            label: "KBD orchestrator".into(),
            severity: Severity::Yellow,
            status: CheckStatus::Skip,
            summary: ".kbd-orchestrator is not initialized".into(),
            details: vec!["KBD state is absent in this working directory.".into()],
            optional: true,
            actions: vec![],
        }
    }
}

fn check_evolver_state() -> CheckResult {
    if Path::new(".evolver").exists() {
        CheckResult {
            id: "state.evolver".into(),
            group: "state".into(),
            label: "Evolver state".into(),
            severity: Severity::Green,
            status: CheckStatus::Pass,
            summary: ".evolver is initialized".into(),
            details: vec![],
            optional: true,
            actions: vec![],
        }
    } else {
        CheckResult {
            id: "state.evolver".into(),
            group: "state".into(),
            label: "Evolver state".into(),
            severity: Severity::Yellow,
            status: CheckStatus::Skip,
            summary: ".evolver is not initialized".into(),
            details: vec!["Evolution state is optional until an evolution cycle is started.".into()],
            optional: true,
            actions: vec![],
        }
    }
}

fn check_trace_store() -> CheckResult {
    let trace_dir = Path::new(".prometheus/traces");
    if trace_dir.exists() {
        let store = prometheus_learn::TraceStore::default_for_project(Path::new("."));
        let count = store.count_all().unwrap_or(0);
        CheckResult {
            id: "learning.trace-store".into(),
            group: "learning".into(),
            label: "Trace store".into(),
            severity: Severity::Green,
            status: CheckStatus::Pass,
            summary: format!("Trace store present with {count} trace(s)"),
            details: vec![],
            optional: true,
            actions: vec![],
        }
    } else {
        CheckResult {
            id: "learning.trace-store".into(),
            group: "learning".into(),
            label: "Trace store".into(),
            severity: Severity::Yellow,
            status: CheckStatus::Skip,
            summary: "No trace store yet".into(),
            details: vec!["Trace capture has not yet produced .prometheus/traces.".into()],
            optional: true,
            actions: vec![],
        }
    }
}

fn check_managed_binaries() -> CheckResult {
    CheckResult {
        id: "binaries.manifest".into(),
        group: "binaries".into(),
        label: "Managed binary manifest".into(),
        severity: Severity::Yellow,
        status: CheckStatus::Skip,
        summary: "Binary refresh manifest is not implemented yet".into(),
        details: vec!["Doctor registry now reserves the binaries group for pinned-source hash and freshness checks.".into()],
        optional: true,
        actions: vec![RepairAction {
            id: "binaries.install-binaries".into(),
            description: "Rebuild and reinstall the managed binaries from the pinned source checkout.".into(),
            safe: true,
            reversible: true,
            dry_run_only: false,
            command_hint: Some("bash scripts/install-binaries.sh --dry-run".into()),
            reason_blocked: None,
        }],
    }
}

fn check_managed_services() -> CheckResult {
    CheckResult {
        id: "services.launch-agents".into(),
        group: "services".into(),
        label: "Managed LaunchAgents".into(),
        severity: Severity::Yellow,
        status: CheckStatus::Skip,
        summary: "LaunchAgent ownership checks are not implemented yet".into(),
        details: vec!["Doctor registry now reserves the services group for plist ownership and health validation.".into()],
        optional: true,
        actions: vec![RepairAction {
            id: "services.install-mcp-services".into(),
            description: "Reload only the managed MCP service definitions owned by this repository.".into(),
            safe: true,
            reversible: true,
            dry_run_only: false,
            command_hint: Some("bash scripts/install-mcp-services.sh --dry-run".into()),
            reason_blocked: None,
        }],
    }
}

fn check_managed_mcp() -> CheckResult {
    CheckResult {
        id: "mcp.config".into(),
        group: "mcp".into(),
        label: "Managed MCP configuration".into(),
        severity: Severity::Yellow,
        status: CheckStatus::Skip,
        summary: "MCP reconciliation checks are not implemented yet".into(),
        details: vec!["Doctor registry now reserves the mcp group for declarative client config verification.".into()],
        optional: true,
        actions: vec![RepairAction {
            id: "mcp.configure-all-tools".into(),
            description: "Reconcile only the managed MCP sections for supported client tools.".into(),
            safe: true,
            reversible: true,
            dry_run_only: false,
            command_hint: Some("bash scripts/configure-mcp-all-tools.sh --dry-run".into()),
            reason_blocked: None,
        }],
    }
}

fn check_managed_hooks() -> CheckResult {
    CheckResult {
        id: "hooks.lifecycle".into(),
        group: "hooks".into(),
        label: "Lifecycle hooks".into(),
        severity: Severity::Yellow,
        status: CheckStatus::Skip,
        summary: "Hook integrity checks are not implemented yet".into(),
        details: vec!["Doctor registry now reserves the hooks group for prompt/stop/subagent hook validation.".into()],
        optional: true,
        actions: vec![RepairAction {
            id: "manual.review-hooks".into(),
            description: "Review hook ownership and payload integrity before any automatic replacement.".into(),
            safe: false,
            reversible: false,
            dry_run_only: false,
            command_hint: None,
            reason_blocked: Some(
                "Doctor must not overwrite unknown hook customizations or local user workflow hooks."
                    .into(),
            ),
        }],
    }
}
