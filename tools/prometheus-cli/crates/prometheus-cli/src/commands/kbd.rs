use anyhow::{anyhow, Context, Result};
use kbd_runtime::{
    project_document::fold_project_events,
    registry::{scan_submodule_pins, ProjectRegistry, RegistryPruneReport},
    rollout::{RolloutObservation, RolloutTracker},
    Actor, ActorKind, Blocker, BoundaryEdge, BoundaryKind, BoundaryOutcome, BoundaryReceipt,
    Checkpoint, ClaimMode, CommandEnvelope, CommandKind, CommandResult, Event, EventKind, GateKind,
    GateOutcome, GateReceipt, GateRun, LifecycleState, ProjectionScope, Runtime, RuntimeError,
    RuntimeState, WorkStatus, EVENT_SCHEMA_VERSION,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use super::control_transport::{ControlResponse, ControlTransport, TransportFailure};

pub enum Action {
    InspectChangeIdentity {
        phase: String,
    },
    Status {
        json: bool,
        receiver_only: bool,
    },
    Projects {
        json: bool,
        prune_missing: bool,
        apply: bool,
    },
    Register {
        path: String,
    },
    Replicas {
        project_id: Option<String>,
        reprobe: bool,
        replica: Option<String>,
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
        command_id: Option<String>,
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
        receiver_only: bool,
    },
    Watch {
        after_event: Option<String>,
        receiver_only: bool,
    },
    Migrate {
        check: bool,
        apply: bool,
    },
    MigrateProjections {
        dry_run: bool,
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
    PhaseClosure(PhaseClosureAction),
    Device(DeviceAction),
}

#[path = "kbd_phase.rs"]
mod phase;
pub use phase::PhaseClosureAction;

#[path = "kbd_device.rs"]
mod device;
pub use device::DeviceAction;

/// Commands the runtime admits only under a signed operator key.
fn requires_operator(command: &CommandKind) -> bool {
    matches!(
        command,
        CommandKind::ChangeRepairIdentity { .. }
            | CommandKind::DeviceEnroll { .. }
            | CommandKind::DeviceRevoke { .. }
            | CommandKind::DeviceRotate { .. }
            | CommandKind::OperatorGrant { .. }
    )
}

pub async fn run(path: &str, action: Action) -> Result<()> {
    let action = match action {
        Action::MigrateProjections { dry_run } => {
            // Projection migration must bypass canonical startup, the control
            // client, legacy journal import and checkpoint/recovery effects.
            let root = find_manifest_project_root(Path::new(path))?;
            let runtime = Runtime::open_projection_migration(&root)?;
            let report = runtime.migrate_phase_projections(dry_run)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            return Ok(());
        }
        Action::Projects {
            json,
            prune_missing,
            apply,
        } => {
            let registry = ProjectRegistry::open();
            if prune_missing {
                let report = registry.prune_missing(apply)?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&report)?);
                } else {
                    print_registry_prune_report(&report);
                }
            } else {
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
            }
            return Ok(());
        }
        Action::Register { path } => {
            let outcome = ProjectRegistry::open().register_existing(&path)?;
            println!("{}", serde_json::to_string_pretty(&outcome)?);
            return Ok(());
        }
        Action::Replicas { project_id, json, reprobe, replica } => {
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
            if reprobe {
                let selected = if let Some(id) = replica {
                    registry.lookup_project(&project_id)?.into_iter()
                        .find(|(_, entry)| entry.replica_id == id)
                        .map(|(root, _)| root)
                        .ok_or_else(|| anyhow!("unknown replica {id} for project {project_id}"))?
                } else {
                    find_manifest_project_root(Path::new(path))?
                };
                let entry = registry.lookup_path(&selected)?
                    .ok_or_else(|| anyhow!("checkout is not registered: {}", selected.display()))?;
                if entry.project_id != project_id {
                    anyhow::bail!("checkout belongs to a different project");
                }
                registry.register_existing(&selected)?;
            }
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
    // A legacy checkout has a readable waypoint before its first typed write
    // creates a project identity. Inspecting it must not register the checkout.
    let read_mode = match &action {
        Action::Status {
            receiver_only: true, ..
        }
        | Action::Audit {
            receiver_only: true, ..
        }
        | Action::Watch {
            receiver_only: true, ..
        } => ControlMode::ReceiverOnly,
        _ if remote_control_enabled() => ControlMode::CompanionWithFallback,
        _ => ControlMode::Local,
    };
    if let Action::Status { json, .. } = &action {
        if read_mode == ControlMode::Local && !root.join(".prometheus/project.json").try_exists()? {
            return print_uninitialized_status(&root, None, *json, ReadSource::Local);
        }
    }
    let runtime = if matches!(
        &action,
        Action::Status { .. }
        | Action::Audit { .. }
        | Action::Conflicts { .. }
        | Action::Claims { .. }
        | Action::Watch { .. }
        | Action::Device(DeviceAction::List { .. })
        | Action::Submodules { scan: false, .. }
        | Action::Migrate { apply: false, .. }
        | Action::RolloutStatus
        | Action::PhaseClosure(PhaseClosureAction::Disposition { .. } | PhaseClosureAction::ValidateExit { .. })
        | Action::GuardEvaluate {
            precommit: true,
            ..
        }
    ) {
        Runtime::open_canonical_snapshot(&root)?
    } else {
        Runtime::open_canonical(&root)?
    };
    let client = ControlClient::new(&runtime, read_mode)?;
    match action {
        Action::Status { json, .. } => status(&root, &runtime, &client, json).await,
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
            command_id,
        } => {
            let state = state_or_initialize(&client, &runtime).await?;
            let mut actor = current_actor(ActorKind::Harness);
            if let Some(holder_id) = holder_id {
                actor.id = holder_id;
            }
            let holder_id = actor.id.clone();
            let command = CommandKind::ClaimAcquire {
                scope,
                mode,
                ttl_seconds,
                holder_id,
            };
            let next = client
                .submit_with_id(&state, actor, command_id, command)
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
            ..
        } => audit(&runtime, &client, since.as_deref(), json, export_git).await,
        Action::Watch { after_event, .. } => watch(&client, after_event.as_deref()).await,
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
            let events = client.read_history().await?.value.events;
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
        Action::InspectChangeIdentity { phase } => {
            let state = runtime.replay_authority()?;
            let selected = state
                .phases
                .get(&phase)
                .ok_or_else(|| anyhow!("phase {phase:?} not found"))?;
            let changes = selected
                .changes
                .iter()
                .map(|(key, change)| {
                    Ok(json!({"key": key, "embeddedId": change.id,
                    "sha256": kbd_runtime::change_identity::change_digest(change)?,
                    "record": change}))
                })
                .collect::<Result<Vec<Value>>>()?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "schema": "kbd.change-identity-inspection/v1", "projectId": state.project_id,
                    "runId": state.run_id, "revision": state.revision, "phaseId": phase,
                    "issues": kbd_runtime::change_identity::phase_issues(selected), "changes": changes,
                    "boundaryObligations": state.boundary_obligations,
                    "unmatchedObligations": state.boundary_obligations.values().filter(|obligation| {
                        if obligation.boundary == BoundaryKind::Phase { return false; }
                        let subject = obligation.subject.as_str();
                        let (change_part, task_part) = subject
                            .split_once('/').or_else(|| subject.split_once("::")).or_else(|| subject.split_once(':'))
                            .map_or((subject, None), |(c, t)| (c, Some(t)));
                        !selected.changes.get(change_part).is_some_and(|change| match task_part {
                            Some(task) => change.tasks.contains_key(task),
                            None => true,
                        })
                    }).map(|obligation| obligation.key.clone()).collect::<Vec<_>>(),
                    "note": "Inspection grants no repair authority; review exact source/task/receipt mappings before repair."
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
            let state = state_or_initialize(&client, &runtime).await?;
            let result = submit_typed(&client, &state, command_id, command).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
            Ok(())
        }
        Action::PhaseClosure(action) => phase::run(&runtime, &client, action).await,
        Action::Device(action) => device::run(&runtime, &client, action).await,
        Action::Projects { .. }
        | Action::MigrateProjections { .. }
        | Action::Register { .. }
        | Action::Replicas { .. }
        | Action::Adopt { .. } => unreachable!("early actions return before project open"),
    }
}

