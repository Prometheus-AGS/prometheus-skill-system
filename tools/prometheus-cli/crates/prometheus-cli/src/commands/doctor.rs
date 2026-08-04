use anyhow::{bail, Result};
use colored::Colorize;
use kbd_runtime::{rollout::RolloutTracker, Runtime};
use prometheus_agents::detect_installed_agents;
use prometheus_learn::memory::SurrealMemoryClient;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Default)]
pub struct DoctorOptions {
    pub json: bool,
    pub check: Option<String>,
    pub exclude: Vec<String>,
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
    #[serde(rename = "contractVersion")]
    pub contract_version: &'static str,
    pub mode: String,
    pub selection: DoctorSelection,
    pub summary: DoctorSummary,
    pub checks: Vec<CheckResult>,
    pub repair_plan: Option<RepairPlan>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DoctorSelection {
    pub check: Option<String>,
    pub excluded: Vec<String>,
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
    let mut checks = Vec::new();
    macro_rules! run_check {
        ($id:literal, $group:literal, $check:expr) => {
            if check_selected(options, $id, $group) {
                checks.push($check);
            }
        };
    }

    run_check!("skills.directory", "skills", check_skills_directory());
    run_check!(
        "skills.installed-agents",
        "skills",
        check_installed_agents()
    );
    run_check!(
        "learning.surreal-memory",
        "learning",
        check_surreal_memory().await
    );
    run_check!(
        "review.judge-gateway",
        "review",
        check_judge_gateway().await
    );
    run_check!("binaries.manifest", "binaries", check_managed_binaries());
    run_check!(
        "services.launch-agents",
        "services",
        check_managed_services()
    );
    run_check!("learning.worker", "learning", check_learning_worker());
    run_check!("learning.snapshots", "learning", check_prompt_snapshots());
    run_check!("hooks.rotation", "hooks", check_hook_log_rotation());
    run_check!("mcp.config", "mcp", check_managed_mcp());
    run_check!("hooks.lifecycle", "hooks", check_managed_hooks());
    run_check!(
        "control.kbd-runtime",
        "control",
        check_kbd_control_plane().await
    );
    run_check!("state.kbd-orchestrator", "state", check_kbd_state());
    run_check!("control.kbd-rollout", "control", check_kbd_rollout());
    run_check!(
        "hooks.harness-adapters",
        "hooks",
        check_harness_adapter_parity()
    );
    run_check!(
        "skills.discovery-budget",
        "skills",
        check_instruction_budgets()
    );
    run_check!("state.evolver", "state", check_evolver_state());
    run_check!("learning.trace-store", "learning", check_trace_store());
    scope_repair_actions(options, &mut checks);

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
        contract_version: "2.0.0",
        mode: mode.to_string(),
        selection: DoctorSelection {
            check: options.check.clone(),
            excluded: options.exclude.clone(),
        },
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

fn check_selected(options: &DoctorOptions, id: &str, group: &str) -> bool {
    let included = options
        .check
        .as_deref()
        .is_none_or(|filter| filter == id || filter == group);
    included
        && !options
            .exclude
            .iter()
            .any(|excluded| excluded == id || excluded == group)
}

fn service_excluded(options: &DoctorOptions, service: &str) -> bool {
    let scope = format!("service:{service}");
    options.exclude.iter().any(|excluded| excluded == &scope)
}

fn scope_repair_actions(options: &DoctorOptions, checks: &mut [CheckResult]) {
    if !service_excluded(options, "sovereign-sync") {
        return;
    }
    for action in checks
        .iter_mut()
        .flat_map(|check| check.actions.iter_mut())
        .filter(|action| action.id.starts_with("services."))
    {
        if let Some(command_hint) = action.command_hint.as_mut() {
            if !command_hint.contains("--exclude sovereign-sync") {
                command_hint.push_str(" --exclude sovereign-sync");
            }
        }
    }
}

fn render_human(report: &DoctorReport, options: &DoctorOptions) {
    println!("{}", "🩺 Health Check".bold());

    if options.fix || options.refresh {
        let mode = if options.refresh { "refresh" } else { "fix" };
        let preview = if options.dry_run { "dry-run " } else { "" };
        println!(
            "{}",
            format!(
                "  Mode: {preview}{mode}{}",
                if options.yes {
                    " (non-interactive)"
                } else {
                    ""
                }
            )
            .cyan()
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
                "Confirmation required: rerun with --yes or use --dry-run for a non-mutating plan."
                    .yellow()
            );
        }
    }

    println!("\n{}", "─".repeat(40));
    if report.summary.failed > 0 {
        println!(
            "{}",
            format!("❌ {} failing required check(s)", report.summary.failed)
                .red()
                .bold()
        );
    } else if report.summary.warned > 0 {
        println!(
            "{}",
            format!(
                "⚠️  {} warning(s), no required failures",
                report.summary.warned
            )
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
        "mutating execution is blocked until --yes is provided for safe, reversible actions"
            .to_string()
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
    surreal_memory_readiness: Option<String>,
    plugin_generation: Option<String>,
    learning_status: Option<String>,
    prompt_snapshot_pointers: Vec<ManifestEntry>,
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

fn execute_safe_actions(
    options: &DoctorOptions,
    report: &DoctorReport,
) -> Result<ExecutionOutcome> {
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
            if service_excluded(options, "sovereign-sync") {
                command.args(["--exclude", "sovereign-sync"]);
            }
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
        status: if success {
            "ok".into()
        } else {
            "degraded".into()
        },
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
            "tools/prometheus-knowledge/target/release/pk",
            "tools/prometheus-knowledge/target/release/pk-cherry",
            "tools/prometheus-knowledge/target/release/prometheus-learning-worker",
            "tools/surreal-memory-server/target/release/surreal-memory-server",
        ]),
        installed_hashes: collect_hashed_paths_from_paths(&[
            local_bin.join("prometheus"),
            local_bin.join("pk"),
            local_bin.join("pk-cherry"),
            local_bin.join("prometheus-learning-worker"),
            local_bin.join("surreal-memory-server"),
        ]),
        service_definition_hashes: collect_hashed_paths(&[
            "shared/launchagents/ai.prometheus.surreal-memory-native.plist",
            "shared/launchagents/ai.prometheus.pk-cherry.plist",
            "shared/launchagents/ai.prometheus.learning-worker.plist",
            "shared/launchagents/ai.prometheus.hooks-logrotate.plist",
            "shared/systemd/ai.prometheus.surreal-memory-native.service",
            "shared/systemd/ai.prometheus.pk-cherry.service",
            "shared/systemd/ai.prometheus.learning-worker.service",
            "shared/systemd/ai.prometheus.hooks-logrotate.service",
        ]),
        catalog_hash: hash_file("config/codex-catalog.txt"),
        mcp_health_snapshot: command_stdout(&[
            "bash",
            "scripts/check-mcp-health.sh",
            "--json",
            "--exclude",
            "sovereign-sync",
        ]),
        surreal_memory_readiness: command_stdout(&["curl", "-fsS", "http://127.0.0.1:23001/ready"]),
        plugin_generation: command_stdout(&[
            "node",
            "scripts/install-plugin-generation.js",
            "--verify",
        ]),
        learning_status: command_stdout(&["prometheus", "learning", "status", "--json"]),
        prompt_snapshot_pointers: collect_prompt_snapshot_pointers(&home_dir),
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
            let sha = parts
                .next()?
                .trim_start_matches(['-', '+', 'U', ' '])
                .to_string();
            let name = parts.next()?.to_string();
            Some(ManifestEntry { name, value: sha })
        })
        .collect()
}

