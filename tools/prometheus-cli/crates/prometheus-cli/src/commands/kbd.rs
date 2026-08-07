use anyhow::{anyhow, Context, Result};
use kbd_runtime::{
    registry::{scan_submodule_pins, ProjectRegistry},
    rollout::{RolloutObservation, RolloutTracker},
    Actor, ActorKind, Checkpoint, ClaimMode, CommandEnvelope, CommandKind, Event, LifecycleState,
    Runtime, RuntimeError, RuntimeState, SignedCommandEnvelope,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;
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
    Conflicts {
        json: bool,
    },
    Resolve {
        conflict_id: String,
        winner_event_id: String,
        reason: String,
    },
    Claims {
        json: bool,
    },
    ClaimAcquire {
        scope: String,
        mode: ClaimMode,
        ttl_seconds: u64,
        holder_id: Option<String>,
    },
    ClaimRenew {
        claim_id: String,
        ttl_seconds: u64,
    },
    ClaimRelease {
        claim_id: String,
    },
    Submodules {
        scan: bool,
        json: bool,
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
        export_git: bool,
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
        replicas: u64,
        successful: bool,
    },
    RolloutPromote,
    Command {
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
                    let access = if replica.read_only {
                        format!(
                            "  read-only ({})",
                            replica.read_only_reason.as_deref().unwrap_or("policy")
                        )
                    } else {
                        String::new()
                    };
                    println!(
                        "{}  {}  {}  {:?}{}",
                        replica.project_id, replica.replica_id, path, replica.kind, access
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
                    let access = if replica.read_only {
                        format!(
                            "  read-only ({})",
                            replica.read_only_reason.as_deref().unwrap_or("policy")
                        )
                    } else {
                        String::new()
                    };
                    println!(
                        "{}  {}  {:?}{}",
                        replica.replica_id,
                        path.display(),
                        replica.kind,
                        access
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
        Action::Conflicts { json } => {
            let state = state_or_replay(&client, &runtime).await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&state.conflicts)?);
            } else if state.conflicts.is_empty() {
                println!("No unresolved or adjudicated conflicts.");
            } else {
                for conflict in state.conflicts.values() {
                    println!(
                        "{}  {}  winner={}{}",
                        conflict.id,
                        conflict.slot,
                        conflict.winner_event_id,
                        conflict
                            .resolved_by_event_id
                            .as_ref()
                            .map(|event_id| format!("  resolved-by={event_id}"))
                            .unwrap_or_default()
                    );
                }
            }
            Ok(())
        }
        Action::Resolve {
            conflict_id,
            winner_event_id,
            reason,
        } => {
            let state = state_or_replay(&client, &runtime).await?;
            let next = client
                .submit_fresh(
                    &state,
                    current_actor(ActorKind::Operator),
                    CommandKind::ConflictResolve {
                        conflict_id,
                        winner_event_id,
                        reason,
                    },
                )
                .await?;
            print_state(&next, false)
        }
        Action::Claims { json } => {
            let state = state_or_replay(&client, &runtime).await?;
            let claim_conflicts = state
                .conflicts
                .values()
                .filter(|conflict| conflict.kind == kbd_runtime::ConflictKind::Claim)
                .collect::<Vec<_>>();
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "claims": state.claims,
                        "conflicts": claim_conflicts
                    }))?
                );
            } else if state.claims.is_empty() {
                println!("No claims.");
            } else {
                for claim in state.claims.values() {
                    println!(
                        "{}  {}  {:?}  holder={}  token={}  expires={}{}",
                        claim.claim_id,
                        claim.scope,
                        claim.mode,
                        claim.holder_id,
                        claim.monotonic_token,
                        claim.expires_at,
                        if claim.released { "  released" } else { "" }
                    );
                }
            }
            Ok(())
        }
        Action::ClaimAcquire {
            scope,
            mode,
            ttl_seconds,
            holder_id,
        } => {
            let state = state_or_replay(&client, &runtime).await?;
            let mut actor = current_actor(ActorKind::Harness);
            if let Some(holder_id) = holder_id {
                actor.id = holder_id;
            }
            let holder_id = actor.id.clone();
            let next = client
                .submit_fresh(
                    &state,
                    actor,
                    CommandKind::ClaimAcquire {
                        scope,
                        mode,
                        ttl_seconds,
                        holder_id,
                    },
                )
                .await?;
            print_state(&next, false)
        }
        Action::ClaimRenew {
            claim_id,
            ttl_seconds,
        } => {
            let state = state_or_replay(&client, &runtime).await?;
            let next = client
                .submit_fresh(
                    &state,
                    current_actor(ActorKind::Harness),
                    CommandKind::ClaimRenew {
                        claim_id,
                        ttl_seconds,
                    },
                )
                .await?;
            print_state(&next, false)
        }
        Action::ClaimRelease { claim_id } => {
            let state = state_or_replay(&client, &runtime).await?;
            let next = client
                .submit_fresh(
                    &state,
                    current_actor(ActorKind::Harness),
                    CommandKind::ClaimRelease { claim_id },
                )
                .await?;
            print_state(&next, false)
        }
        Action::Submodules { scan, json } => {
            if scan {
                let scanned = scan_submodule_pins(&root)?;
                for pin in &scanned.pins {
                    let state = state_or_replay(&client, &runtime).await?;
                    if state.submodule_pins.get(&pin.path) == Some(pin) {
                        continue;
                    }
                    client
                        .submit_fresh(
                            &state,
                            current_actor(ActorKind::Operator),
                            CommandKind::SubmodulePinSet { pin: pin.clone() },
                        )
                        .await?;
                }
                let state = state_or_replay(&client, &runtime).await?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "scan": scanned,
                        "pins": state.submodule_pins,
                        "replicaView": state.replica_view
                    }))?
                );
            } else {
                let state = state_or_replay(&client, &runtime).await?;
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&json!({
                            "pins": state.submodule_pins,
                            "replicaView": state.replica_view
                        }))?
                    );
                } else if state.submodule_pins.is_empty() {
                    println!("No submodule pins.");
                } else {
                    for pin in state.submodule_pins.values() {
                        println!(
                            "{}  {}  {}",
                            pin.path, pin.child_project_id, pin.gitlink_sha
                        );
                    }
                }
            }
            Ok(())
        }
        Action::Pause { reason } => {
            write_emergency_pause(&root, &reason)?;
            // The local emergency-pause file is already written above. Failing here
            // would strand the operator in a half-applied state: paused on disk, with
            // no durable record. Fall back to local replay so the durable pause can
            // still be journaled when the daemon is unreachable.
            let state = state_or_replay(&client, &runtime).await.with_context(|| {
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
            let state = state_or_replay(&client, &runtime).await?;
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
            let state = state_or_replay(&client, &runtime).await?;
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
            // Same half-applied hazard as Pause above: the emergency file is already
            // on disk, so a hard failure here loses the durable cancellation record.
            let state = state_or_replay(&client, &runtime).await.with_context(|| {
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
        Action::Audit {
            since,
            json,
            export_git,
        } => audit(&runtime, &client, since.as_deref(), json, export_git).await,
        Action::Watch => watch(&client).await,
        Action::Migrate { check, apply } => {
            let report = runtime.migrate_legacy_ledgers(false)?;
            if apply {
                let journal = runtime.migrate_v1_journal()?;
                ensure_runtime(&root, &runtime)?;
                let applied = runtime.migrate_legacy_ledgers(true)?;
                runtime.write_compatibility_projections()?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "journal": journal,
                        "ledgers": applied
                    }))?
                );
            } else {
                let _ = check;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "journalMigrationRequired": runtime.v1_journal_migration_required(),
                        "ledgers": report
                    }))?
                );
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
            replicas,
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
            let state = state_or_replay(&client, &runtime).await?;
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
                replicas,
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
            command_id,
            mut command,
        } => {
            if let CommandKind::ActivePathSet { active_path, .. } = &mut command {
                if active_path.commit.is_none() {
                    active_path.commit = git_head(&root);
                }
            }
            // Resolve the current state locally when the daemon is unreachable,
            // otherwise this path dies before it can even build an envelope —
            // the exact failure Codex hit ("signed waypoint remains unchanged
            // ... the typed mutation endpoint refused the connection").
            let state = match client.status().await {
                Ok(state) => state,
                Err(remote_error) => runtime
                    .replay()
                    .with_context(|| format!("control plane unavailable ({remote_error})"))?,
            };
            let envelope = CommandEnvelope {
                schema_version: "2".into(),
                project_id: state.project_id,
                run_id: state.run_id,
                command_id,
                frontier: Some(state.frontier),
                expected_revision: state.revision,
                actor: current_actor(ActorKind::Harness),
                command,
            };
            let result = match client.submit(envelope.clone()).await {
                Ok(result) => result,
                Err(failure) if !failure.may_execute_locally() => return Err(failure.into_error()),
                Err(failure) => {
                    let ambiguous = matches!(failure, ControlFailure::Ambiguous(_));
                    let state =
                        client.execute_locally(envelope, &failure.into_error(), ambiguous)?;
                    json!({
                        "state": state,
                        "committedLocally": true,
                        "remoteStatusUnknown": ambiguous
                    })
                }
            };
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
    runtime: Runtime,
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
            // NOTE: sovereign-sync 1.7.0 serves over a Unix socket by default;
            // TCP :7892 exists only when it is started with an explicit --tcp,
            // which the managed LaunchAgent does not pass. So under the managed
            // configuration this endpoint is normally UNREACHABLE — and that is
            // now a non-event: `submit_fresh` commits through the local runtime
            // instead. reqwest cannot speak Unix sockets without an extra
            // dependency; wiring one up is worthwhile so a running daemon is
            // actually used, but it is no longer required for correctness.
            endpoint: std::env::var("PROMETHEUS_CONTROL_ENDPOINT")
                .unwrap_or_else(|_| "http://127.0.0.1:7892".into())
                .trim_end_matches('/')
                .to_string(),
            project_id,
            runtime: runtime.clone(),
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

    async fn audit_events(&self) -> Result<Vec<Event>> {
        let response = self
            .http
            .get(format!(
                "{}/api/v1/kbd/projects/{}/audit",
                self.endpoint, self.project_id
            ))
            .send()
            .await?;
        let status = response.status();
        let body = response.text().await?;
        if !status.is_success() {
            return Err(anyhow!("control plane returned {status}: {body}"));
        }
        body.lines()
            .filter(|line| !line.trim().is_empty())
            .enumerate()
            .map(|(index, line)| {
                serde_json::from_str(line)
                    .with_context(|| format!("invalid audit JSONL at line {}", index + 1))
            })
            .collect()
    }

    async fn submit(
        &self,
        envelope: CommandEnvelope,
    ) -> std::result::Result<Value, ControlFailure> {
        // Read-only commands must never touch the platform credential store.
        // Resolve the signer only for an actual mutation, and keep the
        // synchronous OS credential lookup off the async executor.
        let runtime = self.runtime.clone();
        let signer = match tokio::task::spawn_blocking(move || runtime.device_signer())
            .await
            .context("device signer task failed")
        {
            Ok(Ok(signer)) => signer,
            // A missing/locked signer is a local problem, not the daemon's, and
            // local execution needs the same signer — so this is terminal.
            Ok(Err(error)) => return Err(ControlFailure::Rejected(error.into())),
            Err(error) => return Err(ControlFailure::Rejected(error)),
        };
        let signed = match SignedCommandEnvelope::sign(envelope, &signer) {
            Ok(signed) => signed,
            Err(error) => return Err(ControlFailure::Rejected(error.into())),
        };
        let response = self
            .http
            .post(format!(
                "{}/api/v1/kbd/projects/{}/commands",
                self.endpoint, self.project_id
            ))
            .json(&signed)
            .send()
            .await
            .map_err(classify_transport_error)?;
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|error| ControlFailure::Unreachable(error.into()))?;
        if !status.is_success() {
            return Err(classify_status(status, &body));
        }
        serde_json::from_str(&body)
            .context("invalid control-plane JSON response")
            .map_err(ControlFailure::Rejected)
    }

    async fn submit_fresh(
        &self,
        state: &RuntimeState,
        actor: Actor,
        command: CommandKind,
    ) -> Result<RuntimeState> {
        // One envelope, reused for both paths. `command_id` is generated ONCE so
        // the local fallback carries the same idempotency key the daemon would
        // have seen — if the daemon later replays this journal it recognises the
        // command instead of double-applying it.
        let envelope = CommandEnvelope {
            schema_version: "2".into(),
            project_id: state.project_id.clone(),
            run_id: state.run_id.clone(),
            command_id: uuid::Uuid::new_v4().to_string(),
            frontier: Some(state.frontier.clone()),
            expected_revision: state.revision,
            actor,
            command,
        };
        match self.submit(envelope.clone()).await {
            Ok(response) => serde_json::from_value(
                response
                    .get("state")
                    .cloned()
                    .ok_or_else(|| anyhow!("control response omitted committed state"))?,
            )
            .context("invalid committed state in control response"),
            // The daemon adjudicated and said no. Honour it — never launder a
            // rejection into a local success.
            Err(failure) if !failure.may_execute_locally() => Err(failure.into_error()),
            // Execute against the same runtime the daemon itself would have used:
            // identical journal, identical signing, identical validation, and
            // mutual exclusion by the same exclusive flock inside
            // `execute_command`. sovereign-sync is a passive replicator and must
            // never gate local KBD work.
            Err(failure) => {
                let ambiguous = matches!(failure, ControlFailure::Ambiguous(_));
                self.execute_locally(envelope, &failure.into_error(), ambiguous)
            }
        }
    }

    /// Commit a command through the in-process runtime.
    ///
    /// This is the same `Runtime::execute_command` the daemon calls, so the
    /// durability contract is unchanged: WAL append + fsync, Loro ingest, folded
    /// checkpoint, all under one exclusive flock. Two CLI processes serialize
    /// exactly as two daemon requests do.
    fn execute_locally(
        &self,
        envelope: CommandEnvelope,
        remote_error: &anyhow::Error,
        ambiguous: bool,
    ) -> Result<RuntimeState> {
        if ambiguous {
            // The command may already be committed in the daemon's journal. The
            // stable `command_id` makes a later merge idempotent, but say so
            // rather than implying a clean local-only commit.
            eprintln!(
                "control plane status UNKNOWN ({remote_error}); the command may already be \
                 committed remotely. Committing locally with the same commandId {} — a later \
                 sync deduplicates on that id. Run `prometheus kbd status` to reconcile.",
                envelope.command_id
            );
        } else {
            eprintln!(
                "control plane unreachable ({remote_error}); committing locally via the canonical runtime"
            );
        }
        let result = self
            .runtime
            .execute_command(envelope)
            .context("local runtime rejected the command")?;
        // `execute_command` records business-logic rejections on the result
        // rather than as an Err, so that a bad command cannot wedge the log.
        // Surface that as a failure instead of reporting a phantom success.
        if let Some(error) = result.apply_error.as_ref() {
            return Err(anyhow!("local runtime rejected the command: {error}"));
        }
        // Refresh the compatibility projections the harnesses actually read.
        //
        // When the daemon commits, it rewrites `current-waypoint.json` and the phase
        // `progress.json` files server-side. The local path must do the same, or the
        // command succeeds in canonical state while every harness keeps reading a stale
        // waypoint — e.g. `revise --exact-next-work` advances the plan to revision 9
        // while `current-waypoint.json` still points at revision 7's already-completed
        // command, sending the next agent to redo finished work.
        //
        // Best-effort: a projection-write failure must not turn a durable, committed
        // command into a reported error. Warn and carry on.
        if let Err(error) = self
            .runtime
            .write_compatibility_projections_from_state(&result.state, chrono::Utc::now())
        {
            eprintln!(
                "warning: command committed at revision {} but compatibility projections \
                 could not be refreshed ({error}); run `prometheus kbd status` to re-derive them",
                result.state.revision
            );
        }
        Ok(result.state)
    }
}

