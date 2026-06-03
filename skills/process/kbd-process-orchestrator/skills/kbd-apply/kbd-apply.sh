#!/usr/bin/env bash
# skills/kbd-apply/kbd-apply.sh
#
# KBD-owned spec-apply driver. Wraps a spec backend (OpenSpec today; Spec Kit
# via change-007) and drives it ONE task at a time so KBD stays the source of
# truth: every task boundary fires the KBD hooks, emits a plain-text position
# signal, and syncs progress.json + the waypoint.
#
# HARD INVARIANT: this driver never invokes a backend's "implement everything"
# command (bare `/opsx:apply`, `/speckit.implement`). It calls the backend per
# task. That is the entire point of this phase (F1).
#
# Subcommands:
#   detect [<dir>]                 → prints backend id ("openspec"|"speckit"|"")
#   list <change>                  → prints tasks as TSV: <id>\t<done 0|1>\t<title>
#   progress <change>              → prints "total complete remaining"
#   begin-task <change> <id> <i> <n> <title>
#                                  → fires task:before + emits "Starting task i of n: title"
#   end-task   <change> <id> <i> <n> <title>
#                                  → mark_done + sync progress.json + fires task:after
#                                    + emits "Completed task i of n: title"
#   mark-done  <change> <id>       → flip one task to done in the backend
#   verify     <change>            → backend verify (non-zero = fail)
#   archive    <change>            → backend archive
#
# The orchestrating model calls `list`, then for each not-done task calls
# `begin-task`, implements that single task, then `end-task`. This keeps the
# per-turn position signal firing no matter how long a task takes.

set -uo pipefail

SELF="kbd-apply"
die()  { printf '%s: %s\n' "$SELF" "$*" >&2; exit 1; }
warn() { printf '%s: warn: %s\n' "$SELF" "$*" >&2; }

command -v jq >/dev/null 2>&1 || die "jq is required"

KBD_ORCHESTRATOR_ROOT="${KBD_ORCHESTRATOR_ROOT:-$HOME/.claude/skills/kbd-process-orchestrator}"
# Source hooks (which now self-sources waypoint.sh). Best-effort.
if [ -f "$KBD_ORCHESTRATOR_ROOT/shared/lib/hooks.sh" ]; then
  # shellcheck source=/dev/null
  . "$KBD_ORCHESTRATOR_ROOT/shared/lib/hooks.sh" 2>/dev/null || true
fi

WP=".kbd-orchestrator/current-waypoint.json"

# ---- backend detection -----------------------------------------------------

backend_detect() {
  if [ -d openspec ] && command -v openspec >/dev/null 2>&1; then
    printf 'openspec'; return 0
  fi
  if [ -d .specify ] || ls specs/*/tasks.md >/dev/null 2>&1; then
    printf 'speckit'; return 0
  fi
  printf ''
}

# ---- OpenSpec adapter ------------------------------------------------------

_os_apply_json() { openspec instructions apply --change "$1" --json 2>/dev/null; }

os_list() {
  local change="$1" js
  js="$(_os_apply_json "$change")" || return 1
  [ -n "$js" ] || return 1
  printf '%s' "$js" | jq -r '.tasks[]? | [.id, (if .done then "1" else "0" end), .description] | @tsv'
}

os_progress() {
  local change="$1" js
  js="$(_os_apply_json "$change")" || return 1
  printf '%s' "$js" | jq -r '.progress | "\(.total) \(.complete) \(.remaining)"'
}

os_mark_done() {
  local change="$1" id="$2"
  local tasks_file="openspec/changes/$change/tasks.md"
  [ -f "$tasks_file" ] || { warn "no tasks.md at $tasks_file"; return 1; }
  # OpenSpec (spec-driven schema) task ids are POSITIONAL: "1" = the first
  # checkbox, "2" = the second, etc. Flip the Nth checkbox line by ordinal.
  # If the id is non-numeric, fall back to a text match on the description.
  local tmp; tmp="$(mktemp)"
  if printf '%s' "$id" | grep -qE '^[0-9]+$'; then
    awk -v id="$id" '
      BEGIN { n=0 }
      {
        if ($0 ~ /^[[:space:]]*-[[:space:]]*\[[ xX]\]/) {
          n++
          if (n == id) sub(/\[[ xX]\]/, "[x]")
        }
        print
      }
    ' "$tasks_file" > "$tmp" && mv "$tmp" "$tasks_file"
  else
    awk -v id="$id" '
      BEGIN { done=0 }
      {
        if (!done && $0 ~ /^[[:space:]]*-[[:space:]]*\[[[:space:]]\]/ && index($0, id)>0) {
          sub(/\[[[:space:]]\]/, "[x]"); done=1
        }
        print
      }
    ' "$tasks_file" > "$tmp" && mv "$tmp" "$tasks_file"
  fi
}

os_verify()  { openspec validate "$1" >/dev/null 2>&1; }
os_archive() { openspec archive "$1" >/dev/null 2>&1; }

# ---- Spec Kit (GitHub) adapter --------------------------------------------
# A "change" for Spec Kit is a feature dir name under specs/. tasks.md uses a
# Markdown checklist: "- [ ] T001 description". Spec Kit has no archive step.

_sk_tasks_file() {
  local change="$1"
  if [ -n "$change" ] && [ -f "specs/$change/tasks.md" ]; then
    printf 'specs/%s/tasks.md' "$change"; return 0
  fi
  # Fallback: the single tasks.md if there is exactly one.
  local f; f="$(ls specs/*/tasks.md 2>/dev/null | head -1)"
  [ -n "$f" ] && printf '%s' "$f"
}

