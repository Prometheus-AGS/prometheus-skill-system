use prometheus_research::{config, http_server, mcp_server};

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use tracing::info;

#[derive(Debug, Clone, ValueEnum)]
enum Mode {
    /// MCP server over stdio — for Claude Code, Kimi, Codex, OpenCode
    Mcp,
    /// Axum HTTP server on :7891 with REST API + AG-UI SSE + A2UI components
    Server,
    /// Check server health without starting a new process
    Status,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Start a background research job
    Start {
        /// Research query
        query: String,
        /// Research depth
        #[arg(long, default_value = "deep")]
        depth: String,
        /// Maximum number of sources to retrieve
        #[arg(long, default_value_t = 20)]
        max_sources: u32,
        /// Citation style
        #[arg(long, default_value = "apa")]
        citation_style: String,
    },
    /// Check the status of a research job
    Status {
        /// Job ID returned by `start`
        job_id: String,
    },
    /// Cancel a running research job
    Cancel {
        /// Job ID to cancel
        job_id: String,
    },
    /// Thread-level operations for a research package
    #[command(subcommand)]
    Threads(ThreadsCommand),
}

#[derive(Debug, Subcommand)]
enum ThreadsCommand {
    /// Dispatch every brief under <package>/threads/ through a bounded scheduler.
    ///
    /// Returns after the workers finish, leaving `threads/index.json` populated
    /// with one row per dispatch — including the failed and timed-out ones.
    Run {
        /// Package directory containing `threads/<tid>/brief.json`
        #[arg(long)]
        package: PathBuf,
        /// Hard concurrency cap. Enforced by a semaphore, not by a prompt.
        #[arg(long, default_value_t = 3)]
        max_parallel: usize,
        /// Worker program to run per thread. Defaults to the resolved harness.
        #[arg(long)]
        worker: Option<PathBuf>,
        /// Per-thread wall-clock budget in seconds (a brief may tighten it).
        #[arg(long, default_value_t = 30 * 60)]
        thread_seconds: u64,
        /// Whole-run wall-clock budget in seconds.
        #[arg(long, default_value_t = 30 * 60)]
        job_seconds: u64,
    },
}

#[derive(Parser, Debug)]
#[command(
    name = "prometheus-research",
    about = "Background research daemon with MCP + HTTP + AG-UI streaming"
)]
struct Cli {
    /// Run as a server or MCP process instead of a one-shot CLI command
    #[arg(long, value_enum)]
    mode: Option<Mode>,

    /// HTTP port (server mode only)
    #[arg(long, default_value_t = 7891)]
    port: u16,

    /// Path to config file
    #[arg(long)]
    config: Option<PathBuf>,

    /// Run one research job to completion (internal: spawned by `start`).
    /// Resolves a headless harness from PATH, runs the deep-research driver
    /// through it, mirrors the package checkpoint, and exits 0 whatever the
    /// job's outcome; the outcome is in the job checkpoint.
    #[arg(long, value_name = "JOB_ID", hide = true)]
    daemon_job: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("prometheus_research=info".parse()?),
        )
        .init();

    let cli = Cli::parse();

    // Process environment is mutated here, before any runtime thread exists:
    // `std::env::set_var` is unsound once other threads may read the
    // environment. The server's daemons inherit these two values.
    if matches!(cli.mode, Some(Mode::Server)) {
        if std::env::var_os("RESEARCH_EVENT_SINK").is_none_or(|v| v.is_empty()) {
            std::env::set_var("RESEARCH_EVENT_SINK", format!("http://127.0.0.1:{}", cli.port));
        }
        // A fresh ingest secret per server process; an operator may pin one
        // for a supervised daemon that must survive server restarts.
        if std::env::var_os("RESEARCH_EVENT_TOKEN").is_none_or(|v| v.is_empty()) {
            std::env::set_var("RESEARCH_EVENT_TOKEN", uuid::Uuid::new_v4().to_string());
        }
    }

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(run(cli))
}

async fn run(cli: Cli) -> anyhow::Result<()> {
    let config_path = cli
        .config
        .unwrap_or_else(config::ResearchConfig::default_path);
    let cfg = config::ResearchConfig::load(&config_path).unwrap_or_default();

    if let Some(job_id) = cli.daemon_job {
        info!("daemon job {job_id} starting");
        prometheus_research::job::daemon::run(&job_id).await?;
        return Ok(());
    }

    match cli.mode {
        Some(Mode::Mcp) => {
            info!("Starting prometheus-research MCP server (stdio)");
            mcp_server::run().await?;
        }
        Some(Mode::Server) => {
            let port = cli.port;
            info!("Starting prometheus-research HTTP server on :{port}");
            http_server::run_server(port, cfg).await?;
        }
        Some(Mode::Status) => {
            warn_legacy_output_root();
            let url = format!("http://127.0.0.1:{}/health", cli.port);
            match reqwest::get(&url).await {
                Ok(resp) => {
                    let body: serde_json::Value = resp.json().await?;
                    println!("{}", serde_json::to_string_pretty(&body)?);
                }
                Err(_) => {
                    eprintln!("prometheus-research is not running on port {}", cli.port);
                    std::process::exit(1);
                }
            }
        }
        None => match cli.command {
            Some(Command::Start {
                query,
                depth,
                max_sources,
                citation_style,
            }) => {
                let job_id = prometheus_research::job::spawn::spawn_job(
                    &query,
                    &depth,
                    max_sources,
                    &citation_style,
                )?;
                println!("{job_id}");
            }
            Some(Command::Status { job_id }) => {
                warn_legacy_output_root();
                let checkpoint = prometheus_research::job::checkpoint::read(&job_id)?;
                println!("{}", serde_json::to_string_pretty(&checkpoint)?);
            }
            Some(Command::Cancel { job_id }) => {
                prometheus_research::job::cancel::cancel_job(&job_id)?;
                println!("Cancelled job {job_id}");
            }
            Some(Command::Threads(ThreadsCommand::Run {
                package,
                max_parallel,
                worker,
                thread_seconds,
                job_seconds,
            })) => {
                run_threads(&package, max_parallel, worker, thread_seconds, job_seconds).await?;
            }
            None => {
                eprintln!("No command or mode specified. Run --help for usage.");
                std::process::exit(1);
            }
        },
    }

    Ok(())
}

