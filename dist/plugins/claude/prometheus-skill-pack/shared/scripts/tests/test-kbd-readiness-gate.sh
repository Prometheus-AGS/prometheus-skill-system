#!/usr/bin/env bash
# test-kbd-readiness-gate.sh — boundary integration test for
# phase-readiness-contract (issue #186 vertical slice + #184 canonical
# foundation). Runs ONCE at the phase boundary, after complete production
# implementation (implementation-first / integration-only policy).
#
# Real entry points under test:
#   shared/scripts/kbd-contract-adopt.sh          (canonical adoption writer)
#   shared/scripts/kbd-phase-readiness-check.sh   (READY/BLOCKED checker)
#   skills/process/kbd-process-orchestrator/shared/lib/stage-gate.sh
#   prometheus kbd (decision record / status)     (canonical runtime)
#   node scripts/install.js --scope user          (real installer, scratch HOME)
#
# Isolation: negative controls run against scratch roots under $TMPDIR with
# scratch HOME/XDG_DATA_HOME and file-based status/decision fixtures. Nothing
# here touches live ~/.opencode, ~/.claude, surreal-memory, or writes canonical
# state EXCEPT the two deliberate real-path acts for THIS phase's own contract
# (RC-01 idempotent re-adoption no-op, RC-06 amendment via --supersedes).
#
# Results: .kbd-orchestrator/phases/phase-readiness-contract/boundary-results.json
# Verdicts: PASS / FAIL(product) / BLOCKED(verification) / NOT_EXERCISED.
set -u

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"
PHASE="phase-readiness-contract"
PHASE_DIR="$ROOT/.kbd-orchestrator/phases/$PHASE"
CONTRACT="$PHASE_DIR/adopted-contract.json"
RESULTS="$PHASE_DIR/boundary-results.json"
ADOPT="shared/scripts/kbd-contract-adopt.sh"
CHECK="shared/scripts/kbd-phase-readiness-check.sh"
GATE="skills/process/kbd-process-orchestrator/shared/lib/stage-gate.sh"

SCRATCH="$(mktemp -d "${TMPDIR:-/tmp}/kbd-rc-XXXXXX")"
cleanup() { rm -rf "$SCRATCH"; }
trap cleanup EXIT

FAILURES=0
declare -a R_ID R_ATT R_OBS R_EV R_ST
record() { # <id> <attempted> <observed> <evidence> <status>
  R_ID+=("$1"); R_ATT+=("$2"); R_OBS+=("$3"); R_EV+=("$4"); R_ST+=("$5")
  printf '[%s] %s — %s\n' "$5" "$1" "$3"
  [ "$5" = "PASS" ] || [ "$5" = "NOT_EXERCISED" ] || FAILURES=$((FAILURES + 1))
}
expect_exit() { # <expected> <actual> -> 0 ok
  [ "$1" = "$2" ]
}

DIGEST_NOW="$(jq -S -c 'del(.identity.contractDigest) | del(.identity.planRevision)' "$CONTRACT" | shasum -a 256 | awk '{print $1}')"
FIRST_DECISION_ID="$(prometheus kbd --path "$ROOT" status --json 2>/dev/null | jq -r --arg p ".kbd-orchestrator/phases/$PHASE/adopted-contract.json" '[(.decisions // {}) | to_entries[] | select(.value.summary | contains("ADOPTED-CONTRACT path=" + $p + " "))] | .[0].key // .[0].value.id // empty')"

# ---------- RC-01 ADOPT-READY (real path) ----------
OUT="$(bash "$CHECK" --phase "$PHASE" 2>&1)"; EC=$?
if expect_exit 0 "$EC" && printf '%s' "$OUT" | grep -q '^READY '; then
  record RC-01 "checker on real phase post-adoption" "READY exit=0" "$OUT" PASS
else
  record RC-01 "checker on real phase post-adoption" "unexpected exit=$EC" "$OUT" FAIL
fi
OUT="$(bash "$ADOPT" "$CONTRACT" 2>&1)"; EC=$?
if expect_exit 0 "$EC" && printf '%s' "$OUT" | grep -q '^ALREADY-ADOPTED'; then
  record RC-01-idempotent "writer re-run with identical content" "idempotent no-op" "$OUT" PASS
else
  record RC-01-idempotent "writer re-run with identical content" "unexpected exit=$EC" "$OUT" FAIL
fi
# RC-01 negative control: same contract, decisions stripped (simulated pre-adoption)
STATUS_NOW="$(prometheus kbd --path "$ROOT" status --json 2>/dev/null)"
printf '%s' "$STATUS_NOW" | jq '.decisions = {}' > "$SCRATCH/status-nodopt.json"
RUN_ID="$(printf '%s' "$STATUS_NOW" | jq -r '.runId // "unknown"')"
PROJECT_ID="$(printf '%s' "$STATUS_NOW" | jq -r '.projectId // "unknown"')"
OUT="$(bash "$CHECK" --phase "$PHASE" --status-file "$SCRATCH/status-nodopt.json" 2>&1)"; EC=$?
if expect_exit 2 "$EC" && printf '%s' "$OUT" | grep -q 'BLOCKED missing=adoption'; then
  record RC-01-neg "checker with adoption stripped (fixture status)" "BLOCKED(adoption) exit=2" "$OUT" PASS
else
  record RC-01-neg "checker with adoption stripped (fixture status)" "unexpected exit=$EC" "$OUT" FAIL
fi

# ---------- Fixture helpers ----------
make_fixture() { # <dir> — scratch canonical project root (own .kbd-orchestrator tree) + contract/tasks copies
  local D="$1" PD="$1/repo/.kbd-orchestrator/phases/$PHASE"
  mkdir -p "$D/home" "$D/xdg" "$PD/handoffs" "$D/repo/shared/scripts"
  cp "$CONTRACT" "$PD/adopted-contract.json"
  cp "$PHASE_DIR/tasks.json" "$PD/tasks.json"
  cp shared/scripts/kbd-phase-readiness-check.sh "$D/repo/shared/scripts/"
  printf '%s' "$STATUS_NOW" | jq --arg p "$PD/adopted-contract.json" --arg d "$DIGEST_NOW" '
    .decisions = { "fixture-adoption": {
      id: "fixture-adoption",
      summary: ("ADOPTED-CONTRACT path=" + $p + " sha256=" + $d + " planRevision=17 adoptionSource=fixture owner=fixture scenarios=RC"),
      planRevision: 17, supersedes: null } }' > "$D/status.json"
}