fn collect_prompt_snapshot_pointers(home: &Path) -> Vec<ManifestEntry> {
    let project_root = std::env::var_os("PROMETHEUS_PROJECT_ROOT")
        .map(PathBuf::from)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));
    let global = home.join(".prometheus/knowledge");
    [
        (
            "project",
            project_root.join(".prometheus/knowledge/.prompt-snapshots/project/current"),
        ),
        (
            "shared",
            global.join("shared/.prompt-snapshots/shared/current"),
        ),
        ("global", global.join(".prompt-snapshots/global/current")),
    ]
    .into_iter()
    .filter_map(|(name, path)| {
        fs::read_to_string(path).ok().map(|value| ManifestEntry {
            name: name.into(),
            value: value.trim().into(),
        })
    })
    .collect()
}

fn collect_hashed_paths(paths: &[&str]) -> Vec<HashedPath> {
    paths
        .iter()
        .filter_map(|path| {
            hash_file(path).map(|sha256| HashedPath {
                path: (*path).to_string(),
                sha256,
            })
        })
        .collect()
}

fn collect_hashed_paths_from_paths(paths: &[PathBuf]) -> Vec<HashedPath> {
    paths
        .iter()
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
                description:
                    "Restore the checked-out skills catalog before any automated repair run.".into(),
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
            details: vec![
                "At least one agent home should exist before broad sync or repair work.".into(),
            ],
            optional: true,
            actions: vec![RepairAction {
                id: "manual.install-agent-home".into(),
                description:
                    "Install or initialize the target agent home before syncing runtime skills."
                        .into(),
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
                description:
                    "Resync the managed Codex skill catalog into the installed Codex runtime."
                        .into(),
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
                description:
                    "Re-render and reload the managed MCP service stack for the configured user."
                        .into(),
                safe: true,
                reversible: true,
                dry_run_only: false,
                command_hint: Some("bash scripts/install-mcp-services.sh --dry-run".into()),
                reason_blocked: None,
            }],
        };
    };

    let health = client.ping().await;
    let readiness = if matches!(health, Ok(true)) {
        client.readiness().await.ok()
    } else {
        None
    };
    let ledger_ready = readiness.as_ref().is_some_and(|payload| {
        payload["status"] == "ready"
            && payload["ingestion_ready"] == true
            && payload["capabilities"]["ledger"] == true
            && payload["capabilities"]["storage"] == true
            && payload["capabilities"]["coordinator"] == true
    });

    match (health, ledger_ready) {
        (Ok(true), true) => CheckResult {
            id: "learning.surreal-memory".into(),
            group: "learning".into(),
            label: "Surreal-memory".into(),
            severity: Severity::Green,
            status: CheckStatus::Pass,
            summary: format!("Durable operation ledger is ready at {}", client.base_url()),
            details: vec![format!(
                "readiness: {}",
                serde_json::to_string(readiness.as_ref().expect("readiness was validated"))
                    .unwrap_or_else(|_| "unavailable".into())
            )],
            optional: false,
            actions: vec![],
        },
        _ => CheckResult {
            id: "learning.surreal-memory".into(),
            group: "learning".into(),
            label: "Surreal-memory".into(),
            severity: Severity::Red,
            status: CheckStatus::Fail,
            summary: format!("Surreal-memory is unhealthy or not ready at {}", client.base_url()),
            details: vec![
                "Required memory substrate must expose healthy ledger, storage, coordinator, and ingestion readiness before learning-loop work proceeds.".into(),
            ],
            optional: false,
            actions: vec![RepairAction {
                id: "services.install-mcp-services".into(),
                description:
                    "Re-render and reload the managed MCP service stack, then rescan health.".into(),
                safe: true,
                reversible: true,
                dry_run_only: false,
                command_hint: Some("bash scripts/install-mcp-services.sh --dry-run".into()),
                reason_blocked: None,
            }],
        },
    }
}

