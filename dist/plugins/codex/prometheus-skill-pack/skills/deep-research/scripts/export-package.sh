#!/usr/bin/env bash
# Assemble a research package: write index.md and a schema-complete manifest.json
# from the package's real state, then fire hooks/post-export.sh.
#
# Usage:
#   bash export-package.sh <package_dir | package_id> [--ingest-palace] [--output-root DIR]
#   PACKAGE_DIR=... bash export-package.sh
#
# The manifest contract is references/schemas/research-manifest.schema.json and
# references/research-package-spec.md (normative). Every required field is emitted;
# values that no artifact provides are emitted as null/0/false and named in a
# `[export-package] defaulted:` line on stderr so a thin package is visible, never
# silently complete. Nothing here is hardcoded to a fixed value.
#
# Package state it reads (all optional except the directory itself):
#   checkpoint.json        job_id, query, depth, scale, citation_style, kb_ids,
#                          model_routing, stages_completed, created_at,
#                          completed_at, integrations{...}
#   report.md              OKF frontmatter: confidence, verification_status,
#                          feynman_grade, misconceptions_absent
#   <slug>.provenance.md   "- **Verification:** PASS|PASS WITH NOTES|BLOCKED"
#   sources/registry.json  sources_count
#   graph.json             claims_count
#   contradictions.json    detected/resolved/unresolved
#
# Requires jq and python3 (the manifest and index.md are assembled by an
# embedded python3 block); both are checked up front so a missing one is a
# named error, not a mid-run "command not found".
# bash 3.2 compatible (constraint C-05): no mapfile, no declare -A.
set -euo pipefail

log() { echo "[export-package] $*" >&2; }
die() { echo "{\"error\": \"$*\"}" >&2; exit 1; }
command -v jq >/dev/null 2>&1 || die "jq is required"
command -v python3 >/dev/null 2>&1 || die "python3 is required"

INGEST_PALACE=0
OUTPUT_ROOT="${RESEARCH_OUTPUT_DIR:-$HOME/.prometheus/research}"
TARGET="${PACKAGE_DIR:-}"
while [ $# -gt 0 ]; do
  case "$1" in
    --ingest-palace) INGEST_PALACE=1 ;;
    --output-root) shift; OUTPUT_ROOT="${1:-}" ;;
    --help|-h) sed -n '2,25p' "$0"; exit 0 ;;
    -*) die "unknown flag $1" ;;
    *) TARGET="$1" ;;
  esac
  shift
done
[ -n "$TARGET" ] || die "package directory or package id is required"

# Accept a directory path or a package id under the output root.
if [ -d "$TARGET" ]; then
  PKG_DIR="$(cd "$TARGET" && pwd)"
elif [ -d "$OUTPUT_ROOT/$TARGET" ]; then
  PKG_DIR="$(cd "$OUTPUT_ROOT/$TARGET" && pwd)"
else
  die "package not found: $TARGET (looked in $OUTPUT_ROOT)"
fi
# The package id is the driver's record in checkpoint.json when present (fixtures and
# relocated packages may sit in a directory with another name); otherwise the directory name.
PACKAGE_ID="$(basename "$PKG_DIR")"
if [ -f "$PKG_DIR/checkpoint.json" ]; then
  CP_ID="$(jq -r '.package_id // empty' "$PKG_DIR/checkpoint.json" 2>/dev/null || true)"
  [ -n "$CP_ID" ] && PACKAGE_ID="$CP_ID"
fi
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SCHEMA="$SCRIPT_DIR/../references/schemas/research-manifest.schema.json"

log "Assembling package $PACKAGE_ID at $PKG_DIR"
mkdir -p "$PKG_DIR/sources"
# Stage artifacts the manifest must point at. A missing one is written empty so
# the package is structurally complete, and each such file is named in the
# `defaulted:` line so a thin package is visible, never silently complete.
FABRICATED=""
[ -f "$PKG_DIR/graph.json" ]          || { echo '{"topics":[],"claims":[],"relations":[]}' > "$PKG_DIR/graph.json"; FABRICATED="$FABRICATED graph.json(empty)"; }
[ -f "$PKG_DIR/citations.json" ]      || { echo '{"style":"APA","citations":[]}' > "$PKG_DIR/citations.json"; FABRICATED="$FABRICATED citations.json(empty)"; }
[ -f "$PKG_DIR/contradictions.json" ] || { echo '{"contradictions":[]}' > "$PKG_DIR/contradictions.json"; FABRICATED="$FABRICATED contradictions.json(empty)"; }

# Build manifest.json from package state. Python assembles the JSON so that no
# value passes through shell interpolation.
# The program lives beside this script rather than inside a heredoc (the
# heredoc form hangs indefinitely on some hosts, observed 2026-09-09).
# Deliberately not `exec`: the palace-ingest block below must still run.
_HERE="$(cd "$(dirname "$0")" && pwd)"
[ -f "$_HERE/export-package.py" ] || { echo "[export-package] export-package.py missing beside this script" >&2; exit 1; }
PKG_DIR="$PKG_DIR" PACKAGE_ID="$PACKAGE_ID" SCHEMA="$SCHEMA" FABRICATED="$FABRICATED" \
  python3 "$_HERE/export-package.py" || exit $?

if [ "$INGEST_PALACE" -eq 1 ]; then
  # palace_ingest is an MCP tool; bash cannot call it. Leave a marker the owning
  # harness session consumes and removes after ingestion.
  date -u +"%Y-%m-%dT%H:%M:%SZ" > "$PKG_DIR/.palace-ingest-requested"
  log "palace ingestion requested; marker written for the harness session"
fi

echo "{\"status\": \"exported\", \"package_id\": \"$PACKAGE_ID\", \"package_dir\": \"$PKG_DIR\", \"ingest_palace\": $INGEST_PALACE}"
log "Package assembled at: $PKG_DIR"

HOOK_DIR="$SCRIPT_DIR/../hooks"
# The driver (run-research.sh) fires post-export itself so it can record the real
# exit code in the checkpoint; it sets RESEARCH_POST_EXPORT_BY_CALLER=1 to say so.
if [ "${RESEARCH_POST_EXPORT_BY_CALLER:-0}" = "1" ]; then
  log "post-export hook deferred to the caller"
elif [ -x "$HOOK_DIR/post-export.sh" ]; then
  # A failing hook is a failed export: capture the hook's own status (not the
  # status of a negation) and propagate it. set -e is suspended only for the call.
  set +e
  RESEARCH_PACKAGE_PATH="$PKG_DIR" PACKAGE_PATH="$PKG_DIR" RESEARCH_JOB_ID="$PACKAGE_ID" bash "$HOOK_DIR/post-export.sh"
  rc=$?
  set -e
  if [ "$rc" -ne 0 ]; then
    log "post-export hook failed with exit $rc; the package is assembled but the export is not complete"
    exit "$rc"
  fi
fi
