use anyhow::{anyhow, Context, Result};
use kbd_runtime::{
    registry::ProjectRegistry,
    rollout::{RolloutObservation, RolloutTracker},
    Actor, ActorKind, Checkpoint, CommandEnvelope, CommandKind, Event, LifecycleState, Runtime,
    RuntimeError, RuntimeState,
};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub enum Action {
    Status {
        json: bool,
    },
    Projects {
        json: bool,
    },
    Register {
        path: String,
    },
    Replicas {
        project_id: Option<String>,
        json: bool,
    },
    Adopt {
        path: String,
        into_project_id: String,
        apply: bool,
    },
    Pause {
        reason: String,
    },
    Revise {
        reason: String,
        exact_next_work: Option<String>,
    },
    Resume {
        plan_revision: Option<u64>,
    },
    Cancel {
        reason: String,
    },
    Audit {
        since: Option<String>,
        json: bool,
    },
    Watch,
    Migrate {
        check: bool,
        apply: bool,
    },
    RolloutStatus,
    RolloutObserve {
        observation_id: String,
        observed_at: Option<String>,
        real_mutations: u64,
        synthetic_replay_mutations: u64,
        unexplained_projection_mismatches: u64,
        harness: Option<String>,
        device: Option<String>,
        voters: u64,
        successful: bool,
    },
    RolloutPromote,
    Command {
        expected_revision: u64,
        command_id: String,
        command: CommandKind,
    },
}

