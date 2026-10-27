//! Replay compatibility for the retired zeespec boundary: 1.12 removed
//! `BoundaryKind::Zeespec`, so every later binary refused journals that 1.11.x
//! had signed with zeespec stage receipts. Fixture lines are copied verbatim
//! from such a journal (agent-team-creator, 2026-09-30).

use kbd_runtime::{
    verify_ed25519_signature, Actor, BoundaryEdge, BoundaryKind, BoundaryOutcome, BoundaryReceipt,
    CommandEnvelope, CommandKind, Event, EventKind, Runtime,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

fn fixture_lines() -> Vec<String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pre-1.12-zeespec-journal/events.jsonl");
    fs::read_to_string(path)
        .expect("read zeespec journal fixture")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(str::to_owned)
        .collect()
}

#[test]
fn signed_zeespec_receipts_parse_and_keep_their_signed_bytes() {
    for line in fixture_lines() {
        let original: Value = serde_json::from_str(&line).unwrap();
        let event: Event = serde_json::from_str(&line).expect("zeespec event parses");

        let EventKind::BoundaryReceiptRecorded { receipt } = &event.kind else {
            panic!("fixture event is not a boundary receipt");
        };
        assert_eq!(receipt.boundary, BoundaryKind::Zeespec);

        // Lossless round trip keeps the canonical bytes, hence hash and signature.
        assert_eq!(serde_json::to_value(&event).unwrap(), original);

        let mut unsigned = original.clone();
        let object = unsigned.as_object_mut().unwrap();
        let signature = object.remove("signature").unwrap();
        object.insert("integrityHash".into(), Value::String(String::new()));
        let bytes = serde_jcs::to_vec(&unsigned).unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(&bytes)),
            original["integrityHash"].as_str().unwrap(),
            "integrity hash of event {}",
            event.event_id
        );
        assert!(verify_ed25519_signature(
            original["signerPublicKey"].as_str().unwrap(),
            &bytes,
            signature.as_str().unwrap(),
        ));
    }
}

#[test]
fn new_zeespec_receipts_are_refused() {
    let fixture = tempfile::tempdir().expect("create runtime fixture");
    let runtime = Runtime::open(fixture.path());
    let actor = Actor::operator("zeespec-integration", "integration-test");
    let state = runtime
        .initialize("zeespec-project", "zeespec-run", actor.clone())
        .expect("initialize runtime");

    let receipt = BoundaryReceipt {
        id: "zeespec-new-receipt".into(),
        boundary: BoundaryKind::Zeespec,
        edge: BoundaryEdge::Before,
        subject: "interrogate".into(),
        phase_id: None,
        change_id: None,
        task_id: None,
        source_revision: state.revision,
        position: "zeespec:interrogate".into(),
        exact_signal: "Starting zeespec stage 1 out of 3: interrogate".into(),
        outcome: BoundaryOutcome::Pass,
        findings: Vec::new(),
        repaired_projections: Vec::new(),
        observed_at: chrono::Utc::now(),
    };
    let error = runtime
        .execute_command(CommandEnvelope {
            schema_version: "1".into(),
            project_id: state.project_id.clone(),
            run_id: state.run_id.clone(),
            command_id: "record-zeespec".into(),
            frontier: None,
            expected_revision: state.revision,
            actor,
            command: CommandKind::BoundaryReceiptRecord { receipt },
        })
        .expect_err("a new zeespec receipt must be refused");
    assert!(
        error
            .to_string()
            .contains("zeespec boundaries were retired"),
        "unexpected error: {error}"
    );
}
