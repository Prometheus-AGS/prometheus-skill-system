//! `--daemon-job <job_id>`: the process that actually runs a research job.
//!
//! `spawn::spawn_job` writes the job checkpoint and re-executes this binary
//! with `--daemon-job`. This module is that child. It resolves a headless
//! harness from `PATH` (`claude` preferred, `codex` fallback), spawns it with a
//! prompt that runs the deep-research stage-contract driver for the job's
//! query, tails the package `checkpoint.json` the driver writes, mirrors stage
//! and progress into the job checkpoint, and forwards `agent.status` /
//! `agent.error` events to the HTTP server's event sink so they reach the SSE
//! stream. Child exit is mapped to `complete`, `blocked`, or `failed` from the
//! package state, never from the harness exit code alone.
//!
//! Hook policy (references/headless-execution.md): the daemon fires no hooks.
//! The harness runs the real driver, which fires the four deep-research hook
//! scripts exactly as in the foreground. `KBD_HOOKS_DISABLED=1` is set in the
//! harness environment and stated in the prompt so the session fires no KBD
//! lifecycle hook.
//!
//! Test seams (all environment variables, all optional in production):
//! `RESEARCH_DRIVER` (driver path), `RESEARCH_EVENT_SINK` (server base URL,
//! set by `run_server`), `RESEARCH_HARNESS_ARGS` (extra harness flags).

use crate::agui::AguiEvent;
use crate::job::checkpoint::{self, now_rfc3339, JobCheckpoint};
use anyhow::Context as _;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

/// Stage names in driver order; index 0 is stage 01.
pub const STAGE_NAMES: [&str; 10] = [
    "planner", "search", "retrieve", "collect", "verify", "resolve", "graph", "cite", "report",
    "export",
];

pub fn stage_name(stage: u32) -> &'static str {
    STAGE_NAMES
        .get((stage as usize).wrapping_sub(1))
        .copied()
        .unwrap_or("unknown")
}

const POLL_INTERVAL: Duration = Duration::from_millis(500);

// ---------------------------------------------------------------------------
// Harness resolution
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Harness {
    pub name: &'static str,
    pub path: PathBuf,
}

/// The first `claude` on `PATH`, else the first `codex`. An empty or unset
/// `PATH` resolves nothing; the caller records the job as blocked.
pub fn resolve_harness() -> Option<Harness> {
    let path = std::env::var_os("PATH")?;
    for name in ["claude", "codex"] {
        for dir in std::env::split_paths(&path) {
            if dir.as_os_str().is_empty() {
                continue;
            }
            let candidate = dir.join(name);
            if is_executable(&candidate) {
                return Some(Harness { name, path: candidate });
            }
        }
    }
    None
}

fn is_executable(path: &Path) -> bool {
    let Ok(meta) = std::fs::metadata(path) else {
        return false;
    };
    if !meta.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        meta.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

/// The stage-contract driver the prompt tells the harness to run.
/// `RESEARCH_DRIVER`, then the plugin root, then the flat install, then the
/// source tree this binary was built from.
pub fn resolve_driver() -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(p) = std::env::var_os("RESEARCH_DRIVER") {
        if !p.is_empty() {
            candidates.push(PathBuf::from(p));
        }
    }
    if let Some(root) = std::env::var_os("CLAUDE_PLUGIN_ROOT") {
        if !root.is_empty() {
            candidates.push(
                PathBuf::from(root).join("skills/research/deep-research/scripts/run-research.sh"),
            );
        }
    }
    if let Some(home) = dirs_next::home_dir() {
        candidates.push(home.join(".claude/skills/deep-research/scripts/run-research.sh"));
    }
    candidates.push(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../skills/research/deep-research/scripts/run-research.sh"),
    );
    candidates.into_iter().find(|p| p.is_file())
}

