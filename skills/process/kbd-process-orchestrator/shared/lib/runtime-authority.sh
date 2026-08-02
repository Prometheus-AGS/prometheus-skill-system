#!/usr/bin/env bash
# shellcheck shell=bash
# Reject direct compatibility-projection writes after KBD runtime migration.

kbd_runtime_authoritative() {
  local root="${1:-.}"
  local waypoint="$root/.kbd-orchestrator/current-waypoint.json"
  [ -f "$waypoint" ] || return 1
  command -v jq >/dev/null 2>&1 || return 1
  [ "$(jq -r '.generatedBy // empty' "$waypoint" 2>/dev/null)" = "kbd-runtime" ]
}

kbd_projection_write_guard() {
  local root="${1:-.}"
  local target="${2:-.kbd-orchestrator projection}"
  if kbd_runtime_authoritative "$root"; then
    printf 'kbd-runtime: direct write rejected for %s; use prometheus kbd typed commands\n' \
      "$target" >&2
    return 1
  fi
  return 0
}

kbd_runtime_status_json() {
  local root="${1:-.}"
  command -v prometheus >/dev/null 2>&1 || {
    printf 'kbd-runtime: prometheus CLI is required\n' >&2
    return 1
  }
  prometheus kbd --path "$root" status --json
}

kbd_runtime_mutation_args() {
  local root="${1:-.}"
  local command_id="$2"
  local state revision
  state="$(kbd_runtime_status_json "$root")" || return 1
  revision="$(printf '%s' "$state" | jq -r '.revision // empty')"
  [ -n "$revision" ] || {
    printf 'kbd-runtime: could not resolve current revision\n' >&2
    return 1
  }
  printf '%s\n%s\n' "$revision" "$command_id"
}
