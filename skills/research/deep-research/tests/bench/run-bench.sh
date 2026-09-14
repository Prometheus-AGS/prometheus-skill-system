#!/usr/bin/env bash
# run-bench.sh — drive the 10-task subset through the pipeline and score it.
#
# Makes "on par with Onyx" falsifiable. It produces four numbers per run:
#
#   RACE overall            scored by the judge against the ADOPTED criteria
#   effective citations     built here (score-fact.py)
#   citation accuracy       built here (score-fact.py)
#   verified-claim ratio    built here — the metric Onyx structurally cannot
#                           report, because it carries no per-claim labels
#
# Scoring against the published rubric is the point: a home-grown rubric
# produces a number comparable to nothing.
#
# A run costs real gateway tokens. The 10-task subset is this phase's ceiling.
#
# Usage:
#   run-bench.sh --packages <dir>            score packages already produced
#   run-bench.sh --packages <dir> --dry-run  check wiring, spend nothing
#
# Exit: 0 scored, 1 usage/IO error, 2 BLOCKED (a gate that could not run —
#       never reported as a pass).
# bash 3.2 compatible (C-05).
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
SKILL="$(cd "$HERE/../.." && pwd -P)"
REPO="$(cd "$SKILL/../../.." && pwd -P)"

QUERIES="$HERE/queries/subset-10.jsonl"
CRITERIA="$HERE/criteria/subset-10-criteria.jsonl"
RESULTS="$HERE/BENCH-RESULTS.md"

PKGS=""; DRY=0
while [ $# -gt 0 ]; do
    case "$1" in
        --packages) PKGS="${2:-}"; shift 2 ;;
        --dry-run)  DRY=1; shift ;;
        -h|--help)  sed -n '2,22p' "$0"; exit 0 ;;
        *) echo "run-bench: unknown argument: $1" >&2; exit 1 ;;
    esac
done

blocked() { echo "run-bench: BLOCKED — $1" >&2; exit 2; }

[ -f "$QUERIES" ]  || blocked "no query subset at $QUERIES"
[ -f "$CRITERIA" ] || blocked "no criteria at $CRITERIA"
command -v jq      >/dev/null 2>&1 || blocked "jq is required"
command -v python3 >/dev/null 2>&1 || blocked "python3 is required"

# The resolver library owns model routing; this script never hardcodes a model.
LIB="$REPO/shared/scripts/lib/kbd-model-resolve.sh"
[ -f "$LIB" ] || blocked "missing $LIB"
# shellcheck source=/dev/null
. "$LIB"

# Secrets live outside any config file (CLAUDE.md); load them if present.
if [ -f "$HOME/.prometheus/kbd/secrets.env" ]; then
    set -a; . "$HOME/.prometheus/kbd/secrets.env"; set +a
fi

JUDGE="$(kbd_resolve_role judge 2>/dev/null || true)"
[ -n "$JUDGE" ] || blocked "no judge model resolves; set it in ~/.prometheus/kbd/models.toml"

# The producer is whatever generated the reports under test. It is recorded so
# score-race.sh can refuse a run where judge and producer are the same model.
PRODUCER="${KBD_PRODUCER_MODEL:-${RESEARCH_PRODUCER_MODEL:-unknown}}"

echo "run-bench: judge=$JUDGE producer=$PRODUCER tasks=$(wc -l < "$QUERIES" | tr -d ' ')"

if [ "$DRY" = "1" ]; then
    kbd_resolve_gateway >/dev/null 2>&1 \
        && echo "run-bench: dry run OK — gateway reachable, judge resolves" \
        || blocked "gateway not reachable"
    exit 0
fi

[ -n "$PKGS" ] || blocked "--packages <dir> is required (a dir of scored research packages)"
[ -d "$PKGS" ] || blocked "no such package directory: $PKGS"

SCORED=0; SKIPPED=0
RACE_SUM=0
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

# One package per task id. A task with no package is SKIPPED and counted, never
# silently dropped: a mean over 3 of 10 tasks reported as "the score" would be
# the most misleading number this script could emit.
while IFS= read -r line; do
    [ -n "$line" ] || continue
    TID="$(printf '%s' "$line" | jq -r '.id')"
    PKG="$PKGS/task-$TID"
    if [ ! -d "$PKG" ] || [ ! -f "$PKG/report.md" ]; then
        echo "  task $TID: SKIPPED (no report at $PKG/report.md)"
        SKIPPED=$((SKIPPED + 1))
        continue
    fi

    R="$(bash "$HERE/score-race.sh" --package "$PKG" --task-id "$TID" \
            --criteria "$CRITERIA" --judge "$JUDGE" --producer "$PRODUCER" 2>&1)" || {
        echo "  task $TID: score-race failed: $(printf '%s' "$R" | tail -1)"
        SKIPPED=$((SKIPPED + 1)); continue; }
    SCORE="$(printf '%s' "$R" | tail -1)"
    echo "  task $TID: RACE $SCORE"
    printf '%s\n' "$SCORE" >> "$TMP/race.txt"
    SCORED=$((SCORED + 1))

    python3 "$HERE/score-fact.py" --package "$PKG" >> "$TMP/fact.jsonl" 2>/dev/null || true
done < "$QUERIES"

[ "$SCORED" -gt 0 ] || blocked "no task produced a score ($SKIPPED skipped)"

echo
echo "run-bench: scored $SCORED of $((SCORED + SKIPPED)) tasks"
python3 - "$TMP" "$SCORED" "$SKIPPED" <<'PY'
import json, os, sys
tmp, scored, skipped = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])

race = [float(x) for x in open(os.path.join(tmp, "race.txt")) if x.strip()]
print(f"  RACE overall:          {sum(race)/len(race):.1f}  (n={len(race)})")

fp = os.path.join(tmp, "fact.jsonl")
rows = [json.loads(l) for l in open(fp)] if os.path.isfile(fp) else []
if rows:
    def mean(k):
        vals = [r[k] for r in rows if r.get(k) is not None]
        return sum(vals) / len(vals) if vals else float("nan")
    print(f"  effective citations:   {mean('effective_citations'):.1f}")
    print(f"  citation accuracy:     {mean('citation_accuracy'):.3f}")
    print(f"  verified-claim ratio:  {mean('verified_claim_ratio'):.3f}")
else:
    print("  FACT metrics: none computed")

if skipped:
    print(f"  NOTE: {skipped} task(s) skipped — the means above cover {scored} tasks only.")
PY

echo
echo "run-bench: record these in $RESULTS with the pin (469cce54), the judge model, and today's date."
