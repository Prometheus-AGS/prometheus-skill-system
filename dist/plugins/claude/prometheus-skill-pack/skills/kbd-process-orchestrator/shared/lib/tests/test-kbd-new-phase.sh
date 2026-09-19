#!/usr/bin/env bash
# shared/lib/tests/test-kbd-new-phase.sh — smoke tests for kbd-new-phase.sh.
# Pure bash + jq. Each test runs in an isolated tmpdir.

set -uo pipefail

cd "$(dirname "$0")"
SKILL_ROOT="$(cd ../../.. && pwd -P)"
SCRIPT="$SKILL_ROOT/skills/kbd-new-phase/kbd-new-phase.sh"
[[ -x "$SCRIPT" ]] || { printf 'FAIL: %s is not executable\n' "$SCRIPT" >&2; exit 1; }

export KBD_ORCHESTRATOR_ROOT="$SKILL_ROOT"

pass() { printf 'pass: %s\n' "$*"; PASSED=$((PASSED + 1)); }
# NOTE: every case below runs as `( subshell ) && pass "..."`, so `fail` exits the
# SUBSHELL, not this script. Before this counter existed the script always exited 0 —
# a failing case merely printed FAIL and skipped its pass line, so the suite could
# never go red. FAILED is incremented by the `||` arm on each case; the exit status
# at the bottom is derived from it.
fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }
PASSED=0
FAILED=0
track() { FAILED=$((FAILED + 1)); }

mk_sandbox() {
  SANDBOX="$(mktemp -d)"
  cd "$SANDBOX"
  trap 'rm -rf "$SANDBOX"' EXIT
}

