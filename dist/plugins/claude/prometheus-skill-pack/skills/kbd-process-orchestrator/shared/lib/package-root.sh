# shellcheck shell=bash
# Package location is independent of the consumer cwd and KBD state.
# Capture the loaded library location now: BASH_SOURCE inside a later caller
# may name a different file. Works in source and flattened payload layouts.
_KBD_PACKAGE_ORCHESTRATOR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"

kbd_package_root() {
  local candidate
  for candidate in "${KBD_PACK_ROOT:-}" "${CLAUDE_PLUGIN_ROOT:-${PLUGIN_ROOT:-}}" \
                   "${KBD_ORCHESTRATOR_ROOT:-}/../.." "${KBD_ORCHESTRATOR_ROOT:-}/../../.." \
                   "$_KBD_PACKAGE_ORCHESTRATOR/../.." "$_KBD_PACKAGE_ORCHESTRATOR/../../.." \
                   "${PROMETHEUS_PLUGIN_ROOT:-$HOME/.prometheus/plugins/prometheus-skill-pack}/current"; do
    [[ -n "$candidate" && "$candidate" != "/../.." && "$candidate" != "/../../.." ]] || continue
    # Identify the package independently of the requested helper, so a missing
    # readiness executable remains an installation error, not missing state.
    if [[ -f "$candidate/shared/scripts/lib/learning_write.py" ]]; then
      (cd "$candidate" 2>/dev/null && pwd -P)
      return 0
    fi
  done
  return 0
}
