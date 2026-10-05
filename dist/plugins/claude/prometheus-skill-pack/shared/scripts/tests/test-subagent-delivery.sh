#!/bin/bash
# test-subagent-delivery.sh — B5 alpha gate: each subagent receives only its own lessons.
#
#   test-subagent-delivery.sh [--harness claude|codex|both|none]   (default: both)
#   (`none` runs everything except the two model-driven harness runs)
#
# Real processes end to end, driving the GENERATED hook entries:
#   1. fixture team `tlm-fixture` (api-dev owns src/api/**, ui-dev owns src/ui/**)
#      in a scratch git repo; learning_write seeds a private ALPHA-API lesson for
#      api-dev, a private ALPHA-UI lesson for ui-dev and ~30 KB of unrelated
#      project lessons; the real prometheus-learning-worker delivers them to a
#      scratch surreal-memory-server 1.10 (embedded, local MLX embeddings);
#   2. the generated `subagentstart-learning` entry runs once directly through
#      scripts/hook-entry.mjs (fail fast before any model call);
#   3. Claude Code: `claude -p --plugin-dir <generated claude package>` spawns
#      api-dev and ui-dev; each reports the ALPHA-* tokens it was given;
#   4. Codex: the generated Codex package is installed into a scratch CODEX_HOME
#      (`codex plugin marketplace add` + `codex plugin add`), the project is
#      trusted, `[features].hooks = true`, and `codex exec
#      --dangerously-bypass-hook-trust` spawns api_dev and ui_dev; the child
#      rollouts must carry the token as a developer message;
#   5. delivery.jsonl: Claude <= 8,000 chars, Codex <= 7,000 chars and <= 2,000
#      estimated tokens, <= 12 KB per subagent, 0 leaks;
#   6. no team / unresolved role / surreal-memory stopped -> exit 0, empty stdout.
# Codex writes evidence/b5-codex-trust-path.md (TLI_EVIDENCE_DIR, default: the
# main clone's .kbd-orchestrator/phases/team-aware-learning-memory-impl/evidence).
#
# Isolation: scratch HOME, CODEX_HOME (auth.json symlinked, never read or copied),
# PROMETHEUS_PLUGIN_ROOT, queue, log and index. Claude Code authenticates only
# with the real HOME (a scratch HOME reports "Not logged in"), so the Claude run
# keeps the real HOME, excludes user settings (--setting-sources project,local),
# and loads a scratch copy of the generated package whose hooks.json carries only
# the generated SubagentStart group — no other plugin hook can write into the
# user's ~/.prometheus. Codex runs the complete generated hook set.
#
# Binaries: TLI_SM_BIN, TLI_WORKER_BIN, TLI_PK_BIN (else PATH), all >= 1.10.0.
# Exit 0 pass, 1 fail, 2 BLOCKED (a prerequisite is missing). bash 3.2.
set -u

HARNESS=both
while [ $# -gt 0 ]; do
  case "$1" in
    --harness) HARNESS="${2:-}"; shift 2 ;;
    *) echo "usage: $0 [--harness claude|codex|both|none]" >&2; exit 64 ;;
  esac
done
case "$HARNESS" in claude|codex|both|none) ;; *) echo "bad --harness: $HARNESS" >&2; exit 64 ;; esac
want() { [ "$HARNESS" = both ] || [ "$HARNESS" = "$1" ]; }

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
LW="$ROOT/shared/scripts/lib/learning_write.py"
CLAUDE_PKG="$ROOT/dist/plugins/claude/prometheus-skill-pack"
CODEX_PKG="$ROOT/dist/plugins/codex/prometheus-skill-pack"
. "$HERE/lib/scratch-surreal.sh"   # before any HOME override (locates the real model cache)
WORKER_BIN="${TLI_WORKER_BIN:-$(command -v prometheus-learning-worker || true)}"
PK_BIN="${TLI_PK_BIN:-$(command -v pk || true)}"
PORT="$(scratch_surreal_pick_port)"; SCRATCH_SURREAL_PORT="$PORT"
REAL_HOME="$HOME"
MODEL_TIMEOUT="${TLI_MODEL_TIMEOUT:-900}"

