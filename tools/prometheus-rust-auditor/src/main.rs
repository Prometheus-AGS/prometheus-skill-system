use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use prometheus_rust_auditor::{
    config::{load, AuditorConfig},
    reporter::OutputFormat,
};

mod phases;

#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[derive(Debug, Parser)]
#[command(
    name = "prometheus-rust-auditor",
    about = "Staged autonomous Rust code quality remediation pipeline",
    version
)]
struct Cli {
    /// Path to prometheus-auditor.toml (defaults to ./prometheus-auditor.toml)
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Output format
    #[arg(short = 'f', long, default_value = "text")]
    format: String,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Run all phases sequentially
    Audit,
    /// Phase 1: enforce Clippy lints
    Enforce,
    /// Phase 2: check formatting with cargo fmt
    Format,
    /// Phase 3: dependency policy (cargo-deny + cargo-audit)
    Deps,
    /// Phase 4: workspace crate inventory
    Inventory,
    /// Phase 5: per-partition architectural invariants
    Partition,
    /// Phase 10: CI workflow generation
    Ci,
    /// Phases 6–9: autonomous AI audit loop (requires claude CLI)
    Autonomous,
}

pub struct AppContext {
    pub config: AuditorConfig,
    pub format: OutputFormat,
    pub verbose: bool,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e:#}");
        std::process::exit(2);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    let config = load(cli.config.as_deref())?;
    let format: OutputFormat = cli.format.parse()?;

    let ctx = AppContext { config, format, verbose: cli.verbose };

    let exit_code = match cli.command {
        Commands::Audit => phases::run_audit(&ctx)?,
        Commands::Enforce => phases::run_enforce(&ctx)?,
        Commands::Format => phases::run_format(&ctx)?,
        Commands::Deps => phases::run_deps(&ctx)?,
        Commands::Inventory => phases::run_inventory(&ctx)?,
        Commands::Partition => phases::run_partition(&ctx)?,
        Commands::Ci => phases::run_ci(&ctx)?,
        Commands::Autonomous => phases::run_autonomous(&ctx)?,
    };

    std::process::exit(exit_code);
}
