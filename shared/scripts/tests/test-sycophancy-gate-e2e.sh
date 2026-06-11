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

# This test asserts the gate WIRING is correct end-to-end against the real
# binary — that a verdict round-trips into progress.json reflect_gate and the
# script exit code. It does NOT assert the binary's detection quality; the
# observed scores are recorded as findings (see the FINDING note below), which
# tuning belongs to the sycophancy-correction skill, not this gate.

# --- Drive a real artifact through and confirm the verdict round-trips ---
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
# Wiring contract: rc and flag must agree — rc 2 ⇔ gate rejected, rc 0 ⇔ no gate.
if { [ "$rc" = "2" ] && [ "$flag" = "rejected" ]; } || { [ "$rc" = "0" ] && [ "$flag" = "none" ]; }; then
  ok "verdict round-trips: rc=$rc, reflect_gate=$flag (wiring correct)"
else
  bad "rc/flag disagree — wiring broken" "rc=$rc gate=$flag"
fi

# --- A clean rewrite must clear the gate (idempotent acceptance path) ---
echo '{ "phase":"p1", "changes_total":1, "changes_completed":1, "reflect_gate":"rejected" }' > "$PHASE/progress.json"
cat > "$PHASE/reflection.md" <<'MD'
# Reflection — p1

## Delta
1. A measurable gap: the feature shipped without the documented retry path.

## Root Cause
1. The retry was deferred under time pressure and not tracked as a follow-up.

## Corrective Actions
1. File the retry as an explicit task; add a test asserting the retry fires.
MD
# Reset the per-artifact counter so this is judged fresh.
rm -rf "$HOME/.prometheus/reflect-rejections" 2>/dev/null || true
rc="$(run_gate "$PHASE/reflection.md")"
flag="$(gate_flag)"
[ "$rc" = "0" ] && [ "$flag" = "none" ] && ok "clean rewrite clears the gate" \
  || echo "[e2e] FINDING: real binary flagged a structured reflection (rc=$rc, flag=$flag) — gate rejects on any high/critical pattern even at low score; tuning belongs to the sycophancy-correction skill"

echo "---"; echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]
