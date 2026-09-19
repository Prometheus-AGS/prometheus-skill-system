//! Bounded thread scheduler for research worker dispatch.
//!
//! # Why the cap lives here and not in a prompt
//!
//! Onyx states its "never more than 3 in parallel" rule four times in prompt
//! prose (`orchestration_layer.py:88,116,181,197`) and enforces it nowhere in
//! code — dispatch runs through `run_functions_tuples_in_parallel` with no
//! arity limit. A prompt is a request, not a bound: a model that decides to
//! dispatch six workers dispatches six workers. Holding the cap in a
//! `Semaphore` makes it structural, so no prompt can exceed it.
//!
//! # What this bounds, and what it cannot
//!
//! This scheduler spawns **harness CLI child processes**, one per thread. It
//! cannot bound in-session subagent dispatch: those run inside the harness's
//! own process, where a `Semaphore` held here has no reach. Strategy A
//! (in-session subagents) is bounded by the director's prompt and, after the
//! fact, by the merge gate. Claiming otherwise would claim an enforcement the
//! architecture cannot deliver.
//!
//! # Every dispatch gets a row
//!
//! A ledger with silent holes cannot tell a director what to re-dispatch, so
//! `complete`, `partial`, `failed` and `timeout` all produce a row. A worker
//! that exceeds its budget is killed and recorded, never left running.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::process::Command;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

/// Ledger schema version written to `threads/index.json`.
pub const THREAD_INDEX_SCHEMA_VERSION: &str = "1.0.0";

/// Terminal state of one dispatched thread.
///
/// Mirrors the `status` enum in `thread-index.schema.json`. `partial` is
/// distinct from `failed`: a budget truncated the work but its output survives
/// and is worth merging, whereas a failed thread produced nothing usable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThreadStatus {
    /// Ran to completion and produced its dossier.
    Complete,
    /// Budget truncated it, but partial output survives.
    Partial,
    /// The worker died — non-zero exit, or it could not be started.
    Failed,
    /// The scheduler killed it for exceeding its budget.
    Timeout,
}

impl ThreadStatus {
    /// Whether a row of this status must carry a `reason`.
    ///
    /// The schema requires one for every non-complete row: a row saying
    /// `failed` with no reason tells a director nothing it can act on.
    #[must_use]
    pub const fn requires_reason(self) -> bool {
        !matches!(self, Self::Complete)
    }
}

/// One row in the dispatch ledger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadRow {
    pub thread_id: String,
    pub task: String,
    pub status: ThreadStatus,
    pub cycle: u32,
    pub started_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claims_count: Option<u32>,
    /// Required whenever `status` is not `complete`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// The dispatch ledger, serialised to `threads/index.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadIndex {
    pub schema_version: String,
    pub package_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub director_cycles: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget_hit: Option<bool>,
    pub threads: Vec<ThreadRow>,
}

/// A worker brief as written by the director, per `thread-brief.schema.json`.
///
/// Only the fields the scheduler acts on are modelled; the rest of the brief
/// belongs to the worker, and `serde` ignores it rather than failing a run over
/// a field the scheduler has no opinion about.
#[derive(Debug, Clone, Deserialize)]
pub struct ThreadBrief {
    pub thread_id: String,
    pub task: String,
    #[serde(default)]
    pub budget: BriefBudget,
}

/// The `budget` object of a brief.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct BriefBudget {
    /// Per-thread wall clock in minutes. Absent means the scheduler default.
    pub minutes: Option<u64>,
}

/// Read every `threads/<tid>/brief.json` under a package, in `thread_id` order.
///
/// A brief that cannot be parsed is an error rather than a skip: silently
/// dropping it would leave the ledger short a row, and a director reading that
/// ledger would believe the thread was never planned.
pub fn read_briefs(package_dir: &Path) -> anyhow::Result<Vec<ThreadBrief>> {
    let dir = package_dir.join("threads");
    if !dir.is_dir() {
        anyhow::bail!("no threads/ directory under {}", package_dir.display());
    }

    let mut briefs = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let brief_path = entry.path().join("brief.json");
        if !brief_path.is_file() {
            continue;
        }
        let raw = std::fs::read_to_string(&brief_path)?;
        let brief: ThreadBrief = serde_json::from_str(&raw)
            .map_err(|e| anyhow::anyhow!("{}: {e}", brief_path.display()))?;

        // The directory name is the id of record; a brief claiming a different
        // id would write its dossier into another thread's directory.
        let dir_name = entry.file_name().to_string_lossy().to_string();
        if brief.thread_id != dir_name {
            anyhow::bail!(
                "{}: thread_id {:?} does not match its directory {:?}",
                brief_path.display(),
                brief.thread_id,
                dir_name
            );
        }
        briefs.push(brief);
    }

    briefs.sort_by(|a, b| a.thread_id.cmp(&b.thread_id));
    Ok(briefs)
}

