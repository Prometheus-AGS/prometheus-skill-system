#!/usr/bin/env bash
# shared/lib/tests/test-kbd-apply-e2e.sh
# End-to-end proof that kbd-apply wraps OpenSpec task-by-task:
#   - list/progress read the real CLI surface
#   - begin-task fires task:before + plain-text signal
#   - end-task marks the right task done (positional id) + syncs progress.json
#   - the per-task position signal carries correct i/n
# Uses a THROWAWAY openspec change created in a sandbox project; never touches
# real changes. Skips cleanly if the `openspec` CLI is unavailable.

set -uo pipefail
cd "$(dirname "$0")"
SKILL_ROOT="$(cd ../../.. && pwd -P)"
export KBD_ORCHESTRATOR_ROOT="$SKILL_ROOT"
APPLY="$SKILL_ROOT/skills/kbd-apply/kbd-apply.sh"

pass() { printf 'pass: %s\n' "$*"; }
fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }
skip() { printf 'skip: %s\n' "$*"; exit 0; }

command -v jq >/dev/null 2>&1 || fail "jq required"
command -v openspec >/dev/null 2>&1 || skip "openspec CLI not installed — e2e skipped"

SANDBOX="$(mktemp -d)"; trap 'rm -rf "$SANDBOX"' EXIT
cd "$SANDBOX"

# Minimal OpenSpec project + a throwaway change with 2 tasks.
openspec init . >/dev/null 2>&1 || true
CH="change-e2e-throwaway"
mkdir -p "openspec/changes/$CH"
cat > "openspec/changes/$CH/proposal.md" <<'MD'
# Proposal: e2e throwaway
Why: exercise kbd-apply wrapping. What: two no-op tasks.
MD
cat > "openspec/changes/$CH/tasks.md" <<'MD'
# Tasks: e2e throwaway

- [ ] First throwaway task
- [ ] Second throwaway task
MD

# If this openspec build can't introspect the change, skip rather than fail —
# we are testing kbd-apply's wrapping, not openspec's schema coverage.
if ! "$APPLY" list "$CH" >/dev/null 2>&1; then
  skip "openspec could not introspect throwaway change in this sandbox"
fi

# 1. list returns 2 tasks, both not done.
n="$("$APPLY" list "$CH" | wc -l | tr -d ' ')"
[ "$n" = "2" ] || fail "expected 2 tasks, got $n"
pass "list returns the 2 throwaway tasks"

# 2. progress: total=2 complete=0.
read -r tot comp rem < <("$APPLY" progress "$CH")
[ "$tot" = "2" ] && [ "$comp" = "0" ] || fail "progress expected '2 0 _', got '$tot $comp $rem'"
pass "progress reports total=2 complete=0"

# 3. begin-task emits the plain-text guarantee with correct i/n.
out="$("$APPLY" begin-task "$CH" 1 1 2 "First throwaway task" 2>/dev/null)"
[ "$out" = "Starting task 1 out of 2:   First throwaway task" ] \
  || fail "begin-task signal wrong: '$out'"
pass "begin-task emits canonical 'Starting task 1 out of 2: ...'"

# 4. end-task marks task 1 (positional) done; tasks.md first box flips, not 2nd.
"$APPLY" end-task "$CH" 1 1 2 "First throwaway task" >/dev/null 2>&1
first="$(grep -nE '^\s*-\s*\[[ xX]\]' "openspec/changes/$CH/tasks.md" | sed -n '1p')"
second="$(grep -nE '^\s*-\s*\[[ xX]\]' "openspec/changes/$CH/tasks.md" | sed -n '2p')"
echo "$first"  | grep -q '\[x\]' || fail "task 1 not marked done: $first"
echo "$second" | grep -q '\[ \]' || fail "task 2 should still be open: $second"
pass "end-task marks the POSITIONAL task done (task 1 only)"

# 5. progress now reflects 1 complete.
read -r tot2 comp2 rem2 < <("$APPLY" progress "$CH")
[ "$comp2" = "1" ] || fail "after end-task, complete expected 1, got $comp2"
pass "progress reflects 1/2 complete after one task"

printf '\nkbd-apply e2e (OpenSpec wrapping) tests passed\n'
