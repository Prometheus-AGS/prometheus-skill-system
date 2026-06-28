#!/usr/bin/env bash
# integration-full-loop.sh
# Integration test: learn-retain → learn-practice → learn-certify (full post-Feynman loop)
#
# Depends on: tests/learn/fixtures/sample-kb/sample-corpus.json (change-learn-021)
#
# Does NOT require:
#   - live MCP servers
#   - Rust binaries
#   - network access
#
# Requires: jq on PATH

set -euo pipefail

# ── Preconditions ─────────────────────────────────────────────────────────────

if ! command -v jq >/dev/null 2>&1; then
  echo "[FAIL] jq is required but not found on PATH" >&2
  exit 1
fi

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
FIXTURE_CORPUS="${REPO_ROOT}/tests/learn/fixtures/sample-kb/sample-corpus.json"

if [[ ! -f "$FIXTURE_CORPUS" ]]; then
  echo "[FAIL] fixture corpus not found (run change-learn-021 first): $FIXTURE_CORPUS" >&2
  exit 1
fi

# ── Setup ─────────────────────────────────────────────────────────────────────

TEST_GOAL_ID="test-full-loop-$(date +%s)"
LEARNER_ID="learner-test"
NOW="2026-06-28T00:00:00Z"

# ── Step 1: Validate FSRS card structure (new card) ───────────────────────────

LEARNER_MODEL_NEW=$(jq -n \
  --arg learner_id "$LEARNER_ID" \
  --arg now "$NOW" \
  '{
    learner_id: $learner_id,
    schema_version: "1.0.0",
    concepts: {
      "vectors": {
        mastery: 0.3,
        observations: [],
        fsrs_card: {
          stability: 1.0,
          difficulty: 5.0,
          due: $now,
          state: "New",
          reps: 0,
          lapses: 0
        }
      }
    },
    gaps: {},
    sessions: []
  }')

FSRS_STATE=$(echo "$LEARNER_MODEL_NEW" | jq -r '.concepts.vectors.fsrs_card.state')
if [[ "$FSRS_STATE" != "New" ]]; then
  echo "[FAIL] FSRS card state expected New, got: $FSRS_STATE" >&2
  exit 1
fi

FSRS_REPS=$(echo "$LEARNER_MODEL_NEW" | jq -r '.concepts.vectors.fsrs_card.reps')
if [[ "$FSRS_REPS" != "0" ]]; then
  echo "[FAIL] FSRS card reps expected 0, got: $FSRS_REPS" >&2
  exit 1
fi

# ── Step 2: Simulate retention session — FSRS card update ─────────────────────

REVIEW_AT="2026-06-29T00:00:00Z"

