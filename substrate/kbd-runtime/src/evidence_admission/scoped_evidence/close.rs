//! Pure canonical closure evidence assembly. No close command or writer.
//! Only the future authenticated scoped event path may supply the complete
//! product observation and repair history. Verified transport facts alone
//! are not Observation values. Never pass this to the legacy close protocol.

use super::{
    evaluate, BoundObservationIdentity, Coverage, Observation, Obligations, RepairResolution,
};
use super::super::{check_observations, refused, sha256_hex};
use crate::phase_closure::{is_digest, require_exact_pass_evidence, ScenarioOutcome, SCENARIO_PASS};
use crate::Result;
use serde_json::{json, Value};

/// Internal scoped closure evidence identity. The future canonical caller
/// must also verify candidate-at-use, actual complete history, unresolved
/// effects, cumulative budget, cleanup, all existing phase gates and the
/// supported new-run protocol. This is not PhaseCloseInput or a wire format.
pub(in crate::evidence_admission) struct ScopedClosureEvidence {
    evidence_sha256: String,
    outcomes: Vec<ScenarioOutcome>,
}

fn identity_value(identity: &BoundObservationIdentity) -> Value {
    json!({
        "projectId": identity.project_id,
        "runId": identity.run_id,
        "phaseId": identity.phase_id,
        "contractDigest": identity.contract_digest,
        "bindingRevision": identity.binding_revision,
        "candidateSha256": identity.candidate_sha256,
        "scenarioId": identity.key.scenario_id,
        "targetId": identity.key.target_id,
        "targetIdentitySha256": identity.target_identity_sha256,
        "requirementsSha256": identity.requirements_sha256,
        "attemptId": identity.attempt_id,
        "receiptSha256": identity.receipt_sha256,
        "receiptItemSha256": identity.receipt_item_sha256,
        "producerProfileSha256": identity.producer_profile_sha256,
        "observerManifestSha256": identity.observer_manifest_sha256
    })
}

/// Commit to each observed row, including rejected, unknown, stale, failed and
/// conflicting history, rather than constructing a new tuple out of
/// independently selected subreceipts.
fn history_value(row: &Observation) -> Value {
    json!({
        "attribution": row.identity.as_ref().map(identity_value),
        "actualOutcome": row.outcome
    })
}

fn repair_value(repair: &RepairResolution) -> Value {
    json!({
        "failedReceiptSha256": repair.failed.receipt_sha256,
        "failedItemSha256": repair.failed.item_sha256,
        "replacementReceiptSha256": repair.replacement.receipt_sha256,
        "replacementItemSha256": repair.replacement.item_sha256,
        "authorityRecordSha256": repair.authority_record_sha256,
        "replacementCandidateSha256": repair.replacement_candidate_sha256
    })
}

/// Assemble the all-scenario result using the canonical evidence decision.
/// Packet selection and caller-declared aggregate PASS are not inputs.
/// A setup-only or unknown row supplies no credit; independently accepted
/// current rows remain in immutable history even when completeness refuses.
pub(in crate::evidence_admission) fn derive_for_close(
    parent: &Obligations<'_>,
    history: &[Observation],
    verified_repairs: &[RepairResolution],
    journal_history_sha256: &str,
) -> Result<ScopedClosureEvidence> {
    if !is_digest(journal_history_sha256) {
        return Err(refused(parent.phase_id, "scoped-closure-journal-identity-missing".into()));
    }
    let coverage = evaluate(parent, history, verified_repairs)?;
    coverage.require_complete(parent.phase_id)?;
    let outcomes = derive_outcomes(parent, history, &coverage)?;
    // Native whole-contract evidence checks are still required after the
    // stronger exact target/receipt/candidate decision.
    check_observations(parent.phase_id, parent.binding.scenario_digests.keys(), &outcomes)?;
    require_exact_pass_evidence(parent.phase_id, parent.binding, &outcomes)?;

    let accepted: Vec<_> = coverage.accepted.iter().map(|(key, indexes)| {
        json!({
            "scenarioId": key.scenario_id,
            "targetId": key.target_id,
            "historyIndexes": indexes
        })
    }).collect();
    let commitment = json!({
        "schema": "kbd.scoped-closure-evidence/v1",
        "projectId": parent.project_id,
        "runId": parent.run_id,
        "phaseId": parent.phase_id,
        "contractDigest": parent.binding.contract_digest,
        "bindingRevision": parent.binding.binding_revision,
        "interpretationSha256": parent.interpretation_sha256,
        "journalHistorySha256": journal_history_sha256,
        "candidateSha256": parent.current_candidate_sha256,
        "history": history.iter().map(history_value).collect::<Vec<_>>(),
        "verifiedRepairs": verified_repairs.iter().map(repair_value).collect::<Vec<_>>(),
        "accepted": accepted,
    });
    let bytes = serde_jcs::to_vec(&commitment)
        .map_err(|_| refused(parent.phase_id, "scoped-closure-evidence-canonicalization".into()))?;
    Ok(ScopedClosureEvidence {
        evidence_sha256: sha256_hex(&bytes),
        outcomes,
    })
}

fn derive_outcomes(
    parent: &Obligations<'_>, history: &[Observation], coverage: &Coverage,
) -> Result<Vec<ScenarioOutcome>> {
    let mut outcomes = Vec::new();
    for (scenario, scenario_digest) in &parent.binding.scenario_digests {
        let mut references = Vec::new();
        let mut tuples = Vec::new();
        for (key, indexes) in &coverage.accepted {
            if &key.scenario_id != scenario { continue; }
            for &index in indexes {
                let observation = history.get(index).ok_or_else(||
                    refused(parent.phase_id, "scoped-closure-history-index-invalid".into()))?;
                let identity = observation.identity.as_ref().ok_or_else(||
                    refused(parent.phase_id, "scoped-closure-attribution-missing".into()))?;
                if !coverage.observations.get(index).is_some_and(|row| row.accepted_current)
                    || &identity.key != key || identity.candidate_sha256 != parent.current_candidate_sha256
                {
                    return Err(refused(parent.phase_id, "scoped-closure-attribution-mismatch".into()));
                }
                // Every reference and tuple names one complete observation.
                // The scenario aggregate contains explicit links; it never
                // pretends a combination of target receipts is one receipt.
                references.push(json!({
                    "attemptId": identity.attempt_id,
                    "receiptSha256": identity.receipt_sha256,
                    "receiptItemSha256": identity.receipt_item_sha256,
                    "targetId": identity.key.target_id,
                    "historyIndex": index
                }));
                tuples.push(history_value(observation));
            }
        }
        if tuples.is_empty() {
            return Err(refused(parent.phase_id, format!("scoped-closure-missing:{scenario}")));
        }
        outcomes.push(ScenarioOutcome {
            id: scenario.clone(),
            status: SCENARIO_PASS.into(),
            scenario_digest: Some(scenario_digest.clone()),
            carried_from_contract: None,
            attempted: Some(json!({"exactObservationReferences": references})),
            observed: Some(json!({"exactObservationReferences": references})),
            evidence: Some(json!({"completeObservationTuples": tuples})),
        });
    }
    Ok(outcomes)
}
