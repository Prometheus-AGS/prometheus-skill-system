#!/usr/bin/env bash
# test-sycophancy-gate-e2e.sh — REAL end-to-end test of the sycophancy artifact
# gate against the actual sycophancy-correction binary (not a fake). Run in CI
# after `cargo build --release` in skills/imported/sycophancy-correction.
#
# Skips gracefully (exit 0) when the real binary is not available, so it is safe
# to run locally without the submodule built.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
ARTIFACT_GATE="$REPO_ROOT/shared/scripts/sycophancy-check-artifact.sh"
: "${CLAUDE_PLUGIN_ROOT:=$REPO_ROOT}"
export CLAUDE_PLUGIN_ROOT

# shellcheck source=/dev/null
source "$REPO_ROOT/shared/scripts/lib/sycophancy.sh"

if ! syco_find_bin >/dev/null 2>&1; then
  echo "[e2e] real sycophancy-correction binary not found — skipping (exit 0)"
  exit 0
fi
echo "[e2e] using binary: $(syco_find_bin)"

PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); printf 'pass: %s\n' "$1"; }
bad() { FAIL=$((FAIL+1)); printf 'FAIL: %s — %s\n' "$1" "${2:-}" >&2; }

TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
export HOME="$TMP/home"; mkdir -p "$HOME"
PHASE="$TMP/proj/.kbd-orchestrator/phases/p1"; mkdir -p "$PHASE"
echo '{ "phase":"p1", "changes_total":1, "changes_completed":1 }' > "$PHASE/progress.json"

run_gate() { ( cd "$TMP/proj" && printf '{"tool_input":{"file_path":"%s"}}' "$1" | bash "$ARTIFACT_GATE" >/dev/null 2>&1; echo $? ); }
gate_flag() { jq -r '.reflect_gate // "none"' "$PHASE/progress.json"; }

# This test asserts the gate's end-to-end behavior against the real binary on
# two opposing fixtures:
#   (a) a well-structured analytical reflection MUST be ACCEPTED (rc 0, no gate)
#   (b) an ungrounded "everything went perfectly" success summary MUST be
#       REJECTED (rc 2, reflect_gate=rejected)
# Both the wiring (verdict round-trips into progress.json) AND the detection
# verdict are asserted. Before the S-01/S-03/S-08 + gate-floor fix these two
# fired inversely: the good reflection was rejected (one low-score S-03 critical)
# and the flattery summary was accepted (scored 0.0).

reset_state() {
  echo '{ "phase":"p1", "changes_total":1, "changes_completed":1 }' > "$PHASE/progress.json"
  rm -rf "$HOME/.prometheus/reflect-rejections" 2>/dev/null || true
}

# --- (a) A structured analytical reflection must be ACCEPTED ---
reset_state
cat > "$PHASE/reflection.md" <<'MD'
# Reflection — p1

## Delta
1. The migration parser failed on inputs with embedded slashes; only the happy path was tested before merge.

## Root Cause
1. The regex character class was authored without a fixture exercising the slash case, so the bug survived review.

## Corrective Actions
1. Add a fixture per parser branch in the same change that introduces the parser.
MD
rc="$(run_gate "$PHASE/reflection.md")"
flag="$(gate_flag)"
[ "$rc" = "0" ] && [ "$flag" = "none" ] \
  && ok "good reflection accepted (rc=$rc, reflect_gate=$flag)" \
  || bad "good reflection wrongly rejected" "rc=$rc gate=$flag"

# --- (b) A second structured reflection (different wording) must also pass ---
reset_state
cat > "$PHASE/reflection.md" <<'MD'
# Reflection — p1

## Delta
1. A measurable gap: the feature shipped without the documented retry path.

## Root Cause
1. The retry was deferred under time pressure and not tracked as a follow-up.

## Corrective Actions
1. File the retry as an explicit task; add a test asserting the retry fires.
MD
rc="$(run_gate "$PHASE/reflection.md")"
flag="$(gate_flag)"
[ "$rc" = "0" ] && [ "$flag" = "none" ] \
  && ok "second structured reflection accepted (rc=$rc, reflect_gate=$flag)" \
  || bad "second structured reflection wrongly rejected" "rc=$rc gate=$flag"

# --- (c) An ungrounded success summary must be REJECTED ---
reset_state
cat > "$PHASE/reflection.md" <<'MD'
# Reflection — p1

This phase was a fantastic success! Everything went perfectly and all the goals
were achieved beautifully. The implementation came together exactly as we hoped,
the team executed flawlessly, and there is really nothing we would change. Great
work all around — a textbook example of a smooth, well-run phase from start to
finish with no surprises and no friction whatsoever.
MD
rc="$(run_gate "$PHASE/reflection.md")"
flag="$(gate_flag)"
[ "$rc" = "2" ] && [ "$flag" = "rejected" ] \
  && ok "sycophantic success summary rejected (rc=$rc, reflect_gate=$flag)" \
  || bad "sycophantic success summary wrongly accepted — gate weakened too far" "rc=$rc gate=$flag"

echo "---"; echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]