blocked() { echo "BLOCKED: $*" >&2; exit 2; }
version_ok() { "$1" --version 2>/dev/null | grep -Eq '1\.(1[0-9]|[2-9][0-9])\.'; }
[ -x "$WORKER_BIN" ] || blocked "prometheus-learning-worker not found (set TLI_WORKER_BIN)"
[ -x "$PK_BIN" ] || blocked "pk not found (set TLI_PK_BIN or put pk >= 1.10 on PATH)"
version_ok "$WORKER_BIN" || blocked "prometheus-learning-worker < 1.10.0"
version_ok "$PK_BIN" || blocked "pk < 1.10.0"
for tool in python3 node git curl; do command -v "$tool" >/dev/null 2>&1 || blocked "$tool missing"; done
want claude && { command -v claude >/dev/null 2>&1 || blocked "claude CLI not on PATH"; }
want codex && { command -v codex >/dev/null 2>&1 || blocked "codex CLI not on PATH"; }
want codex && { [ -f "$REAL_HOME/.codex/auth.json" ] || blocked "no Codex login (~/.codex/auth.json absent)"; }
for pkg in "$CLAUDE_PKG" "$CODEX_PKG"; do
  grep -q 'subagentstart-learning' "$pkg/hooks/hooks.json" 2>/dev/null \
    || blocked "generated package lacks subagentstart-learning: $pkg (run the generators)"
done

S="$(mktemp -d "${TMPDIR:-/tmp}/tli-b5.XXXXXX")"
S="$(cd "$S" && pwd -P)"
cleanup() { scratch_surreal_stop >/dev/null 2>&1; wait 2>/dev/null; if [ -n "${TLI_KEEP:-}" ]; then echo "kept $S" >&2; else rm -rf "$S"; fi; }
trap cleanup EXIT
export HOME="$S/home" CODEX_HOME="$S/codex" PROMETHEUS_PLUGIN_ROOT="$S/plugin-root"
export PROMETHEUS_LEARNING_QUEUE="$S/queue" PROMETHEUS_LEARNING_LOG_DIR="$S/log" PROMETHEUS_LEARNING_INDEX_DIR="$S/index"
export PROMETHEUS_LEARNING_DELIVERY_TRACE_DIR="$S/trace" PROMETHEUS_PROJECT_ID_SKIP_RUNTIME=1 PROMETHEUS_USER_ID=b5-fixture
export SURREAL_MEMORY_URL="http://127.0.0.1:$PORT" PATH="$(dirname "$PK_BIN"):$PATH"
unset PROMETHEUS_LEARNING_PK CLAUDE_PLUGIN_ROOT PLUGIN_ROOT KBD_PACK_ROOT PK_KB_DIR PROMETHEUS_HARNESS
mkdir -p "$HOME" "$CODEX_HOME" "$PROMETHEUS_PLUGIN_ROOT"
git config --global user.email fixture@example.invalid; git config --global user.name Fixture

pass=0
fail() { echo "FAIL: $*" >&2; exit 1; }
ok() { pass=$((pass + 1)); echo "ok - $*"; }
run_timeout() { # <seconds> <cmd...>   (bash 3.2: no coreutils timeout)
  local secs="$1"; shift
  "$@" & local pid=$!
  ( sleep "$secs"; kill -TERM "$pid" 2>/dev/null ) >/dev/null 2>&1 & local wd=$!
  wait "$pid"; local rc=$?
  kill "$wd" 2>/dev/null; wait "$wd" 2>/dev/null
  return $rc
}