/// One unit of work: a harness process to run for a research thread.
#[derive(Debug, Clone)]
pub struct ThreadTask {
    /// Ledger id, `t01`-style. The schema requires `^t[0-9]{2,}$`.
    pub thread_id: String,
    /// Human-readable description of the thread's assignment.
    pub task: String,
    /// Director cycle that dispatched this thread.
    pub cycle: u32,
    /// Program to execute.
    pub program: PathBuf,
    /// Arguments to the program.
    pub args: Vec<String>,
}

/// Budgets bounding one run.
///
/// Defaults are adopted from Onyx and verified at source (analysis D-07):
/// thread hard timeout 30 min (`research_agent.py:88`), thread force-report
/// 12 min (`:91`), job force-report 30 min (`dr_loop.py:85`).
#[derive(Debug, Clone, Copy)]
pub struct Budgets {
    /// Hard per-thread wall clock. Exceeding it kills the worker.
    pub thread_timeout: Duration,
    /// Whole-run wall clock. Exceeding it stops dispatching new threads.
    pub job_timeout: Duration,
}

impl Default for Budgets {
    fn default() -> Self {
        Self {
            thread_timeout: Duration::from_secs(30 * 60),
            job_timeout: Duration::from_secs(30 * 60),
        }
    }
}

/// Outcome of a scheduler run.
#[derive(Debug)]
pub struct RunOutcome {
    pub rows: Vec<ThreadRow>,
    /// Peak number of workers observed running at once. Never exceeds the cap.
    pub peak_concurrency: usize,
    /// True when the job budget stopped dispatch before every task ran.
    pub budget_hit: bool,
}

/// Semaphore-bounded scheduler over a `JoinSet` of harness child processes.
pub struct ThreadScheduler {
    cap: usize,
    budgets: Budgets,
}

impl ThreadScheduler {
    /// Build a scheduler with a hard concurrency cap.
    ///
    /// The cap is a constructor argument and never a prompt instruction; a cap
    /// of 0 is meaningless and is raised to 1 so a run always makes progress.
    #[must_use]
    pub fn new(max_parallel: usize, budgets: Budgets) -> Self {
        Self {
            cap: max_parallel.max(1),
            budgets,
        }
    }

    /// The effective cap, after the zero guard.
    #[must_use]
    pub const fn cap(&self) -> usize {
        self.cap
    }

    /// Run every task, never exceeding the cap, and return one row per task.
    ///
    /// A task that exceeds `thread_timeout` is killed. Because the child is
    /// spawned with `kill_on_drop(true)`, dropping the timed-out future reaps
    /// the process rather than orphaning it — a killed worker leaves no stray
    /// harness running against the package directory.
    pub async fn run(&self, tasks: Vec<ThreadTask>, package_dir: &Path) -> RunOutcome {
        let permits = Arc::new(Semaphore::new(self.cap));
        let live = Arc::new(AtomicUsize::new(0));
        let peak = Arc::new(AtomicUsize::new(0));
        let started = tokio::time::Instant::now();
        let job_timeout = self.budgets.job_timeout;
        let thread_timeout = self.budgets.thread_timeout;

        let mut set: JoinSet<ThreadRow> = JoinSet::new();
        let mut budget_hit = false;
        let mut undispatched = Vec::new();

        // Dispatch acquires the permit HERE, in the loop, rather than inside the
        // spawned future. Spawning all tasks up front and letting each wait on
        // the semaphore internally would drain this loop in microseconds, so
        // `elapsed()` would still be ~0 at the last task and the job budget
        // could never fire — the loop would finish long before any work did.
        // Acquiring first makes the loop advance at the pace of actual
        // completions, which is what the budget is meant to bound.
        let mut tasks = tasks.into_iter();
        for task in tasks.by_ref() {
            if started.elapsed() >= job_timeout {
                budget_hit = true;
                undispatched.push(task);
                break;
            }

            let permit = tokio::select! {
                biased;
                // Whichever comes first: a free slot, or the job budget.
                p = Arc::clone(&permits).acquire_owned() => {
                    p.expect("semaphore is never closed while the scheduler lives")
                }
                () = tokio::time::sleep_until(started + job_timeout) => {
                    budget_hit = true;
                    undispatched.push(task);
                    break;
                }
            };

            let live = Arc::clone(&live);
            let peak = Arc::clone(&peak);
            let dir = package_dir.to_path_buf();

            set.spawn(async move {
                // The permit is held for the life of the child process, so no
                // child exists without one and the cap is structural.
                let _permit = permit;

                let now = live.fetch_add(1, Ordering::SeqCst) + 1;
                peak.fetch_max(now, Ordering::SeqCst);

                let row = run_one(&task, &dir, thread_timeout).await;

                live.fetch_sub(1, Ordering::SeqCst);
                row
            });
        }

        // Anything left after the break was planned but never dispatched.
        for task in tasks {
            undispatched.push(task);
        }

        let mut rows = Vec::new();
        while let Some(joined) = set.join_next().await {
            match joined {
                Ok(row) => rows.push(row),
                // A panicking worker task must still leave a row; losing it
                // would put a silent hole in the ledger.
                Err(e) => rows.push(ThreadRow {
                    thread_id: "t00".to_string(),
                    task: "unknown (worker task panicked)".to_string(),
                    status: ThreadStatus::Failed,
                    cycle: 0,
                    started_at: now_rfc3339(),
                    ended_at: Some(now_rfc3339()),
                    sources_count: None,
                    claims_count: None,
                    reason: Some(format!("scheduler task panicked: {e}")),
                }),
            }
        }

        for task in undispatched {
            rows.push(ThreadRow {
                thread_id: task.thread_id,
                task: task.task,
                status: ThreadStatus::Partial,
                cycle: task.cycle,
                started_at: now_rfc3339(),
                ended_at: Some(now_rfc3339()),
                sources_count: None,
                claims_count: None,
                reason: Some("job budget exhausted before dispatch".to_string()),
            });
        }

        rows.sort_by(|a, b| a.thread_id.cmp(&b.thread_id));

        RunOutcome {
            rows,
            peak_concurrency: peak.load(Ordering::SeqCst),
            budget_hit,
        }
    }
}

