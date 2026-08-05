//! forge — CLI for the forge-rs code enrichment engine.
//!
//! Commands:
//!   forge enrich <task-path>                          — enrich an OpenSpec task
//!   forge reflect <iteration-id>                      — process iteration into Karpathy loop
//!   forge drift [--language <lang>]                   — report skill drift
//!   forge validate <file> --language <l>              — check against constitution
//!   forge mcp [--port 8943]                           — start MCP server
//!   forge init                                        — scaffold .forge/ in current project
//!   forge constitution edit <language>                — open constitution file in $EDITOR
//!   forge skill add <name>                            — pull a skill from the registry
//!   forge skill list                                  — list available skills
//!   forge package-librefang <agent-dir> [--no-build] — package WASM agent as .lf-skill.zip

#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

use anyhow::{Context as _, Result};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "forge",
    version,
    about = "forge-rs: code enrichment for Prometheus AGS"
)]
struct Cli {
    /// Project root (default: current directory)
    #[arg(long, global = true, default_value = ".")]
    project_root: PathBuf,

    /// Prometheus skill pack root (default: auto-detected)
    #[arg(long, global = true)]
    skills_root: Option<PathBuf>,

    /// prometheus-knowledge MCP URL (optional — falls back to pk CLI)
    #[arg(long, global = true, env = "PK_MCP_URL")]
    pk_mcp_url: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Enrich an OpenSpec task with skills, constitution, and Karpathy context
    Enrich {
        /// Path to OpenSpec task folder or tasks.md file
        task_path: PathBuf,
    },

    /// Process a completed iteration into the Karpathy learning loop
    Reflect {
        /// Task ID or iteration ID
        iteration_id: String,
    },

    /// Report skill drift across recent iterations
    Drift {
        /// Filter by language
        #[arg(long)]
        language: Option<String>,
    },

    /// Check a file against the active language constitution
    Validate {
        /// File to validate
        file: PathBuf,
        /// Language (rust, typescript, react, flutter, go, python, tauri)
        #[arg(long)]
        language: String,
    },

    /// Start the forge MCP server
    Mcp {
        /// Port to listen on
        #[arg(long, default_value = "8943")]
        port: u16,
        /// Address to bind to (default: 127.0.0.1; use 0.0.0.0 with caution)
        #[arg(long, default_value = "127.0.0.1")]
        bind: String,
        /// Serve without bearer auth. Loopback binds ONLY — the server refuses
        /// to start if combined with a non-loopback --bind, because that would
        /// publish an unauthenticated file-reading/writing API to the network.
        #[arg(long)]
        no_auth: bool,
    },

    /// Scaffold .forge/ in the current project
    Init,

    /// Show forge configuration and service status
    Status,

    /// Manage forge skills
    Skill {
        #[command(subcommand)]
        action: SkillAction,
    },

    /// Edit a language constitution
    Constitution {
        /// Language to edit (rust, typescript, react, flutter, go, python, tauri)
        language: String,
    },

