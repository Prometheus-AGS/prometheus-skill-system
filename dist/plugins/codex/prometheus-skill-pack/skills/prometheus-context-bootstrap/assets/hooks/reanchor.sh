#!/usr/bin/env bash
# SessionStart + PreCompact — re-inject position, not payload.
#
# stdout is injected into context, so this prints pointers and a handful of
# facts. It must not print file contents: re-injecting the rules on every
# compaction would defeat the reason they were made lean.

set -uo pipefail

root="${CLAUDE_PROJECT_DIR:-$PWD}"

echo "## RE-ANCHOR"

# Select only fields schemaVersion 5 actually carries. This asked for
# `{phase, task, waypoint}` until 2026-09-18; `task` and `waypoint` are not in
# that schema, so every session re-anchored on two nulls and no next step.
#
# `nextChange`/`nextTask` are derived from task state on every projection, so
# they follow completion. `exactNextCommand` is a stored operator string that
# only a plan revision or an explicit path change rewrites — it can name work
# five changes stale, so it is intent, not a selector. Print it labelled.
wp="$root/.kbd-orchestrator/current-waypoint.json"
if [[ -f "$wp" ]]; then
  if command -v jq >/dev/null 2>&1; then
    echo "Waypoint: $(jq -c '{phase, childPointer, change, currentTask, nextChange, nextTask, status}' "$wp" 2>/dev/null || echo '<unparseable>')"
    note="$(jq -r '.exactNextCommand // empty' "$wp" 2>/dev/null)"
    [[ -n "$note" ]] && echo "Operator note (intent, NOT a work selector): $note"
  else
    echo "Waypoint: $wp (jq absent; read it directly)"
  fi
else
  echo "Waypoint: none — run /kbd-init"
fi

[[ -f "$root/versions.toml" ]] && \
  echo "Pins: versions.toml is authoritative for architecture and dependencies."

echo "Authority: signed canonical KBD status/decisions and adopted phase contract; waypoint is a derived position view. Pins remain in versions.toml."

# Recover compact references, not a second policy authority or a pasted payload.
if [[ -f "$wp" ]] && command -v jq >/dev/null 2>&1; then
  method_dir="$root/.kbd-orchestrator/phases"
  method_path_ok=1
  method_count=0
  while IFS= read -r component; do
    case "$component" in ''|.|..|*/*|*\\*|*..*) method_path_ok=0; break ;; esac
    if [[ "$method_count" -gt 0 ]]; then method_dir="$method_dir/children"; fi
    method_dir="$method_dir/$component"
    method_count=$((method_count + 1))
  done < <(jq -r 'if ((.path // []) | length) > 0 then .path[] else (.phase // empty) end' "$wp" 2>/dev/null)
  if [[ "$method_path_ok" == 1 && "$method_count" -gt 0 ]]; then
    echo "Method recovery: $method_dir/method-context.md; contract: $method_dir/adopted-contract.json"
    if [[ -f "$method_dir/method-context.json" ]]; then
      echo "Method snapshot (derived; re-resolve canonical authority): $(jq -c '{selectedMethod, phaseId, sourceRevision, status, blockers, policyConflicts, control}' "$method_dir/method-context.json" 2>/dev/null || echo '<unparseable>')"
    else
      echo "Method snapshot unavailable: resolve selected method and contract before dependent work; do not infer them from old commands or memory."
    fi
  fi
fi
echo "Recovery preserves latest supersession, original evidence, candidate freshness, consumed budgets and unresolved effects. Reconcile uncertain effects before retry."
echo "For selected VLBV: only the complete agreed value-phase boundary with separate authority permits generated-code verification. Host-policy conflicts remain explicit; no generic tool interception is claimed."

# Surface the most recent learned constraints. Three lines, not the file.
g="$root/.prometheus/gotchas.md"
if [[ -s "$g" ]]; then
  echo "Recent gotchas:"
  grep -E '^[-*] ' "$g" 2>/dev/null | tail -3 | sed 's/^/  /'
fi

echo "Rules: AGENTS.md is resident. Stack detail loads from .claude/rules/ on file read."
exit 0