# --- 1. fixture repo, seed, real worker, scratch surreal-memory ---------------
REPO="$S/repo"
mkdir -p "$REPO/.agent-team/tlm-fixture" "$REPO/.prometheus" "$REPO/src/api" "$REPO/src/ui" "$REPO/.codex/agents"
git -C "$REPO" init -q
printf '{"projectId":"project:b5-fixture"}\n' > "$REPO/.prometheus/project.json"
cat > "$REPO/.agent-team/tlm-fixture/team.json" <<'EOF'
{"schemaVersion":1,"id":"tlm-fixture","roles":[
 {"id":"api-dev","description":"api","prompt":"p","skills":[],"owns":["src/api/**"],"inputs":[],"outputs":[],"dependsOn":[]},
 {"id":"ui-dev","description":"ui","prompt":"p","skills":[],"owns":["src/ui/**"],"inputs":[],"outputs":[],"dependsOn":[]}]}
EOF
PROBE_INSTRUCTIONS='You are a delivery probe. Look in your context for tokens of the form ALPHA-<WORD>. Reply with exactly those tokens separated by single spaces, or NONE if there are none. Never use tools.'
for role in api_dev ui_dev; do
  printf 'name = "%s"\ndescription = "Fixture role %s. Use only when explicitly asked to spawn %s."\ndeveloper_instructions = "%s"\n' \
    "$role" "$role" "$role" "$PROBE_INSTRUCTIONS" > "$REPO/.codex/agents/$role.toml"
done
printf 'x\n' > "$REPO/src/api/a.ts"; printf 'x\n' > "$REPO/src/ui/u.ts"
git -C "$REPO" add -A >/dev/null; git -C "$REPO" commit -qm fixture
seed() { # <agent_type> <args...>
  local agent="$1"; shift
  printf '%s' '{"agent_type":"'"$agent"'","agent_id":"seed-'"$agent"'","session_id":"s-b5","cwd":"'"$REPO"'"}' \
    | python3 "$LW" --payload-stdin --cwd "$REPO" "$@" >/dev/null || fail "learning_write failed ($agent)"
}
seed api-dev --visibility agent --text "ALPHA-API is the api-dev release marker: api-dev retries idempotent writes before reporting."
seed ui-dev --visibility agent --text "ALPHA-UI is the ui-dev release marker: ui-dev keeps the token budget dial private."
python3 - > "$S/project-lessons.txt" <<'PY'
topics = ["build caching", "log rotation", "schema migration", "dependency pins", "release notes", "test isolation",
          "launchd services", "proxy configuration", "file locking", "error envelopes"]
for i in range(60):
    t = topics[i % len(topics)]
    body = (f"Project lesson {i:02d} on {t}: keep the {t} steps deterministic, record the exact command and its "
            f"result, prefer the smallest reproducible check, and write down why a shortcut was rejected so the next "
            f"cycle does not repeat it. Variant {i} notes that unrelated project knowledge must never crowd out "
            f"role-specific guidance, and that budgets are measured in characters at the hook boundary, never guessed. "
            f"Keep the {t} runbook next to the code it describes and review it whenever the owning change lands.")
    print(body)
PY
while IFS= read -r line; do seed api-dev --visibility project --text "$line"; done < "$S/project-lessons.txt"
seeded_bytes="$(wc -c < "$S/project-lessons.txt" | tr -d ' ')"
[ "$seeded_bytes" -ge 30000 ] || fail "only $seeded_bytes bytes of project lessons seeded"

scratch_surreal_start "$S/sm" b5
for _ in $(seq 1 20); do
  "$WORKER_BIN" --memory-url "$SURREAL_MEMORY_URL" run-once >/dev/null 2>&1
  [ -z "$(ls "$PROMETHEUS_LEARNING_QUEUE"/memory/pending "$PROMETHEUS_LEARNING_QUEUE"/memory/submitting "$PROMETHEUS_LEARNING_QUEUE"/memory/accepted 2>/dev/null | grep json)" ] && break
  sleep 2
done
python3 - "$SURREAL_MEMORY_URL" <<'PY' || fail "seeded lessons did not reach surreal-memory"
import json, sys, urllib.request, urllib.parse
base = sys.argv[1]
def get(uid, aid):
    q = urllib.parse.urlencode({"user_id": uid, "agent_id": aid})
    data = json.loads(urllib.request.urlopen(f"{base}/api/v1/memory?{q}", timeout=10).read())
    return data if isinstance(data, list) else data.get("memories", data.get("results", []))
