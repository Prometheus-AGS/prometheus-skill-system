#!/usr/bin/env bash
# Fixture stage runner for tests/driver-contract.sh.
#
# Invoked by run-research.sh as: stage-runner.sh <stage> <package_dir>
# Writes a schema-valid artifact for the stage (see references/stage-contracts.md)
# and appends "<stage>" to $FIXTURE_CALLS so a test can assert which stages ran.
#
# Knobs (environment):
#   FIXTURE_SUBQ=<n>          number of sub-question bullets in plan.md (default 4)
#   FIXTURE_FAIL_STAGE=<NN>   write an INVALID artifact for stage NN (contract violation)
#   FIXTURE_EXIT_STAGE=<NN>   exit 9 without writing anything for stage NN (runner failure)
#   FIXTURE_UNRESOLVED=1      contradictions.json carries one unresolved entry
#   FIXTURE_DROP_05_AT_09=1   stage 09 deletes sources/credibility.json after writing the
#                             report (a package that reaches review without verification)
#   FIXTURE_REPORT_GATE=pass  stage 09 records a passing Feynman gate (feynman_grade 0.85,
#                             misconceptions_absent 1.0, feynman_gate_used) and writes
#                             verification_status: verified provisionally; the driver must
#                             confirm or lower it after the review
#   FIXTURE_CALLS=<file>      call log
set -euo pipefail
STAGE="$1"; PKG="$2"
[ -n "${FIXTURE_CALLS:-}" ] && echo "$STAGE" >> "$FIXTURE_CALLS"
if [ "${FIXTURE_EXIT_STAGE:-}" = "$STAGE" ]; then echo "[fixture] stage $STAGE: simulated runner failure" >&2; exit 9; fi
BAD=0; [ "${FIXTURE_FAIL_STAGE:-}" = "$STAGE" ] && BAD=1
mkdir -p "$PKG/sources"
case "$STAGE" in
  01)
    n="${FIXTURE_SUBQ:-4}"
    { echo "# Research Plan: fixture"; echo; echo "## Sub-questions"; echo;
      if [ "$BAD" -eq 0 ]; then i=1; while [ "$i" -le "$n" ]; do echo "- Sub-question $i"; i=$((i+1)); done; fi
      echo; echo "## Search Strategy"; echo; echo "fixture"; } > "$PKG/plan.md" ;;
  02) if [ "$BAD" -eq 0 ]; then echo '{"source_urls":["https://example.org/a","https://example.org/b","https://example.org/c"],"search_metadata":[],"total_found":3,"filtered_count":0}'; else echo '{"urls":"nope"}'; fi > "$PKG/sources/url-list.json" ;;
  03) if [ "$BAD" -eq 0 ]; then echo '{"url":"https://example.org/a","chunk_id":"c1","text":"fixture text","token_count":2,"content_type":"text/html"}' > "$PKG/sources/chunk-1.json"; else echo '{"broken":true}' > "$PKG/sources/chunk-1.json"; fi ;;
  04) if [ "$BAD" -eq 0 ]; then echo '{"sources":[{"url":"https://example.org/a","entity_id":"e1","claims":["fixture claim"],"chunk_ids":["c1"]}]}'; else echo '{}'; fi > "$PKG/sources/registry.json" ;;
  05) if [ "$BAD" -eq 0 ]; then echo '{"verified_sources":[{"url":"https://example.org/a","credibility_score":77,"flags":[],"claims":[{"text":"fixture claim","label":"verified","evidence":"chunk-1: \"fixture text\""}]}],"filtered_sources":[{"url":"https://example.org/c","credibility_score":20}],"credibility_scores":{"https://example.org/a":77}}'; else echo '{"scores":"invalid"}'; fi > "$PKG/sources/credibility.json" ;;
  06) if [ "${FIXTURE_UNRESOLVED:-0}" = "1" ]; then echo '{"contradictions":[{"id":"contra-001","topic":"fixture topic","claim_a":{"text":"A"},"claim_b":{"text":"B"},"strategy_tried":"consensus","resolved":false,"resolution":null,"confidence":0.4,"label":"blocked","audit_trail":"fixture"}]}'; elif [ "$BAD" -eq 0 ]; then echo '{"contradictions":[]}'; else echo '[]'; fi > "$PKG/contradictions.json" ;;
  07) if [ "$BAD" -eq 0 ]; then echo '{"topics":[{"id":"topic-001","name":"fixture","claims":["claim-001"]}],"claims":[{"id":"claim-001","text":"fixture claim","label":"verified","critical":true,"confidence":0.8,"sources":["https://example.org/a"],"contradicts":[],"evidence":"chunk-1: \"fixture text\""}],"relations":[{"from":"claim-001","to":"https://example.org/a","type":"cites"}]}'; else echo '{"graph":null}'; fi > "$PKG/graph.json" ;;
  08) if [ "$BAD" -eq 0 ]; then echo '{"style":"APA","citations":[{"id":"cite-001","url":"https://example.org/a","formatted":"Example (2026). Fixture.","credibility_score":77,"confidence":0.8,"label":"verified"}]}'; else echo '{"style":"APA"}'; fi > "$PKG/citations.json" ;;
  09) # By default the fixture runs no Feynman gate, so the honest provisional label is
      # partial (okf-research-format.md). With FIXTURE_REPORT_GATE=pass it records a
      # passing gate and writes verified provisionally; the review decides the final value.
      if [ "${FIXTURE_REPORT_GATE:-}" = "pass" ]; then
        VS=verified; FG=0.85; MA=1.0
        [ -f "$PKG/checkpoint.json" ] && jq '.integrations.feynman_gate_used = true | .feynman_grade = 0.85 | .misconceptions_absent = 1.0' "$PKG/checkpoint.json" > "$PKG/checkpoint.json.tmp" && mv "$PKG/checkpoint.json.tmp" "$PKG/checkpoint.json"
      else VS=partial; FG=null; MA=null; fi
      if [ "$BAD" -eq 0 ]; then printf -- '---\ntype: research-report\ntitle: "Fixture report"\nquery: "fixture"\ndate: "2026-09-05"\nconfidence: 0.8\nverification_status: %s\nfeynman_grade: %s\nmisconceptions_absent: %s\nsources_count: 1\ncontradictions_resolved: 0\nokf_version: "0.1"\n---\n\n# Fixture report\n\n## Executive Summary\n\nfixture claim [1].\n\n## Evidence Table\n\n| Claim | Label | Source | Credibility | Confidence |\n|---|---|---|---|---|\n| fixture claim * | verified | [1] | 77 | 0.8 |\n\n## References\n\n1. https://example.org/a\n' "$VS" "$FG" "$MA"; else printf 'no front matter\n'; fi > "$PKG/report.md"
      # Tampering after verification: the review step must refuse without stage 05's artifact.
      [ "${FIXTURE_DROP_05_AT_09:-0}" = "1" ] && rm -f "$PKG/sources/credibility.json" ;;
  10) echo "[fixture] stage 10 is run by the driver via export-package.sh; runner should not be called" >&2; exit 8 ;;
  *) echo "[fixture] unknown stage $STAGE" >&2; exit 2 ;;
esac
exit 0
