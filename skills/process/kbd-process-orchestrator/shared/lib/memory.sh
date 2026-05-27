# shellcheck shell=bash
# kbd-process-orchestrator/shared/lib/memory.sh
#
# Detection helper for the optional surreal-memory mirror. Source this file;
# it exports no side effects on import. Soft-fails by design: every probe
# treats failure as "memory unavailable" rather than aborting the caller.
#
#   . shared/lib/memory.sh
#   if kbd_memory_available; then ...; fi
#   url="$(kbd_memory_url)"

_KBD_MEMORY_PROBED=""
_KBD_MEMORY_OK=""
_KBD_MEMORY_URL=""

kbd_memory_url() { printf '%s' "$_KBD_MEMORY_URL"; }

kbd_memory_available() {
  if [[ -n "$_KBD_MEMORY_PROBED" ]]; then
    return "$_KBD_MEMORY_OK"
  fi
  _KBD_MEMORY_PROBED=1

  # 1. Agent's tool list contains the surreal-memory MCP tool name.
  if [[ "${KBD_AVAILABLE_TOOLS:-}" == *create_entity* ]]; then
    _KBD_MEMORY_OK=0
    # URL still helpful for HTTP fallback; pull from env if present.
    _KBD_MEMORY_URL="${UAR_MEMORY_MCP_URL:-${KBD_MEMORY_MCP_URL:-}}"
    return 0
  fi

  # 2. Env var endpoint.
  local url="${UAR_MEMORY_MCP_URL:-${KBD_MEMORY_MCP_URL:-}}"

  # 3. Config file.
  if [[ -z "$url" && -f .kbd-orchestrator/memory.config.json ]] \
     && command -v jq >/dev/null 2>&1; then
    url="$(jq -r '.mcpEndpoint // empty' .kbd-orchestrator/memory.config.json 2>/dev/null || true)"
  fi

  # 4. Probe.
  if [[ -n "$url" ]] && command -v curl >/dev/null 2>&1; then
    if curl -fsS --max-time 2 "$url/healthz" >/dev/null 2>&1; then
      _KBD_MEMORY_OK=0
      _KBD_MEMORY_URL="$url"
      return 0
    fi
  fi

  _KBD_MEMORY_OK=1
  return 1
}