    /// Package an agent directory as a LibreFang WASM skill zip
    PackageLibrefang {
        /// Path to the agent directory (must contain skill.toml)
        agent_dir: PathBuf,

        /// Skip `cargo build` — assume the .wasm binary already exists
        #[arg(long)]
        no_build: bool,

        /// Output path for the zip (default: ./<name>-<version>.lf-skill.zip in cwd)
        #[arg(long, short)]
        output: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum SkillAction {
    /// List available skills
    List {
        #[arg(long)]
        language: Option<String>,
    },
    /// Add a skill from the registry
    Add { name: String },
    /// Sync skills from the skill pack
    Sync,
}

fn resolve_skills_root(provided: Option<PathBuf>) -> PathBuf {
    if let Some(p) = provided {
        return p;
    }
    // Try to auto-detect the prometheus-skill-pack skills/ directory
    // by walking up from the current directory
    let mut current = std::env::current_dir().unwrap_or_default();
    loop {
        let candidate = current.join("skills");
        if candidate.join("rust").exists() || candidate.join("react").exists() {
            return candidate;
        }
        if !current.pop() {
            break;
        }
    }
    // Fall back to ~/.prometheus/skill-pack/skills
    dirs::home_dir()
        .unwrap_or_default()
        .join(".prometheus")
        .join("skill-pack")
        .join("skills")
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env().add_directive("forge=info".parse()?),
        )
        .init();

    let cli = Cli::parse();
    let skills_root = resolve_skills_root(cli.skills_root);

    match cli.command {
        Commands::Enrich { task_path } => {
            let enricher =
                forge_enricher::Enricher::new(&skills_root, &cli.project_root, cli.pk_mcp_url)?;
            let ctx = enricher.enrich(&task_path).await?;

            println!("✅ Enriched: {}", ctx.task_description);
            println!("   Language: {:?}", ctx.language);
            println!("   Skills applied ({}):", ctx.applied_skills.len());
            for skill in &ctx.applied_skills {
                println!("     - {}", skill);
            }
            if ctx.karpathy_focus.is_some() {
                println!("   Karpathy focus: ✅ loaded from prometheus-knowledge");
            }
            println!(
                "   Context written to: .forge/enriched/{}.context.md",
                task_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default()
            );
        }

        Commands::Reflect { iteration_id } => {
            let reflector = forge_reflect::Reflector::new(&cli.project_root);
            let record = reflector.reflect(&iteration_id).await?;

            println!("✅ Reflected iteration: {}", record.task_id);
            println!("   Language: {:?}", record.language);
            println!("   Skills tracked: {}", record.applied_skills.len());
            println!("   Drift records: {}", record.skill_drift.len());
            println!("   Ingested to prometheus-knowledge ✅");
        }

        Commands::Drift { language: _ } => {
            let drift_dir = cli.project_root.join(".forge").join("memory").join("drift");
            if !drift_dir.exists() {
                println!("No drift data yet. Run `forge reflect` after completing iterations.");
                return Ok(());
            }
            println!("Drift reports in: {}", drift_dir.display());
            for entry in std::fs::read_dir(&drift_dir)? {
                let path = entry?.path();
                if let Some(name) = path.file_name() {
                    println!("  - {}", name.to_string_lossy());
                }
            }
        }

        Commands::Validate { file, language } => {
            use std::str::FromStr as _;
            let lang = forge_core::Language::from_str(&language)
                .with_context(|| format!("invalid language '{}'", language))?;
            let content = std::fs::read_to_string(&file)
                .with_context(|| format!("reading {}", file.display()))?;
            let constitution_dir = cli.project_root.join(".forge").join("constitution");
            let constitutions = forge_enricher::load_constitutions(&constitution_dir)?;

            match constitutions.get(&lang) {
                None => {
                    println!(
                        "No constitution found for {} at {}. Run `forge init` to scaffold one.",
                        language,
                        constitution_dir.display()
                    );
                }
                Some(constitution) => {
                    let warnings = forge_enricher::check_constitution(constitution, &content);
                    if warnings.is_empty() {
                        println!(
                            "✅ {} passed {} constitution (no violations).",
                            file.display(),
                            language
                        );
                    } else {
                        let errors: Vec<_> = warnings
                            .iter()
                            .filter(|w| matches!(w.severity, forge_core::Severity::Error))
                            .collect();
                        for w in &warnings {
                            let level = format!("{:?}", w.severity).to_uppercase();
                            println!("[{}] {} — {}", level, w.rule, w.violation);
                        }
                        println!("\n{} violation(s) found.", warnings.len());
                        if !errors.is_empty() {
                            std::process::exit(1);
                        }
                    }
                }
            }
        }

        Commands::Mcp {
            port,
            bind,
            no_auth,
        } => {
            let server = forge_mcp::ForgeServer::with_bind_addr(
                port,
                &bind,
                &skills_root,
                &cli.project_root,
                cli.pk_mcp_url,
            )
            .with_no_auth(no_auth);
            println!("forge-mcp starting on {}:{}...", bind, port);
            println!("   MCP endpoint: http://{}:{}/mcp", bind, port);
            println!("   Health:       http://{}:{}/health", bind, port);
            if no_auth {
                println!("   Auth:         DISABLED (--no-auth, loopback only)");
            }
            server.run().await?;
        }

        Commands::Init => {
            scaffold_forge_dir(&cli.project_root)?;
            println!("✅ Initialized .forge/ in {}", cli.project_root.display());
        }

        Commands::Status => {
            let forge_dir = cli.project_root.join(".forge");

            println!("forge status\n");

            let constitution_dir = forge_dir.join("constitution");
            let constitution_count = if constitution_dir.exists() {
                std::fs::read_dir(&constitution_dir)
                    .map(|d| d.flatten().count())
                    .unwrap_or(0)
            } else {
                0
            };
            println!(
                "Constitutions:  {} file(s) in {}",
                constitution_count,
                constitution_dir.display()
            );

            let drift_dir = forge_dir.join("memory").join("drift");
            let drift_count = if drift_dir.exists() {
                std::fs::read_dir(&drift_dir)
                    .map(|d| d.flatten().count())
                    .unwrap_or(0)
            } else {
                0
            };
            println!("Drift reports:  {} file(s)", drift_count);

            match &cli.pk_mcp_url {
                Some(url) => println!("PK MCP URL:     {} [configured]", url),
                None => {
                    println!("PK MCP URL:     [not configured] — optimise/generate/evolve gated")
                }
            }

            println!("\nActive features:");
            println!("  enrich      YES — forge enrich <task>");
            println!("  reflect     YES — forge reflect <iteration-id>");
            println!("  validate    YES — forge validate <file> --language <lang>");
            println!("  drift       YES — forge drift");
            println!("  mcp         YES — forge mcp [--port 8943]");
            println!("  [EXPERIMENTAL] optimize  — requires --pk-mcp-url");
            println!("  [EXPERIMENTAL] generate  — requires --pk-mcp-url");
            println!("  [EXPERIMENTAL] evolve    — requires --pk-mcp-url");
        }

        Commands::Skill { action } => {
            match action {
                SkillAction::List { language } => {
                    println!("Skills in {}:", skills_root.display());
                    for entry in std::fs::read_dir(&skills_root)? {
                        let path = entry?.path();
                        if path.is_dir() {
                            let lang = path.file_name().unwrap().to_string_lossy().to_string();
                            if let Some(ref filter) = language {
                                if &lang != filter {
                                    continue;
                                }
                            }
                            println!("  {}:", lang);
                            for skill in std::fs::read_dir(&path)? {
                                let skill_path = skill?.path();
                                if skill_path.is_dir() {
                                    println!(
                                        "    - {}",
                                        skill_path.file_name().unwrap().to_string_lossy()
                                    );
                                }
                            }
                        }
                    }
                }
                SkillAction::Add { name } => {
                    println!(
                        "Add skill '{}' — skill registry pull not yet implemented.",
                        name
                    );
                    println!(
                        "Manually copy skill to {}/<language>/{}/",
                        skills_root.display(),
                        name
                    );
                }
                SkillAction::Sync => {
                    println!(
                        "Syncing skills from skill pack at {}...",
                        skills_root.display()
                    );
                    println!("✅ Skills are loaded from the skill pack root on every `forge enrich` call.");
                }
            }
        }

        Commands::Constitution { language } => {
            let constitution_path = cli
                .project_root
                .join(".forge")
                .join("constitution")
                .join(format!("{}.toml", language));

            if !constitution_path.exists() {
                scaffold_constitution(&constitution_path, &language)?;
                println!("Created: {}", constitution_path.display());
            }

            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
            std::process::Command::new(&editor)
                .arg(&constitution_path)
                .status()?;
        }

        Commands::PackageLibrefang {
            agent_dir,
            no_build,
            output,
        } => {
            package_librefang(&agent_dir, no_build, output)?;
        }
    }

    Ok(())
}

fn package_librefang(agent_dir: &PathBuf, no_build: bool, output: Option<PathBuf>) -> Result<()> {
    use std::io::Write as _;

    // 1. Read and parse skill.toml
    let manifest_path = agent_dir.join("skill.toml");
    let manifest_toml = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("Cannot read {}", manifest_path.display()))?;
    let manifest: toml::Value = toml::from_str(&manifest_toml)
        .with_context(|| format!("Invalid TOML in {}", manifest_path.display()))?;