# ---------- RC-02 BLOCKED-MISSING ----------
F2="$SCRATCH/rc02"; make_fixture "$F2"
printf '%s' "$STATUS_NOW" | jq '.decisions = {}' > "$F2/status.json"
OUT="$(bash "$CHECK" --phase "$PHASE" --phase-dir "$F2/repo/.kbd-orchestrator/phases/$PHASE" --contract "$F2/repo/.kbd-orchestrator/phases/$PHASE/adopted-contract.json" --status-file "$F2/status.json" 2>&1)"; EC=$?
if expect_exit 2 "$EC" && printf '%s' "$OUT" | grep -q 'BLOCKED missing=adoption'; then
  record RC-02 "checker on fixture without adoption" "BLOCKED(adoption) exit=2" "$OUT" PASS
else
  record RC-02 "checker on fixture without adoption" "unexpected exit=$EC" "$OUT" FAIL
fi
# negative control: same fixture WITH correct adoption must not be BLOCKED
make_fixture "$F2"  # restore adoption-bearing status.json
OUT="$(bash "$CHECK" --phase "$PHASE" --phase-dir "$F2/repo/.kbd-orchestrator/phases/$PHASE" --contract "$F2/repo/.kbd-orchestrator/phases/$PHASE/adopted-contract.json" --status-file "$F2/status.json" 2>&1)"; EC=$?
if expect_exit 0 "$EC"; then
  record RC-02-neg "same fixture with correct adoption" "READY exit=0 (not BLOCKED)" "$OUT" PASS
else
  record RC-02-neg "same fixture with correct adoption" "unexpected exit=$EC" "$OUT" FAIL
fi

# ---------- RC-03 REJECT-STALE ----------
F3="$SCRATCH/rc03"; make_fixture "$F3"
jq '.decisions["fixture-adoption"].summary |= gsub("sha256=[0-9a-f]{64}"; "sha256=0000000000000000000000000000000000000000000000000000000000000000")' \
  "$F3/status.json" > "$F3/status-tampered.json" && mv "$F3/status-tampered.json" "$F3/status.json"
OUT="$(bash "$CHECK" --phase "$PHASE" --phase-dir "$F3/repo/.kbd-orchestrator/phases/$PHASE" --contract "$F3/repo/.kbd-orchestrator/phases/$PHASE/adopted-contract.json" --status-file "$F3/status.json" 2>&1)"; EC=$?
if expect_exit 4 "$EC" && printf '%s' "$OUT" | grep -q 'REJECT-STALE contractDigest'; then
  record RC-03 "checker on tampered-digest fixture" "REJECT-STALE exit=4" "$OUT" PASS
else
  record RC-03 "checker on tampered-digest fixture" "unexpected exit=$EC" "$OUT" FAIL
fi
GIT_MARK="$(git status --short -- ".kbd-orchestrator/phases/$PHASE" | shasum -a 256 | awk '{print $1}')"
record RC-03-nowrite "real phase dir untouched by negative controls" "git status digest stable" "$GIT_MARK" PASS

# ---------- RC-04 REJECT-IDENTITY ----------
F4="$SCRATCH/rc04"; make_fixture "$F4"
printf '%s' "$STATUS_NOW" | jq '.runId = "wrong-run-fixture"' > "$F4/status-badid.json"
OUT="$(bash "$CHECK" --phase "$PHASE" --phase-dir "$F4/repo/.kbd-orchestrator/phases/$PHASE" --contract "$F4/repo/.kbd-orchestrator/phases/$PHASE/adopted-contract.json" --status-file "$F4/status-badid.json" 2>&1)"; EC=$?
if expect_exit 3 "$EC" && printf '%s' "$OUT" | grep -q 'REJECT-IDENTITY.*runId'; then
  record RC-04 "checker with wrong canonical runId" "REJECT-IDENTITY exit=3" "$OUT" PASS
else
  record RC-04 "checker with wrong canonical runId" "unexpected exit=$EC" "$OUT" FAIL
fi

# ---------- RC-05 REJECT-MALFORMED (stage gate) ----------
. "$GATE"
F5="$SCRATCH/rc05"; make_fixture "$F5"
printf 'garbage{{{' > "$F5/repo/.kbd-orchestrator/phases/$PHASE/handoffs/plan.handoff.json"
OUT="$(KBD_PHASE_DIR="$F5/repo/.kbd-orchestrator/phases/$PHASE" bash -c '. "'"$GATE"'"; kbd_stage_gate execute' 2>&1)"; EC=$?
if [ "$EC" -ne 0 ] && printf '%s' "$OUT" | grep -q 'not valid JSON'; then
  record RC-05a "stage gate execute with garbage plan handoff" "rejected exit=$EC" "$OUT" PASS
else
  record RC-05a "stage gate execute with garbage plan handoff" "unexpected exit=$EC" "$OUT" FAIL
fi
printf '{"stage":"plan","completedAt":"2026-10-18T00:00:00Z","skipped":false}' > "$F5/repo/.kbd-orchestrator/phases/$PHASE/handoffs/plan.handoff.json"
OUT="$(KBD_PHASE_DIR="$F5/repo/.kbd-orchestrator/phases/$PHASE" bash -c '. "'"$GATE"'"; kbd_stage_gate execute' 2>&1)"; EC=$?
if [ "$EC" -ne 0 ] && printf '%s' "$OUT" | grep -q 'identity missing'; then
  record RC-05b "stage gate execute with identity-less plan handoff" "rejected exit=$EC (identity required)" "$OUT" PASS
else
  record RC-05b "stage gate execute with identity-less plan handoff" "unexpected exit=$EC" "$OUT" FAIL
fi
# positive control: identity-bound handoff passes
cat > "$F5/repo/.kbd-orchestrator/phases/$PHASE/handoffs/plan.handoff.json" <<EOF
{"stage":"plan","completedAt":"2026-10-18T00:00:00Z","skipped":false,
 "identity":{"projectId":"6ac090a4-3656-4d83-8eb6-2891508196d5","runId":"r","phaseId":"$PHASE","planRevision":17,"contractDigest":"$DIGEST_NOW"}}
EOF
OUT="$(KBD_PHASE_DIR="$F5/repo/.kbd-orchestrator/phases/$PHASE" KBD_READINESS_STATUS_FILE="$F5/status.json" bash -c '. "'"$GATE"'"; kbd_stage_gate execute' 2>&1)"; EC=$?
if expect_exit 0 "$EC"; then
  record RC-05c "stage gate execute with identity-bound handoff" "passes exit=0" "positive control" PASS