/// Report whether a judge gateway is reachable.
///
/// This exists because the failure it detects is SILENT. When no gateway
/// answers, adversarial review does not error — it falls back to a
/// harness-native model, still returns `PASS`, and records
/// `isolation_mode: harness-native` in the findings artifact. That is how eight
/// consecutive reviews in this repository were Claude grading Claude while the
/// pipeline reported success the entire time.
///
/// Optional by design: a user who never runs an adversarial review needs no
/// gateway, so this is Yellow/Warn rather than Red/Fail. What it must never do
/// is stay quiet, or report "not implemented" — an unreported degradation is
/// indistinguishable from a working gate.
async fn check_judge_gateway() -> CheckResult {
    // Candidates in the same precedence order the shell resolver uses:
    // LITER_LLM_BASE_URL wins, then openai-proxy, then a liter-llm api server.
    let mut candidates: Vec<String> = Vec::new();
    if let Ok(explicit) = std::env::var("LITER_LLM_BASE_URL") {
        if !explicit.trim().is_empty() {
            candidates.push(explicit.trim().trim_end_matches('/').to_string());
        }
    }
    candidates.push("http://localhost:8181/v1".to_string());
    candidates.push("http://localhost:4000/v1".to_string());

    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        // Loopback must not be sent through a corporate proxy; a proxied
        // localhost probe reports a false negative.
        .no_proxy()
        .build()
    {
        Ok(client) => client,
        Err(_) => {
            return CheckResult {
                id: "review.judge-gateway".into(),
                group: "review".into(),
                label: "Adversarial judge gateway".into(),
                severity: Severity::Yellow,
                status: CheckStatus::Skip,
                summary: "could not construct an HTTP client to probe the gateway".into(),
                details: vec![],
                optional: true,
                actions: vec![],
            };
        }
    };

    for base in &candidates {
        let url = format!("{}/models", base);
        // A 401 still proves something is LISTENING and speaking the API — it is
        // an auth problem, not an availability one, and the two need different
        // fixes. Treat any HTTP response as "reachable" and report the status.
        if let Ok(resp) = client.get(&url).send().await {
            let status = resp.status();
            if status.is_success() {
                return CheckResult {
                    id: "review.judge-gateway".into(),
                    group: "review".into(),
                    label: "Adversarial judge gateway".into(),
                    severity: Severity::Green,
                    status: CheckStatus::Pass,
                    summary: format!("Reachable at {}", base),
                    details: vec![
                        "Adversarial review can resolve a judge distinct from the producer.".into(),
                    ],
                    optional: true,
                    actions: vec![],
                };
            }
            return CheckResult {
                id: "review.judge-gateway".into(),
                group: "review".into(),
                label: "Adversarial judge gateway".into(),
                severity: Severity::Yellow,
                status: CheckStatus::Warn,
                summary: format!("{} responded HTTP {}", base, status.as_u16()),
                details: vec![
                    "A gateway is listening but rejected the request. liter-llm returns 401 on \
                     every /v1/* route when [general] master_key is unset."
                        .into(),
                ],
                optional: true,
                actions: vec![RepairAction {
                    id: "review.configure-models".into(),
                    description: "Repair the liter-llm gateway config (merges, never clobbers)."
                        .into(),
                    safe: true,
                    reversible: true,
                    dry_run_only: false,
                    command_hint: Some(
                        "bash skills/process/liter-llm-bridge/scripts/configure-models.sh check"
                            .into(),
                    ),
                    reason_blocked: None,
                }],
            };
        }
    }

    CheckResult {
        id: "review.judge-gateway".into(),
        group: "review".into(),
        label: "Adversarial judge gateway".into(),
        severity: Severity::Yellow,
        status: CheckStatus::Warn,
        summary: format!(
            "No judge gateway reachable (tried {})",
            candidates.join(", ")
        ),
        details: vec![
            "Adversarial reviews will DEGRADE to a same-model self-review: they still \
             return PASS, recording isolation_mode: harness-native."
                .into(),
            "Start openai-proxy (:8181) or a liter-llm api server, then re-run doctor.".into(),
        ],
        optional: true,
        actions: vec![RepairAction {
            id: "review.install-judge-gateway".into(),
            description: "Build and install the optional openai-proxy judge gateway.".into(),
            safe: true,
            reversible: true,
            dry_run_only: false,
            command_hint: Some("bash scripts/install-binaries.sh --dry-run".into()),
            reason_blocked: None,
        }],
    }
}

async fn check_kbd_control_plane() -> CheckResult {
    if !Path::new(".prometheus/project.json").exists() {
        return CheckResult {
            id: "control.kbd-runtime".into(),
            group: "control".into(),
            label: "KBD quorum control plane".into(),
            severity: Severity::Yellow,
            status: CheckStatus::Skip,
            summary: "project identity is not initialized".into(),
            details: vec!["Run `prometheus kbd migrate --check` from a KBD project.".into()],
            optional: true,
            actions: vec![],
        };
    }
    let runtime = Runtime::open(".");
    let project_id = runtime
        .replay()
        .ok()
        .filter(|state| state.revision > 0)
        .map(|state| state.project_id)
        .or_else(|| {
            runtime
                .project_manifest(false)
                .ok()
                .flatten()
                .map(|manifest| manifest.project_id)
        });
    let result = async {
        let project_id = project_id.ok_or_else(|| anyhow::anyhow!("missing project id"))?;
        let endpoint = std::env::var("PROMETHEUS_CONTROL_ENDPOINT")
            .unwrap_or_else(|_| "http://127.0.0.1:7892".into());
        let response = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(2))
            .build()?
            .get(format!(
                "{}/api/v1/kbd/projects/{project_id}/diagnostics",
                endpoint.trim_end_matches('/')
            ))
            .send()
            .await?;
        let status = response.status();
        let body = response.text().await?;
        if !status.is_success() {
            anyhow::bail!("{status}: {body}");
        }
        Ok::<serde_json::Value, anyhow::Error>(serde_json::from_str(&body)?)
    }
    .await;

    match result {
        Ok(diagnostics) => {
            let writable = diagnostics["quorum"]["writable"].as_bool().unwrap_or(false);
            let writer_available = diagnostics["singleWriter"]["available"]
                .as_bool()
                .unwrap_or(false);
            let journal_lamport = diagnostics["journal"]["lastLamport"].as_u64().unwrap_or(0);
            let journal_ingested = diagnostics["journal"]["ingested"]
                .as_bool()
                .unwrap_or(false);
            let document_revision = diagnostics["document"]["derivedRevision"]
                .as_u64()
                .unwrap_or(0);
            let projection_matches = diagnostics["projection"]["matchesRuntime"]
                .as_bool()
                .unwrap_or(false);
            let signatures = diagnostics["integrity"]["signatureChainValid"]
                .as_bool()
                .unwrap_or(false);
            let healthy = writable
                && writer_available
                && journal_ingested
                && projection_matches
                && signatures;
            CheckResult {
                id: "control.kbd-runtime".into(),
                group: "control".into(),
                label: "KBD journal control plane".into(),
                severity: if healthy {
                    Severity::Green
                } else {
                    Severity::Red
                },
                status: if healthy {
                    CheckStatus::Pass
                } else {
                    CheckStatus::Fail
                },
                summary: format!(
                    "writer {}, replica L{}, document r{}, projection {}, signatures {}",
                    if writer_available {
                        "available"
                    } else {
                        "unavailable"
                    },
                    journal_lamport,
                    document_revision,
                    if projection_matches {
                        "current"
                    } else {
                        "stale"
                    },
                    if signatures { "valid" } else { "invalid" }
                ),
                details: vec![
                    format!(
                        "quorum: {}",
                        diagnostics["quorum"]["reason"]
                            .as_str()
                            .unwrap_or("unknown")
                    ),
                    format!(
                        "single writer node/lock: {}/{}",
                        diagnostics["singleWriter"]["nodeId"].as_u64().unwrap_or(0),
                        diagnostics["singleWriter"]["lockPath"]
                            .as_str()
                            .unwrap_or("unknown")
                    ),
                    format!(
                        "journal path/bytes/lamport: {}/{}/{}",
                        diagnostics["journal"]["path"].as_str().unwrap_or("unknown"),
                        diagnostics["journal"]["bytes"].as_u64().unwrap_or(0),
                        journal_lamport
                    ),
                    format!(
                        "project document events/frontier/conflicts: {}/{}/{}",
                        diagnostics["document"]["eventCount"].as_u64().unwrap_or(0),
                        diagnostics["document"]["frontier"],
                        diagnostics["document"]["conflictCount"]
                            .as_u64()
                            .unwrap_or(0)
                    ),
                    format!(
                        "trusted devices active/revoked: {}/{}",
                        diagnostics["trust"]["activeDevices"].as_u64().unwrap_or(0),
                        diagnostics["trust"]["revokedDevices"].as_u64().unwrap_or(0)
                    ),
                    format!(
                        "signed events: {}",
                        diagnostics["integrity"]["eventCount"].as_u64().unwrap_or(0)
                    ),
                ],
                optional: false,
                actions: vec![],
            }
        }
        Err(error) => CheckResult {
            id: "control.kbd-runtime".into(),
            group: "control".into(),
            label: "KBD quorum control plane".into(),
            severity: Severity::Yellow,
            status: CheckStatus::Warn,
            summary: "KBD daemon diagnostics are unreachable".into(),
            details: vec![
                error.to_string(),
                "No direct compatibility-file fallback was used.".into(),
            ],
            optional: false,
            actions: vec![],
        },
    }
}