/// Why a control-plane call did not produce a committed result.
///
/// The distinction is load-bearing for local fallback. `Unreachable` means the
/// daemon never adjudicated the command — nothing was committed anywhere, so
/// executing it locally is safe and produces exactly the state the daemon would
/// have produced. `Rejected` means the daemon DID adjudicate and said no (a
/// revision conflict, an invalid transition, a read-only replica). Retrying a
/// rejection locally would launder a legitimate "no" into a "yes" and is the
/// one thing this fallback must never do.
#[derive(Debug)]
enum ControlFailure {
    /// Delivery is provably impossible: the connection was never established, or
    /// the startup gate refused the route before dispatch (503). The daemon
    /// cannot have adjudicated this command, so committing locally is safe.
    Unreachable(anyhow::Error),
    /// The request may or may not have been committed — a timeout, a reset after
    /// send, or a non-503 5xx. Safe to retry locally ONLY because the command
    /// carries a stable `command_id` and the runtime deduplicates on it, but the
    /// operator is told the remote status is unknown.
    Ambiguous(anyhow::Error),
    /// The control plane adjudicated this command and refused it (4xx). Never
    /// retried locally — that would launder a legitimate "no" into a "yes".
    Rejected(anyhow::Error),
}

impl ControlFailure {
    fn into_error(self) -> anyhow::Error {
        match self {
            Self::Unreachable(error) | Self::Ambiguous(error) | Self::Rejected(error) => error,
        }
    }