/// What the daemon would actually run, resolved now.
///
/// Defect D-B (found by the change-rah-011 evidence run) was invisible until a
/// job had already failed: `resolve_driver()` fell through to an installed
/// plugin generation whose `run-research.sh` was a 1920-byte stub with none of
/// the stage contract, so the harness exited 0 having produced no package. The
/// daemon refused to call that success — correctly — but only *after* spending a
/// harness run to find out.
///
/// This reports the same resolution up front, so a stale install is visible
/// before a job is started rather than after one fails.
pub fn self_check() -> serde_json::Value {
    use serde_json::json;

    let harness = resolve_harness();
    let driver = resolve_driver();

    let driver_json = match driver.as_deref() {
        None => json!({ "status": "missing", "detail": "no run-research.sh found on any candidate path" }),
        Some(path) => {
            let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
            let body = std::fs::read_to_string(path).unwrap_or_default();
            // A driver that implements the stage contract mentions all four.
            // The stub scores 0 on every one; the real driver scores >0 on each.
            // Counting markers beats comparing byte size: size drifts with every
            // edit, but a driver either speaks the contract or it does not.
            let markers = ["--resume", "checkpoint", "next_stage", "RESEARCH_STAGE_RUNNER"];
            let missing: Vec<&str> = markers.iter().copied().filter(|m| !body.contains(m)).collect();
            let stale = !missing.is_empty();
            json!({
                "status": if stale { "stale" } else { "ok" },
                "path": path.display().to_string(),
                "size_bytes": size,
                "missing_stage_contract_markers": missing,
                "detail": if stale {
                    "this driver does not implement the stage contract; a job would exit 0 without a package (defect D-B). Set RESEARCH_DRIVER, or reinstall so the published generation matches the repo."
                } else { "implements the stage contract" }
            })
        }
    };

    json!({
        "harness": match harness {
            Some(h) => json!({
                "status": "ok",
                "name": h.name,
                "path": h.path.display().to_string(),
                // The requirement asks for both paths AND sizes; a harness that
                // resolves to a 0-byte shim is as broken as one that is missing.
                "size_bytes": std::fs::metadata(&h.path).map(|m| m.len()).unwrap_or(0),
            }),
            None => json!({
                "status": "missing",
                "detail": "no harness on PATH (looked for claude, then codex). Under launchd this means the plist grants no PATH (defect D-A)."
            }),
        },
        "driver": driver_json,
    })
}

// ---------------------------------------------------------------------------
// Prompt
// ---------------------------------------------------------------------------

fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// The driver accepts `shallow | deep | exhaustive`; the REST and MCP entry
/// points default to `moderate` and callers write what they like. Map the
/// job's value onto the driver's vocabulary. Anything else is `None`: the
/// value ends up on a command line the harness executes, so an unknown value
/// blocks the job rather than travelling anywhere near a shell.
pub fn canonical_depth(depth: &str) -> Option<&'static str> {
    match depth.trim().to_ascii_lowercase().as_str() {
        "shallow" | "quick" | "light" => Some("shallow"),
        "deep" | "moderate" | "medium" | "standard" | "default" => Some("deep"),
        "exhaustive" | "thorough" | "full" => Some("exhaustive"),
        _ => None,
    }
}

/// `research-manifest.schema.json` enumerates `APA | MLA | Chicago | IEEE |
/// Vancouver`; every entry point defaults to lowercase `apa`. Same rule as
/// `canonical_depth`: known names only.
pub fn canonical_citation_style(style: &str) -> Option<&'static str> {
    match style.trim().to_ascii_lowercase().as_str() {
        "apa" => Some("APA"),
        "mla" => Some("MLA"),
        "chicago" => Some("Chicago"),
        "ieee" => Some("IEEE"),
        "vancouver" => Some("Vancouver"),
        _ => None,
    }
}

/// The values that reach the driver's command line, all canonical.
#[derive(Debug, Clone)]
pub struct RunParams {
    pub depth: &'static str,
    pub citation_style: &'static str,
}

/// Validate the job's caller-supplied fields before any of them is placed in
/// the prompt. The error text is the blocked reason.
pub fn run_params(cp: &JobCheckpoint) -> Result<RunParams, String> {
    let depth = canonical_depth(&cp.depth).ok_or_else(|| {
        format!(
            "depth {:?} is not one of shallow, deep, exhaustive (moderate is accepted as deep)",
            cp.depth
        )
    })?;
    let citation_style = canonical_citation_style(&cp.citation_style).ok_or_else(|| {
        format!(
            "citation_style {:?} is not one of APA, MLA, Chicago, IEEE, Vancouver",
            cp.citation_style
        )
    })?;
    if cp.query.trim().len() < 5 {
        return Err("query is too short (min 5 chars)".to_string());
    }
    Ok(RunParams { depth, citation_style })
}