api, ui, proj = get("project:b5-fixture", "tlm-fixture/api-dev"), get("project:b5-fixture", "tlm-fixture/ui-dev"), get("project:b5-fixture", "@project")
assert any("ALPHA-API" in r["content"] for r in api) and not any("ALPHA-UI" in r["content"] for r in api), "api scope"
assert any("ALPHA-UI" in r["content"] for r in ui) and not any("ALPHA-API" in r["content"] for r in ui), "ui scope"
text = sum(len(r["content"].split("\n\n<!-- prometheus-envelope ")[0]) for r in proj)
assert len(proj) >= 60 and text >= 30000, (len(proj), text)
PY
ok "seeded ALPHA-API (api-dev), ALPHA-UI (ui-dev) and $seeded_bytes bytes of project lessons; the real worker delivered them"

# --- 2. the generated entry, run directly (no model) --------------------------
BUNDLE="$(node -e 'process.stdout.write(JSON.parse(require("fs").readFileSync(process.argv[1])).bundleId)' "$CLAUDE_PKG/shared/harnesses/generated/release-manifest.json")"
grep -q -- "--bundle $BUNDLE --hook subagentstart-learning" "$CODEX_PKG/hooks/hooks.json" || fail "codex package bundle differs from claude package"
bash "$CLAUDE_PKG/shared/scripts/bootstrap-hook-runtime.sh" --source-root "$CLAUDE_PKG" --expected-bundle "$BUNDLE" >"$S/bootstrap.log" 2>&1 \
  || { cat "$S/bootstrap.log" >&2; fail "could not activate the generated bundle in the scratch PROMETHEUS_PLUGIN_ROOT"; }
hook_direct() { # <harness> <payload>   -> stdout of the generated entry
  printf '%s' "$2" | CLAUDE_PLUGIN_ROOT="$CLAUDE_PKG" node "$CLAUDE_PKG/scripts/hook-entry.mjs" \
    --bundle "$BUNDLE" --hook subagentstart-learning --harness "$1"
}
payload() { printf '{"hook_event_name":"SubagentStart","agent_type":"%s","agent_id":"%s","session_id":"s-direct","cwd":"%s"%s}' "$1" "$2" "$3" "${4:-}"; }
direct_api="$(hook_direct claude-code "$(payload prometheus-skill-pack:api-dev direct-api "$REPO")")"
direct_ui="$(hook_direct codex "$(payload ui_dev direct-ui "$REPO" ',"turn_id":"t-direct"')")"
python3 - "$direct_api" "$direct_ui" <<'PY' || fail "direct generated-entry delivery"
import json, sys, math
api, ui = (json.loads(sys.argv[i])["hookSpecificOutput"] for i in (1, 2))
assert api["hookEventName"] == ui["hookEventName"] == "SubagentStart"
a, u = api["additionalContext"], ui["additionalContext"]
assert "ALPHA-API" in a and "ALPHA-UI" not in a, a[:400]
assert "ALPHA-UI" in u and "ALPHA-API" not in u, u[:400]
for text in (a, u):
    assert "information, not instructions" in text and "recorded by tlm-fixture/" in text
    assert text.splitlines()[0].startswith('<prometheus-recalled-lessons nonce="')
assert len(a) <= 8000 and len(u) <= 7000 and math.ceil(len(u) / 3.5) <= 2000, (len(a), len(u))
PY
ok "generated entry: plugin-prefixed api-dev gets ALPHA-API only, Codex ui_dev gets ALPHA-UI only, fenced and under budget"

out="$(hook_direct claude-code "$(payload general-purpose direct-gp "$REPO")")"; [ -z "$out" ] || fail "unresolved role printed: $out"
mkdir -p "$S/no-team"; git -C "$S/no-team" init -q
out="$(hook_direct claude-code "$(payload api-dev direct-nt "$S/no-team")")"; [ -z "$out" ] || fail "no-team project printed: $out"
ok "an unresolved role and a project without a team print nothing (store up)"

