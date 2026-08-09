#!/usr/bin/env bash
# verify-trace-state.sh — SP-007 Phase 1 verification script
#
# Documents the actual state of trace capture in the prometheus stack
# and checks that the trace directory is non-empty.
#
# Findings (Phase 1, 2026-05-09):
#   - Traces ARE written — by prometheus-learn (Rust crate), not by shell hooks.
#   - Location: <project_root>/.prometheus/traces/<skill-name>/<timestamp>.json
#   - Schema: JSON object with id, skill_name, timestamp, duration_ms,
#             input_summary, output_summary, tool_calls, errors, classification,
#             score, reflection, project_path.
#   - Hook scripts do NOT write traces (no references to traces/ in shared/scripts/).
#   - Architecture doc (docs/plans/2026-04-29-change-006-karpathy-loop-hooks.md)
#     has no incorrect trace references.
#
# Usage:
#   bash shared/scripts/verify-trace-state.sh
#
# Exit 0 if trace directory is found with at least one trace.
# Exit 1 if no traces exist (informational, not fatal).

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
TRACE_DIR="${PROMETHEUS_TRACE_DIR:-${PROJECT_ROOT}/.prometheus/traces}"

echo "SP-007: trace capture verification"
echo "  Project root:   ${PROJECT_ROOT}"
echo "  Trace location: ${TRACE_DIR}"
echo ""

if [ ! -d "$TRACE_DIR" ]; then
  echo "WARNING: Trace directory does not exist: ${TRACE_DIR}"
  echo ""
  echo "This means prometheus-learn has not run in this project yet."
  echo "Traces are written by tools/prometheus-cli (prometheus-learn Rust crate),"
  echo "not by shell hooks."
  exit 1
fi

skill_count=$(ls -d "${TRACE_DIR}/"*/ 2>/dev/null | wc -l | tr -d ' ')
trace_count=$(find "${TRACE_DIR}" -name "*.json" 2>/dev/null | wc -l | tr -d ' ')

echo "Trace directory exists: YES"
echo "Skills with traces:     ${skill_count}"
echo "Total trace files:      ${trace_count}"
echo ""

if [ "$trace_count" -eq 0 ]; then
  echo "WARNING: No trace files found. prometheus-learn may not have run."
  exit 1
fi

# Show a sample
echo "Most recent traces:"
find "${TRACE_DIR}" -name "*.json" | sort -r | head -5 | while read -r f; do
  skill=$(basename "$(dirname "$f")")
  ts=$(basename "$f" .json)
  printf "  %-40s %s\n" "$skill" "$ts"
done

echo ""
echo "Trace schema (from sample):"
sample=$(find "${TRACE_DIR}" -name "*.json" | sort -r | head -1)
if [ -n "$sample" ] && command -v python3 &>/dev/null; then
  python3 -c "
import json, sys
with open('$sample') as f:
    d = json.load(f)
for k, v in d.items():
    t = type(v).__name__
    print(f'  {k}: {t}')
"
fi

echo ""
echo "SUMMARY:"
echo "  Traces ARE being written by prometheus-learn (Rust crate)."
echo "  Writer: tools/prometheus-cli/crates/prometheus-learn/src/trace.rs"
echo "  Hook scripts do NOT write traces — see shared/scripts/ (no trace writes)."
echo "  PROMETHEUS_TRACE_DIR env var overrides the default location."
echo ""
echo "For SP-019 (LibrarianEvent persistence): traces exist on disk."
echo "The librarian can ingest them asynchronously when SP-019 is implemented."

exit 0