/// The prompt the harness receives. It is written to `<job_dir>/prompt.md` so
/// an operator can read exactly what the session was asked to do.
pub fn build_prompt(
    cp: &JobCheckpoint,
    params: &RunParams,
    driver: Option<&Path>,
    output_root: &Path,
) -> String {
    let depth = params.depth;
    let citation_style = params.citation_style;
    // Every interpolated value is shell-quoted, the canonical ones included:
    // the line is executed by a session with permissions bypassed, so the
    // quoting is not allowed to depend on which values happen to be safe today.
    let driver_line = match driver {
        Some(d) => format!(
            "    bash {} --query {} --depth {} --citation-style {} --job-id {} --output-root {}",
            shell_quote(&d.display().to_string()),
            shell_quote(&cp.query),
            shell_quote(depth),
            shell_quote(citation_style),
            shell_quote(&cp.job_id),
            shell_quote(&output_root.display().to_string())
        ),
        None => "    (run-research.sh was not found on this machine; use the /deep-research skill's scripts/run-research.sh with the same arguments)".to_string(),
    };
    format!(
        "You are running an unattended deep-research job for prometheus-research. \
Run /deep-research for the query below through the stage-contract driver, headless, \
asking the user nothing.\n\n\
- query: {query}\n- depth: {depth}\n- citation style: {cs}\n- max sources: {ms}\n\
- output root: {root}\n- job id: {job}\n\n\
Invoke the driver:\n\n{driver_line}\n\n\
The driver runs in checkpoint mode: it validates the package, prints one JSON line \
{{\"next_stage\": \"NN\", \"skill\": \"stage-NN-<name>\", \"package_dir\": \"...\"}} and exits 3 \
when a stage is needed. Run that stage skill against the package directory, then re-invoke \
the driver with --resume <package_dir>. Repeat until the driver exits 0 (complete) or 1 \
(blocked; stop and report the sidecar's Blocked line). Do not run stages outside the driver, \
do not edit checkpoint.json, and do not fire KBD lifecycle hooks: KBD_HOOKS_DISABLED=1 is set \
for this session. The deep-research hooks fire from the driver on their own. When finished, \
print the package directory and exit.\n",
        query = cp.query,
        depth = depth,
        cs = citation_style,
        ms = cp.max_sources,
        root = output_root.display(),
        job = cp.job_id,
    )
}

// ---------------------------------------------------------------------------
// Spawn
// ---------------------------------------------------------------------------

fn spawn_harness(
    harness: &Harness,
    prompt: &str,
    cp: &JobCheckpoint,
    params: &RunParams,
    job_dir: &Path,
    output_root: &Path,
) -> anyhow::Result<Child> {
    let log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(job_dir.join("harness.log"))
        .with_context(|| format!("cannot open harness.log in {}", job_dir.display()))?;
    let log_err = log.try_clone()?;
    let mut cmd = Command::new(&harness.path);
    match harness.name {
        "claude" => {
            cmd.arg("-p")
                .arg(prompt)
                .arg("--permission-mode")
                .arg("bypassPermissions");
        }
        _ => {
            cmd.arg("exec")
                .arg("--dangerously-bypass-hook-trust")
                .arg("--full-auto")
                .arg(prompt);
        }
    }
    if let Ok(extra) = std::env::var("RESEARCH_HARNESS_ARGS") {
        for a in extra.split_whitespace() {
            cmd.arg(a);
        }
    }
    cmd.current_dir(job_dir)
        .env("RESEARCH_OUTPUT_DIR", output_root)
        .env("RESEARCH_JOB_ID", &cp.job_id)
        .env("RESEARCH_QUERY", &cp.query)
        .env("RESEARCH_DEPTH", params.depth)
        .env("KBD_HOOKS_DISABLED", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::from(log))
        .stderr(Stdio::from(log_err));
    cmd.spawn()
        .with_context(|| format!("failed to spawn {} ({})", harness.name, harness.path.display()))
}

// ---------------------------------------------------------------------------
// Package checkpoint mirroring
// ---------------------------------------------------------------------------