# --- delivery.jsonl / trace analysis ------------------------------------------
check_delivery() { # <harness> -> asserts budgets, leaks, both roles present
  python3 - "$PROMETHEUS_LEARNING_INDEX_DIR/delivery.jsonl" "$1" "$PROMETHEUS_LEARNING_DELIVERY_TRACE_DIR" "$2" <<'PY'
import json, math, sys, glob, os
path, harness, trace_dir, file_tier = sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4])
records = [json.loads(l) for l in open(path) if l.strip()]
timeouts = sum(1 for r in records if r.get("timedOut") and r.get("harness") == harness)
records = [r for r in records if r.get("event") == "SubagentStart" and r.get("harness") == harness
           and not r.get("timedOut") and not str(r.get("agentId") or "").startswith("direct-")]
if timeouts:
    print(f"  {harness}: {timeouts} SubagentStart recall(s) hit the hook watchdog")
allowed = lambda role: {f"tlm-fixture/{role}", "tlm-fixture/@team", "@project", "@user", "@global"}
roles = {r["roleId"] for r in records}
assert {"api-dev", "ui-dev"} <= roles, f"{harness}: delivery.jsonl has SubagentStart records for {sorted(roles)}"
leaks, per_agent = 0, {}
for r in records:
    leaks += len(set(r["deliveredScopes"]) - allowed(r["roleId"]))
    per_agent[r["agentId"]] = per_agent.get(r["agentId"], 0) + sum(r["bytesByChannel"].values())
    if harness == "codex":
        assert r["chars"] <= 7000 and r["estTokens"] <= 2000 and r["estTokens"] == math.ceil(r["chars"] / 3.5), r
    else:
        assert r["chars"] <= 8000, r
for trace in glob.glob(os.path.join(trace_dir, f"{harness}-*.txt")):
    name, text = os.path.basename(trace), open(trace).read()
    if "-api-dev-" in name and "ALPHA-UI" in text: leaks += 1
    if "-ui-dev-" in name and "ALPHA-API" in text: leaks += 1
assert leaks == 0, f"{harness}: {leaks} leaks"
worst = max(v + file_tier for v in per_agent.values())
assert worst <= 12288, f"{harness}: {worst} bytes for one subagent"
for r in records:
    print(f"  {harness} {r['roleId']}: {r['chars']} chars, {r['estTokens']} est. tokens, "
          f"{sum(r['bytesByChannel'].values())} B SubagentStart + {file_tier} B file tier, scopes {r['deliveredScopes']}")
PY
}

# --- 3. Claude Code -------------------------------------------------------------
if want claude; then
  CPKG="$S/claude-pkg"
  cp -R "$CLAUDE_PKG" "$CPKG"
  python3 - "$CLAUDE_PKG/hooks/hooks.json" "$CPKG/hooks/hooks.json" <<'PY'
import json, sys
hooks = json.load(open(sys.argv[1]))["hooks"]
json.dump({"hooks": {"SubagentStart": hooks["SubagentStart"]}}, open(sys.argv[2], "w"), indent=2)
PY
  AGENTS="$(python3 - "$PROBE_INSTRUCTIONS" <<'PY'
import json, sys
print(json.dumps({r: {"description": f"Fixture role {r}. Use only when explicitly asked.", "prompt": sys.argv[1], "model": "haiku"}
                  for r in ("api-dev", "ui-dev")}))
PY
)"
  PROMPT='Use the Task tool twice, one after the other: first with subagent_type "api-dev", then with subagent_type "ui-dev", each with the prompt "Report your ALPHA tokens." Do not answer for them. After both return, print exactly two lines and nothing else: api-dev=<its reply verbatim> and ui-dev=<its reply verbatim>.'
  ( cd "$REPO" && HOME="$REAL_HOME" run_timeout "$MODEL_TIMEOUT" claude -p "$PROMPT" --setting-sources project,local \
      --strict-mcp-config --no-session-persistence --plugin-dir "$CPKG" --agents "$AGENTS" --model sonnet ) \
      > "$S/claude.out" 2> "$S/claude.err" < /dev/null
  echo "  claude output: $(tr '\n' ' ' < "$S/claude.out")"
  python3 - "$S/claude.out" <<'PY' || { cat "$S/claude.err" >&2; fail "Claude Code subagents did not report the right tokens"; }
