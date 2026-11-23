#!/usr/bin/env bash
set -eu
skill="$(cd "$(dirname "$0")/.." && pwd -P)"
for root in "${KBD_ORCHESTRATOR_ROOT:-}" "$skill/../.." "$skill/../kbd-process-orchestrator"; do
  helper="$root/shared/lib/finalize-reflection.sh"
  if [[ -n "$root" && -r "$helper" ]]; then exec bash "$helper" "$@"; fi
done
echo 'kbd-reflect: installation missing kbd-process-orchestrator/shared/lib/finalize-reflection.sh' >&2
exit 2
