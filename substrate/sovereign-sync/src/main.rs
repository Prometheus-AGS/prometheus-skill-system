use sovereign_sync::{config, health_check, mcp_server, p2p, rest_api};

use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use tracing::{info, warn};

#[derive(Debug, Clone, ValueEnum)]
enum Mode {
    /// Initialize the permission-protected device key used by a headless service
    Init,
    /// MCP server over stdio — for Claude Code, Kimi, Codex, OpenCode, BossFang
    Mcp,
    /// Background P2P sync daemon (HTTP over a Unix socket; --tcp for loopback)
    Daemon,
    /// Axum HTTP server with REST API + AG-UI SSE (Unix socket; --tcp for loopback)
    Server,
    /// Check daemon health over the resolved transport without starting a server
    Status,
    /// Export a redacted-safe pairing ticket to stdout for explicit transfer
    PairExport,
    /// Import a pairing ticket and enroll its endpoint/signing-key binding
    PairImport,
    /// Print the generated OpenAPI 3.1 contract and exit
    Openapi,
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

    #[arg(
        long,
        help = "Loopback TCP port; applies only with --tcp (daemon/server modes)",
        default_value = "7892"
    )]
    port: u16,

    #[arg(long, help = "Expose loopback TCP instead of the default Unix socket")]
    tcp: bool,

    #[arg(
        long,
        help = "Unix socket path; defaults to <data-local-dir>/prometheus/run/sovereign-sync.sock"
    )]
    socket: Option<PathBuf>,

    #[arg(
        long,
        help = "Mode-0600 bearer-token file used only with explicit --tcp"
    )]
    token_file: Option<PathBuf>,

    #[arg(
        long,
        value_enum,
        default_value = "text",
        help = "Output format for status mode"
    )]
    format: OutputFormat,

    #[arg(
        long,
        default_value_t = 1,
        help = "Measured warm-connection health probes in status mode"
    )]
    samples: usize,

    #[arg(
        long,
        default_value_t = 0,
        help = "Unmeasured warm-up health probes in status mode"
    )]
    warmup: usize,

    #[arg(
        long,
        help = "Fail status mode when measured p99 exceeds this many milliseconds"
    )]
    p99_budget_ms: Option<f64>,

    #[arg(
        long,
        help = "Fail status mode when maximum latency exceeds this many milliseconds"
    )]
    max_budget_ms: Option<f64>,

    /// Prefix all MCP tool names with 'sovereign:' (avoids collision in UAR/BossFang)
    #[arg(long)]
    prefix_tools: bool,

    #[arg(long, help = "Pairing ticket for --mode pair-import")]
    ticket: Option<String>,
}