sk_list() {
  local tf; tf="$(_sk_tasks_file "$1")"; [ -n "$tf" ] && [ -f "$tf" ] || return 1
  # Emit id \t done \t title. id = the Txxx token if present, else the ordinal.
  awk '
    /^[[:space:]]*-[[:space:]]*\[[ xX]\]/ {
      n++
      done = ($0 ~ /\[[xX]\]/) ? 1 : 0
      line=$0
      sub(/^[[:space:]]*-[[:space:]]*\[[ xX]\][[:space:]]*/, "", line)
      id=n
      if (match(line, /^T[0-9]+/)) {
        id=substr(line, RSTART, RLENGTH)
        # Strip the "Txxx " token from the displayed title.
        sub(/^T[0-9]+[[:space:]]*/, "", line)
      }
      printf "%s\t%s\t%s\n", id, done, line
    }
  ' "$tf"
}

sk_progress() {
  local out; out="$(sk_list "$1")" || return 1
  local total complete
  total="$(printf '%s\n' "$out" | grep -c . )"
  complete="$(printf '%s\n' "$out" | awk -F'\t' '$2==1' | grep -c . )"
  printf '%s %s %s' "$total" "$complete" "$((total - complete))"
}

sk_mark_done() {
  local change="$1" id="$2" tf tmp
  tf="$(_sk_tasks_file "$change")"; [ -n "$tf" ] && [ -f "$tf" ] || return 1
  tmp="$(mktemp)"
  if printf '%s' "$id" | grep -qE '^[0-9]+$'; then
    awk -v id="$id" '{ if ($0 ~ /^[[:space:]]*-[[:space:]]*\[[ xX]\]/) { n++; if (n==id) sub(/\[[ xX]\]/,"[x]") } print }' "$tf" > "$tmp" && mv "$tmp" "$tf"
  else
    awk -v id="$id" '{ if (!d && $0 ~ /^[[:space:]]*-[[:space:]]*\[[[:space:]]\]/ && index($0,id)>0) { sub(/\[[[:space:]]\]/,"[x]"); d=1 } print }' "$tf" > "$tmp" && mv "$tmp" "$tf"
  fi
}

# ---- backend dispatch ------------------------------------------------------

BACKEND="$(backend_detect)"

b_list()      { case "$BACKEND" in openspec) os_list "$@";; speckit) sk_list "$@";; *) die "no spec backend detected (cwd=$(pwd))";; esac; }
b_progress()  { case "$BACKEND" in openspec) os_progress "$@";; speckit) sk_progress "$@";; *) die "no spec backend detected";; esac; }
b_mark_done() { case "$BACKEND" in openspec) os_mark_done "$@";; speckit) sk_mark_done "$@";; *) die "no spec backend detected";; esac; }
b_verify()    { case "$BACKEND" in openspec) os_verify "$@";; *) return 0;; esac; }   # speckit: /speckit.analyze is model-driven, no CLI gate
b_archive()   { case "$BACKEND" in openspec) os_archive "$@";; *) return 0;; esac; }  # speckit: no archive step

# ---- progress.json sync ----------------------------------------------------