pub async fn run(path: &str, action: Action) -> Result<()> {
    let action = match action {
        Action::Projects { json } => {
            let registry = ProjectRegistry::open();
            let document = registry.load()?;
            if json {
                println!("{}", serde_json::to_string_pretty(&document)?);
            } else {
                println!("Machine: {}", document.machine_id);
                for (path, replica) in document.replicas {
                    println!(
                        "{}  {}  {}  {:?}{}",
                        replica.project_id,
                        replica.replica_id,
                        path,
                        replica.kind,
                        if replica.read_only { "  read-only" } else { "" }
                    );
                }
            }
            return Ok(());
        }
        Action::Register { path } => {
            let outcome = ProjectRegistry::open().register_existing(&path)?;
            println!("{}", serde_json::to_string_pretty(&outcome)?);
            return Ok(());
        }
        Action::Replicas { project_id, json } => {
            let registry = ProjectRegistry::open();
            let project_id = match project_id {
                Some(project_id) => project_id,
                None => {
                    let root = find_manifest_project_root(Path::new(path))?;
                    fs::read(root.join(".prometheus/project.json"))
                        .context("read current project manifest")
                        .and_then(|bytes| {
                            serde_json::from_slice::<kbd_runtime::ProjectManifest>(&bytes)
                                .context("parse current project manifest")
                        })?
                        .project_id
                }
            };
            let replicas = registry.lookup_project(&project_id)?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "projectId": project_id,
                        "replicas": replicas
                    }))?
                );
            } else {
                for (path, replica) in replicas {
                    println!(
                        "{}  {}  {:?}{}",
                        replica.replica_id,
                        path.display(),
                        replica.kind,
                        if replica.read_only { "  read-only" } else { "" }
                    );
                }
            }
            return Ok(());
        }
        Action::Adopt {
            path,
            into_project_id,
            apply,
        } => {
            let registry = ProjectRegistry::open();
            let plan = registry.plan_adoption(&path, &into_project_id)?;
            if !apply {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "dryRun": true,
                        "plan": plan,
                        "applyCommand": format!(
                            "prometheus kbd adopt {:?} --into {} --apply",
                            path, into_project_id
                        )
                    }))?
                );
                return Ok(());
            }
            let result = registry.apply_adoption(&path, &into_project_id)?;
            println!("{}", serde_json::to_string_pretty(&result)?);
            return Ok(());
        }
        action => action,
    };
    let root = find_project_root(Path::new(path))?;
    let runtime = Runtime::open_canonical(&root)?;
    let client = ControlClient::new(&runtime)?;
    match action {
        Action::Status { json } => status(&root, &runtime, &client, json).await,
        Action::Pause { reason } => {
            write_emergency_pause(&root, &reason)?;
            let state = client.status().await.with_context(|| {
                "emergency PAUSE is active locally, but durable pause could not reach the control plane"
            })?;
            let legacy = read_waypoint(&root);
            let checkpoint = Checkpoint {
                reason,
                previous_state: state.lifecycle.clone(),
                last_completed: string_field(&legacy, &["lastCompleted", "lastCompletedChange"]),
                exact_next_work: string_field(&legacy, &["exactNextCommand", "exact_next_command"]),
                decisions: Vec::new(),
                blockers: Vec::new(),
                dirty_work_summary: git_dirty_summary(&root),
                plan_revision: state.plan_revision,
            };
            let next = client
                .submit_fresh(
                    &state,
                    current_actor(ActorKind::Operator),
                    CommandKind::Pause { checkpoint },
                )
                .await?;
            write_pause_valve(&root, &next)?;
            print_state(&next, false)
        }
        Action::Revise {
            reason,
            exact_next_work,
        } => {
            let state = client.status().await?;
            let next = client
                .submit_fresh(
                    &state,
                    current_actor(ActorKind::Harness),
                    CommandKind::PlanRevise {
                        reason,
                        exact_next_work,
                    },
                )
                .await?;
            print_state(&next, false)
        }
        Action::Resume { plan_revision } => {
            let state = client.status().await?;
            let actor = current_actor(ActorKind::Operator);
            let revision = plan_revision.unwrap_or(state.plan_revision);
            let next = client
                .submit_fresh(
                    &state,
                    actor,
                    CommandKind::Resume {
                        plan_revision: revision,
                    },
                )
                .await?;
            release_pause_valve(&root)?;
            print_state(&next, false)
        }
        Action::Cancel { reason } => {
            write_emergency_pause(&root, &reason)?;
            let state = client.status().await.with_context(|| {
                "emergency PAUSE is active locally, but durable cancellation could not reach the control plane"
            })?;
            let next = client
                .submit_fresh(
                    &state,
                    current_actor(ActorKind::Operator),
                    CommandKind::Cancel { reason },
                )
                .await?;
            write_pause_valve(&root, &next)?;
            print_state(&next, false)
        }
        Action::Audit { since, json } => audit(&runtime, &client, since.as_deref(), json).await,
        Action::Watch => watch(&client).await,
        Action::Migrate { check, apply } => {
            let report = runtime.migrate_legacy_ledgers(false)?;
            if apply {
                ensure_runtime(&root, &runtime)?;
                let applied = runtime.migrate_legacy_ledgers(true)?;
                runtime.write_compatibility_projections()?;
                println!("{}", serde_json::to_string_pretty(&applied)?);
            } else {
                let _ = check;
                println!("{}", serde_json::to_string_pretty(&report)?);
            }
            Ok(())
        }
        Action::RolloutStatus => {
            let evidence = RolloutTracker::open(runtime.runtime_root()).load()?;
            let next_gate = evidence.gate();
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "evidence": evidence,
                    "nextGate": next_gate
                }))?
            );
            Ok(())
        }
        Action::RolloutObserve {
            observation_id,
            observed_at,
            real_mutations,
            synthetic_replay_mutations,
            mut unexplained_projection_mismatches,
            harness,
            device,
            voters,
            successful,
        } => {
            let observed_at = observed_at
                .map(|value| {
                    chrono::DateTime::parse_from_rfc3339(&value)
                        .map(|timestamp| timestamp.with_timezone(&chrono::Utc))
                        .with_context(|| format!("invalid RFC 3339 --observed-at {value:?}"))
                })
                .transpose()?
                .unwrap_or_else(chrono::Utc::now);
            let state = client.status().await?;
            let events = client.events().await?;
            let projection_time = events
                .last()
                .map(|event| event.timestamp)
                .ok_or_else(|| anyhow!("cannot compare projections before initialization"))?;
            let projection_mismatches =
                runtime.compatibility_projection_mismatches_from_state(&state, projection_time)?;
            unexplained_projection_mismatches =
                unexplained_projection_mismatches.max(projection_mismatches.len() as u64);
            let tracker = RolloutTracker::open(runtime.runtime_root());
            let evidence = tracker.record(RolloutObservation {
                observation_id,
                observed_at,
                real_mutations,
                synthetic_replay_mutations,
                unexplained_projection_mismatches,
                projection_mismatches,
                harness,
                device,
                voters,
                successful,
            })?;
            let next_gate = evidence.gate();
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "evidence": evidence,
                    "nextGate": next_gate
                }))?
            );
            Ok(())
        }
        Action::RolloutPromote => {
            let tracker = RolloutTracker::open(runtime.runtime_root());
            let evidence = tracker.promote()?;
            let next_gate = evidence.gate();
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "evidence": evidence,
                    "nextGate": next_gate
                }))?
            );
            Ok(())
        }
        Action::Command {
            expected_revision,
            command_id,
            command,
        } => {
            let state = client.status().await?;
            let result = client
                .submit(CommandEnvelope {
                    schema_version: "1".into(),
                    project_id: state.project_id,
                    run_id: state.run_id,
                    command_id,
                    expected_revision,
                    actor: current_actor(ActorKind::Harness),
                    command,
                })
                .await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
            Ok(())
        }
        Action::Projects { .. }
        | Action::Register { .. }
        | Action::Replicas { .. }
        | Action::Adopt { .. } => unreachable!("registry actions return before project open"),
    }
}

