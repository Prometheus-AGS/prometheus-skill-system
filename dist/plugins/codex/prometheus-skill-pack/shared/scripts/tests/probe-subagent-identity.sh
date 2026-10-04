#!/usr/bin/env bash
# probe-subagent-identity.sh — confirm, at runtime, what agent identity each harness
# gives hooks and whether a SubagentStart hook can inject context into ONE subagent.
#
# Behaviours probed (change-tlm-001, phase team-aware-learning-memory):
#   1 claude  SubagentStart additionalContext reaches the subagent
#   2 claude  agent_type is the frontmatter name (project agent) / <plugin>:<name> (plugin agent)
#   3 codex   SubagentStart agent_type == custom agent name, and its output reaches the subagent
#   4 codex   PreToolUse inside a subagent carries agent_id / agent_type
#   5 both    the harness's file-memory tier reaches a subagent (Claude project MEMORY.md,
#             Codex ~/.codex/memories/memory_summary.md)
# Behaviour 6 (Codex per-thread memory controls) is static inspection, recorded separately.
#
# Isolation: nothing is written to ~/.claude, ~/.codex, plugin caches or ~/.prometheus/plugins.
#   claude: --setting-sources project,local --strict-mcp-config --no-session-persistence,
#           with the probe agent and hooks supplied for the session only (--agents, --settings,
#           --plugin-dir), so nothing is written into any project.
#   codex:  --ephemeral --ignore-user-config, re-enabling exactly the user's features
#           (multi_agent, memories) and trusting only the scratch project via -c, with HOME
#           pointed at scratch so Prometheus hook scripts write there.
# Every harness run is bracketed by hash sets of user config, queues and memory stores.
#
# Memory presence is tested without leaking memory content into the prompt: the prompt
# gives an ANCHOR line from the memory file and asks for the line that follows it. Only an
# agent that holds the file in its context can answer, and the answer is checked here.
#
# Usage: probe-subagent-identity.sh --out <dir> [--harness claude|codex|both] [--repo <path>]
# Output: <dir>/probe-result.json plus raw transcripts. Makes real, authenticated model calls.
# bash 3.2 compatible.
set -uo pipefail

OUT="" HARNESS="both" REPO="$PWD"
while [ $# -gt 0 ]; do
  case "$1" in
    --out) OUT="${2:-}"; shift 2 ;;
    --harness) HARNESS="${2:-both}"; shift 2 ;;
    --repo) REPO="${2:-$PWD}"; shift 2 ;;
    *) echo "unknown argument: $1" >&2; exit 64 ;;
  esac
done
[ -n "$OUT" ] || { echo "usage: $0 --out <dir> [--harness claude|codex|both] [--repo <path>]" >&2; exit 64; }
mkdir -p "$OUT"
OUT="$(cd "$OUT" && pwd)"
REPO="$(cd "$REPO" && pwd)"

S="$(mktemp -d "${TMPDIR:-/tmp}/tlm-probe.XXXXXX")"
NONCE="TLM-NONCE-$(od -An -N6 -tx1 /dev/urandom | tr -d ' \n')"
MARKS="$S/marks.jsonl"
# Resolve the real interpreter now: version-manager shims (pyenv) break when a run points
# HOME at scratch, which once made every Codex hook capture fail silently.
PY="$(python3 -c 'import sys; print(sys.executable)')"
: > "$MARKS"

# --- hashing -----------------------------------------------------------------
hash_set() { # prints a JSON object of path -> sha256 (directories hash their sorted listing)
  python3 - <<'PY'
import hashlib, json, os, glob
paths = ['~/.claude/settings.json', '~/.codex/config.toml', '~/.codex/hooks.json',
         '~/.prometheus/plugins/prometheus-skill-pack/pointers/current', '~/.claude.json',
         '~/.prometheus/learning-queue/pending', '~/.prometheus/learning-queue/memory/pending',
         '~/.codex/memories/memory_summary.md', '~/.codex/memories/MEMORY.md']
paths += sorted(p.replace(os.path.expanduser('~'), '~', 1) for p in glob.glob(os.path.expanduser('~/.prometheus/memory-outbox*')))
out = {}
for p in paths:
    q = os.path.expanduser(p)
    if os.path.isdir(q):
        out[p] = hashlib.sha256('\n'.join(sorted(os.listdir(q))).encode()).hexdigest()
    elif os.path.exists(q):
        out[p] = hashlib.sha256(open(q, 'rb').read()).hexdigest()
    else:
        out[p] = 'absent'
print(json.dumps(out, sort_keys=True))
PY
}