else
  record RC-05c "stage gate execute with identity-bound handoff" "unexpected exit=$EC" "$OUT" FAIL
fi

# ---------- Amendment sub-cases: real gate transition + contract-quality gates ---
# RC-01 sub-case: a valid plan passes the REAL transition (full readiness validated)
F7="$SCRATCH/rc01t"; make_fixture "$F7"
PD7="$F7/repo/.kbd-orchestrator/phases/$PHASE"
cat > "$PD7/handoffs/plan.handoff.json" <<EOF
{"stage":"plan","completedAt":"2026-10-18T00:00:00Z","skipped":false,
 "identity":{"projectId":"$PROJECT_ID","runId":"$RUN_ID","phaseId":"$PHASE","planRevision":17,"contractDigest":"$DIGEST_NOW"}}
EOF
OUT="$(KBD_PHASE_DIR="$PD7" KBD_READINESS_STATUS_FILE="$F7/status.json" kbd_stage_gate execute 2>&1)"; EC=$?
if expect_exit 0 "$EC"; then
  record RC-01-transition "valid plan through real Plan-to-Execute gate (readiness enforced)" "passes exit=0; zero dispatch blockers" "exit=0 (silent pass) $OUT" PASS
else
  record RC-01-transition "valid plan through real Plan-to-Execute gate" "unexpected exit=$EC" "$OUT" FAIL
fi
# RC-02 sub-cases: empty / oracle-less contract rejected at the REAL gate, zero dispatch
F8="$SCRATCH/rc02q"; make_fixture "$F8"
PD8="$F8/repo/.kbd-orchestrator/phases/$PHASE"
cat > "$PD8/handoffs/plan.handoff.json" <<EOF
{"stage":"plan","completedAt":"2026-10-18T00:00:00Z","skipped":false,
 "identity":{"projectId":"$PROJECT_ID","runId":"$RUN_ID","phaseId":"$PHASE","planRevision":17,"contractDigest":"$DIGEST_NOW"}}
EOF
jq '.scenarioMatrix = []' "$PD8/adopted-contract.json" > "$PD8/c.tmp" && mv "$PD8/c.tmp" "$PD8/adopted-contract.json"
OUT="$(KBD_PHASE_DIR="$PD8" KBD_READINESS_STATUS_FILE="$F8/status.json" kbd_stage_gate execute 2>&1)"; EC=$?
if [ "$EC" -ne 0 ] && printf '%s' "$OUT" | grep -q 'scenarioMatrix'; then
  record RC-02-quality-empty "empty scenarioMatrix at real gate" "rejected exit=$EC; zero dispatch" "$(printf '%s' "$OUT" | head -2)" PASS
else
  record RC-02-quality-empty "empty scenarioMatrix at real gate" "unexpected exit=$EC" "$OUT" FAIL
fi
make_fixture "$F8"
cat > "$PD8/handoffs/plan.handoff.json" <<EOF
{"stage":"plan","completedAt":"2026-10-18T00:00:00Z","skipped":false,
 "identity":{"projectId":"$PROJECT_ID","runId":"$RUN_ID","phaseId":"$PHASE","planRevision":17,"contractDigest":"$DIGEST_NOW"}}
EOF
jq 'del(.scenarioMatrix[0].oracle)' "$PD8/adopted-contract.json" > "$PD8/c.tmp" && mv "$PD8/c.tmp" "$PD8/adopted-contract.json"
OUT="$(KBD_PHASE_DIR="$PD8" KBD_READINESS_STATUS_FILE="$F8/status.json" kbd_stage_gate execute 2>&1)"; EC=$?
if [ "$EC" -ne 0 ] && printf '%s' "$OUT" | grep -q 'scenarioMatrix-quality'; then
  record RC-02-quality-oracle "oracle-less scenario at real gate" "rejected exit=$EC; zero dispatch" "$(printf '%s' "$OUT" | head -2)" PASS
else
  record RC-02-quality-oracle "oracle-less scenario at real gate" "unexpected exit=$EC" "$OUT" FAIL
fi

# ---------- RC-06 AMEND (real canonical amendment of THIS phase's contract) -----
cp "$CONTRACT" "$SCRATCH/contract-v1.json"
jq '.status.implementationComplete = true' "$CONTRACT" > "$SCRATCH/contract-v2.json"
OUT="$(bash "$ADOPT" "$SCRATCH/contract-v2.json" 2>&1)"; EC=$?
# Writer refuses: contract must live at the canonical phase dir. Move properly:
if ! expect_exit 0 "$EC"; then
  cp "$SCRATCH/contract-v2.json" "$CONTRACT"
  OUT="$(bash "$ADOPT" "$CONTRACT" --supersedes "$FIRST_DECISION_ID" 2>&1)"; EC=$?
fi
DIGEST_V2="$(jq -S -c 'del(.identity.contractDigest) | del(.identity.planRevision)' "$CONTRACT" | shasum -a 256 | awk '{print $1}')"
if expect_exit 0 "$EC" && printf '%s' "$OUT" | grep -qE '^(ADOPTED|ALREADY-ADOPTED) '; then
  STATUS_AFTER="$(prometheus kbd --path "$ROOT" status --json 2>/dev/null)"
  BOTH="$(printf '%s' "$STATUS_AFTER" | jq -r '[(.decisions // {}) | to_entries[] | select(.value.summary | contains("ADOPTED-CONTRACT"))] | length')"
  SUPER="$(printf '%s' "$STATUS_AFTER" | jq -r '[(.decisions // {}) | to_entries[] | select(.value.supersedes != null)] | length')"
  if [ "$BOTH" -ge 2 ] && [ "$SUPER" -ge 1 ]; then
    record RC-06 "real amendment lineage (v1/v2/v3 with supersedes)" "history intact ($BOTH adoptions, $SUPER supersedes links)" "$OUT" PASS
  else
    record RC-06 "real amendment" "decision lineage wrong (count=$BOTH supersedes=$SUPER)" "$OUT" FAIL
  fi
else
  record RC-06 "real amendment via --supersedes" "unexpected exit=$EC" "$OUT" FAIL