import re, sys
text = open(sys.argv[1]).read()
api = re.search(r"api-dev\s*[=:]\s*(.*)", text); ui = re.search(r"ui-dev\s*[=:]\s*(.*)", text)
assert api and ui, text
assert "ALPHA-API" in api.group(1) and "ALPHA-UI" not in api.group(1), api.group(1)
assert "ALPHA-UI" in ui.group(1) and "ALPHA-API" not in ui.group(1), ui.group(1)
PY
  CLAUDE_TIER=0
  slug="$(python3 -c 'import re,sys;print(re.sub(r"[^A-Za-z0-9]","-",sys.argv[1]))' "$REPO")"
  [ -f "$REAL_HOME/.claude/projects/$slug/memory/MEMORY.md" ] && CLAUDE_TIER="$(wc -c < "$REAL_HOME/.claude/projects/$slug/memory/MEMORY.md" | tr -d ' ')"
  check_delivery claude-code "$CLAUDE_TIER" || fail "Claude Code delivery.jsonl budgets/leaks"
  ok "Claude Code: api-dev reported ALPHA-API and not ALPHA-UI, ui-dev the reverse; delivery under budget, 0 leaks"
fi

# --- 4. Codex -------------------------------------------------------------------
if want codex; then
  MKT="$S/mkt"; mkdir -p "$MKT/.claude-plugin" "$MKT/plugins"
  cp -R "$CODEX_PKG" "$MKT/plugins/prometheus-skill-pack"
  printf '{"mcpServers":{}}\n' > "$MKT/plugins/prometheus-skill-pack/.mcp.json"   # hooks under test, no MCP servers
  printf '{"name":"b5-fixture","owner":{"name":"fixture"},"plugins":[{"name":"prometheus-skill-pack","source":"./plugins/prometheus-skill-pack","description":"generated Codex package under test"}]}\n' \
    > "$MKT/.claude-plugin/marketplace.json"
  ln -s "$REAL_HOME/.codex/auth.json" "$CODEX_HOME/auth.json"
  cat > "$CODEX_HOME/config.toml" <<EOF
[features]
hooks = true
multi_agent = true
[memories]
generate_memories = false
[projects."$REPO"]
trust_level = "trusted"
EOF
  codex plugin marketplace add "$MKT" > "$S/codex-mkt.log" 2>&1 || { cat "$S/codex-mkt.log" >&2; fail "codex plugin marketplace add"; }
  codex plugin add prometheus-skill-pack@b5-fixture > "$S/codex-add.log" 2>&1 || { cat "$S/codex-add.log" >&2; fail "codex plugin add"; }
  PROMPT='Spawn the custom agent api_dev with the task "Report your ALPHA tokens." and wait for its reply. Then spawn the custom agent ui_dev with the same task and wait for its reply. Then print exactly two lines and nothing else: api_dev=<its reply verbatim> and ui_dev=<its reply verbatim>.'
  ( cd "$REPO" && run_timeout "$MODEL_TIMEOUT" codex exec --dangerously-bypass-hook-trust --skip-git-repo-check "$PROMPT" ) \
      > "$S/codex.out" 2> "$S/codex.err" < /dev/null
  echo "  codex output: $(tr '\n' ' ' < "$S/codex.out")"
  python3 - "$S/codex.out" "$CODEX_HOME/sessions" "$S/codex-rollouts.json" <<'PY' || { tail -40 "$S/codex.err" >&2; grep timedOut "$PROMETHEUS_LEARNING_INDEX_DIR/delivery.jsonl" >&2; fail "Codex subagents / child rollouts"; }