/// Submit one typed command built from `state`, through the control plane when
/// it is enabled and reachable, otherwise through the local canonical runtime.
async fn submit_typed(
    client: &ControlClient,
    state: &RuntimeState,
    command_id: String,
    command: CommandKind,
) -> Result<Value> {
    let envelope = CommandEnvelope {
        schema_version: "2".into(),
        project_id: state.project_id.clone(),
        run_id: state.run_id.clone(),
        command_id,
        frontier: Some(state.frontier.clone()),
        expected_revision: state.revision,
        actor: current_actor(if requires_operator(&command) {
            ActorKind::Operator
        } else {
            ActorKind::Harness
        }),
        command,
    };
    client.submit(state, envelope).await
}

fn print_registry_prune_report(report: &RegistryPruneReport) {
    let mode = if report.apply_requested {
        "apply"
    } else {
        "dry run"
    };
    println!("Registry prune {mode}");
    println!("Registry: {}", report.registry_path);
    println!("Candidates: {}", report.candidates.len());
    for candidate in &report.candidates {
        println!(
            "Candidate: {}  {}  {}  {:?}{}",
            candidate.project_id,
            candidate.replica_id,
            candidate.path,
            candidate.kind,
            if candidate.read_only {
                "  read-only"
            } else {
                ""
            }
        );
    }
    println!("Removed: {}", report.removed.len());
    println!("Retained registrations: {}", report.retained_registrations);
    if !report.apply_requested {
        println!("No changes applied. Re-run with --prune-missing --apply to remove candidates.");
    } else if !report.applied {
        println!("No registry changes were required.");
    }
    if let Some(path) = &report.backup_path {
        println!("Backup: {path}");
    }
    if let Some(checksum) = &report.backup_sha256 {
        println!("Backup SHA-256: {checksum}");
    }
    if let Some(path) = &report.checksum_path {
        println!("Checksum: {path}");
    }
    if let Some(path) = &report.receipt_path {
        println!("Receipt: {path}");
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

fn guard_context(state: &RuntimeState, boundary: BoundaryKind, subject: &str) -> GuardContext {
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
        BoundaryKind::Change => {
            let phase_id = active_phase_id.clone();
            let phase = phase_id.as_ref().and_then(|id| state.phases.get(id));
            let matches = if let Some(exact) = phase.and_then(|phase| phase.changes.get(subject)) {
                vec![exact]
            } else {
                phase
                    .into_iter()
                    .flat_map(|phase| phase.changes.values())
                    .filter(|change| change.id == subject || change.title == subject)
                    .collect::<Vec<_>>()
            };
            let selected = (matches.len() == 1).then(|| matches[0]);
            let mut changes = phase
                .map(|phase| phase.changes.values().collect::<Vec<_>>())
                .unwrap_or_default();
            changes.sort_by(|left, right| {
                left.sequence
                    .cmp(&right.sequence)
                    .then_with(|| left.id.cmp(&right.id))
            });
            let selected_id = selected
                .map(|change| change.id.clone())
                .unwrap_or_else(|| subject.to_owned());
            let ordinal = changes
                .iter()
                .position(|change| change.id == selected_id)
                .unwrap_or(0)
                + 1;
            let name = selected
                .map(|change| change.title.clone())
                .unwrap_or_else(|| subject.to_owned());
            let mut position = phase_path;
            position.push(selected_id.clone());
            GuardContext {
                phase_id,
                change_id: selected.map(|_| selected_id),
                task_id: None,
                ordinal,
                total: changes.len().max(1),
                name,
                position: position.join(" › "),
                valid: selected.is_some(),
            }
        }
        BoundaryKind::Task => {
            let phase_id = active_phase_id.clone();
            let phase = phase_id.as_ref().and_then(|id| state.phases.get(id));
            let qualified = subject
                .split_once('/')
                .or_else(|| subject.split_once("::"))
                .or_else(|| subject.split_once(':'));
            let qualified_match = qualified.and_then(|(change_id, task_id)| {
                phase
                    .and_then(|phase| phase.changes.get(change_id))
                    .and_then(|change| change.tasks.get(task_id))
                    .map(|task| (change_id.to_owned(), task.id.clone()))
            });
            let (change_id, task_id) = if let Some((change_id, task_id)) = qualified_match {
                (Some(change_id), Some(task_id))
            } else {
                let mut matches = Vec::new();
                for change in phase.into_iter().flat_map(|phase| phase.changes.values()) {
                    for task in change.tasks.values() {
                        if task.id == subject || task.title == subject {
                            matches.push((change.id.clone(), task.id.clone()));
                        }
                    }
                }
                if matches.len() == 1 {
                    let (change_id, task_id) = matches.remove(0);
                    (Some(change_id), Some(task_id))
                } else {
                    (None, None)
                }
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
        // Retired in 1.12: present only in replayed history, never a new subject.
        BoundaryKind::Zeespec => {
            let mut position = phase_path;
            position.push(format!("zeespec:{subject}"));
            GuardContext {
                phase_id: active_phase_id,
                change_id: None,
                task_id: None,
                ordinal: 1,
                total: 1,
                name: subject.to_owned(),
                position: position.join(" › "),
                valid: false,
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
    let label = match boundary {
        BoundaryKind::Task => "task",
        BoundaryKind::Change => "change",
        BoundaryKind::Phase => "phase",
        BoundaryKind::Zeespec => "zeespec stage",
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
    if let Some(phase) = state
        .active_path
        .phase_id
        .as_ref()
        .and_then(|id| state.phases.get(id))
    {
        findings.extend(kbd_runtime::change_identity::phase_issues(phase));
        if !context.valid && boundary == BoundaryKind::Task {
            let qualified = subject
                .split_once('/')
                .or_else(|| subject.split_once("::"))
                .or_else(|| subject.split_once(':'));
            if let Some((change, task)) = qualified {
                findings.push(format!("phase {} missing qualified task key {change}/{task}; inspect registered task keys, not backend display labels", phase.id));
            } else {
                let matches = phase
                    .changes
                    .iter()
                    .flat_map(|(key, change)| {
                        change.tasks.iter().filter_map(move |(task_key, task)| {
                            (task.id == subject || task.title == subject)
                                .then(|| format!("{key}/{task_key}"))
                        })
                    })
                    .collect::<Vec<_>>();
                findings.push(format!(
                    "phase {} task subject {subject:?} has {} matches: {matches:?}",
                    phase.id,
                    matches.len()
                ));
            }
        }
    }
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
    #[cfg(windows)]
    {
        let Ok(output) = Command::new("tasklist.exe")
            .args(["/FO", "CSV", "/NH"])
            .output()
        else {
            return Vec::new();
        };
        return String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|line| {
                line.split(',').next().is_some_and(|name| {
                    matches!(
                        name.trim_matches('"').to_ascii_lowercase().as_str(),
                        "cargo.exe" | "rustc.exe"
                    )
                })
            })
            .map(str::to_owned)
            .collect();
    }
    #[cfg(not(windows))]
    {
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
}

fn command_available(command: &str) -> bool {
    crate::host::find_executable(command).is_some()
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
        for change in phase.changes.values() {
            for task in change.tasks.values() {
                if task.status != WorkStatus::Complete {
                    continue;
                }
                let complete = state.latest_boundary_receipts.values().any(|receipt| {
                    receipt.boundary == BoundaryKind::Task
                        && receipt.phase_id.as_deref() == Some(phase.id.as_str())
                        && receipt.change_id.as_deref() == Some(change.id.as_str())
                        && receipt.task_id.as_deref() == Some(task.id.as_str())
                        && receipt.edge == BoundaryEdge::After
                        && matches!(
                            receipt.outcome,
                            BoundaryOutcome::Pass | BoundaryOutcome::Repaired
                        )
                });
                if !complete {
                    missing.push(format!(
                        "completed task {}/{} has no valid kbd-apply receipt",
                        change.id, task.id
                    ));
                }
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
    let Sourced { value: state, source } =
        client.status().await.map_err(ControlFailure::into_error)?;
    if state.revision == 0 {
        return print_uninitialized_status(root, Some(runtime.runtime_root()), json_output, source);
    }
    source.report();
    if json_output {
        let mut output = serde_json::to_value(&state)?;
        output["readSource"] = json!(source.name());
        println!("{}", serde_json::to_string_pretty(&output)?);
        Ok(())
    } else {
        print_state(&state, false)
    }
}

fn print_uninitialized_status(
    root: &Path,
    runtime_path: Option<&Path>,
    json_output: bool,
    source: ReadSource,
) -> Result<()> {
    source.report();
    let legacy = read_waypoint(root);
    let output = json!({
        "mode": "legacy",
        "phase": string_field(&legacy, &["phase"]),
        "status": string_field(&legacy, &["status", "stage"]),
        "exactNextWork": string_field(&legacy, &["exactNextCommand", "exact_next_command"]),
        "runtimeInitialized": false,
        "initializationRequired": true,
        "initializationAction": "run the intended typed mutation; initialization is automatic",
        "runtimePath": runtime_path,
        "readSource": source.name()
    });
    if json_output {
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("KBD mode: legacy (the first typed mutation initializes automatically)");
        if let Some(path) = runtime_path {
            println!("Runtime: {}", path.display());
        }
        println!("Phase: {}", output["phase"].as_str().unwrap_or("unknown"));
        println!("Status: {}", output["status"].as_str().unwrap_or("unknown"));
    }
    Ok(())
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
    let phase_ids = if state.active_path.phase_path.is_empty() {
        state
            .active_path
            .phase_id
            .iter()
            .cloned()
            .collect::<Vec<_>>()
    } else {
        state.active_path.phase_path.clone()
    };
    let mut cursor = phase_ids
        .iter()
        .filter_map(|phase_id| state.phases.get(phase_id))
        .map(|phase| phase.slug.clone())
        .collect::<Vec<_>>();
    cursor.extend(
        [
            state.active_path.stage_id.as_ref(),
            state.active_path.change_id.as_ref(),
            state.active_path.task_id.as_ref(),
        ]
        .into_iter()
        .flatten()
        .cloned(),
    );
    if !cursor.is_empty() {
        println!("Position: {}", cursor.join(" › "));
    }
    if let Some(change) = state.active_path.change_id.as_deref() {
        match state.active_path.task_id.as_deref() {
            Some(task) => println!("Next work: change {change}, task {task}"),
            None => println!("Next work: change {change}"),
        }
    }
    if let Some(intent) = state.exact_next_work.as_deref() {
        println!("Operator note: {intent}");
    }
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
    refuse_silent_fork(&waypoint)?;
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum ControlMode {
    Local,
    CompanionWithFallback,
    ReceiverOnly,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ReadSource {
    Local,
    LocalFallback,
    Companion,
}

impl ReadSource {
    fn name(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::LocalFallback => "local-fallback",
            Self::Companion => "companion",
        }
    }

    fn report(self) {
        eprintln!("KBD source: {}", self.name());
    }
}

struct Sourced<T> {
    value: T,
    source: ReadSource,
}

struct VerifiedHistory {
    events: Vec<Event>,
    state: RuntimeState,
}

struct ControlClient {
    transport: ControlTransport,
    project_id: String,
    runtime: Runtime,
    mode: ControlMode,
}

pub(super) fn remote_control_enabled() -> bool {
    std::env::var("PROMETHEUS_KBD_CONTROL_PLANE")
        .is_ok_and(|value| matches!(value.as_str(), "1" | "true" | "yes"))
}

impl ControlClient {
    fn new(runtime: &Runtime, mode: ControlMode) -> Result<Self> {
        let manifest = runtime
            .project_manifest(false)?
            .ok_or_else(|| anyhow!("missing .prometheus/project.json"))?;
        Ok(Self {
            // A command that validates and fsyncs the journal can legitimately
            // take longer than the old two-second timeout after daemon startup.
            transport: ControlTransport::new(Duration::from_secs(30))?,
            project_id: manifest.project_id,
            runtime: runtime.clone(),
            mode,
        })
    }

    fn route(&self, resource: &str) -> String {
        format!("/api/v1/kbd/projects/{}/{resource}", self.project_id)
    }

    async fn read_remote(&self, resource: &str) -> Result<ControlResponse, ControlFailure> {
        let response = self
            .transport
            .get_classified(&self.route(resource))
            .await
            .map_err(|failure| match failure {
                TransportFailure::Unreachable(error) | TransportFailure::Disconnected(error) => {
                    ControlFailure::Unreachable(error)
                }
                TransportFailure::Ambiguous(error) | TransportFailure::Invalid(error) => {
                    ControlFailure::Rejected(error)
                }
            })?;
        if response.status.is_success() {
            return Ok(response);
        }
        Err(classify_status(&response))
    }

    async fn status(&self) -> Result<Sourced<RuntimeState>, ControlFailure> {
        let source = match self.mode {
            ControlMode::Local => ReadSource::Local,
            ControlMode::CompanionWithFallback | ControlMode::ReceiverOnly => {
                match self.read_remote_status().await {
                    Ok(value) => return Ok(Sourced { value, source: ReadSource::Companion }),
                    Err(ControlFailure::Unreachable(_))
                        if self.mode == ControlMode::CompanionWithFallback =>
                    {
                        ReadSource::LocalFallback
                    }
                    Err(error) => return Err(error),
                }
            }
        };
        let value = match self.runtime.replay() {
            Ok(state) => state,
            Err(RuntimeError::NotInitialized) => RuntimeState::default(),
            Err(error) => return Err(ControlFailure::Rejected(error.into())),
        };
        Ok(Sourced { value, source })
    }

    async fn read_remote_status(&self) -> Result<RuntimeState, ControlFailure> {
        let response = self.read_remote("status").await?;
        let state: RuntimeState =
            decode_json_response(&response).map_err(ControlFailure::Rejected)?;
        self.validate_remote_state(&state)
            .map_err(ControlFailure::Rejected)?;
        Ok(state)
    }

    fn validate_remote_state(&self, state: &RuntimeState) -> Result<()> {
        if state.schema_version != EVENT_SCHEMA_VERSION || state.revision == 0 {
            anyhow::bail!("incompatible Companion status schema or revision");
        }
        if state.project_id != self.project_id {
            anyhow::bail!(
                "Companion returned project {} for requested project {}",
                state.project_id, self.project_id
            );
        }
        Ok(())
    }

    async fn read_history(&self) -> Result<Sourced<VerifiedHistory>> {
        let source = match self.mode {
            ControlMode::Local => ReadSource::Local,
            ControlMode::CompanionWithFallback | ControlMode::ReceiverOnly => {
                match self.read_remote_history().await {
                    Ok(value) => return Ok(Sourced { value, source: ReadSource::Companion }),
                    Err(ControlFailure::Unreachable(_))
                        if self.mode == ControlMode::CompanionWithFallback =>
                    {
                        ReadSource::LocalFallback
                    }
                    Err(error) => return Err(error.into_error()),
                }
            }
        };
        let value = self.verify_history(self.runtime.events()?)?;
        Ok(Sourced { value, source })
    }

    async fn read_remote_history(&self) -> Result<VerifiedHistory, ControlFailure> {
        let response = self.read_remote("audit").await?;
        self.decode_history(&response)
            .map_err(ControlFailure::Rejected)
    }

    fn decode_history(&self, response: &ControlResponse) -> Result<VerifiedHistory> {
        require_media_type(response, "application/x-ndjson")?;
        let events = response
            .body
            .lines()
            .filter(|line| !line.trim().is_empty())
            .enumerate()
            .map(|(index, line)| {
                serde_json::from_str(line)
                    .with_context(|| format!("invalid Companion audit JSONL at line {}", index + 1))
            })
            .collect::<Result<_>>()?;
        self.verify_history(events)
    }

    fn verify_history(&self, events: Vec<Event>) -> Result<VerifiedHistory> {
        if let Some(event) = events
            .iter()
            .find(|event| event.project_id != self.project_id)
        {
            anyhow::bail!(
                "event {} belongs to project {}, expected {}",
                event.event_id, event.project_id, self.project_id
            );
        }
        let state = fold_project_events(&events).context("invalid signed KBD audit")?;
        Ok(VerifiedHistory { events, state })
    }

    async fn submit(&self, state: &RuntimeState, envelope: CommandEnvelope) -> Result<Value> {
        if state.command_revisions.contains_key(&envelope.command_id)
            && is_session_command(&envelope.command)
        {
            let history = self.read_history().await?;
            Self::audit_receipt(&history.value, &envelope)?
                .ok_or_else(|| anyhow!(
                    "command {} is absent from the selected audit; reconcile before retrying",
                    envelope.command_id
                ))?;
        }
        if self.mode == ControlMode::Local {
            let mut result = serde_json::to_value(self.execute_locally(envelope)?)?;
            result["committedLocally"] = json!(true);
            result["controlPlane"] = json!("disabled");
            return Ok(result);
        }
        match self.submit_remote(&envelope).await {
            Ok(response) => Ok(response),
            Err(ControlFailure::Unreachable(error))
                if self.mode == ControlMode::CompanionWithFallback =>
            {
                eprintln!(
                    "control plane unreachable before dispatch ({error}); committing command {} locally",
                    envelope.command_id
                );
                let mut result = serde_json::to_value(self.execute_locally(envelope)?)?;
                result["committedLocally"] = json!(true);
                result["remoteStatusUnknown"] = json!(false);
                Ok(result)
            }
            Err(ControlFailure::Ambiguous(error)) => {
                self.reconcile_submission(&envelope, error).await
            }
            Err(error) => Err(error.into_error()),
        }
    }

    async fn submit_remote(&self, envelope: &CommandEnvelope) -> Result<Value, ControlFailure> {
        // Read-only commands must never touch the platform credential store.
        // Resolve the signer only for an actual mutation, and keep the
        // synchronous OS credential lookup off the async executor.
        let runtime = self.runtime.clone();
        let signing_state = self
            .runtime
            .replay_authority()
            .map_err(|error| ControlFailure::Rejected(error.into()))?;
        let command_id = envelope.command_id.clone();
        let envelope = envelope.clone();
        let signed = match tokio::task::spawn_blocking(move || {
            runtime.sign_command_envelope(&signing_state, envelope)
        })
        .await
        .context("device signer task failed")
        {
            Ok(Ok(signed)) => signed,
            // A missing/locked signer is a local problem, not the daemon's, and
            // local execution needs the same signer — so this is terminal.
            Ok(Err(error)) => return Err(ControlFailure::Rejected(error.into())),
            Err(error) => return Err(ControlFailure::Rejected(error)),
        };
        let response = self
            .transport
            .post_json(&self.route("commands"), &signed)
            .await
            .map_err(classify_transport_failure)?;
        if !response.status.is_success() {
            return Err(classify_status(&response));
        }
        self.decode_commit(&response, &command_id)
            .with_context(|| format!("invalid Companion response for command {command_id}"))
            .map_err(ControlFailure::Rejected)
    }

    fn decode_commit(&self, response: &ControlResponse, command_id: &str) -> Result<Value> {
        let value: Value = decode_json_response(response)?;
        let result: CommandResult = serde_json::from_value(value.clone())?;
        self.validate_remote_state(&result.state)?;
        if result.command_id != command_id
            || result.committed_revision == 0
            || result.committed_revision > result.state.revision
        {
            anyhow::bail!("Companion did not acknowledge command {command_id}");
        }
        if let Some(error) = &result.apply_error {
            anyhow::bail!("Companion rejected command {command_id}: {error}");
        }
        Ok(value)
    }

    async fn reconcile_submission(
        &self,
        envelope: &CommandEnvelope,
        error: anyhow::Error,
    ) -> Result<Value> {
        let command_id = &envelope.command_id;
        let history = self.read_remote_history().await.map_err(|audit_error| {
            anyhow!(
                "control plane submission ambiguous for command {command_id} ({error}). \
                 Companion audit unavailable ({}). Check `prometheus kbd audit --receiver-only --json` \
                 before retrying with the same command ID",
                audit_error.into_error()
            )
        })?;
        let receipt = Self::audit_receipt(&history, envelope)?.ok_or_else(|| {
            anyhow!(
                "control plane submission ambiguous for command {command_id} ({error}). \
                 No committed effect found in the Companion audit. Recheck the audit before \
                 retrying with the same command ID"
            )
        })?;
        eprintln!(
            "control plane submission ambiguous for command {command_id}. Confirmed by signed Companion audit"
        );
        Ok(receipt)
    }

    fn audit_receipt(history: &VerifiedHistory, envelope: &CommandEnvelope) -> Result<Option<Value>> {
        let Some(committed_revision) = history.state.command_revisions.get(&envelope.command_id) else {
            return Ok(None);
        };
        let mut events = history
            .events
            .iter()
            .filter(|event| event.command_id.as_deref() == Some(envelope.command_id.as_str()));
        let event = events.next().ok_or_else(|| anyhow!(
            "signed audit omitted committed command {}", envelope.command_id
        ))?;
        if events.next().is_some()
            || event.actor.id != envelope.actor.id
            || event.actor.kind != envelope.actor.kind
            || !matches_session_command(&envelope.command, &event.kind)
        {
            anyhow::bail!("signed audit conflicts with command {}", envelope.command_id);
        }
        Ok(Some(json!({
            "commandId": envelope.command_id,
            "committedRevision": committed_revision,
            "duplicate": true,
            "state": history.state,
            "eventId": event.event_id,
            "integrityHash": event.integrity_hash,
            "reconciledFromAudit": true
        })))
    }

    async fn submit_fresh(
        &self,
        state: &RuntimeState,
        actor: Actor,
        command: CommandKind,
    ) -> Result<RuntimeState> {
        self.submit_with_id(state, actor, None, command).await
    }

    async fn submit_with_id(
        &self,
        state: &RuntimeState,
        actor: Actor,
        command_id: Option<String>,
        command: CommandKind,
    ) -> Result<RuntimeState> {
        let envelope = CommandEnvelope {
            schema_version: "2".into(),
            project_id: state.project_id.clone(),
            run_id: state.run_id.clone(),
            command_id: command_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            frontier: Some(state.frontier.clone()),
            expected_revision: state.revision,
            actor,
            command,
        };
        let response = self.submit(state, envelope).await?;
        let committed: CommandResult = serde_json::from_value(response)
            .context("invalid committed result in KBD response")?;
        Ok(committed.state)
    }

    /// Commit a command through the in-process runtime.
    ///
    /// This is the same `Runtime::execute_command` the daemon calls, so the
    /// durability contract is unchanged: WAL append + fsync, Loro ingest, folded
    /// checkpoint, all under one exclusive flock. Two CLI processes serialize
    /// exactly as two daemon requests do.
    fn execute_locally(&self, envelope: CommandEnvelope) -> Result<CommandResult> {
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
        Ok(result)
    }
}

/// Distinguish an unavailable endpoint, a lost response, and an authoritative refusal.
#[derive(Debug)]
enum ControlFailure {
    Unreachable(anyhow::Error),
    Ambiguous(anyhow::Error),
    Uninitialized(anyhow::Error),
    Rejected(anyhow::Error),
}

impl ControlFailure {
    fn into_error(self) -> anyhow::Error {
        match self {
            Self::Unreachable(error)
            | Self::Ambiguous(error)
            | Self::Uninitialized(error)
            | Self::Rejected(error) => error,
        }
    }
}

async fn state_or_replay(client: &ControlClient, _runtime: &Runtime) -> Result<RuntimeState> {
    Ok(client.status().await.map_err(ControlFailure::into_error)?.value)
}

/// Resolve the state required to construct a mutation envelope.
///
/// Registration deliberately establishes identity without inventing a run. If no signed
/// runtime event exists yet, the first mutation crosses that boundary under operator
/// authority using the same legacy-aware initializer as migration. Read-only commands
/// continue to use `state_or_replay` so inspecting status never creates history.
async fn state_or_initialize(client: &ControlClient, runtime: &Runtime) -> Result<RuntimeState> {
    match client.status().await {
        Ok(Sourced { value: state, .. }) if state.revision > 0 => return Ok(state),
        Ok(_) | Err(ControlFailure::Uninitialized(_)) => {}
        Err(error) => return Err(error.into_error()),
    }
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
        Err(error) => Err(error.into()),
    }
}

fn classify_transport_failure(failure: TransportFailure) -> ControlFailure {
    match failure {
        TransportFailure::Unreachable(error) => ControlFailure::Unreachable(error),
        TransportFailure::Disconnected(error) | TransportFailure::Ambiguous(error) => {
            ControlFailure::Ambiguous(error)
        }
        TransportFailure::Invalid(error) => ControlFailure::Rejected(error),
    }
}

fn classify_status(response: &ControlResponse) -> ControlFailure {
    let status = response.status;
    let error = anyhow!("control plane returned {status}: {}", response.body);
    let body = decode_json_response::<Value>(response).unwrap_or_default();
    match (status, body["error"].as_str(), body["status"].as_str()) {
        (
            hyper::StatusCode::SERVICE_UNAVAILABLE,
            Some("local authority routes are not ready"),
            Some("initializing"),
        ) => ControlFailure::Unreachable(error),
        (hyper::StatusCode::NOT_FOUND, Some("kbd runtime is not initialized"), _) => {
            ControlFailure::Uninitialized(error)
        }
        _ if status.is_server_error() => ControlFailure::Ambiguous(error),
        _ => ControlFailure::Rejected(error),
    }
}

fn require_media_type(response: &ControlResponse, expected: &str) -> Result<()> {
    let actual = response.content_type.as_deref()
        .and_then(|value| value.split(';').next())
        .unwrap_or_default()
        .trim();
    if !actual.eq_ignore_ascii_case(expected) {
        anyhow::bail!("incompatible control-plane media type {actual:?}; expected {expected}");
    }
    Ok(())
}

fn decode_json_response<T: serde::de::DeserializeOwned>(response: &ControlResponse) -> Result<T> {
    require_media_type(response, "application/json")?;
    serde_json::from_str(&response.body).context("invalid control-plane JSON response")
}

fn is_session_command(command: &CommandKind) -> bool {
    matches!(
        command,
        CommandKind::BlockerRecord { .. }
            | CommandKind::BlockerClear { .. }
            | CommandKind::ClaimAcquire { .. }
    )
}

fn matches_session_command(command: &CommandKind, event: &EventKind) -> bool {
    match (command, event) {
        (
            CommandKind::BlockerRecord { blocker },
            EventKind::BlockerRecorded { blocker: recorded },
        ) => blocker == recorded,
        (
            CommandKind::BlockerClear { blocker_id, resolution },
            EventKind::BlockerCleared { blocker_id: recorded_id, resolution: recorded_resolution },
        ) => blocker_id == recorded_id && resolution == recorded_resolution,
        (
            CommandKind::ClaimAcquire { scope, holder_id, mode, .. },
            EventKind::ClaimAcquired {
                scope: recorded_scope,
                holder_id: recorded_holder,
                mode: recorded_mode,
                ..
            },
        ) => scope == recorded_scope && holder_id == recorded_holder && mode == recorded_mode,
        _ => !is_session_command(command),
    }
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
    let Sourced { value, source } = client.read_history().await?;
    let events = value.events;
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
    source.report();
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

async fn watch(client: &ControlClient, after_event: Option<&str>) -> Result<()> {
    let mut seen = BTreeMap::new();
    let mut after_event = after_event;
    let mut previous_source = None;
    let interrupt = tokio::signal::ctrl_c();
    tokio::pin!(interrupt);
    loop {
        let Sourced { value, source } = tokio::select! {
            result = &mut interrupt => return result.map_err(Into::into),
            result = client.read_history() => result?,
        };
        let events = value.events;
        let start = match after_event {
            Some(cursor) => match events.iter().position(|event| event.event_id == cursor) {
                Some(index) => {
                    seen.extend(events[..=index].iter().map(|event| {
                        (event.event_id.clone(), event.integrity_hash.clone())
                    }));
                    after_event = None;
                    Some(index + 1)
                }
                // A cursor previously read from Companion may not have replicated
                // to the fallback journal yet. Wait for a snapshot carrying it.
                None if source == ReadSource::LocalFallback => None,
                None => anyhow::bail!("watch cursor event not found: {cursor}"),
            },
            None => Some(0),
        };
        if let Some(start) = start {
            let mut stdout = io::stdout().lock();
            for event in events.into_iter().skip(start) {
                if let Some(previous_hash) = seen.get(&event.event_id) {
                    if previous_hash != &event.integrity_hash {
                        anyhow::bail!("event {} changed its integrity hash", event.event_id);
                    }
                    continue;
                }
                serde_json::to_writer(&mut stdout, &event)?;
                writeln!(stdout)?;
                seen.insert(event.event_id, event.integrity_hash);
            }
            stdout.flush()?;
        }
        if previous_source != Some(source) {
            source.report();
            previous_source = Some(source);
        }
        tokio::select! {
            result = &mut interrupt => return result.map_err(Into::into),
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

/// Set to `1` to deliberately start a new journal when a checked-in plan's journal is lost.
const NEW_JOURNAL_ENV: &str = "PROMETHEUS_KBD_NEW_JOURNAL";

/// A waypoint written by `kbd-runtime` at revision > 0 means this plan already has a signed
/// journal on some machine. Signing a fresh genesis here would make this machine the sole
/// operator of a second journal for the same project and split the plan without warning.
fn refuse_silent_fork(waypoint: &Value) -> Result<()> {
    let revision = waypoint["sourceRevision"].as_u64().unwrap_or(0);
    let opted_in = std::env::var_os(NEW_JOURNAL_ENV).is_some_and(|value| value == "1");
    if waypoint["generatedBy"] != "kbd-runtime" || revision == 0 || opted_in {
        return Ok(());
    }
    Err(anyhow!(
        "this checkout's KBD plan comes from a signed journal that is not on this machine \
         (project {}, run {}, revision {revision}). Writing now would start a separate journal \
         that only this machine controls, splitting the plan. Sync the project's journal to this \
         machine or restore it, and ask a project operator to enroll this device (run \
         `prometheus kbd device identity`). If that journal is permanently lost, rerun with \
         {NEW_JOURNAL_ENV}=1 to start a new one.",
        string_field(waypoint, &["projectId"]).unwrap_or_else(|| "unknown".into()),
        string_field(waypoint, &["runId"]).unwrap_or_else(|| "unknown".into()),
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
    use kbd_runtime::{Change, CommandResult, Phase, Task, EVENT_SCHEMA_VERSION};
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

    #[test]
    fn qualified_task_subject_disambiguates_repeated_task_ids_and_titles() {
        let mut state = RuntimeState::default();
        state.active_path.phase_id = Some("phase".into());
        state.active_path.phase_path = vec!["phase".into()];
        let repeated_title = "Confirm dependency completion";
        let changes = ["change-a", "change-b"]
            .into_iter()
            .map(|change_id| {
                let task = Task {
                    id: "1".into(),
                    title: repeated_title.into(),
                    sequence: 1,
                    status: WorkStatus::Pending,
                    summary: None,
                };
                (
                    change_id.into(),
                    Change {
                        id: change_id.into(),
                        title: change_id.into(),
                        sequence: 1,
                        status: WorkStatus::Pending,
                        depends_on: Vec::new(),
                        implementation_status: WorkStatus::Pending,
                        tasks: [(task.id.clone(), task)].into_iter().collect(),
                    },
                )
            })
            .collect();
        state.phases.insert(
            "phase".into(),
            Phase {
                id: "phase".into(),
                slug: "phase".into(),
                title: "phase".into(),
                parent_phase_id: None,
                status: WorkStatus::InProgress,
                stages: Default::default(),
                changes,
                legacy_completion_baseline: None,
                outcome_first: false,
                outcome: None,
                legacy_read_only: false,
            },
        );

        assert!(!guard_context(&state, BoundaryKind::Task, repeated_title).valid);
        let context = guard_context(&state, BoundaryKind::Task, "change-b/1");
        assert!(context.valid);
        assert_eq!(context.change_id.as_deref(), Some("change-b"));
        assert_eq!(context.task_id.as_deref(), Some("1"));
        assert_eq!(context.name, repeated_title);
        assert_eq!(context.position, "phase › change-b › 1");

        let change = guard_context(&state, BoundaryKind::Change, "change-b");
        assert!(change.valid);
        assert_eq!(change.phase_id.as_deref(), Some("phase"));
        assert_eq!(change.change_id.as_deref(), Some("change-b"));
        assert_eq!(change.task_id, None);
        assert_eq!(change.ordinal, 2);
        assert_eq!(change.total, 2);
        assert_eq!(change.position, "phase › change-b");
    }
}