fn check_kbd_state() -> CheckResult {
    if !Path::new(".kbd-orchestrator").exists() {
        return CheckResult {
            id: "state.kbd-orchestrator".into(),
            group: "state".into(),
            label: "KBD orchestrator".into(),
            severity: Severity::Yellow,
            status: CheckStatus::Skip,
            summary: ".kbd-orchestrator is not initialized".into(),
            details: vec!["KBD state is absent in this working directory.".into()],
            optional: true,
            actions: vec![],
        };
    }

    let waypoint = fs::read(".kbd-orchestrator/current-waypoint.json")
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok());
    let position = fs::read(".kbd-orchestrator/position.json")
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok());
    match (waypoint, position) {
        (Some(waypoint), Some(position)) => {
            let waypoint_revision = waypoint["sourceRevision"]
                .as_u64()
                .or_else(|| waypoint["revision"].as_u64());
            let position_revision = position["sourceRevision"].as_u64();
            let generated = waypoint["generatedBy"] == "kbd-runtime"
                && position["generatedBy"] == "kbd-runtime";
            let healthy =
                generated && waypoint_revision.is_some() && waypoint_revision == position_revision;
            CheckResult {
                id: "state.kbd-orchestrator".into(),
                group: "state".into(),
                label: "KBD compatibility projections".into(),
                severity: if healthy {
                    Severity::Green
                } else {
                    Severity::Yellow
                },
                status: if healthy {
                    CheckStatus::Pass
                } else {
                    CheckStatus::Warn
                },
                summary: format!(
                    "revision {}, projections {}",
                    waypoint_revision
                        .map(|revision| revision.to_string())
                        .unwrap_or_else(|| "unknown".into()),
                    if healthy { "consistent" } else { "mismatched" }
                ),
                details: vec![
                    "Compatibility JSON is projection-only; control.kbd-runtime checks authority."
                        .into(),
                ],
                optional: false,
                actions: vec![],
            }
        }
        (None, None) => CheckResult {
            id: "state.kbd-orchestrator".into(),
            group: "state".into(),
            label: "KBD compatibility projections".into(),
            severity: Severity::Yellow,
            status: CheckStatus::Warn,
            summary: "compatibility projections have not been generated".into(),
            details: vec!["The canonical runtime may still be available via the daemon.".into()],
            optional: false,
            actions: vec![],
        },
        _ => CheckResult {
            id: "state.kbd-orchestrator".into(),
            group: "state".into(),
            label: "KBD compatibility projections".into(),
            severity: Severity::Yellow,
            status: CheckStatus::Warn,
            summary: "only one compatibility projection is readable".into(),
            details: vec!["Regenerate projections from committed runtime state.".into()],
            optional: false,
            actions: vec![],
        },
    }
}

fn check_kbd_rollout() -> CheckResult {
    if !Path::new(".prometheus/project.json").exists() {
        return CheckResult {
            id: "control.kbd-rollout".into(),
            group: "control".into(),
            label: "KBD production rollout gate".into(),
            severity: Severity::Yellow,
            status: CheckStatus::Skip,
            summary: "rollout evidence is not applicable before project initialization".into(),
            details: vec![],
            optional: true,
            actions: vec![],
        };
    }
    let runtime = Runtime::open(".");
    let path = runtime.runtime_root().join("rollout-evidence.json");
    if !path.exists() {
        return CheckResult {
            id: "control.kbd-rollout".into(),
            group: "control".into(),
            label: "KBD production rollout gate".into(),
            severity: Severity::Yellow,
            status: CheckStatus::Warn,
            summary: "shadow evidence collection has not started".into(),
            details: vec![
                "Production remains blocked; run `prometheus kbd rollout observe` from a live control plane.".into(),
            ],
            optional: false,
            actions: vec![],
        };
    }
    match RolloutTracker::open(runtime.runtime_root()).load() {
        Ok(evidence) => {
            let gate = evidence.gate();
            let production = evidence.stage == kbd_runtime::rollout::RolloutStage::Production;
            CheckResult {
                id: "control.kbd-rollout".into(),
                group: "control".into(),
                label: "KBD production rollout gate".into(),
                severity: if production {
                    Severity::Green
                } else {
                    Severity::Yellow
                },
                status: if production {
                    CheckStatus::Pass
                } else {
                    CheckStatus::Warn
                },
                summary: if production {
                    "all staged rollout gates are complete".into()
                } else {
                    format!(
                        "{:?} is active; next promotion {}",
                        evidence.stage,
                        if gate.passed {
                            "is eligible"
                        } else {
                            "is blocked"
                        }
                    )
                },
                details: vec![
                    format!(
                        "successful days: {}; real/synthetic mutations: {}/{}",
                        gate.consecutive_successful_days,
                        gate.real_mutations,
                        gate.synthetic_replay_mutations
                    ),
                    format!(
                        "projection mismatches: {}; harnesses/devices/voters: {}/{}/{}",
                        gate.unexplained_projection_mismatches,
                        gate.harnesses.len(),
                        gate.devices.len(),
                        gate.max_voters
                    ),
                    if gate.failures.is_empty() {
                        "no outstanding gate failures".into()
                    } else {
                        gate.failures.join("; ")
                    },
                ],
                optional: false,
                actions: vec![],
            }
        }
        Err(error) => CheckResult {
            id: "control.kbd-rollout".into(),
            group: "control".into(),
            label: "KBD production rollout gate".into(),
            severity: Severity::Red,
            status: CheckStatus::Fail,
            summary: "rollout evidence is unreadable".into(),
            details: vec![error.to_string()],
            optional: false,
            actions: vec![],
        },
    }
}

