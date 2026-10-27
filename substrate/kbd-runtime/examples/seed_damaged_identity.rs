//! Acceptance-fixture seeder for issue #194: writes a *damaged* canonical journal
//! (two map keys carrying the same embedded change ID) through the real runtime
//! API and signer, then records a genuine Before boundary receipt on the alias
//! task. It exists because the production importer and registration now refuse
//! to create this state; only history written by an older binary can contain it.
//!
//! usage: seed_damaged_identity <project-root> <data-root> <phase-id> <variant>
//!   variant `equal`          alias-a/1 and target/1 are identical and in progress, with a Before receipt
//!   variant `equal-nostart`  same content but no start receipt was ever recorded
//!   variant `conflict`       alias-a/1 is complete while target/1 is in progress

use kbd_runtime::{
    ActivePath, Actor, BoundaryEdge, BoundaryKind, BoundaryOutcome, BoundaryReceipt, Change,
    EventKind, Phase, Runtime, Task, WorkStatus,
};
use std::collections::BTreeMap;

const OPERATOR: &str = "operator@kbd.test";
const TARGET: &str = "target";
const ALIAS: &str = "alias-a";

fn task(status: WorkStatus) -> Task {
    Task { id: "1".into(), title: "1.1 Shared task".into(), sequence: 1, status, summary: None }
}

fn change(status: WorkStatus, task_status: WorkStatus) -> Change {
    Change {
        id: TARGET.into(),
        title: "Damaged change".into(),
        sequence: 1,
        status: status.clone(),
        implementation_status: status,
        tasks: BTreeMap::from([("1".to_string(), task(task_status))]),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let [_, project, data, phase_id, variant] = args.as_slice() else {
        return Err("usage: seed_damaged_identity <project-root> <data-root> <phase-id> <equal|conflict>".into());
    };
    let alias_task_status = match variant.as_str() {
        "equal" | "equal-nostart" => WorkStatus::InProgress,
        "conflict" => WorkStatus::Complete,
        other => return Err(format!("unknown variant {other}").into()),
    };
    // Fixture-only: refuse anything that is not inside an owned `kbd194-*` scratch directory,
    // so a real project or journal is never seeded.
    for path in [project, data] {
        let real = std::fs::canonicalize(path)?;
        let owned = real.components().any(|part| part.as_os_str().to_string_lossy().starts_with("kbd194-"));
        if !owned {
            return Err(format!("refusing to seed {path}: not inside a kbd194-* scratch directory").into());
        }
    }
    let runtime = Runtime::open_canonical_at(project, data)?;
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(std::path::Path::new(project).join(".prometheus/project.json"))?)?;
    let project_id = manifest["projectId"].as_str().or_else(|| manifest["project_id"].as_str())
        .ok_or("project manifest has no project id")?.to_string();
    let actor = || Actor::operator(OPERATOR, "kbd-fixture-seed");
    let state = runtime.initialize(project_id, uuid::Uuid::new_v4().to_string(), actor())?;

    let phase = Phase {
        id: phase_id.clone(),
        slug: phase_id.clone(),
        title: "Damaged identity phase".into(),
        parent_phase_id: None,
        status: WorkStatus::InProgress,
        stages: BTreeMap::new(),
        changes: BTreeMap::from([
            (ALIAS.to_string(), change(WorkStatus::InProgress, alias_task_status)),
            (TARGET.to_string(), change(WorkStatus::InProgress, WorkStatus::InProgress)),
        ]),
        legacy_completion_baseline: None,
        legacy_read_only: false,
    };
    let state = runtime.append_command(
        actor(),
        state.revision,
        "seed-legacy-import",
        EventKind::LegacyStateImported {
            phases: BTreeMap::from([(phase_id.clone(), phase)]),
            active_path: ActivePath { phase_path: vec![phase_id.clone()], phase_id: Some(phase_id.clone()), ..ActivePath::default() },
            completion: BTreeMap::new(),
            decisions: BTreeMap::new(),
            blockers: BTreeMap::new(),
        },
    )?;
    if variant == "equal-nostart" {
        println!("{}", serde_json::json!({ "revision": state.revision, "phase": phase_id, "variant": variant }));
        return Ok(());
    }
    // A real begin-task records a change start, then the task start.
    let change_receipt = BoundaryReceipt {
        id: "seed-receipt-alias-change-start".into(),
        boundary: BoundaryKind::Change,
        edge: BoundaryEdge::Before,
        subject: ALIAS.into(),
        phase_id: Some(phase_id.clone()),
        change_id: Some(ALIAS.into()),
        task_id: None,
        source_revision: state.revision,
        position: format!("{phase_id} > {ALIAS}"),
        exact_signal: format!("Starting change 1 out of 1: {ALIAS}"),
        outcome: BoundaryOutcome::Pass,
        findings: Vec::new(),
        repaired_projections: Vec::new(),
        observed_at: chrono::Utc::now(),
    };
    let state = runtime.append_command(actor(), state.revision, "seed-change-start-receipt", EventKind::BoundaryReceiptRecorded { receipt: change_receipt })?;
    let receipt = BoundaryReceipt {
        id: "seed-receipt-alias-task-start".into(),
        boundary: BoundaryKind::Task,
        edge: BoundaryEdge::Before,
        subject: format!("{ALIAS}/1"),
        phase_id: Some(phase_id.clone()),
        change_id: Some(ALIAS.into()),
        task_id: Some("1".into()),
        source_revision: state.revision,
        position: format!("{phase_id} > {ALIAS} > 1"),
        exact_signal: format!("Starting task 1 out of 1: {ALIAS}/1"),
        outcome: BoundaryOutcome::Pass,
        findings: Vec::new(),
        repaired_projections: Vec::new(),
        observed_at: chrono::Utc::now(),
    };
    let state = runtime.append_command(actor(), state.revision, "seed-start-receipt", EventKind::BoundaryReceiptRecorded { receipt })?;
    println!("{}", serde_json::json!({ "revision": state.revision, "phase": phase_id, "variant": variant }));
    Ok(())
}