async fn status(
    root: &Path,
    runtime: &Runtime,
    client: &ControlClient,
    json_output: bool,
) -> Result<()> {
    match client.status().await {
        Ok(state) => print_state(&state, json_output),
        Err(remote_error) => match runtime.replay() {
            Ok(state) if state.revision > 0 => print_state(&state, json_output),
            Ok(_) | Err(RuntimeError::NotInitialized) => {
                let legacy = read_waypoint(root);
                let output = json!({
                    "mode": "legacy",
                    "phase": string_field(&legacy, &["phase"]),
                    "status": string_field(&legacy, &["status", "stage"]),
                    "exactNextWork": string_field(&legacy, &["exactNextCommand", "exact_next_command"]),
                    "runtimeInitialized": false
                });
                if json_output {
                    println!("{}", serde_json::to_string_pretty(&output)?);
                } else {
                    eprintln!("Control plane unavailable: {remote_error}");
                    println!("KBD mode: legacy (run `prometheus kbd migrate --apply`)");
                    println!("Phase: {}", output["phase"].as_str().unwrap_or("unknown"));
                    println!("Status: {}", output["status"].as_str().unwrap_or("unknown"));
                }
                Ok(())
            }
            Err(error) => Err(error.into()),
        },
    }
}

fn print_state(state: &RuntimeState, json_output: bool) -> Result<()> {
    if json_output {
        println!("{}", serde_json::to_string_pretty(state)?);
        return Ok(());
    }
    println!("Run: {}  revision {}", state.run_id, state.revision);
    println!(
        "Lifecycle: {:?}  plan revision {}",
        state.lifecycle, state.plan_revision
    );
    if let Some(checkpoint) = &state.checkpoint {
        println!("Checkpoint: {}", checkpoint.reason);
        if let Some(next) = &checkpoint.exact_next_work {
            println!("Recorded next: {next}");
        }
    }
    Ok(())
}

fn ensure_runtime(root: &Path, runtime: &Runtime) -> Result<RuntimeState> {
    let state = runtime.replay()?;
    if state.revision > 0 {
        return Ok(state);
    }
    let waypoint = read_waypoint(root);
    let project_id = runtime
        .project_manifest(true)?
        .ok_or_else(|| anyhow!("failed to establish immutable project identity"))?
        .project_id;
    let legacy_phase = string_field(&waypoint, &["phase"]).unwrap_or_else(|| "project".into());
    let run_id = format!(
        "{legacy_phase}-{}",
        chrono::Utc::now().format("%Y%m%dT%H%M%SZ")
    );
    let legacy_status = string_field(&waypoint, &["status", "stage"]).unwrap_or_default();
    let lifecycle = match legacy_status.as_str() {
        "pause_requested" => LifecycleState::PauseRequested,
        "paused" | "suspended" => LifecycleState::Paused,
        "blocked" => LifecycleState::Blocked,
        "phase_complete" | "reflect_complete" | "reflected" | "completed" | "done" => {
            LifecycleState::Completed
        }
        "cancelled" => LifecycleState::Cancelled,
        "failed" => LifecycleState::Failed,
        "executing" | "running" | "execute_ready" => LifecycleState::Running,
        _ => LifecycleState::Ready,
    };
    let exact_next = string_field(&waypoint, &["exactNextCommand", "exact_next_command"]);
    let plan_revision = waypoint["planRevision"].as_u64().unwrap_or(1);
    Ok(runtime.initialize_from_legacy(
        project_id,
        run_id,
        current_actor(ActorKind::Operator),
        lifecycle,
        exact_next,
        plan_revision,
    )?)
}