# --- Test 1: happy path, no goals ---
( mk_sandbox
  "$SCRIPT" first-phase >/dev/null 2>&1 || fail "1: exit non-zero on happy path"
  [[ -d ".kbd-orchestrator/phases/first-phase" ]]                || fail "1: phase dir missing"
  [[ -f ".kbd-orchestrator/phases/first-phase/goals.md" ]]       || fail "1: goals.md missing"
  grep -q '# Goals'                  .kbd-orchestrator/phases/first-phase/goals.md || fail "1: goals.md missing heading"
  grep -q 'TBD: enumerate goals'      .kbd-orchestrator/phases/first-phase/goals.md || fail "1: TBD stub missing"
  jq -e '
    .phase == "first-phase" and .parentPhase == null and
    .childPhases == [] and .childPointer == null and
    .completion.primaryCounter == "implementation" and
    .completion.implementation == {completed:0,total:0,status:"PENDING"} and
    .completion.certification.status == "NOT_TRACKED"
  ' \
     .kbd-orchestrator/phases/first-phase/progress.json >/dev/null || fail "1: progress.json fields wrong"
) && pass "happy path, no goals — phase dir + goals.md (TBD) + progress.json" || track
# --- Test 2: happy path with 3 goals ---
( mk_sandbox
  "$SCRIPT" foo-bar "polish dashboard" "ship dark mode" "audit a11y" >/dev/null 2>&1 || fail "2: exit non-zero"
  bullets="$(grep -c '^- ' .kbd-orchestrator/phases/foo-bar/goals.md)"
  [[ "$bullets" == "3" ]] || fail "2: expected 3 bullets, got $bullets"
  grep -q '^- polish dashboard$'  .kbd-orchestrator/phases/foo-bar/goals.md || fail "2: bullet 1 missing"
  grep -q '^- ship dark mode$'    .kbd-orchestrator/phases/foo-bar/goals.md || fail "2: bullet 2 missing"
  grep -q '^- audit a11y$'        .kbd-orchestrator/phases/foo-bar/goals.md || fail "2: bullet 3 missing"
) && pass "happy path with 3 goals — bullets in order" || track
# --- Test 3: missing name argument ---
( mk_sandbox
  out="$("$SCRIPT" 2>&1)" && fail "3: should exit non-zero"
  echo "$out" | grep -q 'usage: kbd-new-phase.sh' || fail "3: missing usage error, got: $out"
) && pass "missing name → usage error" || track
# --- Test 4: name with uppercase letters ---
( mk_sandbox
  out="$("$SCRIPT" BadName 2>&1)" && fail "4: should exit non-zero"
  echo "$out" | grep -q "must match" || fail "4: expected regex error, got: $out"
) && pass "uppercase name → regex error" || track
# --- Test 5: name containing .. ---
( mk_sandbox
  out="$("$SCRIPT" ..bad 2>&1)" && fail "5: should exit non-zero"
  echo "$out" | grep -qi 'parent traversal' || fail "5: expected traversal error, got: $out"
) && pass "name with .. → traversal error" || track
# --- Test 6: name collision ---
( mk_sandbox
  "$SCRIPT" collide >/dev/null 2>&1 || fail "6a: first create should succeed"
  out="$("$SCRIPT" collide 2>&1)" && fail "6b: second create should fail"
  echo "$out" | grep -qi 'already exists' || fail "6: expected collision error, got: $out"
  # Existing phase untouched
  jq -e '.phase == "collide"' .kbd-orchestrator/phases/collide/progress.json >/dev/null \
    || fail "6: existing phase progress.json corrupted"
) && pass "name collision → refused, existing phase untouched" || track
# --- Test 7: first-waypoint write (no current-waypoint.json initially) ---
( mk_sandbox
  "$SCRIPT" inaugural >/dev/null 2>&1 || fail "7: should succeed"
  [[ -f .kbd-orchestrator/current-waypoint.json ]] || fail "7: waypoint not created"
  jq -e '.phase == "inaugural" and .previousPhase == null' \
     .kbd-orchestrator/current-waypoint.json >/dev/null \
    || fail "7: first waypoint fields wrong"
) && pass "first-waypoint write — phase set, previousPhase null" || track
# --- Test 7b: path[] is reset to the new phase, not inherited ---
# Regression: a top-level flip used to leave the PRIOR phase's path[] in place.
# _kbd_path_from_waypoint prefers an explicit path[] over synthesis, so the stale
# value won and position.json rendered the new phase under the old phase's names.
( mk_sandbox
  "$SCRIPT" inaugural >/dev/null 2>&1 || fail "7b: should succeed"
  jq -e '.path == ["inaugural"]' .kbd-orchestrator/current-waypoint.json >/dev/null \
    || fail "7b: fresh waypoint path[] should be [inaugural], got $(jq -c .path .kbd-orchestrator/current-waypoint.json)"
) && pass "fresh waypoint — path[] is [<phase>]" || track
( mk_sandbox
  mkdir -p .kbd-orchestrator
  # A waypoint carrying a DEEP chain from a previous phase, exactly as observed
  # in the wild (phase flipped, path[] left behind at depth 2).
  jq -n '{phase: "old-phase", path: ["old-phase","some-child"], childPointer: "some-child"}' \
    > .kbd-orchestrator/current-waypoint.json
  "$SCRIPT" fresh-top >/dev/null 2>&1 || fail "7c: should succeed"
  jq -e '.path == ["fresh-top"]' .kbd-orchestrator/current-waypoint.json >/dev/null \
    || fail "7c: stale path[] not reset, got $(jq -c .path .kbd-orchestrator/current-waypoint.json)"
  jq -e '.childPointer == null and .childPhases == []' .kbd-orchestrator/current-waypoint.json >/dev/null \
    || fail "7c: child fields should be cleared alongside path[]"
) && pass "stale deep path[] reset to [<phase>] on top-level flip" || track
# --- Test 8: malformed waypoint aborts without modifying state ---
( mk_sandbox
  mkdir -p .kbd-orchestrator
  printf 'not json at all\n' > .kbd-orchestrator/current-waypoint.json
  out="$("$SCRIPT" attempted 2>&1)" && fail "8a: should exit non-zero on malformed waypoint"
  echo "$out" | grep -qi 'malformed waypoint' || fail "8b: expected malformed error, got: $out"
  [[ ! -d .kbd-orchestrator/phases/attempted ]] || fail "8c: phase dir should NOT have been created"
  # Waypoint left as-is (malformed)
  grep -q 'not json at all' .kbd-orchestrator/current-waypoint.json || fail "8d: waypoint should be untouched"
) && pass "malformed waypoint → abort, no on-disk state changed" || track
# --- Test 9: absent project.json → bootstrap minimal identity and continue ---
( mk_sandbox
  out="$("$SCRIPT" no-project-json 2>&1)" || fail "9: should exit zero"
  echo "$out" | grep -qi 'project.json missing' || fail "9: expected project.json warning, got: $out"
  [[ -d .kbd-orchestrator/phases/no-project-json ]] || fail "9: phase still must be created"
  [[ -f .kbd-orchestrator/project.json ]] || fail "9: project.json should be bootstrapped"
  root_pwd="$(pwd -P)"
  sandbox_name="$(basename "$root_pwd")"
  jq -e \
    --arg root "$root_pwd" \
    --arg sandbox_name "$sandbox_name" \
    '.name == $sandbox_name and
     .activePhase == "no-project-json" and
     .focus_project_path == $root and
     .bootstrappedBy == "kbd-new-phase"' \
    .kbd-orchestrator/project.json >/dev/null || fail "9: bootstrapped project.json fields wrong"
) && pass "absent project.json → bootstrap minimal identity and phase" || track
# --- Test 10: hook fire produces a JSONL entry ---
( mk_sandbox
  "$SCRIPT" hooked >/dev/null 2>&1 || fail "10a: should succeed"
  log=".kbd-orchestrator/phases/hooked/hooks.log.jsonl"
  [[ -f "$log" ]] || fail "10b: hooks.log.jsonl not created at $log"
  # At least one phase:before entry for our phase
  jq -e 'select(.kind == "phase" and .edge == "before" and .name == "hooked" and .index == 1 and .total == 1)' "$log" >/dev/null \
    || fail "10c: no matching phase:before entry in log"
) && pass "hook fire — phase:before entry in hooks.log.jsonl" || track
# --- Test 11: hook subsystem absent → warn, phase still created ---
( mk_sandbox
  out="$(KBD_ORCHESTRATOR_ROOT=/nonexistent/path "$SCRIPT" hookless 2>&1)" || fail "11a: should exit zero"
  echo "$out" | grep -qi 'hooks subsystem unavailable' || fail "11b: expected hooks-unavail warning, got: $out"
  [[ -d .kbd-orchestrator/phases/hookless ]] || fail "11c: phase still must be created"
) && pass "hook subsystem absent → warn, phase still created" || track
# --- Test 12: unknown waypoint keys preserved ---
( mk_sandbox
  mkdir -p .kbd-orchestrator
  jq -n '{phase: "old", myTool_metadata: {custom: "value"}, somethingElse: 42}' \
    > .kbd-orchestrator/current-waypoint.json
  "$SCRIPT" replacing >/dev/null 2>&1 || fail "12a: should succeed"
  jq -e '.myTool_metadata.custom == "value" and .somethingElse == 42' \
     .kbd-orchestrator/current-waypoint.json >/dev/null \
    || fail "12: unknown waypoint keys not preserved"
) && pass "unknown waypoint keys preserved through rewrite" || track
# --- Test 13: terminal canonical run rolls once before phase creation ---
( mk_sandbox
  mkdir -p mock-root/shared/lib mock-bin
  printf '%s\n' 'kbd_runtime_authoritative() { return 0; }' \
    > mock-root/shared/lib/runtime-authority.sh
  printf '%s\n' \
    '#!/usr/bin/env bash' \
    'printf "%s\n" "$*" >> "$PROMETHEUS_LOG"' \
    'case " $* " in' \
    '  *" status --json "*) printf "{\"lifecycle\":\"%s\",\"runId\":\"run-a\"}\n" "$MOCK_LIFECYCLE" ;;' \
    'esac' \
    > mock-bin/prometheus
  chmod +x mock-bin/prometheus
  export PROMETHEUS_LOG="$SANDBOX/prometheus.log"
  export MOCK_LIFECYCLE="cancelled"
  PATH="$SANDBOX/mock-bin:$PATH" \
    KBD_ORCHESTRATOR_ROOT="$SANDBOX/mock-root" \
    "$SCRIPT" successor-phase >/dev/null 2>&1 || fail "13a: terminal rollover should succeed"
  [[ "$(wc -l < "$PROMETHEUS_LOG" | tr -d ' ')" == "5" ]] \
    || fail "13b: expected status, rollover, create, activate, transition commands"
  sed -n '1p' "$PROMETHEUS_LOG" | grep -q 'status --json' \
    || fail "13c: status must be read first"
  sed -n '2p' "$PROMETHEUS_LOG" | grep -Eq \
    'run start --run-id successor-phase-[0-9]{8}T[0-9]{6}Z .*--exact-next-work /kbd-new-phase successor-phase' \
    || fail "13d: successor run must commit before phase creation"
  sed -n '3p' "$PROMETHEUS_LOG" | grep -q 'phase create' \
    || fail "13e: phase create must follow rollover"
  sed -n '4p' "$PROMETHEUS_LOG" | grep -q 'phase activate' \
    || fail "13f: phase activation missing"
  sed -n '5p' "$PROMETHEUS_LOG" | grep -q 'phase transition' \
    || fail "13g: phase start transition missing"
) && pass "terminal runtime → one successor run before phase create + activate" || track
# --- Test 14: non-terminal canonical run creates phase without rollover ---
( mk_sandbox
  mkdir -p mock-root/shared/lib mock-bin
  printf '%s\n' 'kbd_runtime_authoritative() { return 0; }' \
    > mock-root/shared/lib/runtime-authority.sh
  printf '%s\n' \
    '#!/usr/bin/env bash' \
    'printf "%s\n" "$*" >> "$PROMETHEUS_LOG"' \
    'case " $* " in' \
    '  *" status --json "*) printf "{\"lifecycle\":\"%s\",\"runId\":\"run-a\"}\n" "$MOCK_LIFECYCLE" ;;' \
    'esac' \
    > mock-bin/prometheus
  chmod +x mock-bin/prometheus
  export PROMETHEUS_LOG="$SANDBOX/prometheus.log"
  export MOCK_LIFECYCLE="ready"
  PATH="$SANDBOX/mock-bin:$PATH" \
    KBD_ORCHESTRATOR_ROOT="$SANDBOX/mock-root" \
    "$SCRIPT" ordinary-phase >/dev/null 2>&1 || fail "14a: non-terminal phase create should succeed"
  grep -q 'status --json' "$PROMETHEUS_LOG" || fail "14b: canonical status missing"
  ! grep -q 'run start' "$PROMETHEUS_LOG" || fail "14c: non-terminal run must not roll over"
  grep -q 'phase create' "$PROMETHEUS_LOG" || fail "14d: phase create missing"
  grep -q 'phase activate' "$PROMETHEUS_LOG" || fail "14e: phase activate missing"
) && pass "non-terminal runtime → no rollover" || track
# --- Test 15: mixed-history project metadata converges to activePhase ---
( mk_sandbox
  mkdir -p .kbd-orchestrator
  jq -n '{name:"mixed-history", active_phase:"old-phase", activePhase:"newer-phase", custom:true}' \
    > .kbd-orchestrator/project.json
  "$SCRIPT" canonical-phase >/dev/null 2>&1 || fail "15a: mixed-history update should succeed"
  jq -e '
    .activePhase == "canonical-phase" and
    has("active_phase") == false and
    .custom == true
  ' .kbd-orchestrator/project.json >/dev/null \
    || fail "15b: writer did not converge to activePhase-only metadata"
) && pass "mixed-history project metadata → activePhase only" || track
printf '\n'
if [[ "$FAILED" -gt 0 ]]; then
  printf 'kbd-new-phase smoke tests: %d passed, %d FAILED\n' "$PASSED" "$FAILED" >&2
  exit 1
fi
printf 'all kbd-new-phase smoke tests passed (%d/%d)\n' "$PASSED" "$((PASSED + FAILED))"
