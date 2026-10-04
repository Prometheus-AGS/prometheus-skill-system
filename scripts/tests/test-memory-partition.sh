#!/bin/bash
# test-memory-partition.sh - integration test for scripts/memory-index-partition.py (B6).
#
# Runs the real partition tool and the real learning_write.py queue path against
# a COPY of a ~14 KB synthetic auto-memory index in a scratch HOME. Asserts:
#   * dry run (the default) changes nothing and queues nothing;
#   * --apply leaves an index of at most 4,096 bytes and keeps a byte-identical backup;
#   * every bullet removed from the index is a queued learning-write operation
#     addressed to its role (role marker), the team (team marker) or the project
#     (over budget); every other bullet is still in the index verbatim;
#   * a second run queues nothing new (idempotent) and touches no other path.
# Delivery by recall is covered by the SubagentStart gate (B5), not here: this
# test needs no memory service. Nothing touches the user's real HOME or index.
# Exit 0 pass, 1 fail, 2 BLOCKED (a prerequisite is missing). bash 3.2.
set -u

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
PARTITION="$ROOT/scripts/memory-index-partition.py"

blocked() { echo "BLOCKED: $*" >&2; exit 2; }
command -v python3 >/dev/null 2>&1 || blocked "python3 not found"
[ -f "$PARTITION" ] || blocked "memory-index-partition.py missing"
[ -f "$ROOT/shared/scripts/lib/learning_write.py" ] || blocked "learning_write.py missing (B3 not on this branch)"

S="$(mktemp -d)"
trap 'rm -rf "$S"' EXIT
export HOME="$S/home" PROMETHEUS_LEARNING_QUEUE="$S/queue" PROMETHEUS_LEARNING_LOG_DIR="$S/log" PROMETHEUS_LEARNING_INDEX_DIR="$S/index"
export PROMETHEUS_PROJECT_ID_SKIP_RUNTIME=1
mkdir -p "$HOME"
fail() { echo "FAIL: $*" >&2; exit 1; }
ok() { echo "ok - $*"; }

# --- scratch project with a two-role team (same shape as the B3 fixture) -----
REPO="$S/repo"; mkdir -p "$REPO/.agent-team/tlm-fixture" "$REPO/.prometheus"
git -C "$REPO" init -q
printf '{"projectId":"project:b6-fixture"}\n' > "$REPO/.prometheus/project.json"
cat > "$REPO/.agent-team/tlm-fixture/team.json" <<'JSON'
{"schemaVersion":1,"id":"tlm-fixture","roles":[
 {"id":"api-dev","description":"api","prompt":"p","skills":[],"owns":["src/api/**"],"inputs":[],"outputs":[],"dependsOn":[]},
 {"id":"ui-dev","description":"ui","prompt":"p","skills":[],"owns":["src/ui/**"],"inputs":[],"outputs":[],"dependsOn":[]}]}
JSON

# --- synthetic ~14 KB index --------------------------------------------------
MEM="$S/memory/MEMORY.md"; DECOY="$S/other/MEMORY.md"
mkdir -p "$S/memory" "$S/other"
python3 - "$MEM" <<'PY'
import sys
lines = ["# Synthetic auto-memory index", ""]
for i in range(1, 71):
    body = f"entry {i:02d} " + ("lorem ipsum dolor sit amet " * 6).strip()
    if i % 7 == 0:
        lines.append(f"- [UI lesson {i:02d}](ui_{i:02d}.md) role:ui-dev {body}")
    elif i % 11 == 0:
        lines.append(f"- [API lesson {i:02d}](api_{i:02d}.md) agent:api-dev {body}")
    elif i % 13 == 0:
        lines.append(f"- [Team lesson {i:02d}](team_{i:02d}.md) team:* {body}")
    else:
        lines.append(f"- [Project note {i:02d}](proj_{i:02d}.md) - {body}")
open(sys.argv[1], "w").write("\n".join(lines) + "\n")
PY
printf 'decoy\n' > "$DECOY"
cp "$MEM" "$S/original.md"
BEFORE=$(wc -c < "$MEM" | tr -d ' ')
[ "$BEFORE" -ge 13000 ] || fail "synthetic index is only $BEFORE bytes, want ~14 KB"
ok "synthetic index is $BEFORE bytes"