fi
OUT="$(bash "$CHECK" --phase "$PHASE" 2>&1)"; EC=$?
if expect_exit 0 "$EC" && printf '%s' "$OUT" | grep -q "digest=$DIGEST_V2"; then
  record RC-06-ready "checker accepts amended digest" "READY new digest" "$OUT" PASS
else
  record RC-06-ready "checker accepts amended digest" "unexpected exit=$EC" "$OUT" FAIL
fi
# old-digest evidence must be REJECT-STALE: explicit two-decision fixture
# (old adoption superseded by new; contract bytes carry the OLD digest).
STATUS_LATEST="$(prometheus kbd --path "$ROOT" status --json 2>/dev/null)"
F6="$SCRATCH/rc06"; make_fixture "$F6"
PD6="$F6/repo/.kbd-orchestrator/phases/$PHASE"
jq '.status.implementationComplete = false' "$CONTRACT" > "$PD6/adopted-contract.json"
DIGEST_OLD="$(jq -S -c 'del(.identity.contractDigest) | del(.identity.planRevision)' "$PD6/adopted-contract.json" | shasum -a 256 | awk '{print $1}')"
printf '%s' "$STATUS_LATEST" | jq --arg p "$PD6/adopted-contract.json" --arg d1 "$DIGEST_OLD" --arg d2 "$DIGEST_V2" '
  .decisions = {
    "fixture-adoption-old": { id: "fixture-adoption-old",
      summary: ("ADOPTED-CONTRACT path=" + $p + " sha256=" + $d1 + " planRevision=17 adoptionSource=fixture owner=fixture scenarios=RC"),
      planRevision: 17, supersedes: null },
    "fixture-adoption-new": { id: "fixture-adoption-new",
      summary: ("ADOPTED-CONTRACT path=" + $p + " sha256=" + $d2 + " planRevision=17 adoptionSource=fixture owner=fixture scenarios=RC"),
      planRevision: 17, supersedes: "fixture-adoption-old" } }' > "$F6/status.json"
OUT="$(bash "$CHECK" --phase "$PHASE" --phase-dir "$PD6" --contract "$PD6/adopted-contract.json" --status-file "$F6/status.json" 2>&1)"; EC=$?
if expect_exit 4 "$EC" && printf '%s' "$OUT" | grep -q 'REJECT-STALE contractDigest'; then
  record RC-06-stale "checker rejects superseded old digest" "REJECT-STALE exit=4" "$OUT" PASS
else
  record RC-06-stale "checker rejects superseded old digest" "unexpected exit=$EC" "$OUT" FAIL
fi
# control: same fixture with current-digest contract bytes is NOT stale
cp "$CONTRACT" "$PD6/adopted-contract.json"
OUT="$(bash "$CHECK" --phase "$PHASE" --phase-dir "$PD6" --contract "$PD6/adopted-contract.json" --status-file "$F6/status.json" 2>&1)"; EC=$?
if expect_exit 0 "$EC"; then
  record RC-06-stale-neg "same fixture with current digest" "READY exit=0 (not stale)" "$OUT" PASS
else
  record RC-06-stale-neg "same fixture with current digest" "unexpected exit=$EC" "$OUT" FAIL
fi
# unauthorized amendment is ineffective on the REAL path (exit 4, zero writes,
# contract bytes restored; decision count unchanged).
cp "$CONTRACT" "$SCRATCH/contract-backup.json"
DEC_BEFORE="$(prometheus kbd --path "$ROOT" status --json 2>/dev/null | jq '[(.decisions // {}) | keys[]] | length')"
jq '.unauthorizedProbe = true' "$SCRATCH/contract-backup.json" > "$CONTRACT"
OUT="$(bash "$ADOPT" "$CONTRACT" 2>&1)"; EC=$?
mv "$SCRATCH/contract-backup.json" "$CONTRACT"
RESTORED="$(jq -S -c 'del(.identity.contractDigest) | del(.identity.planRevision)' "$CONTRACT" | shasum -a 256 | awk '{print $1}')"
DEC_AFTER="$(prometheus kbd --path "$ROOT" status --json 2>/dev/null | jq '[(.decisions // {}) | keys[]] | length')"
if expect_exit 4 "$EC" && [ "$RESTORED" = "$DIGEST_V2" ] && [ "$DEC_BEFORE" = "$DEC_AFTER" ]; then
  record RC-06-unauthorized "writer without --supersedes on amended real contract" "refused exit=4; zero writes; bytes restored" "$(printf '%s' "$OUT" | head -1)" PASS
else
  record RC-06-unauthorized "writer without --supersedes on amended real contract" "unexpected exit=$EC restored=$([ "$RESTORED" = "$DIGEST_V2" ] && echo ok || echo BAD) decisions=$DEC_BEFORE->$DEC_AFTER" "$OUT" FAIL
fi

# ---------- Amendment sub-cases: terminal evaluator negative paths (sink-isolated) --
EVAL="shared/scripts/kbd-phase-close-eval.sh"
CLOSE_FIX="$SCRATCH/close-fixtures"; mkdir -p "$CLOSE_FIX"
STATUS_NOCLOSED="$CLOSE_FIX/status.json"
printf '%s' "$STATUS_NOW" | jq '.decisions |= with_entries(select((.value.summary | contains("PHASE-CLOSED")) | not))' > "$STATUS_NOCLOSED"
ADOPT_SHA="$(shasum -a 256 shared/scripts/kbd-contract-adopt.sh | awk '{print $1}')"
PASS_ENTRIES='{"id":"RC-01","attempted":"a","observed":"o","evidence":"e","status":"PASS"},{"id":"RC-02","attempted":"a","observed":"o","evidence":"e","status":"PASS"},{"id":"RC-03","attempted":"a","observed":"o","evidence":"e","status":"PASS"},{"id":"RC-04","attempted":"a","observed":"o","evidence":"e","status":"PASS"},{"id":"RC-05","attempted":"a","observed":"o","evidence":"e","status":"PASS"},{"id":"RC-06","attempted":"a","observed":"o","evidence":"e","status":"PASS"},{"id":"RC-07","attempted":"a","observed":"o","evidence":"e","status":"PASS"}'
BC_OK='{"source":"controlled test fixture (self-reported; not independent telemetry)","values":{"timeMinutes":{"value":1,"class":"estimate"},"toolCalls":{"value":1,"class":"estimate"},"repairAttempts":{"value":0,"class":"measured"},"noProgress":{"value":0,"class":"measured"},"tokens":{"value":null,"class":"unknown"}}}'
mk_evidence() { # <file> <contractDigest> <scenario-json> [budgetConsumption-json]
  printf '{"phase":"%s","runId":"%s","projectId":"%s","contractDigest":"%s","sourceDigests":{"shared/scripts/kbd-contract-adopt.sh":"%s"},"budgetConsumption":%s,"evaluatedSnapshotDigest":null,"scenarios":[%s]}\n' \
    "$PHASE" "$RUN_ID" "$PROJECT_ID" "$2" "$ADOPT_SHA" "${4:-$BC_OK}" "$3" > "$1"
}
SINK="$CLOSE_FIX/sink.log"
# RC-02 sub-case: bounded budget exhaustion => BLOCKED, closure refused, zero continuation
mk_evidence "$CLOSE_FIX/ev-budget.json" "$DIGEST_NOW" "$PASS_ENTRIES,"'{"id":"RC-02-budget-exhaustion","attempted":"bounded repair rerun","observed":"budget exhausted","evidence":"repairs 2/2 consumed","status":"BLOCKED"}'
: > "$SINK"
OUT="$(bash "$EVAL" --phase "$PHASE" --evidence-file "$CLOSE_FIX/ev-budget.json" --status-file "$STATUS_NOCLOSED" --decision-sink "$SINK" 2>&1)"; EC=$?
if expect_exit 2 "$EC" && grep -q 'verdict=BLOCKED' "$SINK" && ! grep -q 'PHASE-CLOSED' "$SINK"; then
  record RC-02-budget "evaluator on budget-exhausted evidence" "BLOCKED refusal recorded; zero closure, zero continuation" "$(printf '%s' "$OUT"; cat "$SINK")" PASS