    /// Whether a local commit may proceed. False only for an adjudicated refusal.
    fn may_execute_locally(&self) -> bool {
        !matches!(self, Self::Rejected(_))
    }
}

/// Read current state, falling back to local replay when the daemon is unreachable.
///
/// Every mutating action needs the current state first, to build a `CommandEnvelope`
/// carrying the expected revision and causal frontier. Reading that via a bare
/// `client.status().await?` makes the daemon a hard dependency of *writes* — the
/// command dies at the precondition read and never reaches `submit_fresh`, whose
/// `Unreachable` → `execute_locally` fallback exists precisely so local work is never
/// gated on a passive replicator.
///
/// That is not hypothetical: under the managed configuration sovereign-sync serves over
/// a Unix socket and TCP :7892 is never bound, so `status()` ALWAYS fails and every
/// mutation is unreachable. This is the failure Codex hit ("the typed mutation endpoint
/// refused the connection"), and it silently cost recorded work — a `migrate --apply`
/// then rebuilt the projections from canonical state that no harness had been able to
/// write to, resetting a completed change back to PENDING.
///
/// Local replay is the same journal the daemon itself would read, so the envelope built
/// from it is identical to the one the daemon would have produced.
async fn state_or_replay(client: &ControlClient, runtime: &Runtime) -> Result<RuntimeState> {
    match client.status().await {
        Ok(state) => Ok(state),
        Err(remote_error) => runtime
            .replay()
            .with_context(|| format!("control plane unavailable ({remote_error})")),
    }
}

