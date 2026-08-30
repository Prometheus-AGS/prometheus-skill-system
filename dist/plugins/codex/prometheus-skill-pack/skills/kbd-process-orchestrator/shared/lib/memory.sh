# shellcheck shell=bash
# kbd-process-orchestrator/shared/lib/memory.sh
#
# Detection helper for the optional surreal-memory mirror. Source this file;
# it exports no side effects on import. Soft-fails by design: every probe
# treats failure as "memory unavailable" rather than aborting the caller.
#
#   . shared/lib/memory.sh
#   if kbd_memory_available; then ...; fi
#   rest_base="$(kbd_memory_url)"

_KBD_MEMORY_PROBED=""
_KBD_MEMORY_OK=""
_KBD_MEMORY_URL=""

_KBD_MEMORY_DEFAULT_URL="http://127.0.0.1:23001"

# Returns the normalized REST origin, never an MCP transport path.
kbd_memory_url() { printf '%s' "$_KBD_MEMORY_URL"; }

_kbd_memory_normalize_rest_base() {
  local endpoint="${1:-}"
  endpoint="${endpoint%/}"
  case "$endpoint" in
    http://*|https://*) ;;
    *) return 1 ;;
  esac

  # REST routes are hosted at the service origin even when discovery came from
  # an MCP transport URL such as /mcp/sse or /mcp/http.
  printf '%s' "$endpoint" | sed -E 's#^(https?://[^/]+).*$#\1#'
}

kbd_memory_available() {
  if [[ -n "$_KBD_MEMORY_PROBED" ]]; then
    return "$_KBD_MEMORY_OK"
  fi
  _KBD_MEMORY_PROBED=1
  _KBD_MEMORY_URL=""

  # 1. Explicit endpoint override.
  local endpoint="${UAR_MEMORY_MCP_URL:-${KBD_MEMORY_MCP_URL:-}}"

  # 2. Project configuration. A REST-specific endpoint wins when supplied;
  # legacy mcpEndpoint remains supported and is normalized to the same origin.
  if [[ -z "$endpoint" && -f .kbd-orchestrator/memory.config.json ]] \
     && command -v jq >/dev/null 2>&1; then
    endpoint="$(jq -r '.restEndpoint // .mcpEndpoint // empty' \
      .kbd-orchestrator/memory.config.json 2>/dev/null || true)"
  fi

  # 3. Canonical local installation. This is a discovery default, not an
  # override: an explicit or project-configured endpoint is never replaced.
  [[ -n "$endpoint" ]] || endpoint="$_KBD_MEMORY_DEFAULT_URL"

  # 4. Probe the REST health route at the normalized service origin.
  local rest_base=""
  rest_base="$(_kbd_memory_normalize_rest_base "$endpoint" 2>/dev/null || true)"
  if [[ -n "$rest_base" ]] && command -v curl >/dev/null 2>&1; then
    if curl --noproxy '127.0.0.1,localhost,::1' -fsS \
      --connect-timeout 1 --max-time 2 "${rest_base}/health" \
      >/dev/null 2>&1; then
      _KBD_MEMORY_OK=0
      _KBD_MEMORY_URL="$rest_base"
      return 0
    fi
  fi

  # An in-process MCP tool can still satisfy agent-owned memory operations,
  # but shell callers intentionally receive an empty REST URL in this mode.
  if [[ "${KBD_AVAILABLE_TOOLS:-}" == *create_entity* ]]; then
    _KBD_MEMORY_OK=0
    return 0
  fi

  _KBD_MEMORY_OK=1
  return 1
}