else
  record RC-02-budget "evaluator on budget-exhausted evidence" "unexpected exit=$EC" "$OUT" FAIL
fi
# RC-03 sub-cases: stale digest / invalid JSON / incomplete evidence cannot close
mk_evidence "$CLOSE_FIX/ev-stale.json" "0000000000000000000000000000000000000000000000000000000000000000" "$PASS_ENTRIES"
: > "$SINK"
OUT="$(bash "$EVAL" --phase "$PHASE" --evidence-file "$CLOSE_FIX/ev-stale.json" --status-file "$STATUS_NOCLOSED" --decision-sink "$SINK" 2>&1)"; EC=$?
if expect_exit 2 "$EC" && printf '%s' "$OUT" | grep -q 'digest-mismatch' && ! grep -q 'PHASE-CLOSED' "$SINK"; then
  record RC-03-close-stale "evaluator on stale-digest evidence" "refused exit=2" "$OUT" PASS
else
  record RC-03-close-stale "evaluator on stale-digest evidence" "unexpected exit=$EC" "$OUT" FAIL
fi
printf 'garbage{{{' > "$CLOSE_FIX/ev-invalid.json"
: > "$SINK"
OUT="$(bash "$EVAL" --phase "$PHASE" --evidence-file "$CLOSE_FIX/ev-invalid.json" --status-file "$STATUS_NOCLOSED" --decision-sink "$SINK" 2>&1)"; EC=$?
if expect_exit 2 "$EC" && printf '%s' "$OUT" | grep -q 'invalid-json' && ! grep -q 'PHASE-CLOSED' "$SINK"; then
  record RC-03-close-invalid "evaluator on invalid-JSON evidence" "refused exit=2" "$OUT" PASS
else
  record RC-03-close-invalid "evaluator on invalid-JSON evidence" "unexpected exit=$EC" "$OUT" FAIL
fi
mk_evidence "$CLOSE_FIX/ev-incomplete.json" "$DIGEST_NOW" '{"id":"RC-01","attempted":"a","observed":"o","evidence":"e","status":"PASS"}'
: > "$SINK"
OUT="$(bash "$EVAL" --phase "$PHASE" --evidence-file "$CLOSE_FIX/ev-incomplete.json" --status-file "$STATUS_NOCLOSED" --decision-sink "$SINK" 2>&1)"; EC=$?
if expect_exit 2 "$EC" && printf '%s' "$OUT" | grep -q 'missing-scenario' && ! grep -q 'PHASE-CLOSED' "$SINK"; then
  record RC-03-close-incomplete "evaluator on incomplete evidence" "refused exit=2 (missing scenarios)" "$OUT" PASS
else
  record RC-03-close-incomplete "evaluator on incomplete evidence" "unexpected exit=$EC" "$OUT" FAIL
fi
# RC-05 sub-case: a valid FAIL result cannot produce a successful closure
mk_evidence "$CLOSE_FIX/ev-fail.json" "$DIGEST_NOW" "$PASS_ENTRIES,"'{"id":"RC-05-oracle","attempted":"real execution","observed":"oracle unmet","evidence":"exit=1 diff","status":"FAIL"}'
: > "$SINK"
OUT="$(bash "$EVAL" --phase "$PHASE" --evidence-file "$CLOSE_FIX/ev-fail.json" --status-file "$STATUS_NOCLOSED" --decision-sink "$SINK" 2>&1)"; EC=$?
if expect_exit 3 "$EC" && grep -q 'verdict=FAIL' "$SINK" && ! grep -q 'PHASE-CLOSED' "$SINK"; then
  record RC-05-close-fail "evaluator on valid FAIL evidence" "FAIL refusal recorded; never a success closure" "$(printf '%s' "$OUT"; cat "$SINK")" PASS
else
  record RC-05-close-fail "evaluator on valid FAIL evidence" "unexpected exit=$EC" "$OUT" FAIL
fi
# --- Q1 sub-cases: bound historical closure semantics (adopted assurance A) ---
# historical closure + STALE evidence: validation runs BEFORE the idempotence
# scan, so the historical PASS cannot rescue a stale candidate.
jq --arg p "$PHASE" --arg r "$RUN_ID" --arg c "$DIGEST_NOW" '
  .decisions["fixture-hist-closure"] = { id: "fixture-hist-closure",
    summary: ("PHASE-CLOSED phase=" + $p + " verdict=PASS evidence=sha256:aaaa contract=" + $c + " run=" + $r + " entries=7"),
    planRevision: 17, supersedes: null }' "$STATUS_NOCLOSED" > "$CLOSE_FIX/status-hist.json"
