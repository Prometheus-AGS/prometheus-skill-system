use anyhow::{anyhow, Context, Result};
use kbd_runtime::{
    registry::{scan_submodule_pins, ProjectRegistry},
    rollout::{RolloutObservation, RolloutTracker},
    Actor, ActorKind, Blocker, BoundaryEdge, BoundaryKind, BoundaryOutcome, BoundaryReceipt,
    Checkpoint, ClaimMode, CommandEnvelope, CommandKind, Event, GateKind, GateOutcome, GateReceipt,
    GateRun, LifecycleState, ProjectionScope, Runtime, RuntimeError, RuntimeState,
    SignedCommandEnvelope, WorkStatus,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use super::control_transport::{ControlResponse, ControlTransport, TransportFailure};

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
    RunStart {
        run_id: String,
        reason: String,
        exact_next_work: Option<String>,
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
    GuardEvaluate {
        boundary: BoundaryKind,
        edge: BoundaryEdge,
        subject: String,
        json: bool,
        repair_projections: bool,
        precommit: bool,
    },
    GateRun {
        kind: GateKind,
        scope: String,
        command: Vec<String>,
    },
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
    let runtime = if matches!(
        &action,
        Action::GuardEvaluate {
            precommit: true,
            ..
        }
    ) {
        Runtime::open_canonical_snapshot(&root)?
    } else {
        Runtime::open_canonical(&root)?
    };
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
            let state = state_or_initialize(&client, &runtime).await?;
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
            let state = state_or_initialize(&client, &runtime).await?;
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
            let state = state_or_initialize(&client, &runtime).await?;
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
            let state = state_or_initialize(&client, &runtime).await?;
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
                    let state = state_or_initialize(&client, &runtime).await?;
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
                let state = state_or_initialize(&client, &runtime).await?;
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
        Action::RunStart {
            run_id,
            reason,
            exact_next_work,
            json,
        } => {
            let state = state_or_initialize(&client, &runtime).await?;
            let next = client
                .submit_fresh(
                    &state,
                    current_actor(ActorKind::Operator),
                    CommandKind::RunStart {
                        run_id,
                        reason,
                        exact_next_work,
                    },
                )
                .await?;
            project_successor_and_release_pause(&root, &runtime, &next)?;
            print_state(&next, json)
        }
        Action::Pause { reason } => {
            write_emergency_pause(&root, &reason)?;
            // The local emergency-pause file is already written above. Failing here
            // would strand the operator in a half-applied state: paused on disk, with
            // no durable record. Fall back to local replay so the durable pause can
            // still be journaled when the daemon is unreachable.
            let state = state_or_initialize(&client, &runtime)
                .await
                .with_context(|| {
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
            let state = state_or_initialize(&client, &runtime).await?;
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
            let state = state_or_initialize(&client, &runtime).await?;
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
            let state = state_or_initialize(&client, &runtime)
                .await
                .with_context(|| {
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
        Action::GuardEvaluate {
            boundary,
            edge,
            subject,
            json,
            repair_projections,
            precommit,
        } => {
            guard_evaluate(
                &root,
                &runtime,
                &client,
                boundary,
                edge,
                &subject,
                json,
                repair_projections,
                precommit,
            )
            .await
        }
        Action::GateRun {
            kind,
            scope,
            command,
        } => gate_run(&root, &runtime, &client, kind, &scope, &command).await,
        Action::Command {
            command_id,
            mut command,
        } => {
            if let CommandKind::ActivePathSet { active_path, .. } = &mut command {
                if active_path.commit.is_none() {
                    active_path.commit = git_head(&root);
                }
            }
            let state = state_or_initialize(&client, &runtime).await?;
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

struct GuardContext {
    phase_id: Option<String>,
    change_id: Option<String>,
    task_id: Option<String>,
    ordinal: usize,
    total: usize,
    name: String,
    position: String,
    valid: bool,
}

fn guard_context(
    state: &RuntimeState,
    boundary: BoundaryKind,
    subject: &str,
) -> GuardContext {
    let active_phase_id = state.active_path.phase_id.clone();
    let phase_path = state
        .active_path
        .phase_path
        .iter()
        .filter_map(|phase_id| state.phases.get(phase_id))
        .map(|phase| phase.slug.clone())
        .collect::<Vec<_>>();

    match boundary {
        BoundaryKind::Phase => {
            let selected_phase = state.phases.get(subject).or_else(|| {
                state
                    .phases
                    .values()
                    .find(|phase| phase.slug == subject || phase.title == subject)
            });
            let parent = selected_phase.and_then(|phase| phase.parent_phase_id.as_deref());
            let ordered = state
                .phase_definition_order
                .iter()
                .filter(|phase_id| {
                    state
                        .phases
                        .get(*phase_id)
                        .is_some_and(|phase| phase.parent_phase_id.as_deref() == parent)
                })
                .cloned()
                .collect::<Vec<_>>();
            let phase_id = selected_phase
                .map(|phase| phase.id.clone())
                .unwrap_or_else(|| subject.to_owned());
            let ordinal = ordered
                .iter()
                .position(|candidate| candidate == &phase_id)
                .unwrap_or(0)
                + 1;
            let name = selected_phase
                .map(|phase| phase.title.clone())
                .unwrap_or_else(|| subject.to_owned());
            let position = selected_phase
                .map(|_| {
                    let mut ids = Vec::new();
                    let mut cursor = Some(phase_id.as_str());
                    while let Some(id) = cursor {
                        let Some(phase) = state.phases.get(id) else {
                            break;
                        };
                        ids.push(phase.slug.clone());
                        cursor = phase.parent_phase_id.as_deref();
                    }
                    ids.reverse();
                    ids.join(" › ")
                })
                .unwrap_or_else(|| phase_path.join(" › "));
            GuardContext {
                phase_id: selected_phase.map(|_| phase_id),
                change_id: None,
                task_id: None,
                ordinal,
                total: ordered.len().max(1),
                name,
                position,
                valid: selected_phase.is_some(),
            }
        }
        BoundaryKind::Task => {
            let phase_id = active_phase_id.clone();
            let phase = phase_id.as_ref().and_then(|id| state.phases.get(id));
            let mut matches = Vec::new();
            for change in phase.into_iter().flat_map(|phase| phase.changes.values()) {
                for task in change.tasks.values() {
                    if task.id == subject || task.title == subject {
                        matches.push((change.id.clone(), task.id.clone()));
                    }
                }
            }
            let (change_id, task_id) = if matches.len() == 1 {
                let (change_id, task_id) = matches.remove(0);
                (Some(change_id), Some(task_id))
            } else {
                (None, None)
            };
            let change = change_id
                .as_ref()
                .and_then(|id| phase.and_then(|phase| phase.changes.get(id)));
            let mut tasks = change
                .map(|change| change.tasks.values().collect::<Vec<_>>())
                .unwrap_or_default();
            tasks.sort_by(|left, right| {
                left.sequence
                    .cmp(&right.sequence)
                    .then_with(|| left.id.cmp(&right.id))
            });
            let selected_id = task_id.clone().unwrap_or_else(|| subject.to_owned());
            let ordinal = tasks
                .iter()
                .position(|task| task.id == selected_id || task.title == subject)
                .unwrap_or(0)
                + 1;
            let name = tasks
                .iter()
                .find(|task| task.id == selected_id || task.title == subject)
                .map(|task| task.title.clone())
                .unwrap_or_else(|| subject.to_owned());
            let mut position = phase_path;
            if let Some(change_id) = &change_id {
                position.push(change_id.clone());
            }
            position.push(selected_id.clone());
            let valid = task_id.is_some() && change_id.is_some();
            GuardContext {
                phase_id,
                change_id,
                task_id,
                ordinal,
                total: tasks.len().max(1),
                name,
                position: position.join(" › "),
                valid,
            }
        }
        BoundaryKind::Zeespec => {
            let phases = ["interrogate", "score", "manifest"];
            let ordinal = phases
                .iter()
                .position(|phase| *phase == subject)
                .unwrap_or(0)
                + 1;
            let mut position = phase_path;
            position.push(format!("zeespec:{subject}"));
            GuardContext {
                phase_id: active_phase_id,
                change_id: None,
                task_id: None,
                ordinal,
                total: phases.len(),
                name: subject.to_owned(),
                position: position.join(" › "),
                valid: phases.contains(&subject),
            }
        }
    }
}

fn digest_strings(parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part.as_bytes());
        hasher.update([0]);
    }
    format!("{:x}", hasher.finalize())
}

fn recovery_receipt(
    root: &Path,
    boundary: BoundaryKind,
    edge: BoundaryEdge,
    subject: &str,
    error: &anyhow::Error,
) {
    let recovery_root = root.join(".kbd-orchestrator/recovery/bottleneck");
    if fs::create_dir_all(&recovery_root).is_err() {
        return;
    }
    let id = digest_strings(&[
        &format!("{boundary:?}"),
        &format!("{edge:?}"),
        subject,
        &error.to_string(),
    ]);
    let target = recovery_root.join(format!("{id}.json"));
    let temp = recovery_root.join(format!(".{id}.tmp"));
    let body = json!({
        "schemaVersion": "1",
        "outcome": "blocked",
        "boundary": format!("{boundary:?}").to_ascii_lowercase(),
        "edge": format!("{edge:?}").to_ascii_lowercase(),
        "subject": subject,
        "error": error.to_string(),
        "observedAt": chrono::Utc::now(),
        "canonicalMutation": false
    });
    if fs::write(
        &temp,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&body).unwrap_or_default()
        ),
    )
    .is_ok()
    {
        let _ = fs::rename(temp, target);
    }
}

async fn record_guard_blocker(
    client: &ControlClient,
    state: &RuntimeState,
    receipt: &BoundaryReceipt,
) -> Result<RuntimeState> {
    let blocker_id = format!("bottleneck-{}", receipt.id);
    if state.blockers.contains_key(&blocker_id) {
        return Ok(state.clone());
    }
    client
        .submit_fresh(
            state,
            current_actor(ActorKind::Harness),
            CommandKind::BlockerRecord {
                blocker: Blocker {
                    id: blocker_id,
                    summary: format!(
                        "{:?} {:?} boundary {} blocked: {}",
                        receipt.boundary,
                        receipt.edge,
                        receipt.subject,
                        receipt.findings.join("; ")
                    ),
                    resolved: false,
                    resolution: None,
                },
            },
        )
        .await
}

async fn clear_guard_blockers(
    client: &ControlClient,
    state: &RuntimeState,
    receipt: &BoundaryReceipt,
) -> Result<RuntimeState> {
    let marker = format!(
        "{:?} {:?} boundary {} blocked:",
        receipt.boundary, receipt.edge, receipt.subject
    );
    let blocker_ids = state
        .blockers
        .values()
        .filter(|blocker| {
            !blocker.resolved
                && blocker.id.starts_with("bottleneck-")
                && blocker.summary.starts_with(&marker)
        })
        .map(|blocker| blocker.id.clone())
        .collect::<Vec<_>>();
    let mut current = state.clone();
    for blocker_id in blocker_ids {
        current = client
            .submit_fresh(
                &current,
                current_actor(ActorKind::Harness),
                CommandKind::BlockerClear {
                    blocker_id,
                    resolution: format!(
                        "boundary {} passed at source revision {}",
                        receipt.subject, receipt.source_revision
                    ),
                },
            )
            .await?;
    }
    Ok(current)
}

#[allow(clippy::too_many_arguments)]
async fn guard_evaluate(
    root: &Path,
    runtime: &Runtime,
    client: &ControlClient,
    boundary: BoundaryKind,
    edge: BoundaryEdge,
    subject: &str,
    json_output: bool,
    repair_projections: bool,
    precommit: bool,
) -> Result<()> {
    // Boundary evaluation is a hot-path local reconciliation. The signed local
    // journal is canonical; sovereign-sync passively replicates it and must not
    // add a control-plane round trip to every task/checkpoint boundary.
    let state = match runtime.replay_authority() {
        Ok(state) => state,
        Err(error) => {
            let error = anyhow!(error);
            recovery_receipt(root, boundary, edge, subject, &error);
            return Err(error);
        }
    };
    let projection_time = state
        .last_event_at
        .ok_or_else(|| anyhow!("cannot evaluate boundaries before KBD initialization"))?;
    let context = guard_context(&state, boundary, subject);
    let label = if boundary == BoundaryKind::Task {
        "task"
    } else {
        "phase"
    };
    let verb = if edge == BoundaryEdge::Before {
        "Starting"
    } else {
        "Completed"
    };
    let exact_signal = format!(
        "{verb} {label} {} out of {}: {}",
        context.ordinal, context.total, context.name
    );
    let obligation_key = format!("{boundary:?}:{subject}").to_ascii_lowercase();
    let mut findings = Vec::new();
    if !context.valid {
        findings.push(format!(
            "{boundary:?} subject {subject} is not a unique canonical work item"
        ));
    }
    if edge == BoundaryEdge::Before && state.boundary_obligations.contains_key(&obligation_key) {
        findings.push(format!(
            "boundary {subject} already has an outstanding start receipt"
        ));
    }
    if edge == BoundaryEdge::After && !state.boundary_obligations.contains_key(&obligation_key) {
        findings.push(format!("boundary {subject} has no matching start receipt"));
    }

    let mismatches =
        runtime.compatibility_projection_mismatches_from_state(&state, projection_time)?;
    let mut repaired = Vec::new();
    if !mismatches.is_empty() {
        if repair_projections {
            let source_revision = state.revision;
            runtime.write_compatibility_projections_from_state(&state, projection_time)?;
            let after = runtime.replay()?;
            if after.revision != source_revision {
                return Err(anyhow!(
                    "projection repair changed canonical revision from {source_revision} to {}",
                    after.revision
                ));
            }
            let remaining =
                runtime.compatibility_projection_mismatches_from_state(&state, projection_time)?;
            if !remaining.is_empty() {
                findings.push(format!(
                    "{} projection mismatch(es) remain after repair",
                    remaining.len()
                ));
            } else {
                repaired = mismatches
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect();
            }
        } else {
            findings.push(format!(
                "{} compatibility projection(s) differ from canonical revision {}",
                mismatches.len(),
                state.revision
            ));
        }
    }

    let outcome = if !findings.is_empty() {
        BoundaryOutcome::Blocked
    } else if !repaired.is_empty() {
        BoundaryOutcome::Repaired
    } else {
        BoundaryOutcome::Pass
    };
    let receipt_id = digest_strings(&[
        &state.project_id,
        &state.run_id,
        &context.position,
        context.phase_id.as_deref().unwrap_or_default(),
        context.change_id.as_deref().unwrap_or_default(),
        context.task_id.as_deref().unwrap_or_default(),
        &format!("{boundary:?}"),
        &format!("{edge:?}"),
        subject,
        &state.revision.to_string(),
    ]);
    let receipt = BoundaryReceipt {
        id: receipt_id.clone(),
        boundary,
        edge,
        subject: subject.to_owned(),
        phase_id: context.phase_id,
        change_id: context.change_id,
        task_id: context.task_id,
        source_revision: state.revision,
        position: context.position,
        exact_signal: exact_signal.clone(),
        outcome,
        findings: findings.clone(),
        repaired_projections: repaired.clone(),
        observed_at: chrono::Utc::now(),
    };

    let mut committed_state = if precommit {
        state.clone()
    } else {
        client
            .submit_fresh(
                &state,
                current_actor(ActorKind::Harness),
                CommandKind::BoundaryReceiptRecord {
                    receipt: receipt.clone(),
                },
            )
            .await?
    };
    if outcome == BoundaryOutcome::Blocked {
        committed_state = record_guard_blocker(client, &committed_state, &receipt).await?;
    } else if !precommit {
        committed_state = clear_guard_blockers(client, &committed_state, &receipt).await?;
    }
    let revision = committed_state.revision;
    let output = json!({
        "outcome": format!("{outcome:?}").to_ascii_lowercase(),
        "authoritativeRevision": revision,
        "sourceRevision": state.revision,
        "position": receipt.position,
        "findings": findings,
        "outstandingObligations": committed_state.boundary_obligations,
        "exactSignal": exact_signal,
        "repairedProjections": repaired,
        "receiptId": (!precommit).then_some(receipt_id),
        "precommit": precommit
    });
    if json_output {
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("{exact_signal}");
        println!("Position: {} @ revision {revision}", receipt.position);
    }
    if outcome == BoundaryOutcome::Blocked {
        return Err(anyhow!(
            "boundary evaluation blocked: {}",
            receipt.findings.join("; ")
        ));
    }
    Ok(())
}

fn rust_processes() -> Vec<String> {
    let output = Command::new("ps").args(["-axo", "pid=,command="]).output();
    let Ok(output) = output else {
        return Vec::new();
    };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| {
            let mut fields = line.split_whitespace();
            let _pid = fields.next();
            fields
                .next()
                .and_then(|command| Path::new(command).file_name())
                .and_then(|command| command.to_str())
                .is_some_and(|command| matches!(command, "cargo" | "rustc"))
        })
        .map(str::trim)
        .map(str::to_owned)
        .collect()
}

fn command_available(command: &str) -> bool {
    std::env::var_os("PATH").is_some_and(|path| {
        std::env::split_paths(&path)
            .map(|directory| directory.join(command))
            .any(|candidate| candidate.is_file())
    })
}

fn phase_ready_for_gate(state: &RuntimeState, scope: &str) -> bool {
    state
        .active_path
        .phase_id
        .as_ref()
        .and_then(|phase_id| state.phases.get(phase_id))
        .is_some_and(|phase| {
            !phase.changes.is_empty()
                && phase.changes.values().all(|change| {
                    change.id == scope || change.implementation_status == WorkStatus::Complete
                })
        })
}

fn missing_certification_receipts(state: &RuntimeState) -> Vec<String> {
    // Certification runs immediately before the active phase's completion edge.
    // Its own phase obligation is therefore expected to remain open until the
    // certification gate passes and the completion transition can be recorded.
    let active_phase_obligation = state
        .active_path
        .phase_id
        .as_ref()
        .map(|phase_id| format!("phase:{phase_id}").to_ascii_lowercase());
    let mut missing = state
        .boundary_obligations
        .keys()
        .filter(|key| active_phase_obligation.as_ref() != Some(*key))
        .map(|key| format!("outstanding boundary {key}"))
        .collect::<Vec<_>>();
    if let Some(phase) = state
        .active_path
        .phase_id
        .as_ref()
        .and_then(|phase_id| state.phases.get(phase_id))
    {
        for task in phase
            .changes
            .values()
            .flat_map(|change| change.tasks.values())
        {
            if task.status != WorkStatus::Complete {
                continue;
            }
            let key = format!("task:{}", task.id).to_ascii_lowercase();
            let complete = state
                .latest_boundary_receipts
                .get(&key)
                .is_some_and(|receipt| {
                    receipt.edge == BoundaryEdge::After
                        && matches!(
                            receipt.outcome,
                            BoundaryOutcome::Pass | BoundaryOutcome::Repaired
                        )
                });
            if !complete {
                missing.push(format!(
                    "completed task {} has no valid kbd-apply receipt",
                    task.id
                ));
            }
        }
        let integrated = state.latest_gate_receipts.values().any(|receipt| {
            receipt.phase_id.as_ref() == Some(&phase.id)
                && receipt.kind == GateKind::Integration
                && receipt.outcome == GateOutcome::Passed
        });
        if !integrated {
            missing.push(format!("phase {} has no passed integration gate", phase.id));
        }
    }
    if state.blockers.values().any(|blocker| !blocker.resolved) {
        missing.push("canonical state contains unresolved blockers".to_owned());
    }
    if !state.active_gates.is_empty() {
        missing.push(format!(
            "{} gate(s) have no finish receipt",
            state.active_gates.len()
        ));
    }
    missing
}

async fn finish_gate(
    client: &ControlClient,
    state: &RuntimeState,
    gate: &GateRun,
    outcome: GateOutcome,
    exit_code: Option<i32>,
    duration_ms: u64,
    summary: String,
) -> Result<RuntimeState> {
    let receipt = GateReceipt {
        gate_id: gate.id.clone(),
        kind: gate.kind,
        scope: gate.scope.clone(),
        phase_id: gate.phase_id.clone(),
        source_revision: gate.source_revision,
        outcome,
        exit_code,
        duration_ms,
        finished_at: chrono::Utc::now(),
        summary,
    };
    client
        .submit_fresh(
            state,
            current_actor(ActorKind::Harness),
            CommandKind::GateFinish { receipt },
        )
        .await
}

async fn gate_run(
    root: &Path,
    runtime: &Runtime,
    client: &ControlClient,
    kind: GateKind,
    scope: &str,
    command: &[String],
) -> Result<()> {
    if command.is_empty() {
        return Err(anyhow!("kbd gate run requires an argv command after --"));
    }
    let state = state_or_replay(client, runtime).await?;
    let command_parts = command.iter().map(String::as_str).collect::<Vec<_>>();
    let command_sha256 = digest_strings(&command_parts);
    let gate_id = digest_strings(&[
        &state.project_id,
        &state.run_id,
        &format!("{kind:?}"),
        scope,
        &state.revision.to_string(),
        &command_sha256,
    ]);
    let executable = Path::new(&command[0])
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(&command[0])
        .to_owned();
    let is_rust = matches!(executable.as_str(), "cargo" | "rustc");
    let gate = GateRun {
        id: gate_id.clone(),
        kind,
        scope: scope.to_owned(),
        phase_id: state.active_path.phase_id.clone(),
        source_revision: state.revision,
        executable,
        command_sha256,
        worktree: fs::canonicalize(root)
            .unwrap_or_else(|_| root.to_path_buf())
            .display()
            .to_string(),
        target_dir: Some(
            std::env::var("CARGO_TARGET_DIR")
                .unwrap_or_else(|_| root.join("target").display().to_string()),
        ),
        sccache_available: std::env::var("RUSTC_WRAPPER")
            .is_ok_and(|value| value.contains("sccache"))
            || command_available("sccache"),
        started_at: chrono::Utc::now(),
    };
    let started = client
        .submit_fresh(
            &state,
            current_actor(ActorKind::Harness),
            CommandKind::GateStart { gate: gate.clone() },
        )
        .await?;

    let blocked_reason = if matches!(kind, GateKind::Integration | GateKind::Certification)
        && !phase_ready_for_gate(&state, scope)
    {
        Some("implementation is incomplete for the active phase".to_owned())
    } else {
        let active_rust = is_rust.then(rust_processes).unwrap_or_default();
        if !active_rust.is_empty() {
            Some(format!(
                "another Cargo/rustc process is active: {}",
                active_rust.join(" | ")
            ))
        } else if kind == GateKind::Certification {
            let missing = missing_certification_receipts(&state);
            (!missing.is_empty()).then(|| {
                format!(
                    "certification receipts are incomplete: {}",
                    missing.join("; ")
                )
            })
        } else {
            None
        }
    };
    if let Some(reason) = blocked_reason {
        let finished = finish_gate(
            client,
            &started,
            &gate,
            GateOutcome::Blocked,
            None,
            0,
            reason.clone(),
        )
        .await?;
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "gateId": gate_id,
                "outcome": "blocked",
                "reason": reason,
                "revision": finished.revision
            }))?
        );
        return Err(anyhow!("gate blocked"));
    }

    let timer = Instant::now();
    let status = Command::new(&command[0]).args(&command[1..]).status();
    let duration_ms = timer.elapsed().as_millis().min(u64::MAX as u128) as u64;
    let (outcome, exit_code, summary) = match status {
        Ok(status) if status.success() => (
            GateOutcome::Passed,
            status.code(),
            format!("{} gate passed", format!("{kind:?}").to_ascii_lowercase()),
        ),
        Ok(status) => (
            GateOutcome::Failed,
            status.code(),
            format!("command exited with {}", status.code().unwrap_or(-1)),
        ),
        Err(error) => (
            GateOutcome::Failed,
            None,
            format!("command failed to start: {error}"),
        ),
    };
    let current = state_or_replay(client, runtime).await?;
    let finished = finish_gate(
        client,
        &current,
        &gate,
        outcome,
        exit_code,
        duration_ms,
        summary.clone(),
    )
    .await?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "gateId": gate_id,
            "outcome": format!("{outcome:?}").to_ascii_lowercase(),
            "exitCode": exit_code,
            "durationMs": duration_ms,
            "summary": summary,
            "revision": finished.revision
        }))?
    );
    if outcome != GateOutcome::Passed {
        return Err(anyhow!("gate failed"));
    }
    Ok(())
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
                    "runtimeInitialized": false,
                    "initializationRequired": true,
                    "initializationAction": "run the intended typed mutation; initialization is automatic",
                    "runtimePath": runtime.runtime_root().display().to_string()
                });
                if json_output {
                    println!("{}", serde_json::to_string_pretty(&output)?);
                } else {
                    eprintln!("Control plane unavailable: {remote_error}");
                    println!(
                        "KBD mode: legacy (the first typed mutation initializes automatically)"
                    );
                    println!("Runtime: {}", runtime.runtime_root().display());
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
    match runtime.replay() {
        Ok(state) if state.revision > 0 => return Ok(state),
        Ok(_) | Err(RuntimeError::NotInitialized) => {}
        Err(error) => return Err(error.into()),
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
    transport: ControlTransport,
    project_id: String,
    runtime: Runtime,
}

impl ControlClient {
    fn new(runtime: &Runtime) -> Result<Self> {
        let manifest = runtime
            .project_manifest(false)?
            .ok_or_else(|| anyhow!("missing .prometheus/project.json"))?;
        Ok(Self {
            // A command that validates and fsyncs the journal can legitimately
            // take longer than the old two-second timeout after daemon startup.
            transport: ControlTransport::new(Duration::from_secs(30))?,
            project_id: manifest.project_id,
            runtime: runtime.clone(),
        })
    }

    async fn status(&self) -> Result<RuntimeState> {
        let response = self
            .transport
            .get(&format!("/api/v1/kbd/projects/{}/status", self.project_id))
            .await?;
        decode_response(response)
    }

    async fn events(&self) -> Result<Vec<Event>> {
        let response = self
            .transport
            .get(&format!("/api/v1/kbd/projects/{}/events", self.project_id))
            .await?;
        decode_response(response)
    }

    async fn audit_events(&self) -> Result<Vec<Event>> {
        let response = self
            .transport
            .get(&format!("/api/v1/kbd/projects/{}/audit", self.project_id))
            .await?;
        if !response.status.is_success() {
            return Err(anyhow!(
                "control plane returned {}: {}",
                response.status,
                response.body
            ));
        }
        response
            .body
            .lines()
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
            .transport
            .post_json(
                &format!("/api/v1/kbd/projects/{}/commands", self.project_id),
                &signed,
            )
            .await
            .map_err(classify_transport_failure)?;
        if !response.status.is_success() {
            return Err(classify_status(response.status, &response.body));
        }
        serde_json::from_str(&response.body)
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
        let projection_command = envelope.command.clone();
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
            .write_compatibility_projections_from_state_for_command(
                &result.state,
                chrono::Utc::now(),
                &projection_command,
            )
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

/// Resolve the state required to construct a mutation envelope.
///
/// Registration deliberately establishes identity without inventing a run. If no signed
/// runtime event exists yet, the first mutation crosses that boundary under operator
/// authority using the same legacy-aware initializer as migration. Read-only commands
/// continue to use `state_or_replay` so inspecting status never creates history.
async fn state_or_initialize(client: &ControlClient, runtime: &Runtime) -> Result<RuntimeState> {
    let remote_error = match client.status().await {
        Ok(state) if state.revision > 0 => return Ok(state),
        Ok(_) => None,
        Err(error) => Some(error),
    };
    match runtime.replay() {
        Ok(state) if state.revision > 0 => Ok(state),
        Ok(_) | Err(RuntimeError::NotInitialized) => {
            let initialized =
                ensure_runtime(runtime.project_root(), runtime).with_context(|| {
                    format!(
                        "initialize canonical KBD runtime at {} before typed mutation; \
                         registration succeeded but no signed run exists",
                        runtime.runtime_root().display()
                    )
                })?;
            let inventory = runtime.migrate_legacy_ledgers(false).with_context(|| {
                format!(
                    "inspect legacy KBD ledgers before the first typed mutation at {}",
                    runtime.runtime_root().display()
                )
            })?;
            if initialized.phases.is_empty() && inventory.progress_files > 0 {
                runtime.migrate_legacy_ledgers(true).with_context(|| {
                    format!(
                        "import legacy KBD ledgers before the first typed mutation at {}",
                        runtime.runtime_root().display()
                    )
                })?;
                runtime.replay().with_context(|| {
                    format!(
                        "replay initialized KBD runtime at {} after legacy import",
                        runtime.runtime_root().display()
                    )
                })
            } else {
                Ok(initialized)
            }
        }
        Err(error) => match remote_error {
            Some(remote_error) => {
                Err(error).with_context(|| format!("control plane unavailable ({remote_error})"))
            }
            None => Err(error.into()),
        },
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
fn classify_transport_failure(failure: TransportFailure) -> ControlFailure {
    match failure {
        TransportFailure::Unreachable(error) => ControlFailure::Unreachable(error),
        TransportFailure::Ambiguous(error) => ControlFailure::Ambiguous(error),
    }
}

/// A 503 from the startup gate means the daemon refused the route before any
/// command was dispatched — unreachability wearing an HTTP status. Any other
/// 5xx is ambiguous: the daemon may have committed and then failed to respond.
/// A 4xx is a decision about this command.
fn classify_status(status: hyper::StatusCode, body: &str) -> ControlFailure {
    let error = anyhow!("control plane returned {status}: {body}");
    if status == hyper::StatusCode::SERVICE_UNAVAILABLE {
        ControlFailure::Unreachable(error)
    } else if status.is_server_error() {
        ControlFailure::Ambiguous(error)
    } else {
        ControlFailure::Rejected(error)
    }
}

fn decode_response<T: serde::de::DeserializeOwned>(response: ControlResponse) -> Result<T> {
    if !response.status.is_success() {
        return Err(classify_status(response.status, &response.body).into_error());
    }
    serde_json::from_str(&response.body).context("invalid control-plane JSON response")
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

fn project_successor_and_release_pause(
    root: &Path,
    runtime: &Runtime,
    state: &RuntimeState,
) -> Result<()> {
    runtime
        .write_compatibility_projections_from_state_scoped(
            state,
            chrono::Utc::now(),
            &ProjectionScope::GlobalOnly,
        )
        .with_context(|| {
            format!(
                "successor run {} committed at revision {}, but projections failed; PAUSE remains active",
                state.run_id, state.revision
            )
        })?;
    release_pause_valve(root).with_context(|| {
        format!(
            "successor run {} committed and projected at revision {}, but PAUSE could not be released",
            state.run_id, state.revision
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use kbd_runtime::{CommandResult, EVENT_SCHEMA_VERSION};
    use tempfile::tempdir;

    #[test]
    fn ensure_runtime_initializes_from_legacy_once() {
        let fixture = tempdir().unwrap();
        let root = fixture.path();
        fs::create_dir_all(root.join(".kbd-orchestrator")).unwrap();
        fs::write(
            root.join(".kbd-orchestrator/current-waypoint.json"),
            serde_json::to_vec_pretty(&json!({
                "phase": "legacy-phase",
                "status": "executing",
                "exactNextCommand": "/kbd-execute legacy-phase",
                "planRevision": 7
            }))
            .unwrap(),
        )
        .unwrap();
        let runtime = Runtime::open(root);
        assert_eq!(runtime.replay().unwrap().revision, 0);

        let initialized = ensure_runtime(root, &runtime).unwrap();
        assert_eq!(initialized.revision, 1);
        assert_eq!(initialized.lifecycle, LifecycleState::Running);
        assert_eq!(initialized.plan_revision, 7);
        assert_eq!(
            initialized.exact_next_work.as_deref(),
            Some("/kbd-execute legacy-phase")
        );
        assert!(initialized.run_id.starts_with("legacy-phase-"));
        assert_eq!(runtime.events().unwrap().len(), 1);

        let reopened = ensure_runtime(root, &runtime).unwrap();
        assert_eq!(reopened.revision, initialized.revision);
        assert_eq!(reopened.run_id, initialized.run_id);
        assert_eq!(runtime.events().unwrap().len(), 1);
    }

    fn committed_successor(root: &Path) -> (Runtime, CommandResult) {
        fs::create_dir_all(root.join(".kbd-orchestrator")).unwrap();
        let runtime = Runtime::open(root);
        let operator = Actor::operator("operator", "test");
        let initialized = runtime
            .initialize("project", "run-a", operator.clone())
            .unwrap();
        let cancelled = runtime
            .execute_command(CommandEnvelope {
                schema_version: EVENT_SCHEMA_VERSION.into(),
                project_id: "project".into(),
                run_id: "run-a".into(),
                command_id: "cancel-run-a".into(),
                frontier: Some(initialized.frontier),
                expected_revision: initialized.revision,
                actor: operator.clone(),
                command: CommandKind::Cancel {
                    reason: "terminal".into(),
                },
            })
            .unwrap();
        let successor = runtime
            .execute_command(CommandEnvelope {
                schema_version: EVENT_SCHEMA_VERSION.into(),
                project_id: "project".into(),
                run_id: "run-a".into(),
                command_id: "start-run-b".into(),
                frontier: Some(cancelled.state.frontier.clone()),
                expected_revision: cancelled.state.revision,
                actor: operator,
                command: CommandKind::RunStart {
                    run_id: "run-b".into(),
                    reason: "new work".into(),
                    exact_next_work: Some("/kbd-new-phase".into()),
                },
            })
            .unwrap();
        (runtime, successor)
    }

    #[test]
    fn successor_projection_precedes_pause_release() {
        let fixture = tempdir().unwrap();
        let root = fixture.path();
        let pause = root.join(".kbd-orchestrator/PAUSE");
        fs::create_dir_all(pause.parent().unwrap()).unwrap();
        fs::write(&pause, "cancelled").unwrap();
        let (runtime, successor) = committed_successor(root);

        project_successor_and_release_pause(root, &runtime, &successor.state).unwrap();

        let waypoint: Value = serde_json::from_reader(
            fs::File::open(root.join(".kbd-orchestrator/current-waypoint.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(waypoint["runId"], "run-b");
        assert_eq!(waypoint["status"], "ready");
        assert_eq!(waypoint["implementationCompleted"], 0);
        assert_eq!(waypoint["implementationTotal"], 0);
        assert!(!pause.exists());
        assert!(fs::read_dir(root.join(".kbd-orchestrator"))
            .unwrap()
            .filter_map(|entry| entry.ok())
            .any(|entry| entry
                .file_name()
                .to_string_lossy()
                .starts_with("PAUSE.released.")));
    }

    #[test]
    fn projection_failure_keeps_pause_active_after_successor_commit() {
        let fixture = tempdir().unwrap();
        let root = fixture.path();
        let pause = root.join(".kbd-orchestrator/PAUSE");
        fs::create_dir_all(pause.parent().unwrap()).unwrap();
        fs::write(&pause, "cancelled").unwrap();
        let (runtime, successor) = committed_successor(root);
        fs::create_dir(root.join(".kbd-orchestrator/current-waypoint.json")).unwrap();

        let error = project_successor_and_release_pause(root, &runtime, &successor.state)
            .expect_err("projection failure must stop PAUSE release");

        assert!(error.to_string().contains("PAUSE remains active"));
        assert!(pause.is_file());
        assert_eq!(runtime.replay().unwrap().run_id, "run-b");
    }
}