count_ops() { find "$PROMETHEUS_LEARNING_QUEUE" -name '*.json' 2>/dev/null | wc -l | tr -d ' '; }

# --- dry run changes nothing ---------------------------------------------------
python3 "$PARTITION" "$MEM" --cwd "$REPO" > "$S/plan.json" || fail "dry run exited non-zero"
cmp -s "$MEM" "$S/original.md" || fail "dry run modified the index"
[ "$(count_ops)" = "0" ] || fail "dry run queued operations"
python3 - "$S/plan.json" <<'PY' || exit 1
import json, sys
p = json.load(open(sys.argv[1]))
assert p["applied"] is False and p["within_limit"] is True and p["bytes_after"] <= 4096, p
assert p["moved"], "plan moves nothing"
print("plan: %d bytes -> %d, %d moved" % (p["bytes_before"], p["bytes_after"], len(p["moved"])))
PY
ok "dry run is the default and changes nothing"

# --- apply ----------------------------------------------------------------------
python3 "$PARTITION" "$MEM" --cwd "$REPO" --apply > "$S/applied.json" || fail "--apply exited non-zero"
AFTER=$(wc -c < "$MEM" | tr -d ' ')
[ "$AFTER" -le 4096 ] || fail "index is $AFTER bytes after apply, want <= 4096"
ok "index is $AFTER bytes (<= 4096)"
BACKUP="$(ls "$S/memory"/MEMORY.md.bak-* 2>/dev/null | head -1)"
[ -n "$BACKUP" ] && cmp -s "$BACKUP" "$S/original.md" || fail "backup missing or not byte-identical"
ok "backup is byte-identical to the original"

# --- every removed line is queued for its address, or still in the index --------
python3 - "$S/original.md" "$MEM" "$PROMETHEUS_LEARNING_QUEUE" <<'PY' || exit 1
import glob, json, re, sys
orig = open(sys.argv[1]).read().splitlines()
new = open(sys.argv[2]).read().splitlines()
ops = [json.load(open(f))["arguments"] for f in glob.glob(sys.argv[3] + "/memory/*/*.json")]
def text_of(line):
    t = re.sub(r"^\s*[-*+]\s+", "", line)
    t = re.sub(r"(?:\b(?:role|agent):|@role/)[a-z][a-z0-9-]*\b", "", t)
    t = re.sub(r"(?:\bteam:(?:[a-z][a-z0-9-]*|\*)|\[team\])", "", t)
    return " ".join(t.split())
def find(text):
    return [o for o in ops if o["content"].startswith(text)]
kept = removed = 0
newset = set(new)
for line in orig:
    if line in newset:
        kept += 1
        continue
    removed += 1
    if not re.match(r"^\s*[-*+]\s+", line):
        raise SystemExit("FAIL: non-bullet line removed: %r" % line)
    hits = find(text_of(line))
    if not hits:
        raise SystemExit("FAIL: removed line neither kept nor queued: %r" % line[:80])
    m = re.search(r"(?:\b(?:role|agent):|@role/)([a-z][a-z0-9-]*)\b", line)
    agents = {o["agent_id"] for o in hits}
    if m:
        assert any(a.endswith("/" + m.group(1)) for a in agents), (m.group(1), agents)
    elif re.search(r"\bteam:", line):
        assert any(a.endswith("/@team") for a in agents), agents
    else:
        assert "@project" in agents, agents
print("kept in index: %d, queued: %d, lost: 0" % (kept, removed))
PY
ok "every removed line is queued for its address or remains in the index"

# --- idempotence and path safety --------------------------------------------------
OPS1=$(count_ops)
cp "$S/original.md" "$S/second.md"
python3 "$PARTITION" "$S/second.md" --cwd "$REPO" --apply >/dev/null || fail "second run failed"
[ "$(count_ops)" = "$OPS1" ] || fail "re-partitioning the same index queued new operations"
ok "re-running queues nothing new ($OPS1 operations)"
[ "$(cat "$DECOY")" = "decoy" ] && [ "$(ls "$S/other" | wc -l | tr -d ' ')" = "1" ] || fail "a path the caller did not give was touched"
[ "$(ls "$S/memory" | wc -l | tr -d ' ')" = "2" ] || fail "unexpected files next to the index: $(ls "$S/memory")"
ok "only the given path (and its backup) was touched"

echo "PASS: $((5)) groups"
