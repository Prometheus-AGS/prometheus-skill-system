#!/usr/bin/env bash
# SP-011: Cedar-style gate for SKILL.md edits.
#
# Fires as a PreToolUse hook when a Write or Edit tool targets a SKILL.md file.
# Validates the proposed change against skill authoring rules from AGENTS.md:
#   - name field must be lowercase, hyphens only, pattern ^[a-z0-9]+(-[a-z0-9]+)*$
#   - Required frontmatter fields: name, description, license, metadata.tags, metadata.version
#   - No backslashes in path strings
#
# Exit codes:
#   0 — validation passes
#   2 — violation found; blocked with guidance

set -euo pipefail

HOOK_LOG="${HOME}/.prometheus/hooks.log"
mkdir -p "$(dirname "$HOOK_LOG")"

log_violation() {
  local msg="$1"
  echo "[cedar-skill-gate] VIOLATION: ${msg}" | tee -a "$HOOK_LOG" >&2
}

log_info() {
  local msg="$1"
  echo "[cedar-skill-gate] ${msg}" >> "$HOOK_LOG"
}

# Read tool input JSON from stdin
TOOL_INPUT="$(cat)"

# Check if the target file is a SKILL.md
FILE_PATH="$(echo "$TOOL_INPUT" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d.get('file_path','') or d.get('path',''))" 2>/dev/null || echo "")"

if [[ -z "$FILE_PATH" ]] || [[ "$(basename "$FILE_PATH")" != "SKILL.md" ]]; then
  exit 0
fi

log_info "$(date -u +%Y-%m-%dT%H:%M:%SZ) checking SKILL.md edit: ${FILE_PATH}"

# Extract the proposed content (for Write tool) or new_string (for Edit tool)
CONTENT="$(echo "$TOOL_INPUT" | python3 -c "
import json, sys
d = json.load(sys.stdin)
# Write tool: 'content' field; Edit tool: 'new_string' field
print(d.get('content', d.get('new_string', '')))
" 2>/dev/null || echo "")"

if [[ -z "$CONTENT" ]]; then
  log_info "no content to validate; passing"
  exit 0
fi

VIOLATIONS=()

# Rule 1: name field must match ^[a-z0-9]+(-[a-z0-9]+)*$
NAME_LINE="$(echo "$CONTENT" | grep -m1 '^name:' || echo "")"
if [[ -n "$NAME_LINE" ]]; then
  NAME_VALUE="$(echo "$NAME_LINE" | sed 's/^name:[[:space:]]*//' | tr -d '"'"'")"
  if [[ -n "$NAME_VALUE" ]] && ! echo "$NAME_VALUE" | grep -qE '^[a-z0-9]+(-[a-z0-9]+)*$'; then
    VIOLATIONS+=("name '${NAME_VALUE}' violates pattern ^[a-z0-9]+(-[a-z0-9]+)*$ (lowercase, hyphens only, no spaces)")
  fi
fi

# Rule 2: Required frontmatter fields
for FIELD in "name:" "description:" "license:" "metadata:"; do
  if ! echo "$CONTENT" | grep -qE "^${FIELD}|^  ${FIELD}"; then
    VIOLATIONS+=("required frontmatter field missing: ${FIELD}")
  fi
done

# Also check metadata sub-fields (may be indented under metadata:)
for SUBFIELD in "version:" "tags:"; do
  if ! echo "$CONTENT" | grep -qE "^  ${SUBFIELD}|^    ${SUBFIELD}"; then
    VIOLATIONS+=("required metadata sub-field missing: ${SUBFIELD} (must be under metadata:)")
  fi
done

# Rule 3: No backslashes in path strings
if echo "$CONTENT" | grep -qE '\\\\[^n"]'; then
  VIOLATIONS+=("backslashes found in content — use forward slashes for all paths")
fi

if [[ ${#VIOLATIONS[@]} -gt 0 ]]; then
  for v in "${VIOLATIONS[@]}"; do
    log_violation "$v"
  done

  cat >&2 <<EOF

[cedar-skill-gate] BLOCKED — SKILL.md edit violates authoring rules:

EOF
  for v in "${VIOLATIONS[@]}"; do
    echo "  ✗ $v" >&2
  done

  cat >&2 <<EOF

See AGENTS.md for the complete skill authoring rules.
Required frontmatter:
  ---
  name: my-skill-name        # ^[a-z0-9]+(-[a-z0-9]+)*$
  description: "..."
  license: MIT
  metadata:
    author: your-name
    version: '1.0.0'
    tags: [tag1, tag2]
  ---

EOF
  exit 2
fi

log_info "cedar-skill-gate PASS: ${FILE_PATH}"
exit 0
