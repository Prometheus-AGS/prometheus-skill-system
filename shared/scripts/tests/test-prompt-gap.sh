#!/bin/bash
# test-prompt-gap.sh - integration test for knowledge-gap detection in the prompt
# branch of karpathy-hook-dispatch.sh (change C2, design §4). Runs the real hook
# against the REAL pk (>= 1.10.0) in a scratch HOME and scratch git project.
# Exit 0 = pass, 1 = fail, 2 = BLOCKED (a prerequisite is absent).
# bash 3.2 compatible.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DISPATCH="$HERE/../karpathy-hook-dispatch.sh"
RESOLVE="$HERE/../lib/prompt_gap.py"

command -v jq >/dev/null 2>&1 || { echo "BLOCKED: jq not on PATH" >&2; exit 2; }
command -v python3 >/dev/null 2>&1 || { echo "BLOCKED: python3 not on PATH" >&2; exit 2; }
command -v pk >/dev/null 2>&1 || { echo "BLOCKED: pk not on PATH" >&2; exit 2; }
pk --version 2>/dev/null | grep -Eq " 1\.(1[0-9]|[2-9][0-9])\." \
  || { echo "BLOCKED: pk >= 1.10.0 required" >&2; exit 2; }

SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT
export HOME="$SCRATCH/home"
export CODEX_HOME="$SCRATCH/codex"
export PROMETHEUS_PLUGIN_ROOT="$SCRATCH/plugin"
mkdir -p "$HOME" "$CODEX_HOME" "$PROMETHEUS_PLUGIN_ROOT"
git config --global user.email "fixture@example.invalid"
git config --global user.name "Fixture"
unset PROMETHEUS_PROJECT_ID PROMETHEUS_USER_ID PROMETHEUS_HARNESS PK_KB_DIR || true
export PROMETHEUS_PROJECT_ID_SKIP_RUNTIME=1
export RUST_LOG=error

PROJECT="$SCRATCH/project"
mkdir -p "$PROJECT"
git -C "$PROJECT" init -q
printf '# Scratch\n\nNothing relevant lives here.\n' > "$PROJECT/CLAUDE.md"
cd "$PROJECT"
GAPS="$HOME/.prometheus/knowledge-gaps/gaps.jsonl"

pass=0
fail() { echo "FAIL: $*" >&2; exit 1; }
ok() { pass=$((pass + 1)); echo "ok - $*"; }

payload() { # prompt session [agent_type]
  jq -cn --arg p "$1" --arg s "$2" --arg a "${3:-}" --arg c "$PROJECT" \
    '{hook_event_name:"UserPromptSubmit",prompt:$p,session_id:$s,cwd:$c} + (if $a == "" then {} else {agent_type:$a} end)'
}
hook() { payload "$@" | /bin/bash "$DISPATCH" UserPromptSubmit claude-code 2>/dev/null; }
gap_count() { [ -f "$GAPS" ] && grep -c '"status":"open"' "$GAPS" || echo 0; }

PROBLEM="why does the zorblax compiler fail with error E0432 on quuxify imports?"

# 1. empty KB: first problem prompt emits exactly one gap line, repeat emits none
out="$(hook "$PROBLEM" sess-1)"
[ "$(printf '%s\n' "$out" | grep -c '^\[prometheus-gap\] ')" = "1" ] || fail "expected one gap line, got: $out"
printf '%s' "$out" | grep -q 'consider /learn-goal .* or /feynman-loop' || fail "gap line shape: $out"
[ "$(gap_count)" = "1" ] || fail "gaps.jsonl should hold 1 open record"
out2="$(hook "$PROBLEM" sess-1)"
[ -z "$out2" ] || fail "repeat in the same session must emit nothing, got: $out2"
[ "$(gap_count)" = "1" ] || fail "repeat must not append to gaps.jsonl"
ok "problem prompt on an empty KB emits one gap line; repeat emits none"

# 1b. new session reports the same topic again (once per session per topic)
out3="$(hook "$PROBLEM" sess-2)"
printf '%s' "$out3" | grep -q '^\[prometheus-gap\] ' || fail "a new session should report again: $out3"
ok "once per session, not once per machine"

# 2. a prompt that is not problem-shaped never gaps
out4="$(hook "zorblax quuxify frobnicate widgets" sess-1)"
[ -z "$out4" ] || fail "non-problem prompt emitted: $out4"
ok "non-problem prompt emits nothing"