struct ControlClient {
    http: reqwest::Client,
    endpoint: String,
    project_id: String,
}

impl ControlClient {
    fn new(runtime: &Runtime) -> Result<Self> {
        let manifest = runtime
            .project_manifest(false)?
            .ok_or_else(|| anyhow!("missing .prometheus/project.json"))?;
        let project_id = runtime
            .replay()
            .ok()
            .filter(|state| state.revision > 0)
            .map(|state| state.project_id)
            .unwrap_or(manifest.project_id);
        Ok(Self {
            http: reqwest::Client::builder()
                // 2s was too aggressive: a command that validates and fsyncs
                // the journal can legitimately take longer, especially right
                // after a daemon restart,
                // producing a client-side "operation timed out" even though
                // the write succeeds server-side moments later.
                .timeout(Duration::from_secs(30))
                .build()?,
            endpoint: std::env::var("PROMETHEUS_CONTROL_ENDPOINT")
                .unwrap_or_else(|_| "http://127.0.0.1:7892".into())
                .trim_end_matches('/')
                .to_string(),
            project_id,
        })
    }

    async fn status(&self) -> Result<RuntimeState> {
        let response = self
            .http
            .get(format!(
                "{}/api/v1/kbd/projects/{}/status",
                self.endpoint, self.project_id
            ))
            .send()
            .await?;
        decode_response(response).await
    }

    async fn events(&self) -> Result<Vec<Event>> {
        let response = self
            .http
            .get(format!(
                "{}/api/v1/kbd/projects/{}/events",
                self.endpoint, self.project_id
            ))
            .send()
            .await?;
        decode_response(response).await
    }

    async fn submit(&self, envelope: CommandEnvelope) -> Result<Value> {
        let response = self
            .http
            .post(format!(
                "{}/api/v1/kbd/projects/{}/commands",
                self.endpoint, self.project_id
            ))
            .json(&envelope)
            .send()
            .await?;
        decode_response(response).await
    }

    async fn submit_fresh(
        &self,
        state: &RuntimeState,
        actor: Actor,
        command: CommandKind,
    ) -> Result<RuntimeState> {
        let response = self
            .submit(CommandEnvelope {
                schema_version: "1".into(),
                project_id: state.project_id.clone(),
                run_id: state.run_id.clone(),
                command_id: uuid::Uuid::new_v4().to_string(),
                expected_revision: state.revision,
                actor,
                command,
            })
            .await?;
        serde_json::from_value(
            response
                .get("state")
                .cloned()
                .ok_or_else(|| anyhow!("control response omitted committed state"))?,
        )
        .context("invalid committed state in control response")
    }
}

async fn decode_response<T: serde::de::DeserializeOwned>(response: reqwest::Response) -> Result<T> {
    let status = response.status();
    let body = response.text().await?;
    if !status.is_success() {
        return Err(anyhow!("control plane returned {status}: {body}"));
    }
    serde_json::from_str(&body).context("invalid control-plane JSON response")
}

async fn audit(
    runtime: &Runtime,
    client: &ControlClient,
    since: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let events = match client.events().await {
        Ok(events) => events,
        Err(_) => {
            // A freshly migrated standalone project may have a valid local
            // canonical journal before the daemon has registered the project.
            // Audit must remain available for operator recovery in that state.
            runtime.replay()?;
            runtime.events()?
        }
    };
    let filtered: Vec<&Event> = match since {
        Some(value) => match value.parse::<u64>() {
            Ok(revision) => events
                .iter()
                .filter(|event| event.revision >= revision)
                .collect(),
            Err(_) => {
                let index = events
                    .iter()
                    .position(|event| event.event_id == value)
                    .ok_or_else(|| anyhow!("event not found: {value}"))?;
                events[index..].iter().collect()
            }
        },
        None => events.iter().collect(),
    };
    if json_output {
        println!("{}", serde_json::to_string_pretty(&filtered)?);
    } else {
        for event in filtered {
            println!(
                "r{} {} {} {:?}",
                event.revision, event.timestamp, event.actor.harness, event.kind
            );
        }
    }
    Ok(())
}