async fn run_daemon_until_shutdown(
    mut http: rest_api::HttpService,
    p2p_supervisor: Option<p2p::P2PSupervisor>,
    mut incoming_consumer: Option<tokio::task::JoinHandle<()>>,
) -> anyhow::Result<()> {
    tokio::select! {
        _ = rest_api::shutdown_signal() => info!("shutdown signal received"),
        _ = http.wait_for_exit() => warn!("dedicated HTTP runtime exited"),
    }

    // Stop accepting new requests first, then stop the network runtime and
    // drain its authority consumer before joining the HTTP runtime.
    http.begin_shutdown().await;
    let mut shutdown_error = None;
    if let Some(supervisor) = p2p_supervisor {
        if let Err(error) = supervisor.shutdown().await {
            warn!(%error, "P2P supervisor did not shut down cleanly");
            shutdown_error = Some(error);
        }
    }
    if let Some(consumer) = incoming_consumer.as_mut() {
        if tokio::time::timeout(std::time::Duration::from_secs(5), &mut *consumer)
            .await
            .is_err()
        {
            consumer.abort();
            warn!("P2P authority consumer exceeded shutdown drain bound and was aborted");
        }
    }
    if let Err(error) = http.join().await {
        shutdown_error.get_or_insert(error);
    }
    if let Some(error) = shutdown_error {
        return Err(error);
    }
    Ok(())
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
        .clone()
        .unwrap_or_else(config::SovereignConfig::default_path);
    let cfg = config::SovereignConfig::load(&config_path)?;
    let tcp_token_path = cli
        .token_file
        .clone()
        .unwrap_or_else(|| default_tcp_token_path(&config_path));

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
        Mode::Init => {
            let key_path = config_path
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."))
                .join("device-key.json");
            let signer = kbd_runtime::ensure_device_key_file(&key_path)?;
            println!(
                "{}",
                serde_json::json!({
                    "deviceKeyFile": key_path,
                    "keyId": signer.key_id()
                })
            );
            if cli.tcp {
                rest_api::ensure_tcp_token_file(&tcp_token_path)?;
                println!("{}", serde_json::json!({ "tcpTokenFile": tcp_token_path }));
            }
            let identity_path = PathBuf::from(&cfg.node.p2p_identity_file);
            let identity = p2p::P2PIdentity::load_or_create(&identity_path)?;
            println!(
                "{}",
                serde_json::json!({
                    "p2pIdentityFile": identity_path,
                    "endpointId": identity.endpoint_id().to_string()
                })
            );
        }
        Mode::PairExport => {
            let identity = p2p::P2PIdentity::load_or_create(&cfg.node.p2p_identity_file)?;
            println!("{}", identity.export_ticket()?);
        }
        Mode::PairImport => {
            let ticket = cli
                .ticket
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("--ticket is required for pair-import"))?;
            let mut identity = p2p::P2PIdentity::load_or_create(&cfg.node.p2p_identity_file)?;
            let imported = identity.import_ticket(ticket)?;
            println!(
                "{}",
                serde_json::json!({
                    "pairedEndpointId": imported.endpoint_id,
                    "signingKeyFingerprint": imported.signing_key_fingerprint
                })
            );
        }
        Mode::Openapi => {
            println!(
                "{}",
                serde_json::to_string_pretty(&rest_api::openapi_document())?
            );
        }
        Mode::Mcp => {
            info!("Starting sovereign-sync MCP server (stdio)");
            info!("Skills dir: {}", cfg.node.skills_dir);
            if cli.prefix_tools {
                info!("Tool prefix: sovereign:");
            }
            let skills_path = std::path::Path::new(&cfg.node.skills_dir);
            let server =
                mcp_server::SovereignMcpServer::new(skills_path, cli.prefix_tools, uar_passthrough)
                    .await;
            server.serve_stdio().await?;
        }
        Mode::Daemon => {
            info!("Starting sovereign-sync daemon");
            let skills_path = std::path::Path::new(&cfg.node.skills_dir);
            // The HTTP service owns a dedicated two-worker runtime and proves
            // static liveness before authority or network initialization can
            // begin consuming resources.
            let http_service = if cli.tcp {
                rest_api::HttpService::start_tcp(port, &tcp_token_path).await?
            } else {
                #[cfg(unix)]
                {
                    rest_api::HttpService::start_unix(
                        cli.socket.clone().unwrap_or_else(default_socket_path),
                    )
                    .await?
                }
                #[cfg(not(unix))]
                {
                    anyhow::bail!("Unix sockets are unavailable; use explicit --tcp")
                }
            };
            let startup_gate = http_service.gate().clone();
            let p2p_identity = p2p::P2PIdentity::load_or_create(&cfg.node.p2p_identity_file)?;
            let p2p_handle = p2p::P2PHandle::pending();
            let state = match rest_api::AppState::try_new_with_startup_handle(
                skills_path,
                p2p_handle.clone(),
                &startup_gate,
            )
            .await
            {
                Ok(state) => state,
                Err(error) => {
                    warn!(%error, "local authority initialization failed; diagnostic router remains active");
                    startup_gate
                        .fail("local authority initialization failed; inspect the service log")
                        .await;
                    return run_daemon_until_shutdown(http_service, None, None).await;
                }
            };
            startup_gate.install(state.clone()).await;
            info!("sovereign-sync local authority routes are ready");

            // Only now may the production N0 endpoint create netwatch and join
            // gossip. The node remains entirely on its dedicated runtime.
            let (p2p_supervisor, incoming_consumer) = match p2p::P2PSupervisor::spawn(
                p2p_identity,
                cfg.peers.clone(),
                p2p_handle.clone(),
            ) {
                Ok((supervisor, mut incoming)) => {
                    let consumer_state = state.clone();
                    let consumer = tokio::spawn(async move {
                        while let Some(message) = incoming.recv().await {
                            rest_api::handle_incoming_message(&consumer_state, &message.payload)
                                .await;
                        }
                    });
                    (Some(supervisor), Some(consumer))
                }
                Err(error) => {
                    warn!(%error, "P2P supervisor failed to start; local authority remains available");
                    p2p_handle.mark_failed("P2P supervisor thread could not be created");
                    (None, None)
                }
            };
            info!("sovereign-sync daemon startup completed");
            run_daemon_until_shutdown(http_service, p2p_supervisor, incoming_consumer).await?;
        }
        Mode::Server => {
            info!("Starting sovereign-sync local API server");
            let skills_path = std::path::Path::new(&cfg.node.skills_dir);
            if cli.tcp {
                rest_api::serve_tcp(port, skills_path, &tcp_token_path).await?;
            } else {
                #[cfg(unix)]
                rest_api::serve_unix(
                    cli.socket.clone().unwrap_or_else(default_socket_path),
                    skills_path,
                )
                .await?;
                #[cfg(not(unix))]
                anyhow::bail!("Unix sockets are unavailable; use explicit --tcp");
            }
        }
        Mode::Status => {
            let report = if cli.tcp {
                let token = rest_api::load_tcp_token_file(&tcp_token_path)?;
                health_check::sample_daemon_health_with_token(
                    port,
                    cli.samples,
                    cli.warmup,
                    cli.p99_budget_ms,
                    cli.max_budget_ms,
                    Some(&token),
                )
                .await
            } else {
                #[cfg(unix)]
                {
                    let socket = cli.socket.clone().unwrap_or_else(default_socket_path);
                    health_check::sample_unix_daemon_health(
                        &socket,
                        cli.samples,
                        cli.warmup,
                        cli.p99_budget_ms,
                        cli.max_budget_ms,
                    )
                    .await
                }
                #[cfg(not(unix))]
                anyhow::bail!("Unix sockets are unavailable; use explicit --tcp")
            };
            match cli.format {
                OutputFormat::Text => {
                    println!(
                        "sovereign-sync daemon: {} ({})",
                        report.health.status, report.health.endpoint
                    );
                    println!("{}", report.health.message);
                    println!(
                        "latency ms: p50={} p95={} p99={} max={} failures={} timeouts={}",
                        display_latency(report.latency.p50_ms),
                        display_latency(report.latency.p95_ms),
                        display_latency(report.latency.p99_ms),
                        display_latency(report.latency.maximum_ms),
                        report.latency.failures,
                        report.latency.timeouts
                    );
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

fn display_latency(value: Option<f64>) -> String {
    value.map_or_else(|| "n/a".into(), |value| format!("{value:.3}"))
}

fn default_socket_path() -> PathBuf {
    dirs_next::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("prometheus")
        .join("run")
        .join("sovereign-sync.sock")
}

fn default_tcp_token_path(config_path: &std::path::Path) -> PathBuf {
    config_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("tcp-token")
}
