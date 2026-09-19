#!/usr/bin/env bash
# write-provenance.sh <package_dir> [--interim]
#
# Writes <slug>.provenance.md beside report.md from the package's real state.
# The driver calls it on every exit path (success, block, interruption) and once
# before export so the manifest can copy the verdict. --interim marks a run that
# is still waiting on a stage (checkpoint mode) and never produces BLOCKED.
#
# Verdict rule (research-package-spec.md):
#   BLOCKED         checkpoint.status is "blocked", the run ended without completing,
#                   or the adversarial review of the report returned BLOCK (a CRITICAL
#                   finding); the run still exports so it is auditable
#   PASS WITH NOTES status complete but: scale is direct, the checkpoint carries notes,
#                   any hook exited non-zero, the adversarial review was not run for a
#                   full run, or the run is interim
#   PASS            status complete and none of the above
#
# bash 3.2 compatible (constraint C-05).
set -euo pipefail

PKG="${1:-}"; INTERIM=0
[ "${2:-}" = "--interim" ] && INTERIM=1
[ -d "$PKG" ] || { echo "[write-provenance] package directory required" >&2; exit 1; }
CP="$PKG/checkpoint.json"
[ -f "$CP" ] || { echo "[write-provenance] no checkpoint.json in $PKG" >&2; exit 1; }
command -v jq >/dev/null 2>&1 || { echo "[write-provenance] jq is required" >&2; exit 1; }

PACKAGE_ID="$(basename "$PKG")"
q()   { jq -r "$1" "$CP"; }
# The slug is recorded by the driver at package creation; the suffix-stripping
# form is only the fallback for a checkpoint written before that field existed.
SLUG="$(q '.slug // ""')"
[ -n "$SLUG" ] || SLUG="${PACKAGE_ID%-[0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9]-[0-9a-f][0-9a-f][0-9a-f][0-9a-f]}"
OUT="$PKG/$SLUG.provenance.md"
QUERY="$(q '.query // "query not recorded"')"
STATUS="$(q '.status // "unknown"')"
SCALE="$(q '.scale // "full"')"
CREATED="$(q '.created_at // "unknown"')"
COMPLETED="$(q '.completed_at // "not completed"')"
STAGES_DONE="$(q '.stages_completed | join(" ")')"
STAGES_PLANNED="$(q '.stages_planned | join(" ")')"
# The blocked line reflects the current state only: a recovered run shows "none".
BLOCKED="$(q 'if .status == "blocked" and .blocked then "stage " + (.blocked.stage|tostring) + ": " + .blocked.reason else "none" end')"
NOTES="$(q '.notes | map("  - " + .) | join("\n")')"
HOOK_FAILS="$(q '[.hook_log[] | select(.exit != 0)] | length')"
REVIEW="$(q 'if .review then ((.review.verdict // "unknown") + " (" + ((.review.critical // 0)|tostring) + " CRITICAL, " + ((.review.warning // 0)|tostring) + " WARNING)") elif .integrations.adversarial_review_used == true then "run (details not recorded)" elif (.blocked_review // null) then .blocked_review else "not run" end')"
# An interim sidecar is written before the review runs (the review packet
# carries it): say the review is pending, not that it was skipped.
[ "$INTERIM" -eq 1 ] && [ "$REVIEW" = "not run" ] && REVIEW="pending (this sidecar was written before the report review)"
REVIEW_VERDICT="$(q '.review.verdict // ""')"
REVIEW_CRITICAL="$(q '.review.critical // 0')"
REVIEW_WARNINGS="$(q '(.review.warnings // []) | map("  - " + .) | join("\n")')"

count_json() { # count_json <file> <jq length expr>
  [ -f "$1" ] && jq -r "$2" "$1" 2>/dev/null || echo 0
}
CONSULTED="$(count_json "$PKG/sources/url-list.json" '.source_urls | length')"
ACCEPTED="$(count_json "$PKG/sources/credibility.json" 'if .verified_sources then (.verified_sources|length) elif .credibility_scores then (.credibility_scores|length) else 0 end')"
REJECTED="$(count_json "$PKG/sources/credibility.json" 'if .filtered_sources then (.filtered_sources|length) else 0 end')"
REJECT_REASON="below credibility threshold"
[ "$REJECTED" = "0" ] && REJECT_REASON="none rejected"

# Verdict
if [ "$STATUS" = "blocked" ]; then
  VERDICT="BLOCKED"
elif [ "$INTERIM" -eq 1 ] || [ "$STATUS" != "complete" ]; then
  if [ "$INTERIM" -eq 1 ]; then VERDICT="PASS WITH NOTES"; else VERDICT="BLOCKED"; [ "$BLOCKED" = "none" ] && BLOCKED="run ended with status $STATUS before completing"; fi
elif [ "$REVIEW_VERDICT" = "BLOCK" ]; then
  # A CRITICAL review finding blocks the verdict even though the run completed
  # and exported: the package is auditable, not deliverable as verified.
  VERDICT="BLOCKED"
  [ "$BLOCKED" = "none" ] && BLOCKED="adversarial review: $REVIEW_CRITICAL CRITICAL finding(s) (review/findings.json)"
else
  VERDICT="PASS"
  if [ "$SCALE" = "direct" ] || [ -n "$NOTES" ] || [ "$HOOK_FAILS" != "0" ]; then VERDICT="PASS WITH NOTES"; fi
  if [ "$SCALE" = "full" ] && [ "$REVIEW" = "not run" ] && echo " $STAGES_DONE " | grep -q ' 09 '; then VERDICT="PASS WITH NOTES"; fi
fi

STAGE_FILES=""
for f in plan.md report.md graph.json citations.json contradictions.json manifest.json index.md checkpoint.json sensitivity.json; do
  [ -f "$PKG/$f" ] && STAGE_FILES="$STAGE_FILES$f "
done
SOURCE_FILES=0
[ -d "$PKG/sources" ] && SOURCE_FILES="$(find "$PKG/sources" -type f | wc -l | tr -d ' ')"

{
  echo "# Provenance: $QUERY"
  echo
  echo "- **Package:** $PACKAGE_ID"
  echo "- **Date:** $CREATED to $COMPLETED"
  echo "- **Scale:** $SCALE"
  echo "- **Stages planned:** ${STAGES_PLANNED:-none}"
  echo "- **Stages completed:** ${STAGES_DONE:-none}"
  echo "- **Sources consulted:** $CONSULTED"
  echo "- **Sources accepted:** $ACCEPTED"
  echo "- **Sources rejected:** $REJECTED ($REJECT_REASON)"
  echo "- **Verification:** $VERDICT"
  echo "- **Blocked:** $BLOCKED"
  echo "- **Adversarial review:** $REVIEW"
  if [ -n "$REVIEW_WARNINGS" ]; then echo "- **Review warnings:**"; echo "$REVIEW_WARNINGS"; fi
  echo "- **Plan:** plan.md"
  echo "- **Stage files:** ${STAGE_FILES:-none}; sources/ holds $SOURCE_FILES file(s)"
  if [ -n "$NOTES" ]; then echo "- **Notes:**"; echo "$NOTES"; fi
  [ "$INTERIM" -eq 1 ] && echo "- **State:** interim; the run has not completed (awaiting a stage, or under review)"
  echo
  echo "Written by scripts/write-provenance.sh at $(date -u +"%Y-%m-%dT%H:%M:%SZ") from checkpoint.json and the package artifacts. A verdict here is only as good as the checkpoint the driver wrote; the driver writes it on every exit path."
} > "$OUT"
echo "$OUT"
