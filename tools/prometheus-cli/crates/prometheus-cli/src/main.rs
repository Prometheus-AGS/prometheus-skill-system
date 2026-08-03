use anyhow::Result;
use clap::{Parser, Subcommand};
use std::collections::BTreeMap;

mod commands;

#[derive(Parser)]
#[command(name = "prometheus")]
#[command(
    about = "Self-improving skill execution engine — manage, optimize, and learn from AI skills"
)]
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
    Doctor {
        /// Emit machine-readable JSON output
        #[arg(long)]
        json: bool,
        /// Restrict output to a specific check id or group
        #[arg(long)]
        check: Option<String>,
        /// Exclude a check id, group, or managed service scope (repeatable)
        #[arg(long)]
        exclude: Vec<String>,
        /// Plan or apply safe repairs
        #[arg(long, conflicts_with = "refresh")]
        fix: bool,
        /// Refresh managed binaries, services, and catalogs from pinned source
        #[arg(long)]
        refresh: bool,
        /// Show the repair or refresh plan without mutating
        #[arg(long)]
        dry_run: bool,
        /// Suppress prompts for safe reversible actions
        #[arg(long)]
        yes: bool,
    },

    /// Show current status: Skills.toml, KBD waypoint, evolver state
    Status {
        /// Project path
        #[arg(short, long, default_value = ".")]
        path: String,
    },

    /// Control and audit the canonical KBD runtime
    Kbd {
        /// Project path (walks upward to find .kbd-orchestrator)
        #[arg(short, long, default_value = ".")]
        path: String,
        #[command(subcommand)]
        action: KbdAction,
    },

    /// Evaluate and budget the portable skill instruction plane
    Skill {
        /// Skill tree to inspect
        #[arg(long, default_value = "skills")]
        path: String,
        #[command(subcommand)]
        action: SkillAction,
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
        /// Capture current session traces from all detected platforms
        #[arg(long)]
        capture_session: bool,
        /// Seed traces from Claude Code session history (curated import)
        #[arg(long)]
        seed: bool,
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

    /// Inspect the durable Karpathy learning queue and worker
    Learning {
        #[command(subcommand)]
        action: LearningAction,
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

    /// Manage Cedar governance policies
    Policy {
        #[command(subcommand)]
        action: PolicyAction,
    },

    /// Detect and correct sycophantic patterns
    Sycophancy {
        #[command(subcommand)]
        action: SycophancyAction,
    },

    /// Detect machine setup gaps and interactively install missing components
    Setup {
        /// Assume yes to all install prompts (CI/automation mode)
        #[arg(long)]
        non_interactive: bool,
        /// Show what would be installed without executing
        #[arg(long)]
        dry_run: bool,
        /// Report status only — no install prompts
        #[arg(long)]
        check: bool,
        /// Force rebuild of all binary components from source (implies --non-interactive)
        #[arg(long)]
        rebuild: bool,
    },
}

#[derive(Subcommand)]
enum PolicyAction {
    /// Display currently loaded Cedar policies
    Show,
    /// Validate Cedar policy syntax
    Validate,
    /// Check a specific mutation against current policies
    Check {
        /// Agent ID
        #[arg(short, long, default_value = "pmpo-optimizer")]
        agent: String,
        /// Operation: skill.mutate, skill.generate, skill.promote, trace.capture
        #[arg(short, long)]
        operation: String,
        /// Skill ID
        #[arg(short, long)]
        skill: String,
        /// Environment: development, staging, production
        #[arg(short, long, default_value = "development")]
        environment: String,
    },
}

#[derive(Subcommand)]
enum SycophancyAction {
    /// Detect sycophantic patterns in a file
    Detect {
        /// File to scan (use "-" for stdin)
        file: String,
        /// Strictness level
        #[arg(short, long, default_value = "standard")]
        strictness: String,
    },
    /// Return numeric sycophancy score only (0.0-1.0)
    Score {
        /// File to scan (use "-" for stdin)
        file: String,
    },
    /// Detect patterns and provide correction guidance
    Correct {
        /// File to correct (use "-" for stdin)
        file: String,
        /// Strictness level
        #[arg(short, long, default_value = "standard")]
        strictness: String,
    },
}

#[derive(Subcommand)]
enum SkillAction {
    /// Validate Agent Skills frontmatter, limits, duplicate names, and collisions
    Lint {
        #[arg(long)]
        json: bool,
    },
    /// Measure discovery inventory against a captured harness character budget
    Budget {
        #[arg(long)]
        harness: String,
        #[arg(long)]
        budget_chars: Option<usize>,
        #[arg(long)]
        json: bool,
    },
    /// Validate or grade the critical-skill activation corpus
    Eval {
        #[arg(long)]
        harness: String,
        #[arg(long, default_value = "evals/skill-activation/critical-30.json")]
        corpus: String,
        #[arg(long)]
        trace: Option<String>,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum KbdAction {
    /// Show lifecycle, revision, plan, checkpoint, and workflow state
    Status {
        #[arg(long)]
        json: bool,
    },
    /// List all registered projects and replicas on this machine
    Projects {
        #[arg(long)]
        json: bool,
    },
    /// Register a checkout that already declares .prometheus/project.json
    Register { path: String },
    /// List replicas for a project UUID (defaults to the current project)
    Replicas {
        #[arg(long)]
        project_id: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Adopt an existing checkout into a registered project identity
    Adopt {
        path: String,
        #[arg(long = "into")]
        into_project_id: String,
        /// Apply after printing the same evidence returned by dry-run
        #[arg(long)]
        apply: bool,
    },
    /// List deterministic CRDT conflicts and their candidate events
    Conflicts {
        #[arg(long)]
        json: bool,
    },
    /// Append an operator-signed conflict adjudication
    Resolve {
        conflict_id: String,
        #[arg(long = "winner")]
        winner_event_id: String,
        #[arg(long)]
        reason: String,
    },
    /// Inspect or mutate CRDT work claims
    Claim {
        #[command(subcommand)]
        action: KbdClaimAction,
    },
    /// Inspect or record parent-owned Git submodule pins
    Submodules {
        /// Scan gitlinks and append signed pin events to the parent project
        #[arg(long)]
        scan: bool,
        #[arg(long)]
        json: bool,
    },
    /// Gracefully checkpoint and pause the run
    Pause {
        #[arg(long)]
        reason: String,
    },
    /// Record a course correction as a new immutable plan revision
    Revise {
        #[arg(long)]
        reason: String,
        #[arg(long)]
        exact_next_work: Option<String>,
    },
    /// Resume a paused run at a validated plan revision
    Resume {
        #[arg(long)]
        plan_revision: Option<u64>,
    },
    /// Gracefully terminate the run
    Cancel {
        #[arg(long)]
        reason: String,
    },
    /// Show immutable events
    Audit {
        #[arg(long)]
        since: Option<String>,
        #[arg(long)]
        json: bool,
        /// Export the full converged chain to refs/heads/audit/kbd without checkout
        #[arg(long, conflicts_with_all = ["since", "json"])]
        export_git: bool,
    },
    /// Follow new events until interrupted
    Watch,
    /// Inventory or apply legacy-state migration
    Migrate {
        #[arg(long, conflicts_with = "apply")]
        check: bool,
        #[arg(long)]
        apply: bool,
    },
    /// Record shadow/canary evidence and enforce promotion thresholds
    Rollout {
        #[command(subcommand)]
        action: KbdRolloutAction,
    },
    /// Create, activate, or transition a canonical phase
    Phase {
        #[command(subcommand)]
        action: KbdPhaseAction,
    },
    /// Enter or transition a canonical stage
    Stage {
        #[command(subcommand)]
        action: KbdStageAction,
    },
    /// Register or transition a canonical change
    Change {
        #[command(subcommand)]
        action: KbdChangeAction,
    },
    /// Register or transition a canonical task
    Task {
        #[command(subcommand)]
        action: KbdTaskAction,
    },
    /// Set an independent completion dimension
    Completion {
        #[command(subcommand)]
        action: KbdCompletionAction,
    },
    /// Record an immutable architectural or plan decision
    Decision {
        #[command(subcommand)]
        action: KbdDecisionAction,
    },
    /// Record or clear a blocker
    Blocker {
        #[command(subcommand)]
        action: KbdBlockerAction,
    },
}

#[derive(Clone, clap::ValueEnum)]
enum KbdClaimMode {
    Shared,
    Exclusive,
}

impl From<KbdClaimMode> for kbd_runtime::ClaimMode {
    fn from(value: KbdClaimMode) -> Self {
        match value {
            KbdClaimMode::Shared => Self::Shared,
            KbdClaimMode::Exclusive => Self::Exclusive,
        }
    }
}

#[derive(Subcommand)]
enum KbdClaimAction {
    /// List claims and any claim conflicts
    List {
        #[arg(long)]
        json: bool,
    },
    /// Acquire a shared or exclusive work-scope claim
    Acquire {
        scope: String,
        #[arg(long, value_enum, default_value = "exclusive")]
        mode: KbdClaimMode,
        #[arg(long, default_value_t = 900)]
        ttl: u64,
        #[arg(long)]
        holder: Option<String>,
    },
    /// Renew a claim with a larger monotonic token
    Renew {
        claim_id: String,
        #[arg(long, default_value_t = 900)]
        ttl: u64,
    },
    /// Release a claim
    Release { claim_id: String },
}

#[derive(Subcommand)]
enum KbdRolloutAction {
    /// Show the current rollout stage and next promotion gate
    Status,
    /// Add one idempotent, non-authoritative observation
    Observe {
        #[arg(long)]
        observation_id: String,
        #[arg(long)]
        observed_at: Option<String>,
        #[arg(long, default_value_t = 0)]
        real_mutations: u64,
        #[arg(long, default_value_t = 0)]
        synthetic_replay_mutations: u64,
        #[arg(long, default_value_t = 0)]
        unexplained_projection_mismatches: u64,
        #[arg(long)]
        harness: Option<String>,
        #[arg(long)]
        device: Option<String>,
        #[arg(long, default_value_t = 1)]
        voters: u64,
        #[arg(long)]
        failed: bool,
    },
    /// Advance only when every acceptance threshold for the current stage passes
    Promote,
}

#[derive(clap::Args, Clone)]
struct KbdMutationArgs {
    #[arg(long)]
    command_id: String,
}

#[derive(Clone, clap::ValueEnum)]
enum KbdWorkStatus {
    Pending,
    InProgress,
    Blocked,
    Complete,
    Cancelled,
}

impl From<KbdWorkStatus> for kbd_runtime::WorkStatus {
    fn from(value: KbdWorkStatus) -> Self {
        match value {
            KbdWorkStatus::Pending => Self::Pending,
            KbdWorkStatus::InProgress => Self::InProgress,
            KbdWorkStatus::Blocked => Self::Blocked,
            KbdWorkStatus::Complete => Self::Complete,
            KbdWorkStatus::Cancelled => Self::Cancelled,
        }
    }
}

#[derive(Clone, clap::ValueEnum)]
enum KbdCompletionDimension {
    Implementation,
    Evidence,
    Certification,
    Publication,
}

impl From<KbdCompletionDimension> for kbd_runtime::CompletionDimension {
    fn from(value: KbdCompletionDimension) -> Self {
        match value {
            KbdCompletionDimension::Implementation => Self::Implementation,
            KbdCompletionDimension::Evidence => Self::Evidence,
            KbdCompletionDimension::Certification => Self::Certification,
            KbdCompletionDimension::Publication => Self::Publication,
        }
    }
}

#[derive(Subcommand)]
enum KbdPhaseAction {
    Create {
        #[command(flatten)]
        mutation: KbdMutationArgs,
        #[arg(long)]
        id: String,
        #[arg(long)]
        slug: Option<String>,
        #[arg(long)]
        title: String,
        #[arg(long)]
        parent: Option<String>,
    },
    Activate {
        #[command(flatten)]
        mutation: KbdMutationArgs,
        #[arg(long)]
        id: String,
        #[arg(long = "ancestor")]
        ancestors: Vec<String>,
        #[arg(long)]
        exact_next_work: Option<String>,
    },
    Transition {
        #[command(flatten)]
        mutation: KbdMutationArgs,
        #[arg(long)]
        id: String,
        #[arg(long, value_enum)]
        status: KbdWorkStatus,
    },
}

#[derive(Subcommand)]
enum KbdStageAction {
    Enter {
        #[command(flatten)]
        mutation: KbdMutationArgs,
        #[arg(long)]
        phase: String,
        #[arg(long)]
        id: String,
        #[arg(long)]
        title: String,
        #[arg(long, default_value_t = 0)]
        sequence: u64,
    },
    Transition {
        #[command(flatten)]
        mutation: KbdMutationArgs,
        #[arg(long)]
        phase: String,
        #[arg(long)]
        id: String,
        #[arg(long, value_enum)]
        status: KbdWorkStatus,
    },
}

#[derive(Subcommand)]
enum KbdChangeAction {
    Register {
        #[command(flatten)]
        mutation: KbdMutationArgs,
        #[arg(long)]
        phase: String,
        #[arg(long)]
        id: String,
        #[arg(long)]
        title: String,
        #[arg(long, default_value_t = 0)]
        sequence: u64,
    },
    Transition {
        #[command(flatten)]
        mutation: KbdMutationArgs,
        #[arg(long)]
        phase: String,
        #[arg(long)]
        id: String,
        #[arg(long, value_enum)]
        status: KbdWorkStatus,
    },
}

#[derive(Subcommand)]
enum KbdTaskAction {
    Register {
        #[command(flatten)]
        mutation: KbdMutationArgs,
        #[arg(long)]
        phase: String,
        #[arg(long)]
        change: String,
        #[arg(long)]
        id: String,
        #[arg(long)]
        title: String,
        #[arg(long, default_value_t = 0)]
        sequence: u64,
    },
    Transition {
        #[command(flatten)]
        mutation: KbdMutationArgs,
        #[arg(long)]
        phase: String,
        #[arg(long)]
        change: String,
        #[arg(long)]
        id: String,
        #[arg(long, value_enum)]
        status: KbdWorkStatus,
        #[arg(long)]
        summary: Option<String>,
    },
}

#[derive(Subcommand)]
enum KbdCompletionAction {
    Set {
        #[command(flatten)]
        mutation: KbdMutationArgs,
        #[arg(long, value_enum)]
        dimension: KbdCompletionDimension,
        #[arg(long)]
        completed: u64,
        #[arg(long)]
        total: u64,
        #[arg(long, value_enum)]
        status: KbdWorkStatus,
        #[arg(long)]
        summary: Option<String>,
        #[arg(long)]
        blocker: Vec<String>,
    },
}

#[derive(Subcommand)]
enum KbdDecisionAction {
    Record {
        #[command(flatten)]
        mutation: KbdMutationArgs,
        #[arg(long)]
        id: String,
        #[arg(long)]
        summary: String,
        #[arg(long)]
        supersedes: Option<String>,
    },
}

#[derive(Subcommand)]
enum KbdBlockerAction {
    Record {
        #[command(flatten)]
        mutation: KbdMutationArgs,
        #[arg(long)]
        id: String,
        #[arg(long)]
        summary: String,
    },
    Clear {
        #[command(flatten)]
        mutation: KbdMutationArgs,
        #[arg(long)]
        id: String,
        #[arg(long)]
        resolution: String,
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
    /// Install surreal-memory server
    Install {
        /// Dry run — show what would happen
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
enum LearningAction {
    /// Show queue, retry, dead-letter, and memory-delivery status
    Status {
        #[arg(long, default_value_t = false)]
        json: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "warn".into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Install {
            source,
            agent,
            local,
            no_symlink,
            plugin,
        } => commands::install::run(&source, agent.as_deref(), local, no_symlink, plugin).await,
        Commands::Uninstall { name, agent } => commands::uninstall::run(&name, agent.as_deref()),
        Commands::List {
            all,
            global,
            project,
            verbose,
        } => commands::list::run(all, global, project, verbose),
        Commands::Search { query, limit } => commands::search::run(&query, limit).await,
        Commands::Audit { path } => commands::audit::run(path.as_deref()),
        Commands::Verify { update } => commands::verify::run(update),
        Commands::Doctor {
            json,
            check,
            exclude,
            fix,
            refresh,
            dry_run,
            yes,
        } => {
            commands::doctor::run(commands::doctor::DoctorOptions {
                json,
                check,
                exclude,
                fix,
                refresh,
                dry_run,
                yes,
            })
            .await
        }
        Commands::Status { path } => commands::status::run(&path),
        Commands::Kbd { path, action } => {
            let typed_command = |mutation: KbdMutationArgs, command: kbd_runtime::CommandKind| {
                commands::kbd::Action::Command {
                    command_id: mutation.command_id,
                    command,
                }
            };
            let action = match action {
                KbdAction::Status { json } => commands::kbd::Action::Status { json },
                KbdAction::Projects { json } => commands::kbd::Action::Projects { json },
                KbdAction::Register { path } => commands::kbd::Action::Register { path },
                KbdAction::Replicas { project_id, json } => {
                    commands::kbd::Action::Replicas { project_id, json }
                }
                KbdAction::Adopt {
                    path,
                    into_project_id,
                    apply,
                } => commands::kbd::Action::Adopt {
                    path,
                    into_project_id,
                    apply,
                },
                KbdAction::Conflicts { json } => commands::kbd::Action::Conflicts { json },
                KbdAction::Resolve {
                    conflict_id,
                    winner_event_id,
                    reason,
                } => commands::kbd::Action::Resolve {
                    conflict_id,
                    winner_event_id,
                    reason,
                },
                KbdAction::Claim { action } => match action {
                    KbdClaimAction::List { json } => commands::kbd::Action::Claims { json },
                    KbdClaimAction::Acquire {
                        scope,
                        mode,
                        ttl,
                        holder,
                    } => commands::kbd::Action::ClaimAcquire {
                        scope,
                        mode: mode.into(),
                        ttl_seconds: ttl,
                        holder_id: holder,
                    },
                    KbdClaimAction::Renew { claim_id, ttl } => commands::kbd::Action::ClaimRenew {
                        claim_id,
                        ttl_seconds: ttl,
                    },
                    KbdClaimAction::Release { claim_id } => {
                        commands::kbd::Action::ClaimRelease { claim_id }
                    }
                },
                KbdAction::Submodules { scan, json } => {
                    commands::kbd::Action::Submodules { scan, json }
                }
                KbdAction::Pause { reason } => commands::kbd::Action::Pause { reason },
                KbdAction::Revise {
                    reason,
                    exact_next_work,
                } => commands::kbd::Action::Revise {
                    reason,
                    exact_next_work,
                },
                KbdAction::Resume { plan_revision } => {
                    commands::kbd::Action::Resume { plan_revision }
                }
                KbdAction::Cancel { reason } => commands::kbd::Action::Cancel { reason },
                KbdAction::Audit {
                    since,
                    json,
                    export_git,
                } => commands::kbd::Action::Audit {
                    since,
                    json,
                    export_git,
                },
                KbdAction::Watch => commands::kbd::Action::Watch,
                KbdAction::Migrate { check, apply } => {
                    commands::kbd::Action::Migrate { check, apply }
                }
                KbdAction::Rollout { action } => match action {
                    KbdRolloutAction::Status => commands::kbd::Action::RolloutStatus,
                    KbdRolloutAction::Observe {
                        observation_id,
                        observed_at,
                        real_mutations,
                        synthetic_replay_mutations,
                        unexplained_projection_mismatches,
                        harness,
                        device,
                        voters,
                        failed,
                    } => commands::kbd::Action::RolloutObserve {
                        observation_id,
                        observed_at,
                        real_mutations,
                        synthetic_replay_mutations,
                        unexplained_projection_mismatches,
                        harness,
                        device,
                        voters,
                        successful: !failed,
                    },
                    KbdRolloutAction::Promote => commands::kbd::Action::RolloutPromote,
                },
                KbdAction::Phase { action } => match action {
                    KbdPhaseAction::Create {
                        mutation,
                        id,
                        slug,
                        title,
                        parent,
                    } => typed_command(
                        mutation,
                        kbd_runtime::CommandKind::PhaseDefine {
                            phase: kbd_runtime::Phase {
                                slug: slug.unwrap_or_else(|| id.clone()),
                                id,
                                title,
                                parent_phase_id: parent,
                                status: kbd_runtime::WorkStatus::Pending,
                                stages: BTreeMap::new(),
                                changes: BTreeMap::new(),
                                legacy_read_only: false,
                            },
                        },
                    ),
                    KbdPhaseAction::Activate {
                        mutation,
                        id,
                        mut ancestors,
                        exact_next_work,
                    } => {
                        ancestors.push(id.clone());
                        typed_command(
                            mutation,
                            kbd_runtime::CommandKind::ActivePathSet {
                                active_path: kbd_runtime::ActivePath {
                                    phase_path: ancestors,
                                    phase_id: Some(id),
                                    ..Default::default()
                                },
                                exact_next_work,
                            },
                        )
                    }
                    KbdPhaseAction::Transition {
                        mutation,
                        id,
                        status,
                    } => typed_command(
                        mutation,
                        kbd_runtime::CommandKind::PhaseTransition {
                            phase_id: id,
                            to: status.into(),
                        },
                    ),
                },
                KbdAction::Stage { action } => match action {
                    KbdStageAction::Enter {
                        mutation,
                        phase,
                        id,
                        title,
                        sequence,
                    } => typed_command(
                        mutation,
                        kbd_runtime::CommandKind::StageEnter {
                            phase_id: phase,
                            stage: kbd_runtime::Stage {
                                id,
                                title,
                                sequence,
                                status: kbd_runtime::WorkStatus::InProgress,
                            },
                        },
                    ),
                    KbdStageAction::Transition {
                        mutation,
                        phase,
                        id,
                        status,
                    } => typed_command(
                        mutation,
                        kbd_runtime::CommandKind::StageTransition {
                            phase_id: phase,
                            stage_id: id,
                            to: status.into(),
                        },
                    ),
                },
                KbdAction::Change { action } => match action {
                    KbdChangeAction::Register {
                        mutation,
                        phase,
                        id,
                        title,
                        sequence,
                    } => typed_command(
                        mutation,
                        kbd_runtime::CommandKind::ChangeRegister {
                            phase_id: phase,
                            change: kbd_runtime::Change {
                                id,
                                title,
                                sequence,
                                status: kbd_runtime::WorkStatus::Pending,
                                implementation_status: kbd_runtime::WorkStatus::Pending,
                                tasks: BTreeMap::new(),
                            },
                        },
                    ),
                    KbdChangeAction::Transition {
                        mutation,
                        phase,
                        id,
                        status,
                    } => typed_command(
                        mutation,
                        kbd_runtime::CommandKind::ChangeTransition {
                            phase_id: phase,
                            change_id: id,
                            to: status.into(),
                        },
                    ),
                },
                KbdAction::Task { action } => match action {
                    KbdTaskAction::Register {
                        mutation,
                        phase,
                        change,
                        id,
                        title,
                        sequence,
                    } => typed_command(
                        mutation,
                        kbd_runtime::CommandKind::TaskRegister {
                            phase_id: phase,
                            change_id: change,
                            task: kbd_runtime::Task {
                                id,
                                title,
                                sequence,
                                status: kbd_runtime::WorkStatus::Pending,
                                summary: None,
                            },
                        },
                    ),
                    KbdTaskAction::Transition {
                        mutation,
                        phase,
                        change,
                        id,
                        status,
                        summary,
                    } => typed_command(
                        mutation,
                        kbd_runtime::CommandKind::TaskTransition {
                            phase_id: phase,
                            change_id: change,
                            task_id: id,
                            to: status.into(),
                            summary,
                        },
                    ),
                },
                KbdAction::Completion { action } => match action {
                    KbdCompletionAction::Set {
                        mutation,
                        dimension,
                        completed,
                        total,
                        status,
                        summary,
                        blocker,
                    } => typed_command(
                        mutation,
                        kbd_runtime::CommandKind::CompletionSet {
                            dimension: dimension.into(),
                            completion: kbd_runtime::Completion {
                                completed,
                                total,
                                status: status.into(),
                                summary,
                                blockers: blocker,
                            },
                        },
                    ),
                },
                KbdAction::Decision { action } => match action {
                    KbdDecisionAction::Record {
                        mutation,
                        id,
                        summary,
                        supersedes,
                    } => typed_command(
                        mutation,
                        kbd_runtime::CommandKind::DecisionRecord {
                            decision: kbd_runtime::Decision {
                                id,
                                summary,
                                plan_revision: 0,
                                supersedes,
                            },
                        },
                    ),
                },
                KbdAction::Blocker { action } => match action {
                    KbdBlockerAction::Record {
                        mutation,
                        id,
                        summary,
                    } => typed_command(
                        mutation,
                        kbd_runtime::CommandKind::BlockerRecord {
                            blocker: kbd_runtime::Blocker {
                                id,
                                summary,
                                resolved: false,
                                resolution: None,
                            },
                        },
                    ),
                    KbdBlockerAction::Clear {
                        mutation,
                        id,
                        resolution,
                    } => typed_command(
                        mutation,
                        kbd_runtime::CommandKind::BlockerClear {
                            blocker_id: id,
                            resolution,
                        },
                    ),
                },
            };
            commands::kbd::run(&path, action).await
        }
        Commands::Skill { path, action } => match action {
            SkillAction::Lint { json } => commands::skill::lint(std::path::Path::new(&path), json),
            SkillAction::Budget {
                harness,
                budget_chars,
                json,
            } => commands::skill::budget(std::path::Path::new(&path), &harness, budget_chars, json),
            SkillAction::Eval {
                harness,
                corpus,
                trace,
                json,
            } => commands::skill::eval(
                std::path::Path::new(&corpus),
                &harness,
                trace.as_deref().map(std::path::Path::new),
                json,
            ),
        },
        Commands::Generate { path, language } => {
            commands::generate::run(&path, language.as_deref())
        }
        Commands::Validate { path } => commands::validate::run(path.as_deref()),
        Commands::Build {
            service,
            overlay,
            gitops_path,
        } => commands::build::run(&gitops_path, &service, &overlay),
        Commands::Memory { action } => match action {
            MemoryAction::Ping => commands::memory::ping().await,
            MemoryAction::Stats => commands::memory::stats().await,
            MemoryAction::Search { query, r#type } => {
                commands::memory::search(&query, r#type.as_deref()).await
            }
            MemoryAction::Install { dry_run } => commands::memory::install(dry_run).await,
        },
        Commands::Evolve {
            name,
            domain,
            phase,
        } => commands::evolve::run(&name, &domain, phase.as_deref()),
        Commands::Learn {
            capture_session,
            seed,
            compile,
            lint,
            dry_run,
        } => commands::learn::run(capture_session, seed, compile, lint, dry_run).await,
        Commands::Learning { action } => match action {
            LearningAction::Status { json } => commands::learning::status(json),
        },
        Commands::Policy { action } => match action {
            PolicyAction::Show => commands::policy::show(),
            PolicyAction::Validate => commands::policy::validate(),
            PolicyAction::Check {
                agent,
                operation,
                skill,
                environment,
            } => commands::policy::check(&agent, &operation, &skill, &environment),
        },
        Commands::Optimize {
            skill,
            min_traces,
            dry_run,
        } => commands::optimize::run(&skill, min_traces, dry_run).await,
        Commands::Sycophancy { action } => match action {
            SycophancyAction::Detect { file, strictness } => {
                commands::sycophancy::detect(&file, &strictness)
            }
            SycophancyAction::Score { file } => commands::sycophancy::score(&file),
            SycophancyAction::Correct { file, strictness } => {
                commands::sycophancy::correct(&file, &strictness)
            }
        },
        Commands::Setup {
            non_interactive,
            dry_run,
            check,
            rebuild,
        } => commands::setup::run(non_interactive, dry_run, check, rebuild),
    }
}