/// Dispatch a package's thread briefs through the bounded scheduler.
///
/// Exits non-zero when any thread did not complete, so a shell gate can tell a
/// clean run from one that lost a thread without parsing the ledger.
async fn run_threads(
    package: &std::path::Path,
    max_parallel: usize,
    worker: Option<PathBuf>,
    thread_seconds: u64,
    job_seconds: u64,
) -> anyhow::Result<()> {
    use prometheus_research::threads::{Budgets, ThreadScheduler, ThreadStatus, ThreadTask};
    use std::time::Duration;

    let briefs = prometheus_research::threads::read_briefs(package)?;
    if briefs.is_empty() {
        anyhow::bail!("no briefs found under {}/threads/", package.display());
    }

    // Resolving the harness here, once, keeps the failure legible: a missing
    // harness is one clear error rather than N identical spawn failures.
    let program = match worker {
        Some(p) => p,
        None => {
            let harness = prometheus_research::job::daemon::resolve_harness().ok_or_else(|| {
                anyhow::anyhow!(
                    "no harness binary on PATH (looked for claude, then codex); \
                     pass --worker to name one explicitly"
                )
            })?;
            info!(
                "resolved harness {} at {}",
                harness.name,
                harness.path.display()
            );
            harness.path
        }
    };

    let package_id = package
        .file_name()
        .map_or_else(|| package.display().to_string(), |n| n.to_string_lossy().to_string());

    let default_thread = Duration::from_secs(thread_seconds);
    let tasks: Vec<ThreadTask> = briefs
        .iter()
        .map(|b| ThreadTask {
            thread_id: b.thread_id.clone(),
            task: b.task.clone(),
            cycle: 0,
            program: program.clone(),
            args: vec![
                "--thread".to_string(),
                b.thread_id.clone(),
                "--package".to_string(),
                package.display().to_string(),
            ],
        })
        .collect();

    // A brief may bound its own thread more tightly than the flag; the tighter
    // of the two wins, so a brief can never widen the operator's budget.
    let thread_budget = briefs
        .iter()
        .filter_map(|b| b.budget.minutes)
        .map(|m| Duration::from_secs(m * 60))
        .min()
        .map_or(default_thread, |m| m.min(default_thread));

    let scheduler = ThreadScheduler::new(
        max_parallel,
        Budgets {
            thread_timeout: thread_budget,
            job_timeout: Duration::from_secs(job_seconds),
        },
    );

    let outcome = scheduler.run(tasks, package).await;
    let path = prometheus_research::threads::write_index(package, &package_id, &outcome, None)?;

    let completed = outcome
        .rows
        .iter()
        .filter(|r| r.status == ThreadStatus::Complete)
        .count();
    let total = outcome.rows.len();

    println!(
        "{completed}/{total} threads complete (peak concurrency {}, cap {}) -> {}",
        outcome.peak_concurrency,
        scheduler.cap(),
        path.display()
    );
    for row in &outcome.rows {
        if row.status != ThreadStatus::Complete {
            println!(
                "  {} {:?}: {}",
                row.thread_id,
                row.status,
                row.reason.as_deref().unwrap_or("(no reason recorded)")
            );
        }
    }

    if completed != total {
        std::process::exit(1);
    }
    Ok(())
}

/// Name the legacy job root on stderr while it still exists. Nothing reads
/// the legacy `~/.research-jobs/` after change-rah-002 (the root is `~/.prometheus/research/`
/// or `RESEARCH_OUTPUT_DIR`); the notice exists so an operator can delete the
/// old directories by hand. Analysis decision D-14: no migration.
fn warn_legacy_output_root() {
    let legacy = prometheus_research::job::checkpoint::legacy_output_root();
    if !legacy.is_dir() {
        return;
    }
    let entries = std::fs::read_dir(&legacy)
        .map(|rd| rd.filter_map(Result::ok).count())
        .unwrap_or(0);
    eprintln!(
        "note: legacy job root {} still exists ({entries} entries); it is no longer read. \
         Research output now lives under {}. Delete the legacy directory by hand when convenient.",
        legacy.display(),
        prometheus_research::job::checkpoint::output_root().display()
    );
}