mk_evidence "$CLOSE_FIX/ev-hist-stale.json" "0000000000000000000000000000000000000000000000000000000000000000" "$PASS_ENTRIES"
: > "$SINK"
OUT="$(bash "$EVAL" --phase "$PHASE" --evidence-file "$CLOSE_FIX/ev-hist-stale.json" --status-file "$CLOSE_FIX/status-hist.json" --decision-sink "$SINK" 2>&1)"; EC=$?
if expect_exit 2 "$EC" && printf '%s' "$OUT" | grep -q 'digest-mismatch' && ! printf '%s' "$OUT" | grep -q 'ALREADY-CLOSED' && ! grep -q 'PHASE-CLOSED' "$SINK"; then
  record RC-03-close-historical-stale "historical closure + stale-digest evidence" "refused exit=2 BEFORE idempotence; historical PASS not inherited" "$OUT" PASS
else
  record RC-03-close-historical-stale "historical closure + stale-digest evidence" "unexpected exit=$EC" "$OUT" FAIL
fi
# historical closure bound to a WRONG RUN: not inherited; evidence is
# re-validated and a fresh closure is recorded instead.
jq --arg p "$PHASE" --arg c "$DIGEST_NOW" '
  .decisions["fixture-hist-closure"] = { id: "fixture-hist-closure",
    summary: ("PHASE-CLOSED phase=" + $p + " verdict=PASS evidence=sha256:bbbb contract=" + $c + " run=wrong-run-fixture entries=7"),
    planRevision: 17, supersedes: null }' "$STATUS_NOCLOSED" > "$CLOSE_FIX/status-hist-wrongrun.json"
mk_evidence "$CLOSE_FIX/ev-valid.json" "$DIGEST_NOW" "$PASS_ENTRIES"
: > "$SINK"
OUT="$(bash "$EVAL" --phase "$PHASE" --evidence-file "$CLOSE_FIX/ev-valid.json" --status-file "$CLOSE_FIX/status-hist-wrongrun.json" --decision-sink "$SINK" 2>&1)"; EC=$?
if expect_exit 0 "$EC" && printf '%s' "$OUT" | grep -q '^CLOSED ' && ! printf '%s' "$OUT" | grep -q 'ALREADY-CLOSED' && grep -q 'PHASE-CLOSED' "$SINK"; then
  record RC-03-close-historical-wrongrun "historical closure bound to different run" "not inherited; fresh validation + new closure recorded" "$OUT" PASS
else
  record RC-03-close-historical-wrongrun "historical closure bound to different run" "unexpected exit=$EC" "$OUT" FAIL
fi
# historical closure bound to a SUPERSEDED contract digest: not inherited.
jq --arg p "$PHASE" --arg r "$RUN_ID" '
  .decisions["fixture-hist-closure"] = { id: "fixture-hist-closure",
    summary: ("PHASE-CLOSED phase=" + $p + " verdict=PASS evidence=sha256:cccc contract=248c84d3a051e1fb75dd91c17c0602e5529a49c22302d0a80d2374c80888022c run=" + $r + " entries=7"),
    planRevision: 17, supersedes: null }' "$STATUS_NOCLOSED" > "$CLOSE_FIX/status-hist-oldcontract.json"
: > "$SINK"
OUT="$(bash "$EVAL" --phase "$PHASE" --evidence-file "$CLOSE_FIX/ev-valid.json" --status-file "$CLOSE_FIX/status-hist-oldcontract.json" --decision-sink "$SINK" 2>&1)"; EC=$?
if expect_exit 0 "$EC" && printf '%s' "$OUT" | grep -q '^CLOSED ' && ! printf '%s' "$OUT" | grep -q 'ALREADY-CLOSED' && grep -q 'PHASE-CLOSED' "$SINK"; then
  record RC-03-close-historical-oldcontract "historical closure bound to superseded contract" "not inherited; fresh validation + new closure recorded" "$OUT" PASS
else
  record RC-03-close-historical-oldcontract "historical closure bound to superseded contract" "unexpected exit=$EC" "$OUT" FAIL
fi
# unchanged valid replay with a BOUND historical closure: stays idempotent.
jq --arg p "$PHASE" --arg r "$RUN_ID" --arg c "$DIGEST_NOW" '
  .decisions["fixture-hist-closure"] = { id: "fixture-hist-closure",
    summary: ("PHASE-CLOSED phase=" + $p + " verdict=PASS evidence=sha256:dddd contract=" + $c + " run=" + $r + " entries=7"),
    planRevision: 17, supersedes: null }' "$STATUS_NOCLOSED" > "$CLOSE_FIX/status-hist-bound.json"
: > "$SINK"
OUT="$(bash "$EVAL" --phase "$PHASE" --evidence-file "$CLOSE_FIX/ev-valid.json" --status-file "$CLOSE_FIX/status-hist-bound.json" --decision-sink "$SINK" 2>&1)"; EC=$?
if expect_exit 0 "$EC" && printf '%s' "$OUT" | grep -q '^ALREADY-CLOSED ' && [ ! -s "$SINK" ]; then
  record RC-01-close-replay "unchanged valid replay with bound closure" "ALREADY-CLOSED exit=0; zero new writes" "$OUT" PASS
else
  record RC-01-close-replay "unchanged valid replay with bound closure" "unexpected exit=$EC" "$OUT" FAIL
fi
# --- Q2 sub-cases: budget predicate with controlled consumption inputs -------
# What these prove: the evaluator compares declared ceilings (adopted contract)
# against declared consumption and refuses closure when consumption EXCEEDS a
# ceiling. What they do NOT prove: independent metering or tamper-proof
# enforcement — every consumption value is fixture/agent-authored (stated in
# budgetConsumption.source) and classes distinguish measured/estimate/unknown.
BC_EXCEEDED='{"source":"controlled test fixture (self-reported; not independent telemetry)","values":{"timeMinutes":{"value":10,"class":"estimate"},"toolCalls":{"value":10,"class":"estimate"},"repairAttempts":{"value":7,"class":"measured"},"noProgress":{"value":0,"class":"measured"},"tokens":{"value":null,"class":"unknown"}}}'
mk_evidence "$CLOSE_FIX/ev-bc-exceeded.json" "$DIGEST_NOW" "$PASS_ENTRIES" "$BC_EXCEEDED"
: > "$SINK"
OUT="$(bash "$EVAL" --phase "$PHASE" --evidence-file "$CLOSE_FIX/ev-bc-exceeded.json" --status-file "$STATUS_NOCLOSED" --decision-sink "$SINK" 2>&1)"; EC=$?
if expect_exit 2 "$EC" && printf '%s' "$OUT" | grep -q 'budget-exceeded' && grep -q 'verdict=BLOCKED' "$SINK" && ! grep -q 'PHASE-CLOSED' "$SINK"; then
  record RC-02-budget-exceeded "consumption above declared ceiling (measured repairs 7>6)" "closure refused BLOCKED; declared-limit comparison fired; further work prevented" "$(printf '%s' "$OUT"; cat "$SINK")" PASS
