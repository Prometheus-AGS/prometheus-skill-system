#!/usr/bin/env bash
# shared/lib/tests/test-hooks.sh — smoke tests for hooks.sh.
# Pure bash + jq. Exits non-zero on first failure.

set -uo pipefail

cd "$(dirname "$0")"
SKILL_ROOT="$(cd ../../.. && pwd -P)"
export KBD_ORCHESTRATOR_ROOT="$SKILL_ROOT"

# shellcheck source=/dev/null
. "$SKILL_ROOT/shared/lib/waypoint.sh"
# shellcheck source=/dev/null
. "$SKILL_ROOT/shared/lib/hooks.sh"

pass() { printf 'pass: %s\n' "$*"; }
fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }

# Isolate each test in a tmp working dir so .kbd-orchestrator/ writes are sandboxed.
SANDBOX="$(mktemp -d)"
trap 'rm -rf "$SANDBOX"' EXIT

# Test 1: default reporter on *:before
cd "$SANDBOX"
out="$(kbd_hooks_fire phase before alpha 1 1 2>&1 >/dev/null)"
echo "$out" | grep -q '^starting phase alpha \[1/1\]$' \
  || fail "test 1 — expected default 'starting phase alpha [1/1]' line, got: $out"
pass "default reporter emits 'starting' on *:before"

# Test 2: default reporter on *:after
out="$(kbd_hooks_fire phase after alpha 1 1 2>&1 >/dev/null)"
echo "$out" | grep -q '^ending phase alpha \[1/1\]$' \
  || fail "test 2 — expected 'ending phase alpha [1/1]', got: $out"
pass "default reporter emits 'ending' on *:after"

# Test 3: project-layer override suppresses default reporter
mkdir -p .kbd-orchestrator
cat > .kbd-orchestrator/hooks-config.json <<'CONFIG'
{
  "hooks": [
    {
      "id": "override-progress",
      "event": "*:*",
      "mode": "override",
      "action": {
        "type": "command",
        "command": "printf 'OVERRIDE %s %s\\n' \"$KBD_HOOK_EDGE\" \"$KBD_HOOK_NAME\" >&2"
      }
    }
  ]
}
CONFIG
out="$(kbd_hooks_fire plan before beta 1 1 2>&1 >/dev/null)"
echo "$out" | grep -q '^OVERRIDE before beta$' \
  || fail "test 3a — expected override output, got: $out"
echo "$out" | grep -q '^starting ' \
  && fail "test 3b — default reporter should have been suppressed, got: $out"
pass "project-layer override suppresses default reporter"

# Test 4: per-task with on_change_complete sentinel
cat > .kbd-orchestrator/hooks-config.json <<'CONFIG'
{
  "hooks": [
    {
      "id": "legacy-change-listener",
      "event": "on_change_complete",
      "action": {
        "type": "command",
        "command": "printf 'CHANGE_DONE %s\\n' \"$KBD_HOOK_NAME\" >&2"
      }
    }
  ]
}
CONFIG
# Mid-task (not last): on_change_complete should NOT fire
out="$(kbd_hooks_fire task after t1 1 3 2>&1 >/dev/null)"
echo "$out" | grep -q '^CHANGE_DONE' \
  && fail "test 4a — on_change_complete should not fire mid-task, got: $out"
pass "on_change_complete does not fire mid-task"

# Last task: on_change_complete SHOULD fire
out="$(kbd_hooks_fire task after t3 3 3 2>&1 >/dev/null)"
echo "$out" | grep -q '^CHANGE_DONE t3$' \
  || fail "test 4b — on_change_complete should fire on final task, got: $out"
pass "on_change_complete fires on final task:after (index==total)"

# Test 5: alias - on_phase_complete fires on canonical phase:after
cat > .kbd-orchestrator/hooks-config.json <<'CONFIG'
{
  "hooks": [
    {
      "id": "legacy-phase-listener",
      "event": "on_phase_complete",
      "action": {
        "type": "command",
        "command": "printf 'PHASE_DONE %s\\n' \"$KBD_HOOK_NAME\" >&2"
      }
    }
  ]
}
CONFIG
out="$(kbd_hooks_fire phase after gamma 1 1 2>&1 >/dev/null)"
echo "$out" | grep -q '^PHASE_DONE gamma$' \
  || fail "test 5 — on_phase_complete alias should fire on phase:after, got: $out"
pass "on_phase_complete alias fires on phase:after"

# Test 6: legacy <step>:begin / :end aliases
cat > .kbd-orchestrator/hooks-config.json <<'CONFIG'
{
  "hooks": [
    {
      "id": "legacy-begin-end",
      "event": "assess:begin",
      "action": {
        "type": "command",
        "command": "printf 'ASSESS_BEGIN\\n' >&2"
      }
    }
  ]
}
CONFIG
out="$(kbd_hooks_fire assess before delta 1 1 2>&1 >/dev/null)"
echo "$out" | grep -q '^ASSESS_BEGIN$' \
  || fail "test 6 — assess:begin should map to assess:before, got: $out"
pass "<step>:begin alias maps to <step>:before"

# Test 7: JSONL log written with full key set
rm -f .kbd-orchestrator/hooks.log.jsonl
kbd_hooks_fire phase before epsilon 1 1 >/dev/null 2>&1
[[ -f .kbd-orchestrator/hooks.log.jsonl ]] \
  || fail "test 7a — hooks.log.jsonl should be created at fallback path"
last="$(tail -n 1 .kbd-orchestrator/hooks.log.jsonl)"
for k in ts kind edge name index total phasePath sourceTool hookId layer mode status; do
  echo "$last" | jq -e ".$k" >/dev/null \
    || fail "test 7b — log entry missing key: $k (line: $last)"
done
pass "JSONL log entry contains all required keys"

# Test 8: invalid event in config — warning + skip, no abort
cat > .kbd-orchestrator/hooks-config.json <<'CONFIG'
{
  "hooks": [
    {
      "id": "bad-event",
      "event": "garbage",
      "action": { "type": "command", "command": "true" }
    },
    {
      "id": "good-event",
      "event": "phase:before",
      "action": { "type": "command", "command": "printf 'GOOD\\n' >&2" }
    }
  ]
}
CONFIG
# Garbage event shouldn't match phase:before — only "good-event" should fire.
# (The dispatcher's matcher silently drops non-matching entries; this verifies
# that an unrecognised event doesn't crash dispatch.)
out="$(kbd_hooks_fire phase before zeta 1 1 2>&1 >/dev/null)"
echo "$out" | grep -q '^GOOD$' \
  || fail "test 8 — good-event should still fire alongside garbage event, got: $out"
pass "invalid event does not abort dispatch"

# Test 9: override conflict — two project-layer overrides on same event
cat > .kbd-orchestrator/hooks-config.json <<'CONFIG'
{
  "hooks": [
    {
      "id": "ov-1",
      "event": "phase:before",
      "mode": "override",
      "action": { "type": "command", "command": "printf 'OV1\\n' >&2" }
    },
    {
      "id": "ov-2",
      "event": "phase:before",
      "mode": "override",
      "action": { "type": "command", "command": "printf 'OV2\\n' >&2" }
    }
  ]
}
CONFIG
out="$(kbd_hooks_fire phase before eta 1 1 2>&1 >/dev/null)"
echo "$out" | grep -q 'override conflict' \
  || fail "test 9 — expected 'override conflict' warning, got: $out"
pass "multiple overrides on same event emit conflict warning"

cd /
printf '\nall hook smoke tests passed\n'
