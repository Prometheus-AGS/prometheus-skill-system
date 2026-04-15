use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

mod commands;

#[derive(Parser)]
#[command(name = "prometheus")]
#[command(about = "Self-improving skill execution engine — manage, optimize, and learn from AI skills")]
#[command(version)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install skills from a GitHub repository or local path
    Install {
        /// GitHub repo (user/repo) or local path
        source: String,
        /// Target specific platform(s) — comma-separated
        #[arg(short, long)]
        agent: Option<String>,
        /// Install to project scope instead of global
        #[arg(long)]
        local: bool,
        /// Copy files instead of creating symlinks
        #[arg(long)]
        no_symlink: bool,
        /// Install as plugin (preserve full repo structure)
        #[arg(long)]
        plugin: bool,
    },

    /// Remove installed skills
    Uninstall {
        /// Skill name to remove
        name: String,
        /// Target specific platform(s)
        #[arg(short, long)]
        agent: Option<String>,
    },

    /// List installed skills
    List {
        /// Show all scopes
        #[arg(long)]
        all: bool,
        /// Show only global skills
        #[arg(long)]
        global: bool,
        /// Show only project skills
        #[arg(long)]
        project: bool,
        /// Verbose output with symlink targets
        #[arg(short, long)]
        verbose: bool,
    },

    /// Search GitHub for skill repositories
    Search {
        /// Search query
        query: String,
        /// Maximum results
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Security audit of installed skills
    Audit {
        /// Path to scan (default: all installed skills)
        #[arg(short, long)]
        path: Option<String>,
    },

    /// Verify skill integrity against Skills.lock checksums
    Verify {
        /// Update checksums instead of checking
        #[arg(long)]
        update: bool,
    },

    /// Health check — verify directories, platforms, connectivity
    Doctor,

    /// Show current status: Skills.toml, KBD waypoint, evolver state
    Status {
        /// Project path
        #[arg(short, long, default_value = ".")]
        path: String,
    },

    /// Generate skills from source code repositories
    Generate {
        /// Path to source code
        path: String,
        /// Target language filter
        #[arg(short, long)]
        language: Option<String>,
    },

    /// Validate skills against agentskills.io specification
    Validate {
        /// Specific skill path (default: all)
        path: Option<String>,
    },

    /// Build Kustomize overlay and validate output
    Build {
        /// Service name
        #[arg(short, long)]
        service: String,
        /// Target overlay (e.g., gke-prod)
        #[arg(short, long)]
        overlay: String,
        /// GitOps root directory
        #[arg(long, default_value = "./gitops")]
        gitops_path: String,
    },

    /// Query surreal-memory server
    Memory {
        #[command(subcommand)]
        action: MemoryAction,
    },

    /// Trigger an iterative evolution cycle
    Evolve {
        /// Evolution name for cross-session retrieval
        name: String,
        /// Domain (software, business, product, etc.)
        #[arg(short, long, default_value = "software")]
        domain: String,
        /// Specific phase to run
        #[arg(short, long)]
        phase: Option<String>,
    },

    /// Run the self-learning pipeline on execution traces
    Learn {
        /// Capture current session traces
        #[arg(long)]
        capture_session: bool,
        /// Compile traces into knowledge wiki
        #[arg(long)]
        compile: bool,
        /// Run lint on compiled knowledge
        #[arg(long)]
        lint: bool,
        /// Dry run — show what would happen without writing
        #[arg(long)]
        dry_run: bool,
    },

    /// Optimize a skill's prompts using dspy-rs
    Optimize {
        /// Path to the skill directory
        skill: String,
        /// Minimum trace count required
        #[arg(long, default_value = "10")]
        min_traces: usize,
        /// Dry run — show optimization plan without applying
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
enum MemoryAction {
    /// Check server health
    Ping,
    /// Show entity and memory stats
    Stats,
    /// Search the knowledge graph
    Search {
        /// Search query
        query: String,
        /// Filter by entity type
        #[arg(short, long)]
        r#type: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "warn".into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Install { source, agent, local, no_symlink, plugin } => {
            commands::install::run(&source, agent.as_deref(), local, no_symlink, plugin).await
        }
        Commands::Uninstall { name, agent } => {
            commands::uninstall::run(&name, agent.as_deref())
        }
        Commands::List { all, global, project, verbose } => {
            commands::list::run(all, global, project, verbose)
        }
        Commands::Search { query, limit } => {
            commands::search::run(&query, limit).await
        }
        Commands::Audit { path } => {
            commands::audit::run(path.as_deref())
        }
        Commands::Verify { update } => {
            commands::verify::run(update)
        }
        Commands::Doctor => {
            commands::doctor::run().await
        }
        Commands::Status { path } => {
            commands::status::run(&path)
        }
        Commands::Generate { path, language } => {
            commands::generate::run(&path, language.as_deref())
        }
        Commands::Validate { path } => {
            commands::validate::run(path.as_deref())
        }
        Commands::Build { service, overlay, gitops_path } => {
            commands::build::run(&gitops_path, &service, &overlay)
        }
        Commands::Memory { action } => match action {
            MemoryAction::Ping => commands::memory::ping().await,
            MemoryAction::Stats => commands::memory::stats().await,
            MemoryAction::Search { query, r#type } => {
                commands::memory::search(&query, r#type.as_deref()).await
            }
        },
        Commands::Evolve { name, domain, phase } => {
            commands::evolve::run(&name, &domain, phase.as_deref())
        }
        Commands::Learn { capture_session, compile, lint, dry_run } => {
            commands::learn::run(capture_session, compile, lint, dry_run).await
        }
        Commands::Optimize { skill, min_traces, dry_run } => {
            commands::optimize::run(&skill, min_traces, dry_run).await
        }
    }
}