/// Run one worker under its budget and classify the outcome.
async fn run_one(task: &ThreadTask, package_dir: &Path, budget: Duration) -> ThreadRow {
    let started_at = now_rfc3339();

    let spawned = Command::new(&task.program)
        .args(&task.args)
        .current_dir(package_dir)
        .env("RESEARCH_THREAD_ID", &task.thread_id)
        .env("RESEARCH_PACKAGE_DIR", package_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        // Without this a timed-out child outlives the dropped future and keeps
        // writing into the package directory — the exact stall this bounds.
        .kill_on_drop(true)
        .spawn();

    let mut child = match spawned {
        Ok(c) => c,
        Err(e) => {
            return row_for(
                task,
                ThreadStatus::Failed,
                started_at,
                Some(format!("could not spawn {}: {e}", task.program.display())),
            )
        }
    };

    match tokio::time::timeout(budget, child.wait()).await {
        Ok(Ok(status)) if status.success() => {
            row_for(task, ThreadStatus::Complete, started_at, None)
        }
        Ok(Ok(status)) => row_for(
            task,
            ThreadStatus::Failed,
            started_at,
            Some(match status.code() {
                Some(c) => format!("worker exited with code {c}"),
                None => "worker terminated by signal".to_string(),
            }),
        ),
        Ok(Err(e)) => row_for(
            task,
            ThreadStatus::Failed,
            started_at,
            Some(format!("could not wait on worker: {e}")),
        ),
        Err(_) => {
            // Kill explicitly rather than relying on kill_on_drop alone, so the
            // process is reaped before the row is written and the ledger cannot
            // claim a terminal state while the worker is still alive.
            let _ = child.kill().await;
            row_for(
                task,
                ThreadStatus::Timeout,
                started_at,
                Some(format!(
                    "exceeded its {}s thread budget and was killed",
                    budget.as_secs()
                )),
            )
        }
    }
}

fn row_for(
    task: &ThreadTask,
    status: ThreadStatus,
    started_at: String,
    reason: Option<String>,
) -> ThreadRow {
    debug_assert!(
        !status.requires_reason() || reason.is_some(),
        "a non-complete row must carry a reason"
    );
    ThreadRow {
        thread_id: task.thread_id.clone(),
        task: task.task.clone(),
        status,
        cycle: task.cycle,
        started_at,
        ended_at: Some(now_rfc3339()),
        sources_count: None,
        claims_count: None,
        reason,
    }
}

/// Write the ledger to `<package_dir>/threads/index.json`.
pub fn write_index(
    package_dir: &Path,
    package_id: &str,
    outcome: &RunOutcome,
    director_cycles: Option<u32>,
) -> anyhow::Result<PathBuf> {
    let dir = package_dir.join("threads");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("index.json");
    let index = ThreadIndex {
        schema_version: THREAD_INDEX_SCHEMA_VERSION.to_string(),
        package_id: package_id.to_string(),
        director_cycles,
        budget_hit: Some(outcome.budget_hit),
        threads: outcome.rows.clone(),
    };
    std::fs::write(&path, format!("{}\n", serde_json::to_string_pretty(&index)?))?;
    Ok(path)
}

fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}