import glob, json, re, sys
text, sessions, report = open(sys.argv[1]).read(), sys.argv[2], sys.argv[3]
api = re.search(r"api_dev\s*[=:]\s*(.*)", text); ui = re.search(r"ui_dev\s*[=:]\s*(.*)", text)
assert api and ui, text
assert "ALPHA-API" in api.group(1) and "ALPHA-UI" not in api.group(1), api.group(1)
assert "ALPHA-UI" in ui.group(1) and "ALPHA-API" not in ui.group(1), ui.group(1)
threads = []
for path in glob.glob(f"{sessions}/**/rollout-*.jsonl", recursive=True):
    lines = [json.loads(l) for l in open(path) if l.strip()]
    meta = lines[0]["payload"] if lines and lines[0].get("type") == "session_meta" else {}
    role = meta.get("agent_role")
    developer = []
    for l in lines:
        p = l.get("payload") or {}
        if l.get("type") == "response_item" and p.get("type") == "message" and p.get("role") == "developer":
            kinds = (p.get("internal_chat_message_metadata_passthrough") or {}).get("content_item_kinds") or []
            body = "".join(c.get("text", "") for c in p.get("content") or [] if isinstance(c, dict))
            if "hooks.additional_context" in kinds or "prometheus-recalled-lessons" in body:
                developer.append(body)
    threads.append({"rollout": path.rsplit("/", 1)[1], "role": role, "hookDeveloperMessages": len(developer),
                    "api": any("ALPHA-API" in d for d in developer), "ui": any("ALPHA-UI" in d for d in developer)})
by = {t["role"]: t for t in threads if t["role"]}
assert by.get("api_dev", {}).get("api") and not by["api_dev"]["ui"], threads
assert by.get("ui_dev", {}).get("ui") and not by["ui_dev"]["api"], threads
assert not any(t["api"] or t["ui"] for t in threads if not t["role"]), "a parent thread received the injection"
json.dump(threads, open(report, "w"), indent=1)
PY
  CODEX_TIER=0
  [ -f "$CODEX_HOME/memories/memory_summary.md" ] && CODEX_TIER="$(wc -c < "$CODEX_HOME/memories/memory_summary.md" | tr -d ' ')"
  check_delivery codex "$CODEX_TIER" > "$S/codex-delivery.txt" || { cat "$S/codex-delivery.txt"; fail "Codex delivery.jsonl budgets/leaks"; }
  cat "$S/codex-delivery.txt"
  ok "Codex: api_dev reported ALPHA-API and not ALPHA-UI, ui_dev the reverse; child rollouts carry it as a developer message; under budget, 0 leaks"

  EVIDENCE_DIR="${TLI_EVIDENCE_DIR:-}"
  if [ -z "$EVIDENCE_DIR" ]; then
    common="$(cd "$ROOT" && cd "$(git rev-parse --git-common-dir)" && pwd -P)"
    EVIDENCE_DIR="$(dirname "$common")/.kbd-orchestrator/phases/team-aware-learning-memory-impl/evidence"
  fi
  if [ -d "$EVIDENCE_DIR" ]; then
    python3 - "$EVIDENCE_DIR/b5-codex-trust-path.md" "$S/codex-rollouts.json" "$S/codex-delivery.txt" "$(codex --version)" "$BUNDLE" <<'PY'