# --- hook capture script shared by both harnesses -----------------------------
cat > "$S/hook.sh" <<EOF
#!/usr/bin/env bash
# Logs the hook's stdin JSON and injects the nonce on SubagentStart.
event="\${1:-unknown}"
input="\$(cat)"
"$PY" -c 'import json,sys; print(json.dumps({"event": sys.argv[1], "stdin": json.loads(sys.argv[2] or "{}")}))' "\$event" "\$input" >> "$MARKS" 2>/dev/null \
  || printf '{"event":"%s","raw":%s}\n' "\$event" "\$(printf '%s' "\$input" | "$PY" -c 'import json,sys;print(json.dumps(sys.stdin.read()))')" >> "$MARKS"
if [ "\$event" = "SubagentStart" ]; then
  printf '{"hookSpecificOutput":{"hookEventName":"SubagentStart","additionalContext":"%s"}}\n' "$NONCE"
fi
exit 0
EOF
chmod +x "$S/hook.sh"

# --- memory anchors -----------------------------------------------------------
anchor_pair() { # <file> -> prints "anchor\nexpected" for a distinctive adjacent pair
  python3 - "$1" <<'PY'
import sys, os
p = os.path.expanduser(sys.argv[1])
if not os.path.exists(p):
    sys.exit(0)
lines = [l.rstrip('\n') for l in open(p, encoding='utf-8', errors='replace')]
cands = [(i, l) for i, l in enumerate(lines[:-1]) if len(l.strip()) > 40 and len(lines[i + 1].strip()) > 40]
if not cands:
    sys.exit(0)
i, l = cands[len(cands) // 2]
print(l); print(lines[i + 1])
PY
}

PROMPT_TASK() { # <anchor-or-empty>
  if [ -n "$1" ]; then
    cat <<EOF
Answer in exactly two lines and nothing else.
Line 1: the full text of any line in your context that begins with "TLM-NONCE-", or NONE.
Line 2: some file content may have been loaded into your context before this task. Find the line that is exactly: <<<$1>>> and output the single line that immediately follows it in that content, verbatim, or NONE if that line is not in your context. Do not guess.
EOF
  else
    printf 'Answer in exactly one line: the full text of any line in your context that begins with "TLM-NONCE-", or NONE.\n'
  fi
}

python3 - "$OUT/probe-result.json" "$NONCE" <<'PY'
import json, sys
json.dump({"nonce": sys.argv[2], "runs": []}, open(sys.argv[1], 'w'))
PY

record_run() { # <harness> <label> <cwd> <before-json> <after-json> <stdout-file> <expected-next-or-empty>
  python3 - "$OUT/probe-result.json" "$@" "$MARKS" <<'PY'
import json, sys
res_path, harness, label, cwd, before, after, stdout_file, expected, marks = sys.argv[1:10]
res = json.load(open(res_path))
marks_lines = [json.loads(l) for l in open(marks) if l.strip()]
res["runs"].append({
    "harness": harness, "label": label, "cwd": cwd,
    "before": json.loads(before), "after": json.loads(after),
    "stdout": open(stdout_file, encoding='utf-8', errors='replace').read()[-4000:],
    "expected_memory_next_line": expected,
    "hook_events": marks_lines,
})
json.dump(res, open(res_path, 'w'), indent=2)
PY
  : > "$MARKS"
}

# --- Claude Code ---------------------------------------------------------------
run_claude() { # <label> <cwd> <agents-json> <spawn-name> <anchor> <expected> [extra args...]
  label="$1"; cwd="$2"; agents="$3"; spawn="$4"; anchor="$5"; expected="$6"; shift 6
  cat > "$S/claude-settings.json" <<EOF
{"hooks":{
  "SubagentStart":[{"hooks":[{"type":"command","command":"bash $S/hook.sh SubagentStart","timeout":10}]}],
  "SubagentStop":[{"hooks":[{"type":"command","command":"bash $S/hook.sh SubagentStop","timeout":10}]}],
  "PreToolUse":[{"matcher":"*","hooks":[{"type":"command","command":"bash $S/hook.sh PreToolUse","timeout":10}]}]
}}
EOF
  task="$(PROMPT_TASK "$anchor")"
  prompt="Use the Agent tool exactly once to run the subagent named \"$spawn\" with this task, then print the subagent's answer verbatim and nothing else.

TASK:
$task"
  before="$(hash_set)"
  ( cd "$cwd" && timeout 300 claude -p "$prompt" \
      --setting-sources project,local --strict-mcp-config --no-session-persistence \
      --settings "$S/claude-settings.json" --agents "$agents" "$@" \
      < /dev/null > "$S/claude-$label.out" 2> "$S/claude-$label.err" ) || true
  after="$(hash_set)"
  record_run claude "$label" "$cwd" "$before" "$after" "$S/claude-$label.out" "$expected"
}

if [ "$HARNESS" = "claude" ] || [ "$HARNESS" = "both" ]; then
  mkdir -p "$S/claude-proj"
  AGENTS_JSON='{"tlm-probe-role":{"description":"Probe agent for the team-aware-learning-memory identity probe. Use only when explicitly asked to run tlm-probe-role.","prompt":"You are a probe. Follow the task exactly and answer only what it asks. Never use tools."}}'
  run_claude project-agent "$S/claude-proj" "$AGENTS_JSON" tlm-probe-role "" ""
  # Plugin-scoped agent name (behaviour 2, plugin form), loaded for this session only.
  mkdir -p "$S/plugin/.claude-plugin" "$S/plugin/agents"
  printf '{"name":"tlmprobe","version":"0.0.1","description":"session-only probe plugin"}\n' > "$S/plugin/.claude-plugin/plugin.json"
  printf -- '---\nname: tlm-plug-role\ndescription: Probe plugin agent. Use only when explicitly asked to run tlmprobe:tlm-plug-role.\n---\nYou are a probe. Follow the task exactly. Never use tools.\n' > "$S/plugin/agents/tlm-plug-role.md"
  run_claude plugin-agent "$S/claude-proj" '{}' "tlmprobe:tlm-plug-role" "" "" --plugin-dir "$S/plugin"
  # Behaviour 5: the repository's own project auto-memory, in its real directory.
  slug="$(printf '%s' "$REPO" | sed 's#[/.]#-#g')"
  MEM="$HOME/.claude/projects/$slug/memory/MEMORY.md"
  pair="$(anchor_pair "$MEM")"
  anchor="$(printf '%s\n' "$pair" | sed -n 1p)"; expected="$(printf '%s\n' "$pair" | sed -n 2p)"
  run_claude repo-memory "$REPO" "$AGENTS_JSON" tlm-probe-role "$anchor" "$expected"
fi

# --- Codex ---------------------------------------------------------------------
if [ "$HARNESS" = "codex" ] || [ "$HARNESS" = "both" ]; then
  P="$S/codex-proj"; mkdir -p "$P/.codex/agents" "$S/home"
  # Canonical path: the trust override must match the path Codex resolves (/private/var on macOS).
  P="$(cd "$P" && pwd -P)"
  # Codex agent names allow only [a-z0-9_]; a kebab-case role id must be normalised.
  cat > "$P/.codex/agents/tlm_probe_role.toml" <<'EOF'
name = "tlm_probe_role"
description = "Probe agent for the team-aware-learning-memory identity probe. Use only when explicitly asked to spawn tlm_probe_role."
developer_instructions = "You are a probe. Follow the task exactly and answer only what it asks. Never run commands."
EOF
  cat > "$P/.codex/hooks.json" <<EOF
{"hooks":{
  "SubagentStart":[{"hooks":[{"type":"command","command":"bash $S/hook.sh SubagentStart","timeout":10}]}],
  "SubagentStop":[{"hooks":[{"type":"command","command":"bash $S/hook.sh SubagentStop","timeout":10}]}],
  "PreToolUse":[{"hooks":[{"type":"command","command":"bash $S/hook.sh PreToolUse","timeout":10}]}]
}}
EOF
  pair="$(anchor_pair "$HOME/.codex/memories/memory_summary.md")"
  anchor="$(printf '%s\n' "$pair" | sed -n 1p)"; expected="$(printf '%s\n' "$pair" | sed -n 2p)"
  task="$(PROMPT_TASK "$anchor")"
  prompt="Spawn exactly one subagent with agent_type \"tlm_probe_role\" and give it this task, wait for it, then print its answer verbatim and nothing else.

TASK:
$task"
  REAL_CODEX_HOME="${CODEX_HOME:-$HOME/.codex}"
  run_codex() { # <label> [extra -c overrides...]
    label="$1"; shift
    before="$(hash_set)"
    # Not --ephemeral: Codex subagents need the parent thread's rollout ("no rollout found
    # for thread" otherwise). memories.generate_memories=false keeps the persisted probe
    # thread out of memory consolidation; use_memories=true keeps behaviour 5 observable.
    ( cd "$P" && HOME="$S/home" CODEX_HOME="$REAL_CODEX_HOME" timeout 300 codex exec \
        --ignore-user-config --skip-git-repo-check \
        -c features.multi_agent=true -c features.memories=true \
        -c memories.use_memories=true -c memories.generate_memories=false \
        -c "projects.\"$P\".trust_level=\"trusted\"" "$@" \
        --dangerously-bypass-hook-trust "$prompt" \
        < /dev/null > "$S/codex-$label.out" 2> "$S/codex-$label.err" ) || true
    after="$(hash_set)"
    record_run codex "$label" "$P" "$before" "$after" "$S/codex-$label.out" "$expected"
  }
  run_codex multi-agent-v1
  run_codex multi-agent-v2 -c features.multi_agent_v2=true
fi

cp "$S"/*.out "$S"/*.err "$OUT/" 2>/dev/null || true
echo "probe complete: $OUT/probe-result.json (scratch $S)"
