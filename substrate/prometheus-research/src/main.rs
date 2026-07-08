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

    #[command(subcommand)]
    command: Option<Command>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("prometheus_research=info".parse()?),
        )
        .init();

    let cli = Cli::parse();

    let config_path = cli
        .config
        .unwrap_or_else(config::ResearchConfig::default_path);
    let cfg = config::ResearchConfig::load(&config_path).unwrap_or_default();

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
                let checkpoint =
                    prometheus_research::job::checkpoint::read(&job_id)?;
                println!("{}", serde_json::to_string_pretty(&checkpoint)?);
            }
            Some(Command::Cancel { job_id }) => {
                prometheus_research::job::cancel::cancel_job(&job_id)?;
                println!("Cancelled job {job_id}");
            }
            None => {
                eprintln!("No command or mode specified. Run --help for usage.");
                std::process::exit(1);
            }
        },
    }

    Ok(())
}