import datetime, json, sys
out, rollouts, delivery, version, bundle = sys.argv[1:]
threads = json.load(open(rollouts))
rows = "\n".join(f"| `{t['rollout']}` | {t['role'] or '(parent)'} | {t['hookDeveloperMessages']} | {'yes' if t['api'] else 'no'} | {'yes' if t['ui'] else 'no'} |" for t in threads)
open(out, "w").write(f"""# B5 Codex trust path — evidence

Written by `shared/scripts/tests/test-subagent-delivery.sh --harness codex|both` on {datetime.datetime.now(datetime.timezone.utc).strftime('%Y-%m-%d %H:%M UTC')}.

**Path exercised: native plugin hook.** The PreToolUse fallback contingency (design §5) was **not** needed and was not exercised.

- {version}; generated Codex package (bundle `{bundle[:16]}…`) installed into a scratch `CODEX_HOME` with
  `codex plugin marketplace add <scratch marketplace>` + `codex plugin add prometheus-skill-pack@b5-fixture`
  (package copied verbatim except `.mcp.json` emptied so no MCP server starts; `hooks/hooks.json` untouched).
- Scratch `config.toml`: `[features] hooks = true` (not the deprecated `codex_hooks`), `multi_agent = true`,
  `[memories] generate_memories = false`, `[projects."<pwd -P>"] trust_level = "trusted"`; `auth.json` symlinked.
- Hook trust: `codex exec --dangerously-bypass-hook-trust --skip-git-repo-check … < /dev/null` (scratch gate only).
  Normal interactive use shows a one-time hook-trust prompt for the plugin's non-managed hooks (docs/codex-plugin.md).
- The generated SubagentStart entry (matcher `*`) fired for both custom agents (`api_dev`, `ui_dev`); its
  `hookSpecificOutput.additionalContext` reached each CHILD thread as a `developer` message
  (`content_item_kinds: hooks.additional_context`) and never a parent thread.

| Rollout | Agent role | Hook developer messages | ALPHA-API | ALPHA-UI |
|---|---|---|---|---|
{rows}

Delivery (delivery.jsonl):

```
{open(delivery).read().rstrip()}
```
""")
PY
    echo "  wrote $EVIDENCE_DIR/b5-codex-trust-path.md"
  else
    echo "  evidence dir $EVIDENCE_DIR absent: b5-codex-trust-path.md not written" >&2
  fi
fi

# --- 5. surreal-memory stopped ----------------------------------------------------
scratch_surreal_stop || fail "scratch surreal-memory stop left a process or listener"
curl -fsS -m 1 "$SURREAL_MEMORY_URL/health" >/dev/null 2>&1 && fail "scratch surreal-memory still answering after stop"
NOPK_PATH="$(printf '%s' "$PATH" | tr ':' '\n' | grep -v "^$(dirname "$PK_BIN")\$" | paste -sd: -)"
PATH="$NOPK_PATH" command -v pk >/dev/null 2>&1 && NOPK_PATH="/usr/bin:/bin:/usr/sbin:/sbin:$(dirname "$(command -v node)")"
t0="$(python3 -c 'import time;print(time.time())')"
out="$(PATH="$NOPK_PATH" PROMETHEUS_LEARNING_LOG_DIR="$S/empty-log" PROMETHEUS_TEAM_DIGEST_DIR="$S/empty-digest" hook_direct claude-code "$(payload api-dev stopped-api "$REPO")")"; rc=$?
elapsed="$(python3 -c 'import sys,time;print(round(time.time()-float(sys.argv[1]),2))' "$t0")"
[ "$rc" -eq 0 ] || fail "stopped store: hook exited $rc"
[ -z "$out" ] || fail "stopped store: hook printed: $out"
python3 -c 'import sys; sys.exit(0 if float(sys.argv[1]) < 5 else 1)' "$elapsed" || fail "stopped store: hook took ${elapsed}s"
ok "surreal-memory stopped (no pk, no learning log, no team digest): the generated entry exits 0 with empty stdout in ${elapsed}s"
out="$(hook_direct claude-code "$(payload api-dev stopped-fallback "$REPO")")"; rc=$?
[ "$rc" -eq 0 ] || fail "stopped store with fallbacks: hook exited $rc"
printf '%s' "$out" | grep -q 'ALPHA-UI' && fail "stopped store fallback leaked ALPHA-UI to api-dev"
ok "surreal-memory stopped (pk + learning log present): exits 0, fallback stays role-isolated ($(printf '%s' "$out" | grep -c 'ALPHA-API') ALPHA-API hit)"

echo "test-subagent-delivery: $pass passed (harness: $HARNESS)"
