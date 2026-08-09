#!/usr/bin/env bash
# validate-gitops-write.sh — PostToolUse hook: validates written files conform to TJ-CICD-001
set -euo pipefail

HOOK_LOG_LIB="$(cd "$(dirname "$0")" && pwd)/lib/hook-log.sh"
[ -f "$HOOK_LOG_LIB" ] && source "$HOOK_LOG_LIB"
hook_log_start "PostToolUse" "validate-gitops-write.sh"

INPUT=$(cat)
FILE=$(echo "$INPUT" | jq -r '.tool_input.file_path // ""' 2>/dev/null || echo "")

if [ -z "$FILE" ]; then
  hook_log_end 0
  exit 0
fi

if [ ! -f "$FILE" ]; then
  hook_log_end 0
  exit 0
fi

# Check workflow files for violations
if echo "$FILE" | grep -q ".github/workflows"; then
  if grep -q "kubectl apply" "$FILE" 2>/dev/null; then
    echo "WARNING: $FILE contains kubectl apply — verify this is not a production deploy step" >&2
  fi
  if grep -qE "credentials_json|AWS_ACCESS_KEY_ID|AZURE_CREDENTIALS" "$FILE" 2>/dev/null; then
    echo "WARNING: $FILE may contain static credentials — migrate to keyless OIDC" >&2
  fi
fi

# Check Kubernetes manifest files for Secret values
if echo "$FILE" | grep -qE "\.ya?ml$"; then
  if grep -q "^kind: Secret" "$FILE" 2>/dev/null && grep -q "^  data:\|^  stringData:" "$FILE" 2>/dev/null; then
    echo "WARNING: $FILE is a Kubernetes Secret with encoded values — use ExternalSecret CR instead" >&2
  fi
fi

hook_log_end 0
exit 0