/// Classify a transport error by whether delivery is *provably* impossible.
///
/// Only a failure to establish the connection proves the daemon never saw the
/// command. Once bytes are on the wire, a timeout or a reset is AMBIGUOUS: the
/// daemon may have validated, journaled, fsynced and committed, and only the
/// response died. Committing locally in that case is still safe — the command
/// carries the same `command_id`, and `Runtime::execute_command` checks
/// `state.command_revisions` for that id BEFORE validating the frontier, so a
/// replay short-circuits to `duplicate: true` with the original revision rather
/// than double-applying. But the local runtime cannot see a commit that only
/// exists in the daemon's journal yet, so we surface the ambiguity instead of
/// silently reconciling it.
fn classify_transport_error(error: reqwest::Error) -> ControlFailure {
    if error.is_connect() {
        return ControlFailure::Unreachable(error.into());
    }
    ControlFailure::Ambiguous(error.into())
}

/// A 503 from the startup gate means the daemon refused the route before any
/// command was dispatched — unreachability wearing an HTTP status. Any other
/// 5xx is ambiguous: the daemon may have committed and then failed to respond.
/// A 4xx is a decision about this command.
fn classify_status(status: reqwest::StatusCode, body: &str) -> ControlFailure {
    let error = anyhow!("control plane returned {status}: {body}");
    if status == reqwest::StatusCode::SERVICE_UNAVAILABLE {
        ControlFailure::Unreachable(error)
    } else if status.is_server_error() {
        ControlFailure::Ambiguous(error)
    } else {
        ControlFailure::Rejected(error)
    }
}

async fn decode_response<T: serde::de::DeserializeOwned>(response: reqwest::Response) -> Result<T> {
    let status = response.status();
    let body = response.text().await?;
    if !status.is_success() {
        return Err(classify_status(status, &body).into_error());
    }
    serde_json::from_str(&body).context("invalid control-plane JSON response")
}

async fn audit(
    runtime: &Runtime,
    client: &ControlClient,
    since: Option<&str>,
    json_output: bool,
    export_git: bool,
) -> Result<()> {
    if export_git {
        let exported = runtime.export_audit_to_git()?;
        println!("{}", serde_json::to_string_pretty(&exported)?);
        return Ok(());
    }
    let events = match client.audit_events().await {
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
    let mut seen = BTreeSet::new();
    loop {
        let fresh: Vec<_> = client
            .events()
            .await?
            .into_iter()
            .filter(|event| seen.insert(event.event_id.clone()))
            .collect();
        for event in fresh {
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

fn git_head(root: &Path) -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let head = String::from_utf8(output.stdout).ok()?;
    let head = head.trim();
    (!head.is_empty()).then(|| head.to_owned())
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
