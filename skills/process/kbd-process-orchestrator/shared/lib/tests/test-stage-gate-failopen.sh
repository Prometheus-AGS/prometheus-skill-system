#!/usr/bin/env bash
# Fail-open regression tests for kbd_stage_gate.
#
# WHY
# ---
# On 2026-08-12 an agent authored a full KBD phase (assess -> analyze -> spec ->
# plan) while canonical state still pointed at a DIFFERENT, already-closed
# phase. The gate never objected, because every one of its failure branches
# `return 0`. A second harness then read the stale position file and stalled.
#
# Each test below drives ONE fail-open path. Before the fix they exit 0 (the
# defect). After the fix they must exit non-zero. A gate that has never been
# observed to refuse is indistinguishable from one that always passes, so these
# run in both directions and are asserted, not eyeballed.
#
# Usage:  bash test-stage-gate-failopen.sh            # assert FIXED behaviour
#         EXPECT=failopen bash test-stage-gate-failopen.sh   # document CURRENT

set -uo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LIB="$HERE/.."
EXPECT="${EXPECT:-fixed}"

pass=0; fail=0
scratch="$(mktemp -d)"
trap 'rm -rf "$scratch"' EXIT

# shellcheck disable=SC1091
. "$LIB/stage-gate.sh"

_mk_phase() { # <root> <waypoint-phase> <active-phase> [handoffs?]
  local root="$1" wp_phase="$2" active="$3" with_handoffs="${4:-no}"
  mkdir -p "$root/.kbd-orchestrator/phases/$wp_phase"
  [ "$with_handoffs" = "yes" ] && mkdir -p "$root/.kbd-orchestrator/phases/$wp_phase/handoffs"
  cat > "$root/.kbd-orchestrator/current-waypoint.json" <<EOF
{"phase":"$wp_phase","activePhaseId":"$active","status":"running"}
EOF
}

_assert() { # <name> <actual-exit> <what-fixed-should-be>
  local name="$1" got="$2" want_fixed="$3" want
  if [ "$EXPECT" = "failopen" ]; then want=0; else want="$want_fixed"; fi
  if [ "$got" -eq "$want" ]; then
    printf '  PASS  %-52s exit=%s\n' "$name" "$got"; pass=$((pass+1))
  else
    printf '  FAIL  %-52s exit=%s want=%s\n' "$name" "$got" "$want"; fail=$((fail+1))
  fi
}

echo "stage-gate fail-open suite (mode: $EXPECT)"

# 1. Canonical phase differs from the phase being worked. THE bug of 2026-08-12.
#
#    The prior handoff MUST be present here. Without it the gate blocks on
#    "assess handoff missing" — a different rule — and the test would report a
#    pass it did not earn. Isolate one variable: the ONLY defect under test is
#    the unchecked activePhaseId.
r="$scratch/t1"; _mk_phase "$r" phase-X phase-DIFFERENT yes
printf '{"stage":"assess"}' > "$r/.kbd-orchestrator/phases/phase-X/handoffs/assess.handoff.json"
( cd "$r" && kbd_stage_gate analyze >/dev/null 2>&1 ); _assert "mismatched activePhaseId refused" "$?" 2

# 2. `assess` is stage index 0 and returns 0 before any check runs. Opening a
#    phase is exactly when canonical state must exist, so exemption is backwards.
r="$scratch/t2"; _mk_phase "$r" phase-X phase-DIFFERENT yes
( cd "$r" && kbd_stage_gate assess >/dev/null 2>&1 ); _assert "assess not exempt from phase match" "$?" 2

# 3. Missing handoffs/ disables the gate as "legacy" — but every NEW phase also
#    lacks handoffs/, so this exempts precisely the phases most at risk.
r="$scratch/t3"; _mk_phase "$r" phase-X phase-X no
( cd "$r" && kbd_stage_gate plan >/dev/null 2>&1 ); _assert "missing handoffs/ not treated as legacy" "$?" 2

# 4. Unresolvable phase dir warns and passes.
r="$scratch/t4"; mkdir -p "$r"
( cd "$r" && kbd_stage_gate plan >/dev/null 2>&1 ); _assert "unresolvable phase refused" "$?" 2

# 5. GREEN: aligned state with a prior handoff must still PASS. A gate that
#    refuses everything is as useless as one that refuses nothing.
r="$scratch/t5"; _mk_phase "$r" phase-X phase-X yes
printf '{"stage":"assess"}' > "$r/.kbd-orchestrator/phases/phase-X/handoffs/assess.handoff.json"
( cd "$r" && kbd_stage_gate analyze >/dev/null 2>&1 ); _assert "aligned state still passes" "$?" 0

echo "  ---- pass=$pass fail=$fail"
[ "$fail" -eq 0 ]