/// What the driver's `checkpoint.json` tells the daemon.
#[derive(Debug, Clone, Default, PartialEq)]
struct PackageState {
    status: String,
    current_stage: Option<u32>,
    planned: Vec<u32>,
    completed: Vec<u32>,
    blocked: Option<String>,
}

fn parse_stage(s: &str) -> Option<u32> {
    s.trim().parse::<u32>().ok()
}

fn read_package_state(package_dir: &Path) -> Option<PackageState> {
    let raw = std::fs::read_to_string(package_dir.join("checkpoint.json")).ok()?;
    let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let stages = |key: &str| -> Vec<u32> {
        v.get(key)
            .and_then(|a| a.as_array())
            .map(|a| a.iter().filter_map(|s| s.as_str().and_then(parse_stage)).collect())
            .unwrap_or_default()
    };
    Some(PackageState {
        status: v.get("status").and_then(|s| s.as_str()).unwrap_or("").to_string(),
        current_stage: v.get("current_stage").and_then(|s| s.as_str()).and_then(parse_stage),
        planned: stages("stages_planned"),
        completed: stages("stages_completed"),
        blocked: v.get("blocked").and_then(|b| {
            let stage = b.get("stage").and_then(|s| s.as_str())?;
            let reason = b.get("reason").and_then(|s| s.as_str())?;
            Some(format!("stage {stage}: {reason}"))
        }),
    })
}

/// The driver names the package after the query, not the job, so the daemon
/// finds it by the `job_id` the driver records in the package checkpoint. The
/// output root is shared across runs, so a candidate must also have been
/// created no earlier than this job started: a stale package that happens to
/// carry the same job id is never adopted.
fn find_package_dir(output_root: &Path, job_id: &str, started_at: &str) -> Option<PathBuf> {
    let started = chrono::DateTime::parse_from_rfc3339(started_at).ok();
    let entries = std::fs::read_dir(output_root).ok()?;
    for entry in entries.flatten() {
        let dir = entry.path();
        if !dir.is_dir() || dir.file_name().and_then(|n| n.to_str()) == Some(job_id) {
            continue;
        }
        let cp = dir.join("checkpoint.json");
        let Ok(raw) = std::fs::read_to_string(&cp) else {
            continue;
        };
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) else {
            continue;
        };
        let is_package = v.get("stages_planned").is_some() || v.get("package_id").is_some();
        if !is_package || v.get("job_id").and_then(|j| j.as_str()) != Some(job_id) {
            continue;
        }
        let created = v
            .get("created_at")
            .and_then(|c| c.as_str())
            .and_then(|c| chrono::DateTime::parse_from_rfc3339(c).ok());
        match (started, created) {
            (Some(s), Some(c)) if c < s => {
                tracing::warn!(
                    "ignoring {} : carries job id {job_id} but was created at {c}, before this job started at {s}",
                    dir.display()
                );
                continue;
            }
            (Some(_), None) => {
                tracing::warn!("ignoring {}: no parseable created_at", dir.display());
                continue;
            }
            _ => return Some(dir),
        }
    }
    None
}

fn progress_of(state: &PackageState) -> u32 {
    if state.planned.is_empty() {
        return 0;
    }
    ((state.completed.len() as u64 * 100) / state.planned.len() as u64).min(100) as u32
}

// ---------------------------------------------------------------------------
// Event sink
// ---------------------------------------------------------------------------

struct Sink {
    base: Option<String>,
    token: Option<String>,
    client: reqwest::Client,
    warned: bool,
}

/// Header carrying the per-server ingest token (`RESEARCH_EVENT_TOKEN`).
pub const EVENT_TOKEN_HEADER: &str = "x-research-event-token";

impl Sink {
    fn from_env() -> Self {
        Self {
            base: std::env::var("RESEARCH_EVENT_SINK")
                .ok()
                .filter(|s| !s.is_empty()),
            token: std::env::var("RESEARCH_EVENT_TOKEN")
                .ok()
                .filter(|s| !s.is_empty()),
            // Loopback only: an ambient HTTP(S)_PROXY must never intercept
            // events bound for 127.0.0.1 (the same lesson as the doctor's
            // surreal-memory probe).
            client: reqwest::Client::builder()
                .no_proxy()
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            warned: false,
        }
    }

