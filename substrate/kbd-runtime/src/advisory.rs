//! Advisory bottleneck recommendations (issue #256).
//!
//! A pure reading of canonical state. It writes no event, stops nothing,
//! creates nothing and changes no disposition or contract. It answers one
//! question: has the same blocker been attempted again without new evidence,
//! and if so, which single next step is worth proposing to the operator.
//!
//! Only blockers recorded with a `fingerprint` take part. Blockers recorded
//! before the field existed are never counted.

use crate::{Blocker, BlockerClass, RuntimeState};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Completed attempts on one fingerprint before a recommendation is made.
pub const REPEAT_ATTEMPT_THRESHOLD: usize = 2;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RecommendationKind {
    /// The environment or release setup failed; repair it once, apart from
    /// the product change.
    SetupRepair,
    /// The fixture, oracle or runner failed; the product result is unknown.
    MeasurementRepair,
    /// The next step needs an authority the agent does not hold.
    AuthorityHandoff,
    /// The product approach keeps failing; propose a scoped child phase.
    ArchitectureChild,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Recommendation {
    pub kind: RecommendationKind,
    pub fingerprint: String,
    /// Completed attempts recorded for this fingerprint.
    pub attempts: usize,
    /// Distinct evidence references across those attempts.
    pub distinct_evidence: usize,
    pub blocker_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_ref: Option<String>,
    /// What to propose to the operator, in plain words.
    pub proposal: String,
    /// A command the operator may choose to run. The advisor never runs it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WaitingBlocker {
    pub blocker_id: String,
    pub class: BlockerClass,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InFlightGate {
    pub gate_id: String,
    pub scope: String,
    pub started_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BottleneckAdvice {
    /// Always true: advice never stops, creates or changes anything.
    pub advisory: bool,
    /// The revision the advice was read from. Unchanged by reading it.
    pub revision: u64,
    /// Gates started and not finished. While any is running, no
    /// recommendation is made.
    pub in_flight: Vec<InFlightGate>,
    /// Waits on an approval or a resource. Never counted as failed attempts.
    pub waiting: Vec<WaitingBlocker>,
    pub recommendations: Vec<Recommendation>,
    /// Recommendations held back because a gate is still running.
    pub withheld_while_in_flight: usize,
    pub guidance: String,
}

fn recommendation_kind(class: BlockerClass) -> Option<RecommendationKind> {
    match class {
        BlockerClass::Product => Some(RecommendationKind::ArchitectureChild),
        BlockerClass::Setup => Some(RecommendationKind::SetupRepair),
        BlockerClass::Measurement => Some(RecommendationKind::MeasurementRepair),
        BlockerClass::Authority => Some(RecommendationKind::AuthorityHandoff),
        BlockerClass::ApprovalWait | BlockerClass::ResourceWait => None,
    }
}

fn proposal(kind: RecommendationKind, latest: &Blocker) -> (String, Option<String>) {
    match kind {
        RecommendationKind::SetupRepair => (
            "Repair the setup once, apart from the product change, then run the \
             same check again. Repeating it unchanged will fail the same way."
                .into(),
            None,
        ),
        RecommendationKind::MeasurementRepair => (
            "Repair the fixture, oracle or runner. The product result is not \
             established: report it as not exercised, not as failed."
                .into(),
            None,
        ),
        RecommendationKind::AuthorityHandoff => (
            format!(
                "Hand this one decision to its owner and continue with work that \
                 does not depend on it: {}",
                latest.summary
            ),
            None,
        ),
        RecommendationKind::ArchitectureChild => (
            "The same product failure repeated without new evidence. Propose a \
             scoped child phase for it so the rest of the phase can proceed."
                .into(),
            Some("bash skills/kbd-new-child/kbd-new-child.sh <child-name> \"<goal>\"".into()),
        ),
    }
}

/// Read advisory bottleneck recommendations from canonical state.
pub fn bottleneck_advice(state: &RuntimeState) -> BottleneckAdvice {
    let in_flight: Vec<InFlightGate> = state
        .active_gates
        .values()
        .map(|gate| InFlightGate {
            gate_id: gate.id.clone(),
            scope: gate.scope.clone(),
            started_at: gate.started_at,
        })
        .collect();

    let mut waiting = Vec::new();
    let mut attempts: BTreeMap<&str, Vec<&Blocker>> = BTreeMap::new();
    for blocker in state.blockers.values() {
        let Some(fingerprint) = blocker.fingerprint.as_deref() else {
            continue;
        };
        let class = blocker.class.unwrap_or(BlockerClass::Product);
        if recommendation_kind(class).is_none() {
            if !blocker.resolved {
                waiting.push(WaitingBlocker {
                    blocker_id: blocker.id.clone(),
                    class,
                    summary: blocker.summary.clone(),
                });
            }
            continue;
        }
        attempts.entry(fingerprint).or_default().push(blocker);
    }

    let mut recommendations = Vec::new();
    for (fingerprint, blockers) in attempts {
        // A fingerprint with nothing open is cleared work, not a bottleneck.
        let Some(latest) = blockers.iter().rev().find(|blocker| !blocker.resolved) else {
            continue;
        };
        if blockers.len() < REPEAT_ATTEMPT_THRESHOLD {
            continue;
        }
        let evidence: BTreeSet<Option<&str>> = blockers
            .iter()
            .map(|blocker| blocker.evidence_ref.as_deref())
            .collect();
        // Every attempt brought different evidence: the work is still learning.
        if evidence.len() >= blockers.len() {
            continue;
        }
        let class = latest.class.unwrap_or(BlockerClass::Product);
        let Some(kind) = recommendation_kind(class) else {
            continue;
        };
        let (proposal, suggested_command) = proposal(kind, latest);
        recommendations.push(Recommendation {
            kind,
            fingerprint: fingerprint.to_string(),
            attempts: blockers.len(),
            distinct_evidence: evidence.len(),
            blocker_ids: blockers.iter().map(|blocker| blocker.id.clone()).collect(),
            evidence_ref: latest.evidence_ref.clone(),
            proposal,
            suggested_command,
        });
    }

    let withheld_while_in_flight = if in_flight.is_empty() {
        0
    } else {
        recommendations.len()
    };
    if !in_flight.is_empty() {
        recommendations.clear();
    }
    let guidance = if !in_flight.is_empty() {
        "in-flight: do not stop. A gate is still running; wait for its result.".to_string()
    } else if !recommendations.is_empty() {
        "Advisory only. Put the proposal to the operator; nothing was stopped, created or changed."
            .to_string()
    } else if !waiting.is_empty() {
        "Waiting on an approval or a resource. This is not a failed attempt; continue with work that does not depend on it."
            .to_string()
    } else {
        "No repeated blocker without new evidence.".to_string()
    };

    BottleneckAdvice {
        advisory: true,
        revision: state.revision,
        in_flight,
        waiting,
        recommendations,
        withheld_while_in_flight,
        guidance,
    }
}