# 3. covering document in the KB + snapshot -> no gap, output == pk context --format hook
mkdir -p "$PROJECT/.prometheus/knowledge/wiki"
cat > "$PROJECT/.prometheus/knowledge/wiki/zorblax-quuxify.md" <<'DOC'
---
type: Lesson
title: Zorblax compiler E0432 quuxify imports
tags: [zorblax]
---

The zorblax compiler fails with error E0432 when quuxify imports are unresolved.
Fix the zorblax quuxify import path and rebuild.
DOC
pk snapshot >/dev/null 2>&1 || fail "pk snapshot failed"
PROMPT2="why does the zorblax compiler fail with error E0432 on quuxify imports? help"
out5="$(hook "$PROMPT2" sess-3)"
printf '%s' "$out5" | grep -q '^\[prometheus-gap\]' && fail "gap emitted despite a covering document: $out5"
expected="$(pk context "$PROMPT2" --scope project --scope shared --scope global \
  --limit 8 --max-candidates 128 --max-bytes 6000 --format hook 2>/dev/null)"
[ -n "$expected" ] || fail "pk returned no context for the covering document"
[ "$out5" = "$expected" ] || fail "hook output differs from pk context --format hook"
ok "covering document: no gap line, output equals pk context --format hook"

# 4. gap inside a subagent carries the team role
mkdir -p "$PROJECT/.agent-team/gap-team"
cat > "$PROJECT/.agent-team/gap-team/team.json" <<'JSON'
{"schemaVersion":1,"id":"gap-team","outcome":"fixture","scope":"project","harness":"claude",
 "roles":[{"id":"backend-dev","description":"api","prompt":"p","skills":[],"owns":["src/api/**"],"inputs":[],"outputs":[],"dependsOn":[]}]}
JSON
SUBPROMPT="how should the wibblesnort daemon handle flarbnitz retries after failure?"
hook "$SUBPROMPT" sess-4 "prometheus-skill-pack:backend-dev" >/dev/null
rec="$(grep 'wibblesnort' "$GAPS" | tail -n 1)"
[ -n "$rec" ] || fail "no gap record for the subagent prompt"
[ "$(printf '%s' "$rec" | jq -r .roleId)" = "backend-dev" ] || fail "roleId missing: $rec"
[ "$(printf '%s' "$rec" | jq -r .teamId)" = "gap-team" ] || fail "teamId missing: $rec"
[ "$(printf '%s' "$rec" | jq -r .sessionId)" = "sess-4" ] || fail "sessionId missing: $rec"
printf '%s' "$rec" | jq -e '.projectId and .promptHash and .topic and .ts' >/dev/null || fail "record fields: $rec"
first="$(head -n 1 "$GAPS")"
printf '%s' "$first" | jq -e 'has("roleId") | not' >/dev/null || fail "main-agent gap must not carry roleId: $first"
ok "subagent gap carries teamId and roleId; main-agent gap does not"

# 5. resolve: the learn-skill closing step marks the gap resolved
resolved="$(python3 "$RESOLVE" resolve --topic "wibblesnort daemon flarbnitz retries" --by learn-goal --scope project)"
[ -n "$resolved" ] || fail "resolve matched nothing"
last="$(tail -n 1 "$GAPS")"
[ "$(printf '%s' "$last" | jq -r .status)" = "resolved" ] || fail "resolved record: $last"
[ "$(printf '%s' "$last" | jq -r .resolvedBy)" = "learn-goal" ] || fail "resolvedBy: $last"
again="$(python3 "$RESOLVE" resolve --topic "wibblesnort daemon flarbnitz retries")"
[ -z "$again" ] || fail "an already-resolved gap must not resolve twice: $again"
ok "resolve appends a resolved record once"

# 6. no pk on PATH: unchanged behaviour (stderr notice, empty stdout, exit 0)
NOPK_PATH="$SCRATCH/nopk"; mkdir -p "$NOPK_PATH"
for tool in jq python3 bash git cat head dirname mktemp; do
  ln -sf "$(command -v "$tool")" "$NOPK_PATH/$tool"
done
out6="$(payload "$PROBLEM" sess-9 | PATH="$NOPK_PATH" /bin/bash "$DISPATCH" UserPromptSubmit claude-code 2>"$SCRATCH/nopk.err")"
[ -z "$out6" ] || fail "no-pk run printed to stdout: $out6"
grep -q 'reason=pk-not-found' "$SCRATCH/nopk.err" || fail "no-pk notice missing"
ok "without pk the hook is unchanged"

echo "test-prompt-gap: $pass checks passed"
