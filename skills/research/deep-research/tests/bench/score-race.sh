#!/usr/bin/env bash
# score-race.sh — score one report against the adopted RACE criteria.
#
# Drives the pack's own gateway with the `judge` role rather than porting the
# upstream Python scorer, so model routing stays in one place
# (~/.prometheus/kbd/models.toml) and the judge is provably not the producer.
#
# THE REFUSAL THAT MATTERS: a judge that is the same model as the producer
# shares the producer's blind spots. It will not notice what the producer failed
# to notice, so a high score from a same-model judge measures agreement, not
# quality. That is a failure, not a fallback — this script exits 2 BLOCKED
# rather than emitting a number nobody should trust.
#
# Usage:
#   score-race.sh --package <dir> --task-id <n> --criteria <file> \
#                 --judge <model> --producer <model>
#
# Prints the overall score (0-100) on the last stdout line.
# Exit: 0 scored, 1 usage/IO error, 2 BLOCKED.
# bash 3.2 compatible (C-05).
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
SKILL="$(cd "$HERE/../.." && pwd -P)"
REPO="$(cd "$SKILL/../../.." && pwd -P)"

PKG=""; TASK=""; CRIT=""; JUDGE=""; PRODUCER=""
while [ $# -gt 0 ]; do
    case "$1" in
        --package)  PKG="${2:-}";      shift 2 ;;
        --task-id)  TASK="${2:-}";     shift 2 ;;
        --criteria) CRIT="${2:-}";     shift 2 ;;
        --judge)    JUDGE="${2:-}";    shift 2 ;;
        --producer) PRODUCER="${2:-}"; shift 2 ;;
        -h|--help)  sed -n '2,20p' "$0"; exit 0 ;;
        *) echo "score-race: unknown argument: $1" >&2; exit 1 ;;
    esac
done

blocked() { echo "score-race: BLOCKED — $1" >&2; exit 2; }

[ -n "$PKG" ]   || { echo "score-race: --package is required" >&2; exit 1; }
[ -n "$TASK" ]  || { echo "score-race: --task-id is required" >&2; exit 1; }
[ -n "$CRIT" ]  || { echo "score-race: --criteria is required" >&2; exit 1; }
[ -f "$PKG/report.md" ] || blocked "no report.md under $PKG"
[ -f "$CRIT" ]  || blocked "no criteria file at $CRIT"
command -v jq      >/dev/null 2>&1 || blocked "jq is required"
command -v python3 >/dev/null 2>&1 || blocked "python3 is required"

# --- the judge must not be the producer ------------------------------------
[ -n "$JUDGE" ] || blocked "no judge model given"
if [ -n "$PRODUCER" ] && [ "$PRODUCER" != "unknown" ] && [ "$JUDGE" = "$PRODUCER" ]; then
    blocked "judge ($JUDGE) is the same model as the producer. A same-model judge \
shares the producer's blind spots, so its score measures agreement rather than \
quality. Route the judge role to a different model in ~/.prometheus/kbd/models.toml."
fi

LIB="$REPO/shared/scripts/lib/kbd-model-resolve.sh"
[ -f "$LIB" ] || blocked "missing $LIB"
# shellcheck source=/dev/null
. "$LIB"

if [ -f "$HOME/.prometheus/kbd/secrets.env" ]; then
    set -a; . "$HOME/.prometheus/kbd/secrets.env"; set +a
fi

# --- build the judge prompt from the ADOPTED criteria -----------------------
# The criteria row is passed through verbatim: rewriting it would silently make
# the score incomparable to every other run of this benchmark.
PROMPT_FILE="$(mktemp)"; trap 'rm -f "$PROMPT_FILE"' EXIT
python3 - "$CRIT" "$TASK" "$PKG/report.md" > "$PROMPT_FILE" <<'PY'
import json, sys
crit_path, task_id, report_path = sys.argv[1], sys.argv[2], sys.argv[3]

row = None
for line in open(crit_path, encoding="utf-8"):
    if not line.strip():
        continue
    r = json.loads(line)
    if str(r.get("id")) == str(task_id):
        row = r
        break
if row is None:
    sys.stderr.write(f"no criteria row for task {task_id}\n")
    sys.exit(2)

with open(report_path, encoding="utf-8") as fh:
    report = fh.read()

# A very long report would blow the judge's context and truncate the criteria
# with it; cap the report, never the rubric.
MAX = 60000
if len(report) > MAX:
    report = report[:MAX] + "\n\n[report truncated at 60000 characters for scoring]"

print(json.dumps({
    "task": row.get("prompt", ""),
    "dimension_weight": row.get("dimension_weight", {}),
    "criterions": row.get("criterions", []),
    "report": report,
}, ensure_ascii=False))
PY

SYS="You are a strict, meticulous and objective evaluator of research reports. \
You score only against the criteria given to you, you do not invent criteria, \
and you do not reward length. Answer with JSON only."

USR="$(python3 - "$PROMPT_FILE" <<'PY'
import json, sys
d = json.load(open(sys.argv[1], encoding="utf-8"))
crit = json.dumps(d["criterions"], ensure_ascii=False, indent=2)
w = json.dumps(d["dimension_weight"], ensure_ascii=False)
print(f"""**Research task**
{d['task']}

**Dimension weights**
{w}

**Criteria**
{crit}

**Report under evaluation**
{d['report']}

Score the report against EVERY criterion above. For each dimension
(readability, insight, comprehensiveness, instruction_following) give a score
from 0 to 100. Then compute `overall` as the weighted mean using the dimension
weights given.

Reply with JSON only, exactly:
{{"readability": <n>, "insight": <n>, "comprehensiveness": <n>,
  "instruction_following": <n>, "overall": <n>}}""")
PY
)"

RAW="$(kbd_complete "$JUDGE" "$SYS" "$USR" 1500)" || blocked "judge call failed (gateway unreachable or refused)"

# A judge that returns prose instead of JSON has not scored anything; refusing
# beats parsing a number out of an apology.
SCORE="$(printf '%s' "$RAW" | python3 -c "
import json, re, sys
raw = sys.stdin.read()
m = re.search(r'\{.*\}', raw, re.S)
if not m:
    sys.exit(1)
try:
    d = json.loads(m.group(0))
except Exception:
    sys.exit(1)
v = d.get('overall')
if not isinstance(v, (int, float)):
    sys.exit(1)
print(f'{float(v):.1f}')
" 2>/dev/null)" || blocked "judge did not return a parseable score for task $TASK"

echo "$SCORE"