    async fn send(&mut self, job_id: &str, event: &AguiEvent) {
        let Some(base) = &self.base else {
            return;
        };
        let url = format!("{}/api/v1/jobs/{job_id}/events", base.trim_end_matches('/'));
        let mut req = self.client.post(&url).json(event);
        if let Some(token) = &self.token {
            req = req.header(EVENT_TOKEN_HEADER, token);
        }
        match req.send().await {
            Ok(resp) if resp.status().is_success() => {}
            Ok(resp) => {
                if !self.warned {
                    tracing::warn!("event sink {url} answered {}", resp.status());
                    self.warned = true;
                }
            }
            Err(e) => {
                if !self.warned {
                    tracing::warn!("event sink {url} unreachable: {e}");
                    self.warned = true;
                }
            }
        }
    }

    async fn status(&mut self, cp: &JobCheckpoint) {
        let event = AguiEvent::AgentStatus {
            job_id: cp.job_id.clone(),
            stage: cp.stage,
            stage_name: cp.stage_name.clone(),
            progress: cp.progress,
            status: cp.status.clone(),
            tokens: cp.tokens_used,
            timestamp: now_rfc3339(),
        };
        self.send(&cp.job_id, &event).await;
    }

    async fn error(&mut self, cp: &JobCheckpoint, message: &str) {
        let event = AguiEvent::AgentError {
            job_id: cp.job_id.clone(),
            message: message.to_string(),
            stage: cp.stage,
            timestamp: now_rfc3339(),
        };
        self.send(&cp.job_id, &event).await;
    }
}

// ---------------------------------------------------------------------------
// The job
// ---------------------------------------------------------------------------

/// Write the job checkpoint unless an operator has cancelled the job under
/// us. `research_cancel` / `DELETE /api/v1/jobs/{id}` rewrite the file with
/// `cancelled`; the daemon re-reads immediately before every write and never
/// overwrites that status, so the window in which a cancel could be lost is
/// the read-write pair here, not the seconds a poll interval or an HTTP
/// round-trip takes. Returns `Ok(false)` when the job is cancelled on disk;
/// callers stop the harness and return.
fn save(cp: &mut JobCheckpoint) -> anyhow::Result<bool> {
    if let Ok(on_disk) = checkpoint::read(&cp.job_id) {
        if on_disk.status == "cancelled" {
            // The operator's record wins. Merge in only what the daemon alone
            // knows (pids, harness, package, exit code) and leave every other
            // field as the operator's cancel path left it.
            let mut merged = on_disk;
            merged.pid = cp.pid.or(merged.pid);
            merged.harness = cp.harness.clone().or(merged.harness);
            merged.harness_pid = cp.harness_pid.or(merged.harness_pid);
            merged.package_dir = cp.package_dir.clone().or(merged.package_dir);
            merged.package_id = cp.package_id.clone().or(merged.package_id);
            merged.exit_code = cp.exit_code.or(merged.exit_code);
            merged.error = merged
                .error
                .or_else(|| Some("cancelled by operator".to_string()));
            merged.last_updated_at = now_rfc3339();
            checkpoint::write(&merged)?;
            *cp = merged;
            return Ok(false);
        }
    }
    cp.last_updated_at = now_rfc3339();
    checkpoint::write(cp)?;
    Ok(true)
}

async fn finish_blocked(
    cp: &mut JobCheckpoint,
    sink: &mut Sink,
    reason: &str,
) -> anyhow::Result<()> {
    cp.status = "blocked".to_string();
    cp.error = Some(reason.to_string());
    if !save(cp)? {
        tracing::info!("job {} cancelled before it could be marked blocked", cp.job_id);
        return Ok(());
    }
    tracing::warn!("job {} blocked: {reason}", cp.job_id);
    sink.error(cp, reason).await;
    sink.status(cp).await;
    Ok(())
}

async fn finish_failed(cp: &mut JobCheckpoint, sink: &mut Sink, reason: String) -> anyhow::Result<()> {
    cp.status = "failed".to_string();
    cp.error = Some(reason.clone());
    if !save(cp)? {
        return Ok(());
    }
    tracing::warn!("job {} failed: {reason}", cp.job_id);
    sink.error(cp, &reason).await;
    sink.status(cp).await;
    Ok(())
}

