use sovereign_sync::{config, health_check, mcp_server, rest_api};

use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use tracing::info;

#[derive(Debug, Clone, ValueEnum)]
enum Mode {
    /// MCP server over stdio — for Claude Code, Kimi, Codex, OpenCode, BossFang
    Mcp,
    /// Background P2P sync daemon (HTTP on :7892)
    Daemon,
    /// Axum HTTP server with REST API + AG-UI SSE
    Server,
    /// Check daemon health on localhost without starting a server
    Status,
}

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Parser, Debug)]
#[command(
    name = "sovereign-sync",
    about = "P2P CRDT sync for prometheus-skill-pack"
)]
struct Cli {
    #[arg(long, value_enum, default_value = "mcp")]
    mode: Mode,

    #[arg(long, help = "Path to config.toml")]
    config: Option<PathBuf>,

    #[arg(long, help = "HTTP port (daemon/server modes)", default_value = "7892")]
    port: u16,

    #[arg(
        long,
        value_enum,
        default_value = "text",
        help = "Output format for status mode"
    )]
    format: OutputFormat,

    /// Prefix all MCP tool names with 'sovereign:' (avoids collision in UAR/BossFang)
    #[arg(long)]
    prefix_tools: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("sovereign_sync=info".parse()?),
        )
        .init();

    let cli = Cli::parse();

    let config_path = cli
        .config
        .unwrap_or_else(config::SovereignConfig::default_path);
    let cfg = config::SovereignConfig::load(&config_path)?;

    // Detect UAR passthrough mode
    let uar_passthrough = std::env::var("UAR_SKILL_SERVICE_URL").is_ok();
    if uar_passthrough {
        info!("UAR_SKILL_SERVICE_URL detected — enabling passthrough mode (sync tools only)");
    }

    let port = if cli.port != 7892 {
        cli.port
    } else {
        cfg.server.port
    };

    match cli.mode {
        Mode::Mcp => {
            info!("Starting sovereign-sync MCP server (stdio)");
            info!("Skills dir: {}", cfg.node.skills_dir);
            if cli.prefix_tools {
                info!("Tool prefix: sovereign:");
            }
            let skills_path = std::path::Path::new(&cfg.node.skills_dir);
            let server =
                mcp_server::SovereignMcpServer::new(skills_path, cli.prefix_tools, uar_passthrough);
            server.serve_stdio().await?;
        }
        Mode::Daemon => {
            info!("Starting sovereign-sync daemon on port {port}");
            let skills_path = std::path::Path::new(&cfg.node.skills_dir);
            rest_api::serve(port, skills_path).await?;
        }
        Mode::Server => {
            info!("Starting sovereign-sync HTTP server on port {port}");
            let skills_path = std::path::Path::new(&cfg.node.skills_dir);
            rest_api::serve(port, skills_path).await?;
        }
        Mode::Status => {
            let report = health_check::detect_daemon_health(port).await;
            match cli.format {
                OutputFormat::Text => {
                    println!(
                        "sovereign-sync daemon: {} ({})",
                        report.status, report.endpoint
                    );
                    println!("{}", report.message);
                }
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&report)?);
                }
            }
            std::process::exit(report.exit_code());
        }
    }

    Ok(())
}
