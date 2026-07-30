#!/usr/bin/env bash
# changelog-fetch.sh — Fetch and extract features from a competitor's GitHub releases
# Usage: changelog-fetch.sh <owner/repo> [--since-tag <tag>] [--evolution-name <name>]
# [MODEL_ROUTING] phase=evolver-changelog-extract class=medium
set -euo pipefail

REPO="${1:?Usage: changelog-fetch.sh <owner/repo> [--since-tag <tag>] [--evolution-name <name>]}"
SINCE_TAG=""
EVOLUTION_NAME="default"

shift
while [[ $# -gt 0 ]]; do
  case "$1" in
    --since-tag) SINCE_TAG="${2:-}"; shift 2 ;;
    --evolution-name) EVOLUTION_NAME="${2:-}"; shift 2 ;;
    *) shift ;;
  esac
done

CHANGELOG_DIR=".evolver/${EVOLUTION_NAME}/changelogs"
mkdir -p "${CHANGELOG_DIR}"

COMPETITOR_ID=$(echo "${REPO}" | tr '/' '-')
TIMESTAMP=$(date -u +%Y%m%dT%H%M%SZ 2>/dev/null || python3 -c "from datetime import datetime,timezone; print(datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ'))")
OUTPUT_FILE="${CHANGELOG_DIR}/${COMPETITOR_ID}-${TIMESTAMP}.json"

echo "[changelog-fetch] Fetching releases for ${REPO} since tag: ${SINCE_TAG:-<all recent>}"

# Check gh is available
if ! command -v gh > /dev/null 2>&1; then
  echo "[changelog-fetch] ERROR: gh CLI not available" >&2
  echo "{\"repo\": \"${REPO}\", \"error\": \"gh CLI not available\", \"features_added\": [], \"breaking_changes\": [], \"deprecations\": []}" > "${OUTPUT_FILE}"
  cat "${OUTPUT_FILE}"
  exit 0
fi