fn stop_harness(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

/// Entry point for `--daemon-job <job_id>`. Returns `Ok(())` for every job
/// outcome, including blocked and failed: the outcome lives in the checkpoint,
/// and a non-zero exit here would only be a second, less informative record.
pub async fn run(job_id: &str) -> anyhow::Result<()> {
    let mut cp = checkpoint::read(job_id)?;
    if cp.status == "cancelled" {
        tracing::info!("job {job_id} was cancelled before the daemon started; nothing to run");
        return Ok(());
    }
    let output_root = checkpoint::output_root();
    let job_dir = checkpoint::job_dir(job_id);
    std::fs::create_dir_all(&job_dir)?;
    let mut sink = Sink::from_env();
    // The daemon is the process the job runs in; it records its own pid so
    // `research_cancel` can signal it. `spawn_job` writes nothing after the
    // spawn succeeds, so this record is never raced by the parent.
    cp.pid = Some(std::process::id());

    // Caller-supplied values are validated before any of them can reach the
    // prompt: the driver line is executed by a session with permissions bypassed.
    let params = match run_params(&cp) {
        Ok(p) => p,
        Err(reason) => return finish_blocked(&mut cp, &mut sink, &reason).await,
    };

    let Some(harness) = resolve_harness() else {
        return finish_blocked(
            &mut cp,
            &mut sink,
            "no harness binary on PATH (looked for claude, then codex); install one or set PATH for the daemon",
        )
        .await;
    };

    let driver = resolve_driver();
    let prompt = build_prompt(&cp, &params, driver.as_deref(), &output_root);
    std::fs::write(job_dir.join("prompt.md"), &prompt)?;
    if driver.is_none() {
        tracing::warn!("run-research.sh not found; the prompt names the skill without a path");
    }

    let mut child = match spawn_harness(&harness, &prompt, &cp, &params, &job_dir, &output_root) {
        Ok(c) => c,
        Err(e) => {
            cp.harness = Some(harness.name.to_string());
            return finish_failed(&mut cp, &mut sink, format!("harness spawn failed: {e:#}")).await;
        }
    };

    cp.status = "running".to_string();
    cp.harness = Some(harness.name.to_string());
    cp.harness_pid = Some(child.id());
    cp.stage = 0;
    cp.stage_name = "starting".to_string();
    cp.progress = 0;
    if !save(&mut cp)? {
        stop_harness(&mut child);
        tracing::info!("job {job_id} cancelled at start; harness killed");
        return Ok(());
    }
    sink.status(&cp).await;
    tracing::info!(
        "job {job_id}: {} (pid {}) running the driver",
        harness.name,
        child.id()
    );

    #[cfg(unix)]
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;

    let mut package_dir: Option<PathBuf> = None;
    let mut last_state = PackageState::default();
    let exit_status = loop {
        #[cfg(unix)]
        tokio::select! {
            _ = tokio::time::sleep(POLL_INTERVAL) => {}
            _ = sigterm.recv() => {
                stop_harness(&mut child);
                cp.status = "cancelled".to_string();
                cp.error = Some("daemon received SIGTERM; harness killed".to_string());
                cp.last_updated_at = now_rfc3339();
                checkpoint::write(&cp)?;
                sink.status(&cp).await;
                return Ok(());
            }
        }
        #[cfg(not(unix))]
        tokio::time::sleep(POLL_INTERVAL).await;

        // An operator cancel rewrites the job checkpoint under us; `save` also
        // re-checks immediately before every write.
        if let Ok(current) = checkpoint::read(job_id) {
            if current.status == "cancelled" {
                stop_harness(&mut child);
                tracing::info!("job {job_id} cancelled; harness killed");
                return Ok(());
            }
        }

        if package_dir.is_none() {
            package_dir = find_package_dir(&output_root, job_id, &cp.started_at);
            if let Some(dir) = &package_dir {
                cp.package_dir = Some(dir.display().to_string());
                cp.package_id = dir.file_name().map(|n| n.to_string_lossy().to_string());
                if !save(&mut cp)? {
                    stop_harness(&mut child);
                    return Ok(());
                }
            }
        }
        if let Some(state) = package_dir.as_deref().and_then(read_package_state) {
            if state != last_state {
                if !mirror(&mut cp, &mut sink, &last_state, &state).await? {
                    stop_harness(&mut child);
                    tracing::info!("job {job_id} cancelled during mirroring; harness killed");
                    return Ok(());
                }
                last_state = state;
            }
        }

        match child.try_wait()? {
            Some(status) => break status,
            None => continue,
        }
    };

    // Final state comes from the package, not the harness exit code: a harness
    // can exit 0 after the driver blocked, and exit non-zero after it completed.
    let exit_code = exit_status.code();
    cp.exit_code = exit_code;
    if package_dir.is_none() {
        package_dir = find_package_dir(&output_root, job_id, &cp.started_at);
        if let Some(dir) = &package_dir {
            cp.package_dir = Some(dir.display().to_string());
            cp.package_id = dir.file_name().map(|n| n.to_string_lossy().to_string());
        }
    }
    let final_state = package_dir.as_deref().and_then(read_package_state);
    if let Some(state) = &final_state {
        if *state != last_state && !mirror(&mut cp, &mut sink, &last_state, state).await? {
            return Ok(());
        }
    }
    let exit_text = match exit_code {
        Some(c) => format!("exit code {c}"),
        None => "terminated by signal".to_string(),
    };
    match final_state {
        Some(state) if state.status == "complete" => {
            cp.status = "complete".to_string();
            cp.stage = 10;
            cp.stage_name = "export".to_string();
            cp.progress = 100;
            cp.error = None;
            if !save(&mut cp)? {
                return Ok(());
            }
            sink.status(&cp).await;
            tracing::info!("job {job_id} complete ({exit_text})");
        }
        Some(state) if state.status == "blocked" => {
            let reason = state
                .blocked
                .unwrap_or_else(|| "driver blocked without a recorded reason".to_string());
            finish_blocked(&mut cp, &mut sink, &format!("{reason} (harness {exit_text})")).await?;
        }
        Some(state) if state.status == "awaiting_stage" => {
            let stage = state
                .current_stage
                .map(|s| format!("{s:02}"))
                .unwrap_or_else(|| "?".to_string());
            finish_blocked(
                &mut cp,
                &mut sink,
                &format!("harness exited ({exit_text}) while the run was awaiting stage {stage}; resume with the driver's --resume"),
            )
            .await?;
        }
        Some(state) => {
            finish_failed(
                &mut cp,
                &mut sink,
                format!("harness {exit_text}; package status {}", state.status),
            )
            .await?;
        }
        None => {
            let reason = match &package_dir {
                Some(dir) => format!(
                    "harness {exit_text}; package {} exists but its checkpoint.json is missing or unreadable",
                    dir.display()
                ),
                None => format!(
                    "harness {exit_text} without producing a package under {}",
                    output_root.display()
                ),
            };
            finish_failed(&mut cp, &mut sink, reason).await?;
        }
    }
    Ok(())
}

/// Mirror a package checkpoint change into the job checkpoint, emitting one
/// `agent.status` per newly completed stage (so a fast driver that finishes
/// several stages between two polls still produces one event per stage) and
/// one for the stage now in progress. Returns `Ok(false)` when the job was
/// cancelled on disk, in which case nothing was overwritten.
async fn mirror(
    cp: &mut JobCheckpoint,
    sink: &mut Sink,
    before: &PackageState,
    after: &PackageState,
) -> anyhow::Result<bool> {
    let mut newly_completed: Vec<u32> = after
        .completed
        .iter()
        .copied()
        .filter(|s| !before.completed.contains(s))
        .collect();
    newly_completed.sort_unstable();
    for stage in newly_completed {
        cp.stage = stage;
        cp.stage_name = stage_name(stage).to_string();
        cp.progress = ((after.completed.iter().filter(|s| **s <= stage).count() as u64 * 100)
            / after.planned.len().max(1) as u64)
            .min(100) as u32;
        cp.status = "running".to_string();
        sink.status(cp).await;
    }
    if let Some(current) = after.current_stage {
        cp.stage = current;
        cp.stage_name = stage_name(current).to_string();
    }
    cp.progress = progress_of(after);
    if after.status == "blocked" {
        if let Some(reason) = &after.blocked {
            cp.error = Some(reason.clone());
            sink.error(cp, reason).await;
        }
    }
    if !save(cp)? {
        return Ok(false);
    }
    sink.status(cp).await;
    Ok(true)
}