async fn watch(client: &ControlClient) -> Result<()> {
    let mut revision = 0;
    loop {
        let seen = revision;
        let fresh: Vec<_> = client
            .events()
            .await?
            .into_iter()
            .filter(|event| event.revision > seen)
            .collect();
        for event in fresh {
            revision = event.revision;
            println!("{}", serde_json::to_string(&event)?);
        }
        tokio::select! {
            _ = tokio::signal::ctrl_c() => return Ok(()),
            _ = tokio::time::sleep(Duration::from_secs(1)) => {}
        }
    }
}

fn current_actor(kind: ActorKind) -> Actor {
    let harness = std::env::var("PROMETHEUS_HARNESS").unwrap_or_else(|_| {
        if std::env::var_os("CODEX_THREAD_ID").is_some() {
            "codex".into()
        } else if std::env::var_os("CLAUDE_SESSION_ID").is_some() {
            "claude-code".into()
        } else {
            "cli".into()
        }
    });
    harness_actor(&harness, kind)
}

fn harness_actor(harness: &str, kind: ActorKind) -> Actor {
    let mut actor = Actor::operator(
        std::env::var("USER").unwrap_or_else(|_| "operator".into()),
        harness,
    );
    actor.kind = kind;
    actor
}

fn find_project_root(start: &Path) -> Result<PathBuf> {
    let start = fs::canonicalize(start)
        .with_context(|| format!("cannot resolve project path {}", start.display()))?;
    for candidate in start.ancestors() {
        if candidate.join(".kbd-orchestrator").is_dir() {
            return Ok(candidate.to_path_buf());
        }
    }
    Err(anyhow!(
        "no .kbd-orchestrator directory found from {}",
        start.display()
    ))
}

fn find_manifest_project_root(start: &Path) -> Result<PathBuf> {
    let start = fs::canonicalize(start)
        .with_context(|| format!("cannot resolve project path {}", start.display()))?;
    for candidate in start.ancestors() {
        if candidate.join(".prometheus/project.json").is_file() {
            return Ok(candidate.to_path_buf());
        }
    }
    Err(anyhow!(
        "no .prometheus/project.json found from {}",
        start.display()
    ))
}

fn read_waypoint(root: &Path) -> Value {
    fs::read(root.join(".kbd-orchestrator/current-waypoint.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_else(|| json!({}))
}

fn string_field(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_str))
        .map(str::to_string)
}

fn git_dirty_summary(root: &Path) -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["status", "--short"])
        .current_dir(root)
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    let count = text.lines().count();
    Some(format!("{count} changed paths"))
}

fn write_pause_valve(root: &Path, state: &RuntimeState) -> Result<()> {
    fs::write(
        root.join(".kbd-orchestrator/PAUSE"),
        format!(
            "runId={}\nrevision={}\nlifecycle={:?}\n",
            state.run_id, state.revision, state.lifecycle
        ),
    )?;
    Ok(())
}

fn write_emergency_pause(root: &Path, reason: &str) -> Result<()> {
    let directory = root.join(".kbd-orchestrator");
    fs::create_dir_all(&directory)?;
    fs::write(
        directory.join("PAUSE"),
        format!(
            "requestedAt={}\nreason={}\nlifecycle=pause_requested\n",
            chrono::Utc::now().to_rfc3339(),
            reason.replace('\n', " ")
        ),
    )?;
    Ok(())
}

fn release_pause_valve(root: &Path) -> Result<()> {
    let path = root.join(".kbd-orchestrator/PAUSE");
    if path.exists() {
        let audit = root.join(format!(
            ".kbd-orchestrator/PAUSE.released.{}",
            chrono::Utc::now().format("%Y%m%dT%H%M%SZ")
        ));
        fs::rename(path, audit)?;
    }
    Ok(())
}