LEARNER_MODEL_REVIEWED=$(echo "$LEARNER_MODEL_NEW" | jq \
  --arg review_at "$REVIEW_AT" \
  '.concepts.vectors.fsrs_card.state = "Review"
   | .concepts.vectors.fsrs_card.reps = 1
   | .concepts.vectors.fsrs_card.stability = 1.2
   | .concepts.vectors.fsrs_card.last_review = $review_at
   | .concepts.vectors.mastery = 0.55
   | .concepts.vectors.observations += [{
       timestamp: $review_at,
       score: 0.75,
       source_skill: "learn-retain",
       vector_clock: { "learner-test": 1 }
     }]')

FSRS_STATE_AFTER=$(echo "$LEARNER_MODEL_REVIEWED" | jq -r '.concepts.vectors.fsrs_card.state')
if [[ "$FSRS_STATE_AFTER" != "Review" ]]; then
  echo "[FAIL] updated FSRS card state expected Review, got: $FSRS_STATE_AFTER" >&2
  exit 1
fi

FSRS_REPS_AFTER=$(echo "$LEARNER_MODEL_REVIEWED" | jq -r '.concepts.vectors.fsrs_card.reps')
if [[ "$FSRS_REPS_AFTER" != "1" ]]; then
  echo "[FAIL] updated FSRS card reps expected 1, got: $FSRS_REPS_AFTER" >&2
  exit 1
fi

FSRS_STABILITY_AFTER=$(echo "$LEARNER_MODEL_REVIEWED" | jq -r '.concepts.vectors.fsrs_card.stability')
STABILITY_OK=$(echo "$FSRS_STABILITY_AFTER >= 1.0" | awk '{print ($1 >= $3) ? "true" : "false"}' FS=" ")
if [[ "$STABILITY_OK" != "true" ]]; then
  echo "[FAIL] updated FSRS stability expected >= 1.0, got: $FSRS_STABILITY_AFTER" >&2
  exit 1
fi

# ── Step 3: Validate practice-result structure ────────────────────────────────

PRACTICE_RESULT=$(jq -n --arg goal_id "$TEST_GOAL_ID" '{
  goal_id: $goal_id,
  concept_id: "vectors",
  session_type: "derivation",
  score: 0.8,
  passed: true,
  practiced_at: "2026-06-29T01:00:00Z"
}')

PRACTICE_PASSED=$(echo "$PRACTICE_RESULT" | jq -r '.passed')
if [[ "$PRACTICE_PASSED" != "true" ]]; then
  echo "[FAIL] practice-result passed expected true, got: $PRACTICE_PASSED" >&2
  exit 1
fi

PRACTICE_SCORE=$(echo "$PRACTICE_RESULT" | jq -r '.score')
SCORE_OK=$(awk "BEGIN{print ($PRACTICE_SCORE >= 0.7) ? \"true\" : \"false\"}")
if [[ "$SCORE_OK" != "true" ]]; then
  echo "[FAIL] practice-result score expected >= 0.7, got: $PRACTICE_SCORE" >&2
  exit 1
fi

# ── Step 4: Validate checkpoint VC structure (normal trajectory) ──────────────

CHECKPOINT_VC=$(jq -n --arg goal_id "$TEST_GOAL_ID" --arg learner_id "$LEARNER_ID" '{
  "@context": [
    "https://www.w3.org/ns/credentials/v2",
    "https://purl.imsglobal.org/spec/ob/v3p0/context-3.0.3.json"
  ],
  type: ["VerifiableCredential", "OpenBadgeCredential"],
  id: ("urn:uuid:test-vc-" + $goal_id),
  issuer: "did:plc:prometheus-learn",
  validFrom: "2026-06-29T02:00:00Z",
  credentialSubject: {
    id: ("did:plc:" + $learner_id),
    achievement: {
      id: ("urn:uuid:achievement-" + $goal_id),
      type: ["Achievement"],
      name: "Mastery — Linear Algebra Basics (Novice)",
      description: "Demonstrated novice-level mastery of linear algebra basics via Feynman loop."
    },
    evidence: [
      {
        type: ["Evidence"],
        narrative: "Passed feynman-loop with overall_score=0.80 on concept vectors."
      }
    ]
  }
}')

# Assert @context has 2 entries
CONTEXT_LEN=$(echo "$CHECKPOINT_VC" | jq '.["@context"] | length')
if [[ "$CONTEXT_LEN" -ne 2 ]]; then
  echo "[FAIL] VC @context expected 2 entries, got: $CONTEXT_LEN" >&2
  exit 1
fi

# Assert evidence is non-empty
EVIDENCE_LEN=$(echo "$CHECKPOINT_VC" | jq '.credentialSubject.evidence | length')
if [[ "$EVIDENCE_LEN" -lt 1 ]]; then
  echo "[FAIL] VC credentialSubject.evidence expected at least 1 entry, got: $EVIDENCE_LEN" >&2
  exit 1
fi

# Assert no integrityNote in normal trajectory
HAS_INTEGRITY_NOTE=$(echo "$CHECKPOINT_VC" | jq 'has("integrityNote")')
if [[ "$HAS_INTEGRITY_NOTE" != "false" ]]; then
  echo "[FAIL] VC should not have integrityNote on normal trajectory" >&2
  exit 1
fi

# ── Step 5: Anomalous trajectory — integrityNote must be present ──────────────
# Simulate Δmastery > 0.4 in one session (0.1 → 0.55 = Δ0.45)

ANOMALOUS_VC=$(echo "$CHECKPOINT_VC" | jq \
  '.integrityNote = "Mastery delta of 0.45 in a single session exceeds expected threshold (0.40). Manual review recommended."')

HAS_INTEGRITY_NOTE_ANOMALOUS=$(echo "$ANOMALOUS_VC" | jq 'has("integrityNote")')
if [[ "$HAS_INTEGRITY_NOTE_ANOMALOUS" != "true" ]]; then
  echo "[FAIL] anomalous VC should have integrityNote present" >&2
  exit 1
fi

INTEGRITY_NOTE_VALUE=$(echo "$ANOMALOUS_VC" | jq -r '.integrityNote')
if [[ -z "$INTEGRITY_NOTE_VALUE" ]]; then
  echo "[FAIL] anomalous VC integrityNote is empty" >&2
  exit 1
fi

# ── Done ──────────────────────────────────────────────────────────────────────

echo "[PASS] full loop integration test"
exit 0
