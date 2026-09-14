#!/usr/bin/env bash
# Fixture judge for tests/driver-contract.sh (change-rah-006).
#
# Invoked by run-research.sh through RESEARCH_JUDGE_CMD as:
#   judge.sh <packet.json> <findings.json>
#
# It stands in for adversarial-review's dispatch-judge.sh and writes a
# shape-valid findings document. It also PROVES the driver sent a real research
# packet: a packet without research_report, research_provenance, and
# research_plan makes it exit 7, so a driver that skipped the packet build (or
# built the wrong target) fails the scenario instead of passing on an empty judge.
#
# Knobs (environment):
#   FIXTURE_JUDGE=pass|critical|warning|unavailable   (default pass)
#   FIXTURE_JUDGE_LOG=<file>  one line per call: "<stages_completed at call time> <verdict>"
set -euo pipefail
PACKET="${1:?packet path}"; OUT="${2:?findings path}"
PKG="$(cd "$(dirname "$PACKET")/.." && pwd)"
MODE="${FIXTURE_JUDGE:-pass}"

if [ "$MODE" = "unavailable" ]; then
  echo "[fixture judge] simulating no reachable gateway" >&2
  exit 3
fi
for key in research_report research_provenance research_plan goals review_focus truncation; do
  jq -e --arg k "$key" 'has($k) and .[$k] != null' "$PACKET" >/dev/null 2>&1 || { echo "[fixture judge] packet lacks $key" >&2; exit 7; }
done
[ "$(jq -r '.target' "$PACKET")" = "research" ] || { echo "[fixture judge] packet target is not research" >&2; exit 7; }

case "$MODE" in
  critical) FINDINGS='[{"severity":"CRITICAL","file":"report.md","line":0,"claim":"executive summary states a figure no evidence row carries","evidence":"fixture: \"fixture claim\" row has no numeric value"},{"severity":"WARNING","file":"plan.md","claim":"sub-question Q3 is not answered","evidence":"fixture: report has no section for Q3"}]'; VERDICT=BLOCK ;;
  warning)  FINDINGS='[{"severity":"WARNING","file":"report.md","claim":"one inferred claim appears in the executive summary without the inline marker","evidence":"fixture: summary sentence 2"}]'; VERDICT=PASS ;;
  pass)     FINDINGS='[]'; VERDICT=PASS ;;
  *) echo "[fixture judge] unknown FIXTURE_JUDGE=$MODE" >&2; exit 2 ;;
esac
jq -n --arg v "$VERDICT" --argjson f "$FINDINGS" \
  '{mode:"artifact", verdict:$v, judge_model:"fixture/judge", isolation_mode:"fixture", producer_model:"fixture/producer", cross_model_check:"verified-distinct", findings:$f, checked_classes: (if ($f|length)==0 then ["fixture: all classes checked, none apply"] else [] end)}' > "$OUT"
if [ -n "${FIXTURE_JUDGE_LOG:-}" ]; then
  # What the judge saw: stages at call time, the frontmatter status in the packet,
  # whether the packet's sidecar said the review was pending, and the producer.
  stages="$(jq -r '.stages_completed | join(" ")' "$PKG/checkpoint.json" 2>/dev/null || echo "?")"
  seen="$(jq -r '.research_report' "$PACKET" | grep -o '^verification_status: [a-z]*' | head -1 | sed 's/verification_status: //')"
  pending="$(jq -r '.research_provenance' "$PACKET" | grep -q 'Adversarial review:\*\* pending' && echo pending || echo not-pending)"
  producer="$(jq -r '.producer_model' "$PACKET")"
  printf '%s | %s | saw=%s | %s | producer=%s\n' "$stages" "$VERDICT" "${seen:-?}" "$pending" "$producer" >> "$FIXTURE_JUDGE_LOG"
fi
exit 0