# Fetch releases
if [ -n "${SINCE_TAG}" ]; then
  RELEASES_JSON=$(gh api "repos/${REPO}/releases" --paginate 2>/dev/null | \
    python3 -c "
import json, sys
releases = json.load(sys.stdin)
tag = '${SINCE_TAG}'
filtered = [r for r in releases if r.get('tag_name', '') > tag]
print(json.dumps(filtered[:10]))
" 2>/dev/null || echo "[]")
else
  RELEASES_JSON=$(gh api "repos/${REPO}/releases" 2>/dev/null | \
    python3 -c "import json, sys; print(json.dumps(json.load(sys.stdin)[:5]))" 2>/dev/null || echo "[]")
fi

RELEASE_COUNT=$(echo "${RELEASES_JSON}" | python3 -c "import json,sys; print(len(json.load(sys.stdin)))" 2>/dev/null || echo "0")
echo "[changelog-fetch] Found ${RELEASE_COUNT} releases to process"

if [ "${RELEASE_COUNT}" -eq 0 ]; then
  python3 -c "
import json
from datetime import datetime, timezone
result = {
  'repo': '${REPO}',
  'fetched_at': datetime.now(timezone.utc).isoformat().replace('+00:00', 'Z'),
  'since_tag': '${SINCE_TAG}',
  'release_count': 0,
  'features_added': [],
  'breaking_changes': [],
  'deprecations': []
}
with open('${OUTPUT_FILE}', 'w') as f:
    json.dump(result, f, indent=2)
print(json.dumps(result, indent=2))
"
  exit 0
fi

# Build release notes text for LLM extraction
RELEASE_NOTES=$(echo "${RELEASES_JSON}" | python3 -c "
import json, sys
releases = json.load(sys.stdin)
notes = []
for r in releases:
    tag = r.get('tag_name', 'unknown')
    date = r.get('published_at', '')[:10]
    body = (r.get('body') or '').strip()[:3000]
    notes.append(f'## {tag} ({date})\n{body}')
print('\n\n---\n\n'.join(notes))
" 2>/dev/null || echo "No release notes available")

# [MODEL_ROUTING] phase=evolver-changelog-extract class=medium
echo "[changelog-fetch] Extracting features via model=medium"

# Dispatch over OpenAI REST via the shared helper. This previously called
# `liter-llm complete --model medium` — a subcommand that does not exist (the
# binary ships only `api` and `mcp`) — and paired it with
# `2>/dev/null || echo "{}"`, so the contract mismatch was completely invisible:
# `command -v liter-llm` succeeded, the call failed, and extraction silently
# yielded {} with no warning on that path.
EXTRACTED="{}"
_EXTRACT_LIB=""
for _cand in \
  "$(cd "$(dirname "$0")" && pwd)/../../../../shared/scripts/lib/kbd-model-resolve.sh" \
  "${CLAUDE_PLUGIN_ROOT:-}/shared/scripts/lib/kbd-model-resolve.sh" \
  "${PLUGIN_ROOT:-}/shared/scripts/lib/kbd-model-resolve.sh"; do
  if [ -n "$_cand" ] && [ -f "$_cand" ]; then _EXTRACT_LIB="$_cand"; break; fi
done

if [ -z "$_EXTRACT_LIB" ]; then
  echo "[changelog-fetch] WARN: model-resolve library not found — skipping extraction" >&2
  echo "[changelog-fetch]       (returning raw release count only)" >&2
elif [ -z "${RELEASE_NOTES:-}" ] || [ "${RELEASE_NOTES}" = "No release notes available" ]; then
  echo "[changelog-fetch] no release notes to extract from — skipping model call" >&2
else
  # shellcheck source=/dev/null
  . "$_EXTRACT_LIB"
  _model="$(kbd_resolve_role critic 2>/dev/null || echo kbd-critic)"
  _sys='Extract features, breaking changes, and deprecations from these release notes. Output ONLY valid JSON in this exact format: {"features_added": ["string"], "breaking_changes": ["string"], "deprecations": ["string"]}. No markdown, no explanation.'
  # Errors are REPORTED, not swallowed: a silent {} is indistinguishable from
  # "this release genuinely changed nothing", which is a misleading input to the
  # evolver's next decision.
  if _out="$(kbd_complete "$_model" "$_sys" "${RELEASE_NOTES}" 2048)"; then
    if printf '%s' "$_out" | python3 -c 'import json,sys; json.load(sys.stdin)' 2>/dev/null; then
      EXTRACTED="$_out"
    else
      echo "[changelog-fetch] WARN: model returned non-JSON — keeping empty extraction" >&2
      echo "[changelog-fetch]       first 160 chars: $(printf '%s' "$_out" | head -c 160)" >&2
    fi
  else
    echo "[changelog-fetch] WARN: extraction model call failed (see message above) —" >&2
    echo "[changelog-fetch]       continuing with raw release count only" >&2
  fi
fi

# Combine into final output.
#
# Values are passed through the ENVIRONMENT, never interpolated into the python
# source. The previous version embedded ${EXTRACTED} inside a '''...''' literal,
# which was only safe while extraction was permanently broken and always returned
# "{}". Once real extraction started working, release-note text containing an
# apostrophe (e.g. "a user's computer") terminated the triple-quoted string and
# the block died with a SyntaxError — silenced by `2>/dev/null`, so the script
# exited 1 with no output and no explanation.
EXTRACTED_JSON="$EXTRACTED" RELEASES_RAW="$RELEASES_JSON" \
SINCE_TAG_IN="$SINCE_TAG" REPO_IN="$REPO" \
RELEASE_COUNT_IN="$RELEASE_COUNT" OUTPUT_FILE_IN="$OUTPUT_FILE" \
python3 <<'PY'
import json, os
from datetime import datetime, timezone

try:
    extracted = json.loads(os.environ.get("EXTRACTED_JSON") or "{}")
    if not isinstance(extracted, dict):
        raise ValueError("extraction was not a JSON object")
except Exception as exc:
    print("[changelog-fetch] WARN: could not parse extraction (%s)" % exc)
    extracted = {}

try:
    releases_raw = json.loads(os.environ.get("RELEASES_RAW") or "[]")
except Exception:
    releases_raw = []

tags = [r.get("tag_name", "") for r in releases_raw if isinstance(r, dict)]
from_tag = os.environ.get("SINCE_TAG_IN") or (tags[-1] if tags else "")
to_tag = tags[0] if tags else ""

try:
    release_count = int(os.environ.get("RELEASE_COUNT_IN") or 0)
except ValueError:
    release_count = 0

result = {
    "repo": os.environ.get("REPO_IN", ""),
    "fetched_at": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
    "since_tag": from_tag,
    "to_tag": to_tag,
    "release_count": release_count,
    "features_added": extracted.get("features_added", []),
    "breaking_changes": extracted.get("breaking_changes", []),
    "deprecations": extracted.get("deprecations", []),
}

out_path = os.environ.get("OUTPUT_FILE_IN")
if out_path:
    with open(out_path, "w") as fh:
        json.dump(result, fh, indent=2)
print(json.dumps(result, indent=2))
PY

echo "[changelog-fetch] Output: ${OUTPUT_FILE}"
