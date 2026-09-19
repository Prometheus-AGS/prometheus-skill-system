#!/usr/bin/env bash
# Thin wrapper (change-rah-008): the one grounding script is
# shared/scripts/content-grounding-kb.sh. This file locates it and execs it with
# the same arguments, so every entry point emits byte-identical corpora.
# Roots tried: explicit plugin/repo root, installed generation
# (<root>/skills/<skill>/scripts), repository (<root>/skills/learn/<skill>/scripts).
set -euo pipefail
_self_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
for _root in "${CLAUDE_PLUGIN_ROOT:-}" "${PLUGIN_ROOT:-}" "${REPO_ROOT:-}" \
             "${_self_dir}/../../.." "${_self_dir}/../../../.."; do
  [ -n "$_root" ] && [ -f "${_root}/shared/scripts/content-grounding-kb.sh" ] || continue
  exec "${BASH:-bash}" "${_root}/shared/scripts/content-grounding-kb.sh" "$@"   # same interpreter as this wrapper
done
echo "[content-grounding-kb] ERROR: shared/scripts/content-grounding-kb.sh not found from ${_self_dir}; set CLAUDE_PLUGIN_ROOT" >&2
echo '{"status":"error","message":"shared content-grounding-kb.sh not found; set CLAUDE_PLUGIN_ROOT"}'
exit 1