fn read_json_for_doctor(
    path: &Path,
    label: &str,
    failures: &mut Vec<String>,
) -> Option<serde_json::Value> {
    match fs::read(path) {
        Ok(bytes) => match serde_json::from_slice(&bytes) {
            Ok(value) => Some(value),
            Err(error) => {
                failures.push(format!("{label} is invalid JSON: {error}"));
                None
            }
        },
        Err(error) => {
            failures.push(format!(
                "{label} is unreadable at {}: {error}",
                path.display()
            ));
            None
        }
    }
}

fn collect_hook_commands(value: &serde_json::Value, commands: &mut Vec<String>) {
    match value {
        serde_json::Value::Object(object) => {
            for (key, child) in object {
                if key == "command" {
                    if let Some(command) = child.as_str() {
                        commands.push(command.to_string());
                    }
                }
                collect_hook_commands(child, commands);
            }
        }
        serde_json::Value::Array(array) => {
            for child in array {
                collect_hook_commands(child, commands);
            }
        }
        _ => {}
    }
}

fn validate_codex_hook_graph(
    root: &Path,
    plugin_manifest: &Path,
    expected_bundle: &str,
    expected_hook_count: usize,
    expected_hooks_sha: &str,
    label: &str,
    failures: &mut Vec<String>,
) -> Option<String> {
    let plugin = read_json_for_doctor(
        plugin_manifest,
        &format!("{label} plugin manifest"),
        failures,
    )?;
    let version = plugin["version"].as_str().unwrap_or_default().to_string();
    let hooks_reference = match plugin["hooks"].as_str() {
        Some("./hooks/codex-hooks.json") => "hooks/codex-hooks.json",
        Some(other) => {
            failures.push(format!(
                "{label} selects {other}, not ./hooks/codex-hooks.json"
            ));
            return Some(version);
        }
        None => {
            failures.push(format!("{label} has no hooks manifest selection"));
            return Some(version);
        }
    };
    let hooks_path = root.join(hooks_reference);
    let hooks = match read_json_for_doctor(
        &hooks_path,
        &format!("{label} selected hooks manifest"),
        failures,
    ) {
        Some(hooks) => hooks,
        None => return Some(version),
    };
    let actual_sha = hash_path(&hooks_path).unwrap_or_default();
    if actual_sha != expected_hooks_sha {
        failures.push(format!(
            "{label} selected hooks differ from the generated source manifest"
        ));
    }
    let mut commands = Vec::new();
    collect_hook_commands(&hooks, &mut commands);
    if commands.len() != expected_hook_count {
        failures.push(format!(
            "{label} exposes {} hook commands; expected {expected_hook_count}",
            commands.len()
        ));
    }
    for (index, command) in commands.iter().enumerate() {
        let invalid = !command.contains("runtime/v1/run-hook")
            || !command.contains("bootstrap-hook-runtime.sh")
            || !command.contains("--bundle")
            || !command.contains(expected_bundle)
            || !command.contains("--harness")
            || !command.contains("'codex'")
            || command.contains("/stable/")
            || command.contains("/current/");
        if invalid {
            failures.push(format!(
                "{label} hook command {} is not pinned to bundle {expected_bundle}",
                index + 1
            ));
        }
    }
    Some(version)
}

fn check_harness_adapter_parity() -> CheckResult {
    let mut failures = Vec::new();
    let source_root = Path::new(".");
    let release_path = source_root.join("shared/harnesses/generated/release-manifest.json");
    let contract_path = source_root.join("shared/harnesses/hook-contract.json");
    let release = read_json_for_doctor(&release_path, "release manifest", &mut failures);
    let contract = read_json_for_doctor(&contract_path, "hook contract", &mut failures);
    let bundle = release
        .as_ref()
        .and_then(|value| value["bundleId"].as_str())
        .unwrap_or_default()
        .to_string();
    if bundle.len() != 64
        || !bundle
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        failures.push("release manifest has no valid bundle id".into());
    }
    if release
        .as_ref()
        .and_then(|value| value["dispatcherAbi"].as_str())
        != Some("hook-runtime-v1")
    {
        failures.push("release manifest does not select hook-runtime-v1".into());
    }
    let expected_hook_count = contract
        .as_ref()
        .and_then(|value| value["events"].as_array())
        .map(|events| {
            events
                .iter()
                .filter_map(|event| event["hooks"].as_array())
                .map(Vec::len)
                .sum()
        })
        .unwrap_or_default();
    if expected_hook_count == 0 {
        failures.push("hook contract contains no hooks".into());
    }
    let source_hooks = source_root.join("hooks/codex-hooks.json");
    let expected_hooks_sha = hash_path(&source_hooks).unwrap_or_default();
    if expected_hooks_sha.is_empty() {
        failures.push(format!("{} is unreadable", source_hooks.display()));
    }
    let source_version = validate_codex_hook_graph(
        source_root,
        &source_root.join(".codex-plugin/plugin.json"),
        &bundle,
        expected_hook_count,
        &expected_hooks_sha,
        "source",
        &mut failures,
    )
    .unwrap_or_default();

    if let Some(home) = dirs::home_dir() {
        let plugin_root = home.join(".prometheus/plugins/prometheus-skill-pack");
        let generations = fs::canonicalize(plugin_root.join("generations"));
        let active = fs::canonicalize(plugin_root.join("current"));
        match (generations, active) {
            (Ok(generations), Ok(active)) if active.starts_with(&generations) => {
                validate_codex_hook_graph(
                    &active,
                    &active.join(".codex-plugin/plugin.json"),
                    &bundle,
                    expected_hook_count,
                    &expected_hooks_sha,
                    "active immutable generation",
                    &mut failures,
                );
                let generation_manifest = read_json_for_doctor(
                    &active.join("manifest.json"),
                    "active generation receipt",
                    &mut failures,
                );
                let installed_bundle = generation_manifest
                    .as_ref()
                    .and_then(|value| value["bundleId"].as_str())
                    .unwrap_or_default();
                if installed_bundle != bundle {
                    failures.push(format!(
                        "active immutable generation is bundle {installed_bundle}, not {bundle}"
                    ));
                }
                let runner = plugin_root.join("runtime/v1/run-hook");
                let expected_runner_sha = generation_manifest
                    .as_ref()
                    .and_then(|value| value["hookRuntime"]["runnerSha256"].as_str())
                    .unwrap_or_default();
                if hash_path(&runner).as_deref() != Some(expected_runner_sha) {
                    failures.push("fixed hook runtime differs from its generation receipt".into());
                }
                match fs::canonicalize(plugin_root.join("bundles").join(&bundle)) {
                    Ok(indexed) if indexed == active => {}
                    Ok(indexed) => failures.push(format!(
                        "bundle index resolves to {}, not the active generation",
                        indexed.display()
                    )),
                    Err(error) => {
                        failures.push(format!("bundle index {bundle} is not resolvable: {error}"))
                    }
                }
            }
            (Ok(_), Ok(active)) => failures.push(format!(
                "active generation escapes the immutable store: {}",
                active.display()
            )),
            (_, Err(error)) => failures.push(format!("active generation is unavailable: {error}")),
            (Err(error), _) => failures.push(format!("generation store is unavailable: {error}")),
        }

        if source_version.is_empty() {
            failures.push("source plugin version is missing".into());
        } else {
            let cache_root = home
                .join(".codex/plugins/cache/prometheus-skill-pack/prometheus-skill-pack")
                .join(&source_version);
            validate_codex_hook_graph(
                &cache_root,
                &cache_root.join(".codex-plugin/plugin.json"),
                &bundle,
                expected_hook_count,
                &expected_hooks_sha,
                "Codex native cache",
                &mut failures,
            );
        }
    } else {
        failures.push("home directory cannot be resolved".into());
    }

    CheckResult {
        id: "hooks.harness-adapters".into(),
        group: "hooks".into(),
        label: "Installed Codex hook graph".into(),
        severity: if failures.is_empty() {
            Severity::Green
        } else {
            Severity::Red
        },
        status: if failures.is_empty() {
            CheckStatus::Pass
        } else {
            CheckStatus::Fail
        },
        summary: if failures.is_empty() {
            format!(
                "source, immutable generation, bundle index, and Codex cache agree on {expected_hook_count} pinned hooks"
            )
        } else {
            format!("{} installed hook graph defect(s)", failures.len())
        },
        details: if failures.is_empty() {
            vec![
                format!("bundle: {bundle}"),
                "No selected command resolves business logic through stable or current.".into(),
            ]
        } else {
            failures
        },
        optional: false,
        actions: vec![],
    }
}