    let skill = manifest
        .get("skill")
        .context("[skill] table missing from skill.toml")?;
    let name = skill
        .get("name")
        .and_then(|v| v.as_str())
        .context("skill.name missing from [skill]")?
        .to_string();
    let version = skill
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("0.1.0")
        .to_string();
    let entry = manifest
        .get("runtime")
        .and_then(|r| r.get("entry"))
        .and_then(|e| e.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{name}.wasm"));

    // 2. Optionally build the WASM binary
    if !no_build {
        tracing::info!("Running cargo build --release --target wasm32-unknown-unknown");
        let status = std::process::Command::new("cargo")
            .args(["build", "--release", "--target", "wasm32-unknown-unknown"])
            .current_dir(agent_dir)
            .status()
            .context("Failed to spawn `cargo build`")?;
        anyhow::ensure!(
            status.success(),
            "`cargo build --release --target wasm32-unknown-unknown` failed in {}",
            agent_dir.display()
        );
    }

    // 3. Locate the WASM binary
    let wasm_path = agent_dir
        .join("target/wasm32-unknown-unknown/release")
        .join(&entry);
    anyhow::ensure!(
        wasm_path.exists(),
        "WASM binary not found: {}\nRun `cargo build --release --target wasm32-unknown-unknown` first, or remove --no-build.",
        wasm_path.display()
    );
    let wasm_bytes = std::fs::read(&wasm_path)
        .with_context(|| format!("Cannot read {}", wasm_path.display()))?;

    // 4. Write the zip
    let zip_path =
        output.unwrap_or_else(|| PathBuf::from(format!("{name}-{version}.lf-skill.zip")));
    let zip_file = std::fs::File::create(&zip_path)
        .with_context(|| format!("Cannot create {}", zip_path.display()))?;
    let mut zip = zip::ZipWriter::new(zip_file);
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("skill.toml", opts)?;
    zip.write_all(manifest_toml.as_bytes())?;

    zip.start_file(&entry, opts)?;
    zip.write_all(&wasm_bytes)?;

    zip.finish()?;

    println!("✅ Packaged: {}", zip_path.display());
    println!("   Skill:    {} v{}", name, version);
    println!("   WASM:     {} ({} KB)", entry, wasm_bytes.len() / 1024);
    println!(
        "   Install:  librefang skill install {}",
        zip_path.display()
    );
    Ok(())
}

fn scaffold_forge_dir(project_root: &Path) -> Result<()> {
    let dirs = [
        ".forge/constitution",
        ".forge/enriched",
        ".forge/memory/iterations",
        ".forge/memory/drift",
        ".forge/skills",
    ];
    for dir in &dirs {
        std::fs::create_dir_all(project_root.join(dir))?;
    }

    // Write default Rust constitution
    let rust_const_path = project_root.join(".forge/constitution/rust.toml");
    if !rust_const_path.exists() {
        scaffold_constitution(&rust_const_path, "rust")?;
    }

    Ok(())
}

fn scaffold_constitution(path: &PathBuf, language: &str) -> Result<()> {
    let content = match language {
        "rust" => include_str!("../../../constitution-templates/rust.toml"),
        "typescript" | "react" => include_str!("../../../constitution-templates/typescript.toml"),
        "flutter" => include_str!("../../../constitution-templates/flutter.toml"),
        "go" => include_str!("../../../constitution-templates/go.toml"),
        "python" => include_str!("../../../constitution-templates/python.toml"),
        "tauri" => include_str!("../../../constitution-templates/tauri.toml"),
        _ => "[language]\nname = \"unknown\"\n",
    };
    std::fs::create_dir_all(path.parent().unwrap())?;
    std::fs::write(path, content)?;
    Ok(())
}