_phase_dir() {
  # Child-aware: when a child loop is active the waypoint keeps `.phase` as the
  # PARENT and names the active child in `.childPointer`. The child's
  # progress.json lives under phases/<parent>/children/<child>/. Resolve to the
  # child dir in that case so /kbd-apply syncs the inner loop, not the parent.
  [ -f "$WP" ] || return 1
  local phase child; phase="$(jq -r '.phase // ""' "$WP" 2>/dev/null)"
  [ -n "$phase" ] || return 1
  child="$(jq -r '.childPointer // ""' "$WP" 2>/dev/null)"
  if [ -n "$child" ] && [ "$child" != "null" ]; then
    printf '.kbd-orchestrator/phases/%s/children/%s' "$phase" "$child"
  else
    printf '.kbd-orchestrator/phases/%s' "$phase"
  fi
}

sync_progress() {
  # sync_progress <change> <complete> <total>
  local change="$1" complete="$2" total="$3" pdir pj tmp
  pdir="$(_phase_dir)" || return 0
  pj="$pdir/progress.json"
  [ -f "$pj" ] || return 0
  tmp="$(mktemp)"
  jq --arg c "$change" --argjson done "$complete" --argjson tot "$total" '
    (.changes[]? | select(.id==$c) | .tasks_done) = $done
    | (.changes[]? | select(.id==$c) | .tasks_total) = $tot
  ' "$pj" > "$tmp" 2>/dev/null && mv "$tmp" "$pj" || rm -f "$tmp"
}

# Fire a hook. Do NOT swallow its stderr — that is where the default reporter
# and any user-defined override/augment hooks write. The driver's own
# plain-text stdout signal is the user-facing guarantee; the hook output is the
# extensibility layer and must remain visible. Never let a hook failure abort
# the driver, though.
fire() { command -v kbd_hooks_fire >/dev/null 2>&1 && { kbd_hooks_fire "$@" || true; }; }

# ---- subcommands -----------------------------------------------------------

# Testability: when sourced with KBD_APPLY_LIB_ONLY=1, define functions and
# return without dispatching, so tests can call internal resolvers directly.
if [ "${KBD_APPLY_LIB_ONLY:-}" = "1" ]; then
  return 0 2>/dev/null || true
fi

cmd="${1:-}"; shift || true
case "$cmd" in
  detect)
    printf '%s\n' "$BACKEND" ;;

  list)
    [ -n "${1:-}" ] || die "usage: list <change>"
    [ -n "$BACKEND" ] || die "no spec backend detected in $(pwd)"
    b_list "$1" ;;

  progress)
    [ -n "${1:-}" ] || die "usage: progress <change>"
    b_progress "$1" ;;

  begin-task)
    # begin-task <change> <id> <i> <n> <title...>
    change="${1:-}"; id="${2:-}"; i="${3:-1}"; n="${4:-1}"; shift 4 || true; title="$*"
    [ -n "$change" ] && [ -n "$id" ] || die "usage: begin-task <change> <id> <i> <n> <title>"
    fire task before "$change:$id" "$i" "$n"
    printf 'Starting task %s of %s: %s\n' "$i" "$n" "$title" ;;

  end-task)
    change="${1:-}"; id="${2:-}"; i="${3:-1}"; n="${4:-1}"; shift 4 || true; title="$*"
    [ -n "$change" ] && [ -n "$id" ] || die "usage: end-task <change> <id> <i> <n> <title>"
    b_mark_done "$change" "$id"
    # Recompute progress from the backend so the count is authoritative.
    read -r tot comp rem < <(b_progress "$change" 2>/dev/null || echo "$n $i 0")
    sync_progress "$change" "${comp:-$i}" "${tot:-$n}"
    fire task after "$change:$id" "$i" "$n"
    printf 'Completed task %s of %s: %s\n' "$i" "$n" "$title" ;;

  mark-done)
    [ -n "${1:-}" ] && [ -n "${2:-}" ] || die "usage: mark-done <change> <id>"
    b_mark_done "$1" "$2" ;;

  verify)
    [ -n "${1:-}" ] || die "usage: verify <change>"
    if b_verify "$1"; then echo "verify: PASS"; else echo "verify: FAIL"; exit 1; fi ;;

  archive)
    [ -n "${1:-}" ] || die "usage: archive <change>"
    b_archive "$1" && echo "archived: $1" ;;

  ""|-h|--help)
    sed -n '2,40p' "$0" ;;

  *)
    die "unknown subcommand: $cmd (try --help)" ;;
esac