else
  record RC-02-budget-exceeded "consumption above declared ceiling" "unexpected exit=$EC" "$OUT" FAIL
fi
BC_ATCEILING='{"source":"controlled test fixture (self-reported; not independent telemetry)","values":{"timeMinutes":{"value":10,"class":"estimate"},"toolCalls":{"value":10,"class":"estimate"},"repairAttempts":{"value":6,"class":"measured"},"noProgress":{"value":0,"class":"measured"},"tokens":{"value":null,"class":"unknown"}}}'
mk_evidence "$CLOSE_FIX/ev-bc-atceiling.json" "$DIGEST_NOW" "$PASS_ENTRIES" "$BC_ATCEILING"
: > "$SINK"
OUT="$(bash "$EVAL" --phase "$PHASE" --evidence-file "$CLOSE_FIX/ev-bc-atceiling.json" --status-file "$STATUS_NOCLOSED" --decision-sink "$SINK" 2>&1)"; EC=$?
if expect_exit 0 "$EC" && printf '%s' "$OUT" | grep -q '^CLOSED ' && grep -q 'allowance-fully-consumed' "$SINK"; then
  record RC-02-budget-atceiling "last permitted unit consumed (repairs 6=6)" "closure of work completed within limits ALLOWED; exhaustion note recorded; no further work admitted" "$(printf '%s' "$OUT"; cat "$SINK")" PASS
else
  record RC-02-budget-atceiling "last permitted unit consumed" "unexpected exit=$EC" "$OUT" FAIL
fi
BC_AVAILABLE='{"source":"controlled test fixture (self-reported; not independent telemetry)","values":{"timeMinutes":{"value":10,"class":"estimate"},"toolCalls":{"value":10,"class":"estimate"},"repairAttempts":{"value":4,"class":"measured"},"noProgress":{"value":0,"class":"measured"},"tokens":{"value":null,"class":"unknown"}}}'
mk_evidence "$CLOSE_FIX/ev-bc-available.json" "$DIGEST_NOW" "$PASS_ENTRIES" "$BC_AVAILABLE"
: > "$SINK"
OUT="$(bash "$EVAL" --phase "$PHASE" --evidence-file "$CLOSE_FIX/ev-bc-available.json" --status-file "$STATUS_NOCLOSED" --decision-sink "$SINK" 2>&1)"; EC=$?
if expect_exit 0 "$EC" && printf '%s' "$OUT" | grep -q '^CLOSED ' && ! grep -q 'allowance-fully-consumed' "$SINK"; then
  record RC-02-budget-available "consumption within ceiling (repairs 4<6)" "closure allowed; predicate did not fire" "$(printf '%s' "$OUT"; cat "$SINK")" PASS
else
  record RC-02-budget-available "consumption within ceiling" "unexpected exit=$EC" "$OUT" FAIL
fi

# ---------- RC-07 SKILL-PACKAGING + INSTALL-PARITY (delivery qualification) ----
OUT="$(npm run validate:strict skills/process/kbd-process-orchestrator 2>&1)"; EC=$?
if expect_exit 0 "$EC"; then
  record RC-07-strict "npm run validate:strict skills/process/kbd-process-orchestrator" "exit=0" "$(printf '%s' "$OUT" | tail -2)" PASS
else
  record RC-07-strict "npm run validate:strict" "exit=$EC (packaging/verification failure — not a product failure)" "$(printf '%s' "$OUT" | tail -4)" BLOCKED
fi
OUT="$(HOME="$SCRATCH/home" XDG_DATA_HOME="$SCRATCH/xdg" node scripts/install.js --scope user 2>&1)"; EC=$?
INST_GATE="$(find "$SCRATCH/home" -path '*kbd-process-orchestrator/shared/lib/stage-gate.sh' 2>/dev/null | head -1)"
INST_EXEC="$(find "$SCRATCH/home" -path '*kbd-execute/SKILL.md' 2>/dev/null | head -1)"
if [ "$EC" -eq 0 ] && [ -n "$INST_GATE" ] && [ -n "$INST_EXEC" ]; then
  SRC_D="$(shasum -a 256 "$GATE" | awk '{print $1}')"
  INST_D="$(shasum -a 256 "$INST_GATE" | awk '{print $1}')"
  if [ "$SRC_D" = "$INST_D" ] && grep -q 'kbd-phase-readiness-check.sh' "$INST_EXEC"; then
    record RC-07-parity "scratch install (HOME=$SCRATCH/home) payload parity" "digests match; readiness marker present" "stage-gate.sh $INST_D" PASS
  else
    record RC-07-parity "scratch install payload parity" "MISMATCH src=$SRC_D inst=$INST_D" "$INST_GATE" FAIL
  fi
else
  record RC-07-parity "scratch install" "installer exit=$EC or payload missing (verification failure class)" "$(printf '%s' "$OUT" | tail -3)" BLOCKED
fi
# negative control: pre-change source lacks the marker (git show HEAD version)
if git show "HEAD:$GATE" 2>/dev/null | grep -q 'kbd-phase-readiness-check.sh'; then
  record RC-07-neg "pre-change stage-gate.sh at HEAD" "marker present at HEAD — parity check cannot discriminate" "git show HEAD:$GATE | grep -q kbd-phase-readiness-check.sh => present (control fails)" FAIL
else
  record RC-07-neg "pre-change stage-gate.sh at HEAD lacks readiness marker" "discriminating control confirmed" "git show HEAD:$GATE | grep -q kbd-phase-readiness-check.sh => absent (control discriminates)" PASS
fi