fn check_instruction_budgets() -> CheckResult {
    let inventory = super::skill::inventory(Path::new("skills"));
    let baseline_path = Path::new("evals/skill-activation/harness-budgets.json");
    let baseline = fs::read(baseline_path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok());
    let mut failures = Vec::new();
    let (skills, characters) = match inventory {
        Ok(inventory) => inventory,
        Err(error) => {
            failures.push(error.to_string());
            (0, 0)
        }
    };
    let mut measured = 0;
    for harness in ["claude-code", "codex", "opencode", "kimi"] {
        let entry = baseline
            .as_ref()
            .and_then(|value| value["harnesses"].get(harness));
        match entry {
            Some(entry)
                if entry["measured"] == true
                    && entry["budgetChars"].as_u64().is_some()
                    && entry["sourceTrace"].as_str().is_some() =>
            {
                measured += 1;
                if entry["inventoryChars"].as_u64() != Some(characters as u64) {
                    failures.push(format!(
                        "{harness} baseline was measured against a different inventory"
                    ));
                }
            }
            Some(_) => failures.push(format!("{harness} discovery budget is not measured")),
            None => failures.push(format!("{harness} discovery baseline is missing")),
        }
    }
    let healthy = failures.is_empty() && skills == 145;
    CheckResult {
        id: "skills.discovery-budget".into(),
        group: "skills".into(),
        label: "Harness instruction discovery budgets".into(),
        severity: if healthy {
            Severity::Green
        } else {
            Severity::Yellow
        },
        status: if healthy {
            CheckStatus::Pass
        } else {
            CheckStatus::Warn
        },
        summary: format!(
            "{skills} skills, {characters} discovery characters, {measured}/4 measured harnesses"
        ),
        details: if failures.is_empty() {
            vec!["Every budget is trace-backed and matches the current inventory.".into()]
        } else {
            failures
        },
        optional: false,
        actions: vec![],
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
            details: vec![
                "Evolution state is optional until an evolution cycle is started.".into(),
            ],
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
    use std::os::unix::fs::PermissionsExt;

    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
    let bin_dir = home.join(".local/bin");
    let binaries = [
        (
            "prometheus",
            "tools/prometheus-cli/target/release/prometheus",
        ),
        ("pk", "tools/prometheus-knowledge/target/release/pk"),
        (
            "pk-cherry",
            "tools/prometheus-knowledge/target/release/pk-cherry",
        ),
        (
            "prometheus-learning-worker",
            "tools/prometheus-knowledge/target/release/prometheus-learning-worker",
        ),
        (
            "surreal-memory-server",
            "tools/surreal-memory-server/target/release/surreal-memory-server",
        ),
    ];
    let mut failures = Vec::new();
    let mut details = Vec::new();
    for (name, source) in binaries {
        let installed = bin_dir.join(name);
        let executable = fs::metadata(&installed).ok().is_some_and(|metadata| {
            metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
        });
        let Some(installed_hash) = hash_path(&installed) else {
            failures.push(format!("{name} is missing from {}", bin_dir.display()));
            continue;
        };
        if !executable {
            failures.push(format!("{name} is not executable"));
            continue;
        }
        let signed = !cfg!(target_os = "macos")
            || Command::new("codesign")
                .args(["--verify", "--strict"])
                .arg(&installed)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .is_ok_and(|status| status.success());
        if !signed {
            failures.push(format!("{name} does not have a valid macOS code signature"));
        }
        let source_hash = hash_file(source).unwrap_or_else(|| "not-built".into());
        details.push(format!(
            "{name}: installed sha256 {installed_hash}; source sha256 {source_hash}; signature {}",
            if signed { "valid" } else { "invalid" }
        ));
    }
    let healthy = failures.is_empty();
    CheckResult {
        id: "binaries.manifest".into(),
        group: "binaries".into(),
        label: "Managed binary manifest".into(),
        severity: if healthy {
            Severity::Green
        } else {
            Severity::Red
        },
        status: if healthy {
            CheckStatus::Pass
        } else {
            CheckStatus::Fail
        },
        summary: if healthy {
            "5/5 managed binaries are executable, hashed, and signed".into()
        } else {
            format!("{} managed binary defect(s)", failures.len())
        },
        details: if healthy { details } else { failures },
        optional: false,
        actions: vec![RepairAction {
            id: "binaries.install-binaries".into(),
            description:
                "Rebuild and reinstall the managed binaries from the pinned source checkout.".into(),
            safe: true,
            reversible: true,
            dry_run_only: false,
            command_hint: Some("bash scripts/install-binaries.sh --dry-run".into()),
            reason_blocked: None,
        }],
    }
}

fn check_managed_services() -> CheckResult {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
    let labels = [
        "ai.prometheus.surreal-memory-native",
        "ai.prometheus.pk-cherry",
        "ai.prometheus.learning-worker",
        "ai.prometheus.hooks-logrotate",
    ];
    let mut failures = Vec::new();
    let mut details = Vec::new();
    for label in labels {
        let definition = if cfg!(target_os = "macos") {
            home.join("Library/LaunchAgents")
                .join(format!("{label}.plist"))
        } else {
            home.join(".config/systemd/user")
                .join(format!("{label}.service"))
        };
        if !definition.is_file() {
            failures.push(format!(
                "missing service definition: {}",
                definition.display()
            ));
            continue;
        }
        let loaded = if cfg!(target_os = "macos") {
            let target = format!(
                "gui/{}/{}",
                command_stdout(&["id", "-u"]).unwrap_or_else(|| "0".into()),
                label
            );
            Command::new("launchctl")
                .args(["print", &target])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .is_ok_and(|status| status.success())
        } else {
            Command::new("systemctl")
                .args(["--user", "is-enabled", label])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .is_ok_and(|status| status.success())
        };
        if loaded {
            details.push(format!("{label}: loaded from {}", definition.display()));
        } else {
            failures.push(format!("{label} is installed but not loaded"));
        }
    }
    let healthy = failures.is_empty();
    CheckResult {
        id: "services.launch-agents".into(),
        group: "services".into(),
        label: "Managed LaunchAgents".into(),
        severity: if healthy {
            Severity::Green
        } else {
            Severity::Red
        },
        status: if healthy {
            CheckStatus::Pass
        } else {
            CheckStatus::Fail
        },
        summary: if healthy {
            "4/4 deterministic learning services are installed and loaded".into()
        } else {
            format!("{} managed service defect(s)", failures.len())
        },
        details: if healthy { details } else { failures },
        optional: false,
        actions: vec![RepairAction {
            id: "services.install-mcp-services".into(),
            description:
                "Reload only the managed MCP service definitions owned by this repository.".into(),
            safe: true,
            reversible: true,
            dry_run_only: false,
            command_hint: Some("bash scripts/install-mcp-services.sh --dry-run".into()),
            reason_blocked: None,
        }],
    }
}

fn check_learning_worker() -> CheckResult {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
    let queue = home.join(".prometheus/learning-queue");
    let count = |relative: &str| -> usize {
        fs::read_dir(queue.join(relative))
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("json"))
            .count()
    };
    let pending = count("pending");
    let processing = count("processing");
    let completed = count("completed");
    let rejected = count("rejected");
    let memory_pending = count("memory/pending");
    let memory_submitting = count("memory/submitting");
    let memory_accepted = count("memory/accepted");
    let memory_completed = count("memory/completed");
    let memory_rejected = count("memory/rejected");
    let legacy_retry = count("retry") + count("memory/retry");
    let legacy_dead = count("dead-letter") + count("memory/dead-letter");
    let worker = home.join(".local/bin/prometheus-learning-worker");
    let loaded = if cfg!(target_os = "macos") {
        let target = format!(
            "gui/{}/ai.prometheus.learning-worker",
            command_stdout(&["id", "-u"]).unwrap_or_else(|| "0".into())
        );
        Command::new("launchctl")
            .args(["print", &target])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
    } else {
        Command::new("systemctl")
            .args(["--user", "is-enabled", "ai.prometheus.learning-worker.path"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
    };
    let healthy = worker.is_file()
        && loaded
        && pending == 0
        && processing == 0
        && memory_pending == 0
        && memory_submitting == 0
        && memory_accepted == 0
        && legacy_retry == 0
        && legacy_dead == 0;
    CheckResult {
        id: "learning.worker".into(),
        group: "learning".into(),
        label: "Asynchronous learning worker".into(),
        severity: if healthy { Severity::Green } else { Severity::Red },
        status: if healthy { CheckStatus::Pass } else { CheckStatus::Fail },
        summary: format!(
            "worker {}, service {}, jobs {pending}/{processing}/{completed}/{rejected}, memory {memory_pending}/{memory_submitting}/{memory_accepted}/{memory_completed}/{memory_rejected}, legacy retry/dead {legacy_retry}/{legacy_dead}",
            if worker.is_file() { "installed" } else { "missing" },
            if loaded { "loaded" } else { "unloaded" },
        ),
        details: vec![
            format!("queue: {}", queue.display()),
            "job states: pending/processing/completed/rejected; memory states: pending/submitting/accepted/completed/rejected".into(),
        ],
        optional: false,
        actions: vec![RepairAction {
            id: "services.install-mcp-services".into(),
            description: "Install the learning worker and reload its supervised queue service.".into(),
            safe: true,
            reversible: true,
            dry_run_only: false,
            command_hint: Some("bash scripts/install-mcp-services.sh --restart".into()),
            reason_blocked: None,
        }],
    }
}

fn check_hook_log_rotation() -> CheckResult {
    use std::os::unix::fs::PermissionsExt;
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
    let root = home.join(".prometheus");
    let log = root.join("hooks.log");
    let config = root.join("logrotate/prometheus-hooks.conf");
    let mode = fs::metadata(&log)
        .ok()
        .map(|metadata| metadata.permissions().mode() & 0o777);
    let plist = home.join("Library/LaunchAgents/ai.prometheus.hooks-logrotate.plist");
    let logrotate = configured_rotation_dependency(
        &plist,
        "EnvironmentVariables.PROMETHEUS_LOGROTATE_BIN",
        &[
            "/usr/local/sbin/logrotate",
            "/opt/homebrew/opt/logrotate/sbin/logrotate",
            "/usr/sbin/logrotate",
        ],
    );
    let flock = configured_rotation_dependency(
        &plist,
        "EnvironmentVariables.PROMETHEUS_FLOCK_BIN",
        &[
            "/usr/local/bin/flock",
            "/opt/homebrew/bin/flock",
            "/usr/bin/flock",
        ],
    );
    let logrotate_ready = logrotate.as_deref().is_some_and(is_executable);
    let flock_ready = flock.as_deref().is_some_and(is_executable);
    let loaded = if cfg!(target_os = "macos") {
        let target = format!(
            "gui/{}/ai.prometheus.hooks-logrotate",
            command_stdout(&["id", "-u"]).unwrap_or_else(|| "0".into())
        );
        Command::new("launchctl")
            .args(["print", &target])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
    } else {
        Command::new("systemctl")
            .args([
                "--user",
                "is-enabled",
                "ai.prometheus.hooks-logrotate.timer",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
    };
    let healthy =
        config.is_file() && loaded && mode == Some(0o600) && logrotate_ready && flock_ready;
    CheckResult {
        id: "hooks.rotation".into(),
        group: "hooks".into(),
        label: "Hook log rotation".into(),
        severity: if healthy {
            Severity::Green
        } else {
            Severity::Red
        },
        status: if healthy {
            CheckStatus::Pass
        } else {
            CheckStatus::Fail
        },
        summary: format!(
            "config {}, service {}, dependencies {}, hook-log mode {}",
            if config.is_file() {
                "installed"
            } else {
                "missing"
            },
            if loaded { "loaded" } else { "unloaded" },
            if logrotate_ready && flock_ready {
                "ready"
            } else {
                "missing"
            },
            mode.map(|value| format!("{value:04o}"))
                .unwrap_or_else(|| "missing".into())
        ),
        details: vec![
            dependency_detail("logrotate", logrotate.as_deref(), logrotate_ready),
            dependency_detail("flock", flock.as_deref(), flock_ready),
            "30 daily archives, delayed compression, and writer-lock coordination are required."
                .into(),
        ],
        optional: false,
        actions: vec![RepairAction {
            id: "services.install-mcp-services".into(),
            description: "Render and load the owner-only hook rotation service.".into(),
            safe: true,
            reversible: true,
            dry_run_only: false,
            command_hint: Some("bash scripts/install-mcp-services.sh --restart".into()),
            reason_blocked: None,
        }],
    }
}

fn configured_rotation_dependency(plist: &Path, key: &str, fallbacks: &[&str]) -> Option<PathBuf> {
    if cfg!(target_os = "macos") && plist.is_file() {
        let output = Command::new("plutil")
            .args(["-extract", key, "raw", "-o", "-"])
            .arg(plist)
            .output()
            .ok();
        if let Some(path) = output
            .filter(|output| output.status.success())
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|path| PathBuf::from(path.trim()))
            .filter(|path| !path.as_os_str().is_empty())
        {
            return Some(path);
        }
    }

    fallbacks
        .iter()
        .map(PathBuf::from)
        .find(|path| is_executable(path))
}

fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path)
        .ok()
        .is_some_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}

fn dependency_detail(name: &str, path: Option<&Path>, ready: bool) -> String {
    format!(
        "{name}: {} ({})",
        path.map(|path| path.display().to_string())
            .unwrap_or_else(|| "not found".into()),
        if ready { "executable" } else { "missing" }
    )
}

fn check_prompt_snapshots() -> CheckResult {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
    let project_root = std::env::var_os("PROMETHEUS_PROJECT_ROOT")
        .map(PathBuf::from)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));
    let global = home.join(".prometheus/knowledge");
    let snapshots = [
        ("project", project_root.join(".prometheus/knowledge")),
        ("shared", global.join("shared")),
        ("global", global),
    ];
    let mut failures = Vec::new();
    let mut details = Vec::new();
    for (scope, knowledge_root) in snapshots {
        let snapshot_root = knowledge_root.join(".prompt-snapshots").join(scope);
        let pointer = snapshot_root.join("current");
        let generation = fs::read_to_string(&pointer)
            .ok()
            .map(|value| value.trim().to_owned())
            .filter(|value| {
                value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
            });
        match generation {
            Some(generation)
                if snapshot_root
                    .join("generations")
                    .join(format!("{generation}.json"))
                    .is_file() =>
            {
                details.push(format!("{scope}: committed generation {generation}"));
            }
            _ => failures.push(format!(
                "{scope}: no valid committed snapshot at {}",
                pointer.display()
            )),
        }
    }
    let healthy = failures.is_empty();
    CheckResult {
        id: "learning.snapshots".into(),
        group: "learning".into(),
        label: "Immutable prompt snapshots".into(),
        severity: if healthy {
            Severity::Green
        } else {
            Severity::Red
        },
        status: if healthy {
            CheckStatus::Pass
        } else {
            CheckStatus::Fail
        },
        summary: if healthy {
            "project/shared/global committed snapshot pointers are valid".into()
        } else {
            format!("{} prompt snapshot defect(s)", failures.len())
        },
        details: if healthy { details } else { failures },
        optional: false,
        actions: vec![RepairAction {
            id: "manual.publish-prompt-snapshots".into(),
            description: "Publish project, shared, and global snapshots with `pk snapshot`.".into(),
            safe: false,
            reversible: true,
            dry_run_only: false,
            command_hint: Some(
                "pk snapshot --scope project --scope shared; pk snapshot --scope global".into(),
            ),
            reason_blocked: Some(
                "Snapshot publication requires an explicit knowledge-root choice.".into(),
            ),
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
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
    let plugin_root = home.join(".prometheus/plugins/prometheus-skill-pack");
    let output = Command::new("node")
        .args([
            "scripts/install-plugin-generation.js",
            "--verify",
            "--plugin-root",
        ])
        .arg(&plugin_root)
        .arg("--home")
        .arg(&home)
        .output();
    let generation = output
        .as_ref()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout.clone()).ok())
        .map(|value| value.trim().to_owned())
        .filter(|value| value.len() == 64);
    let healthy = generation.is_some();
    CheckResult {
        id: "hooks.lifecycle".into(),
        group: "hooks".into(),
        label: "Lifecycle hooks".into(),
        severity: if healthy { Severity::Green } else { Severity::Red },
        status: if healthy { CheckStatus::Pass } else { CheckStatus::Fail },
        summary: generation
            .as_ref()
            .map(|generation| format!("active generation {generation} passed manifest, dispatcher, and 14-target verification"))
            .unwrap_or_else(|| "active plugin generation verification failed".into()),
        details: if healthy {
            vec![format!("plugin root: {}", plugin_root.display())]
        } else {
            vec![output
                .ok()
                .and_then(|output| String::from_utf8(output.stderr).ok())
                .filter(|stderr| !stderr.trim().is_empty())
                .unwrap_or_else(|| "plugin verifier did not return a certified generation".into())]
        },
        optional: false,
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
