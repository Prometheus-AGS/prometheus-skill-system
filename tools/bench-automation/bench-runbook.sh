#!/usr/bin/env bash
# bench-runbook.sh — run the bench (run-bench.sh) on the bench-g5 directory,
# fix the nested→flat layout (run-bench expects <dir>/task-<id>/report.md,
# while run-research.sh creates <dir>/task-<id>/<slug>-<id>/report.md), and
# append per-task results to tests/bench/BENCH-RESULTS.md.
#
# Usage: bench-runbook.sh
set -euo pipefail

BENCH_G5="${BENCH_G5:-$HOME/.prometheus/research/bench-g5}"
BENCH_RESULTS="skills/research/deep-research/tests/bench/BENCH-RESULTS.md"
RUN_BENCH="skills/research/deep-research/tests/bench/run-bench.sh"
SCORE_RACE="skills/research/deep-research/tests/bench/score-race.sh"
SCORE_FACT="skills/research/deep-research/tests/bench/score-fact.py"
LABEL_CLAIMS="skills/research/deep-research/scripts/label-claims.py"
export KBD_PRODUCER_MODEL="${KBD_PRODUCER_MODEL:-glm-5.3}"

[ -d "$BENCH_G5" ] || { echo "bench-runbook: $BENCH_G5 not found" >&2; exit 2; }

# 1) Label all packages first (closes the drt-006 verified_claim_ratio gap).
for PKG in "$BENCH_G5"/task-*/*/; do
  [ -f "$PKG/sources/registry.json" ] || continue
  echo "label-claims: ${PKG}"
  python3 "$LABEL_CLAIMS" "$PKG" || true
done

# 2) Fix the nested→flat layout for run-bench (which expects <root>/task-<id>/).
for TD in "$BENCH_G5"/task-*/; do
  TASK_ID="$(basename "$TD")"
  # If the dir is a real dir (not already a symlink) and contains a single nested
  # slug-dir, link the slug-dir's contents up to task-<id>/.
  if [ -d "$TD" ] && [ ! -L "$TD" ]; then
    # If task-<id>/report.md exists directly, nothing to do
    if [ -f "$TD/report.md" ]; then continue; fi
    # Find the nested slug-dir
    NESTED="$(find "$TD" -mindepth 1 -maxdepth 1 -type d 2>/dev/null | head -1)"
    [ -n "$NESTED" ] && [ -f "$NESTED/report.md" ] && {
      # Replace the empty task-<id>/dir with a symlink to the nested slug-dir
      rmdir "$TD" 2>/dev/null || true
      ln -sfn "$NESTED" "$TD" 2>/dev/null || true
    }
  fi
done

# 3) Run the bench
echo "run-bench: $RUN_BENCH --packages $BENCH_G5"
bash "$RUN_BENCH" --packages "$BENCH_G5" || true

# 4) Per-task score + append to BENCH-RESULTS.md
TS_NOW="$(date -Iseconds)"
{
  echo ""
  echo "## Bench run — $TS_NOW"
  echo ""
  echo "| Task | RACE | Effective citations | Citation accuracy | Verified-claim ratio | Label |"
  echo "| --- | --- | --- | --- | --- | --- |"
  for TD in "$BENCH_G5"/task-*/; do
    TID="$(basename "$TD")"
    PKG="$(readlink -f "$TD")"
    [ -f "$PKG/report.md" ] || continue
    RACE=$(KBD_PRODUCER_MODEL=glm-5.3 bash "$SCORE_RACE" --package "$PKG" --task-id "$TID" --criteria skills/research/deep-research/tests/bench/criteria/subset-10-criteria.jsonl --judge gpt-5.5 --producer glm-5.3 2>/dev/null | tail -1)
    FACT=$(KBD_PRODUCER_MODEL=glm-5.3 python3 "$SCORE_FACT" --package "$PKG" 2>/dev/null | tr -d '\n ')
    VCR=$(echo "$FACT" | python3 -c "import json,sys; d=json.loads(sys.stdin.read()); print(d.get('verified_claim_ratio','null'))" 2>/dev/null)
    EC=$(echo "$FACT" | python3 -c "import json,sys; d=json.loads(sys.stdin.read()); print(d.get('effective_citations','null'))" 2>/dev/null)
    CA=$(echo "$FACT" | python3 -c "import json,sys; d=json.loads(sys.stdin.read()); print(d.get('citation_accuracy','null'))" 2>/dev/null)
    echo "| $TID | $RACE | $EC | $CA | $VCR | $([ -f "$PKG/sources/registry.json" ] && python3 -c "import json; d=json.load(open('$PKG/sources/registry.json')); ls=[c.get('label','-') for s in d.get('sources',[]) for c in s.get('claims',[])]; v=ls.count('verified'); p=ls.count('partial'); u=ls.count('unverified'); print(f'v={v} p={p} u={u}')" 2>/dev/null || echo "-") |"
  done
} | tee -a "$BENCH_RESULTS" || true

# 5) Summary JSON
SUMMARY="$BENCH_G5/bench-summary.json"
{
  echo "{"
  echo "  \"generated_at\": \"$TS_NOW\","
  echo "  \"packages\": ["
  first=1
  for TD in "$BENCH_G5"/task-*/; do
    TID="$(basename "$TD")"
    PKG="$(readlink -f "$TD")"
    [ -f "$PKG/report.md" ] || continue
    RACE=$(KBD_PRODUCER_MODEL=glm-5.3 bash "$SCORE_RACE" --package "$PKG" --task-id "$TID" --criteria skills/research/deep-research/tests/bench/criteria/subset-10-criteria.jsonl --judge gpt-5.5 --producer glm-5.3 2>/dev/null | tail -1)
    [ $first -eq 0 ] && echo ","
    first=0
    printf "    {\"task_id\": \"%s\", \"RACE\": %s}" "$TID" "${RACE:-null}"
  done
  echo ""
  echo "  ]"
  echo "}"
} > "$SUMMARY" 2>/dev/null || true

echo "bench-runbook: done at $TS_NOW — results appended to $BENCH_RESULTS"