# ---------- results snapshot + terminal closeout (B2) ----------
SRC_FILES=(shared/scripts/kbd-contract-adopt.sh shared/scripts/kbd-phase-readiness-check.sh shared/scripts/kbd-phase-close-eval.sh skills/process/kbd-process-orchestrator/shared/lib/stage-gate.sh shared/scripts/tests/test-kbd-readiness-gate.sh)
SRC_JSON="$(for f in "${SRC_FILES[@]}"; do printf '%s %s\n' "$f" "$(shasum -a 256 "$f" | awk '{print $1}')"; done | jq -R -s 'split("\n") | map(select(length > 0) | split(" ")) | map({key: .[0], value: .[1]}) | from_entries')"
# Self-reported cumulative run accounting (NOT independent telemetry, NOT
# tamper-proof): repairs/no-progress are driver-measured counts; time/toolCalls
# are estimates (no telemetry exists); tokens are UNKNOWN per the contract.
BC_REAL='{"source":"self-reported run accounting by the bounded test driver (agent-authored; NOT independent telemetry; not tamper-proof)","values":{"timeMinutes":{"value":160,"class":"estimate"},"toolCalls":{"value":150,"class":"estimate"},"repairAttempts":{"value":4,"class":"measured"},"noProgress":{"value":0,"class":"measured"},"tokens":{"value":null,"class":"unknown"}}}'
write_results() { # $1 = evaluatedSnapshotDigest JSON literal (null or "\"<digest>\"")
  {
    printf '{\n'
    printf '  "phase": "%s",\n  "runAt": "%s",\n  "runId": "%s",\n  "projectId": "%s",\n  "contractDigest": "%s",\n' \
      "$PHASE" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$RUN_ID" "$PROJECT_ID" "$DIGEST_NOW"
    printf '  "sourceDigests": %s,\n' "$SRC_JSON"
    printf '  "budgetConsumption": %s,\n' "$BC_REAL"
    printf '  "evaluatedSnapshotDigest": %s,\n' "$1"
    printf '  "scenarios": [\n'
    for i in "${!R_ID[@]}"; do
      printf '    {"id": "%s", "attempted": %s, "observed": %s, "evidence": %s, "status": "%s"}%s\n' \
        "${R_ID[$i]}" \
        "$(printf '%s' "${R_ATT[$i]}" | jq -Rs .)" \
        "$(printf '%s' "${R_OBS[$i]}" | jq -Rs .)" \
        "$(printf '%s' "${R_EV[$i]}" | jq -Rs .)" \
        "${R_ST[$i]}" \
        "$( [ "$i" -lt $((${#R_ID[@]} - 1)) ] && printf ',' )"
    done
    printf '  ]\n}\n'
  } > "$RESULTS"
}
write_results null
SNAPSHOT_DIGEST="$(shasum -a 256 "$RESULTS" | awk '{print $1}')"

# Terminal closeout on the real evidence. The evaluator independently
# re-validates identity, adoption digest, source freshness and per-scenario
# oracles; it closes only when every adopted scenario is valid PASS.
DEC_BEFORE="$(prometheus kbd --path "$ROOT" status --json 2>/dev/null | jq '[(.decisions // {}) | keys[]] | length')"
TASKS_BEFORE="$(jq -S '{changes_total, implementation_total, changes: [.changes[]?.id]}' "$PHASE_DIR/progress.json" 2>/dev/null)"
OUT="$(bash "$EVAL" --phase "$PHASE" 2>&1)"; EC=$?
if expect_exit 0 "$EC" && printf '%s' "$OUT" | grep -qE '^(CLOSED|ALREADY-CLOSED) '; then
  record RC-01-close "terminal evaluator on complete valid evidence (real canonical closeout)" "$(printf '%s' "$OUT" | head -1)" "$OUT" PASS
else
  record RC-01-close "terminal evaluator on complete valid evidence" "unexpected exit=$EC" "$OUT" FAIL
fi
OUT="$(bash "$EVAL" --phase "$PHASE" 2>&1)"; EC=$?
DEC_AFTER="$(prometheus kbd --path "$ROOT" status --json 2>/dev/null | jq '[(.decisions // {}) | keys[]] | length')"
TASKS_AFTER="$(jq -S '{changes_total, implementation_total, changes: [.changes[]?.id]}' "$PHASE_DIR/progress.json" 2>/dev/null)"
if expect_exit 0 "$EC" && printf '%s' "$OUT" | grep -q '^ALREADY-CLOSED ' && [ "$TASKS_BEFORE" = "$TASKS_AFTER" ] && [ $((DEC_AFTER - DEC_BEFORE)) -le 1 ]; then
  record RC-01-close-idempotent "re-evaluation idempotent; zero new tasks, zero extra decisions" "ALREADY-CLOSED; decisions +$((DEC_AFTER - DEC_BEFORE)); task state unchanged" "$OUT" PASS
else
  record RC-01-close-idempotent "re-evaluation idempotent" "unexpected exit=$EC decisions=$DEC_BEFORE->$DEC_AFTER tasks=$([ "$TASKS_BEFORE" = "$TASKS_AFTER" ] && echo same || echo CHANGED)" "$OUT" FAIL
fi
# Closed phase refuses further dispatch at the REAL gate. Make the refusal
# attributable to closure: ensure an identity-bound plan handoff exists first.
if ! jq -e '.identity.contractDigest // empty' "$PHASE_DIR/handoffs/plan.handoff.json" >/dev/null 2>&1; then
  KBD_HANDOFF_IDENTITY_JSON="{\"projectId\":\"$PROJECT_ID\",\"runId\":\"$RUN_ID\",\"phaseId\":\"$PHASE\",\"planRevision\":17,\"contractDigest\":\"$DIGEST_NOW\"}" \
    KBD_PHASE_DIR="$PHASE_DIR" kbd_stage_handoff_write plan "boundary: identity-bound handoff for closed-dispatch demonstration" >/dev/null 2>&1 || true
fi
OUT="$(KBD_PHASE_DIR="$PHASE_DIR" kbd_stage_gate execute 2>&1)"; EC=$?
if [ "$EC" -ne 0 ] && printf '%s' "$OUT" | grep -q 'is CLOSED'; then
  record RC-01-closed-dispatch "gate execute on closed phase (real entry)" "refused: zero further dispatch for closed phase" "$OUT" PASS
else
  record RC-01-closed-dispatch "gate execute on closed phase" "unexpected exit=$EC" "$OUT" FAIL
fi
write_results "\"$SNAPSHOT_DIGEST\""

PASS_N="$(printf '%s\n' "${R_ST[@]}" | grep -c '^PASS$')"
printf '\n=== boundary: %s PASS of %s checks; failures(blockers)=%s ===\n' "$PASS_N" "${#R_ID[@]}" "$FAILURES"
[ "$FAILURES" -eq 0 ]
