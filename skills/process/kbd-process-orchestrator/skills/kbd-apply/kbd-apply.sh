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
#                                  → fires task:before + emits "Starting task i out of n:   title"
#   end-task   <change> <id> <i> <n> <title>
#                                  → mark_done + sync progress.json + fires task:after
#                                    + emits "Completed task i out of n:   title"
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
# Source the position model so the apply loop keeps position.json fresh on every
# task boundary (Phase 1 carry-forward CF-5). Best-effort.
if [ -f "$KBD_ORCHESTRATOR_ROOT/shared/lib/position.sh" ]; then
  # shellcheck source=/dev/null
  . "$KBD_ORCHESTRATOR_ROOT/shared/lib/position.sh" 2>/dev/null || true
fi

WP=".kbd-orchestrator/current-waypoint.json"

# ---- backend detection -----------------------------------------------------

backend_detect() {
  # Optional $1: the change id being driven. When given, resolution is
  # scoped to that change's own directory shape instead of a repo-wide
  # guess — a repo can carry more than one backend's change directories at
  # once (e.g. legacy openspec/changes/ alongside native-kbd
  # .kbd-orchestrator/changes/), and the mere presence of an openspec/ dir
  # elsewhere in the repo must not shadow a change that actually lives
  # under a different backend.
  local change="${1:-}"

  # Explicit override wins: project.json.specBackend pins the backend.
  local pinned=""
  if [ -f .kbd-orchestrator/project.json ]; then
    pinned="$(jq -r '.specBackend // empty' .kbd-orchestrator/project.json 2>/dev/null)"
  fi
  case "$pinned" in
    openspec|native-kbd|speckit) printf '%s' "$pinned"; return 0 ;;
    auto|""|*) : ;;  # fall through to auto-detection
  esac

  if [ -n "$change" ]; then
    # native-kbd checked first: it's the KBD-owned change store and the
    # most specific match, so it can't be shadowed by an unrelated
    # openspec/ directory sitting elsewhere in the repo.
    if [ -f ".kbd-orchestrator/changes/$change/tasks.json" ] \
       || [ -f ".kbd-orchestrator/changes/$change/change.md" ]; then
      printf 'native-kbd'; return 0
    fi
    if { [ -f "openspec/changes/$change/proposal.md" ] || [ -f "openspec/changes/$change/tasks.md" ]; } \
       && command -v openspec >/dev/null 2>&1; then
      printf 'openspec'; return 0
    fi
    if [ -f "specs/$change/tasks.md" ]; then
      printf 'speckit'; return 0
    fi
    # Change id didn't match a known shape under any backend — fall through
    # to the repo-wide heuristic below rather than failing outright.
  fi

  if [ -d openspec ] && command -v openspec >/dev/null 2>&1; then
    printf 'openspec'; return 0
  fi
  if [ -d .specify ] || ls specs/*/tasks.md >/dev/null 2>&1; then
    printf 'speckit'; return 0
  fi
  # Native PMPO backend: the always-available fallback. Active when a native
  # change directory exists (tasks.json source-of-truth, or legacy change.md
  # that nk_list lazily migrates).
  if ls .kbd-orchestrator/changes/*/tasks.json >/dev/null 2>&1 \
     || ls .kbd-orchestrator/changes/*/change.md >/dev/null 2>&1; then
    printf 'native-kbd'; return 0
  fi
  printf ''
}

# ---- native-kbd adapter ----------------------------------------------------
# Source of truth: .kbd-orchestrator/changes/<change>/tasks.json
# (schema: references/schemas/change-tasks.schema.json). tasks.md is a
# regenerated human view. Legacy change.md checkbox lists are lazily migrated
# into tasks.json on first nk_list (original preserved).

_nk_change_dir() { printf '.kbd-orchestrator/changes/%s' "$1"; }
_nk_now() { date -u +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || echo unknown; }

# Regenerate tasks.md from tasks.json (generated view; banner warns editors).
_nk_render_md() {
  local cd="$1" tj="$1/tasks.json" md="$1/tasks.md"
  [ -f "$tj" ] || return 0
  {
    printf '<!-- GENERATED by kbd-apply from tasks.json — do not edit; edits are overwritten. -->\n\n'
    printf '# Tasks — %s\n\n' "$(jq -r '.changeId // ""' "$tj")"
    jq -r '.tasks[] | "- [" + (if .done then "x" else " " end) + "] " + .id + " " + .title' "$tj"
  } > "$md" 2>/dev/null || true
}

# Lazy one-way migration: legacy change.md checkbox list → tasks.json.
_nk_migrate_changemd() {
  local cd="$1" cm="$1/change.md" tj="$1/tasks.json" id
  [ -f "$cm" ] || return 1
  id="$(basename "$cd")"
  # Parse "## Tasks" checkbox lines: "- [ ] 1. title" / "- [x] title".
  local tasks_json
  tasks_json="$(awk '
    /^[[:space:]]*-[[:space:]]*\[[ xX\/]\]/ {
      n++
      done = ($0 ~ /\[[xX]\]/) ? "true" : "false"
      line=$0
      sub(/^[[:space:]]*-[[:space:]]*\[[ xX\/]\][[:space:]]*/, "", line)
      id=n
      if (match(line, /^[0-9]+\./)) { id=substr(line,1,RLENGTH-1); sub(/^[0-9]+\.[[:space:]]*/,"",line) }
      gsub(/\\/,"\\\\",line); gsub(/"/,"\\\"",line)
      printf "%s{\"id\":\"%s\",\"title\":\"%s\",\"done\":%s,\"doneAt\":null,\"doneBy\":null}", (n>1?",":""), id, line, done
    }
  ' "$cm")"
  printf '{"changeId":"%s","schemaVersion":"1","tasks":[%s]}' "$id" "$tasks_json" \
    | jq '.' > "$tj" 2>/dev/null || return 1
  _nk_render_md "$cd"
}

_nk_ensure_tasks() {
  local cd; cd="$(_nk_change_dir "$1")"
  [ -d "$cd" ] || return 1
  if [ ! -f "$cd/tasks.json" ]; then
    _nk_migrate_changemd "$cd" || return 1
  fi
  printf '%s' "$cd"
}

nk_list() {
  local cd; cd="$(_nk_ensure_tasks "$1")" || return 1
  jq -r '.tasks[] | [.id, (if .done then "1" else "0" end), .title] | @tsv' "$cd/tasks.json"
}

nk_progress() {
  local cd; cd="$(_nk_ensure_tasks "$1")" || return 1
  jq -r '[.tasks[]] as $t
    | ($t|length) as $tot
    | ([$t[]|select(.done)]|length) as $c
    | "\($tot) \($c) \($tot-$c)"' "$cd/tasks.json"
}

nk_mark_done() {
  local change="$1" id="$2" cd tmp now tool
  cd="$(_nk_ensure_tasks "$change")" || return 1
  now="$(_nk_now)"; tool="${KBD_TOOL:-claude-code}"
  tmp="$(mktemp)"
  jq --arg id "$id" --arg now "$now" --arg tool "$tool" '
    (.tasks[] | select(.id == $id)) |= (.done = true | .doneAt = $now | .doneBy = $tool)
  ' "$cd/tasks.json" > "$tmp" 2>/dev/null && mv "$tmp" "$cd/tasks.json" || { rm -f "$tmp"; return 1; }
  _nk_render_md "$cd"
}

nk_verify() {
  local cd; cd="$(_nk_ensure_tasks "$1")" || return 1
  # Run any per-task verify commands first.
  local cmds; cmds="$(jq -r '.tasks[] | select(.verify != null and .verify != "") | .verify' "$cd/tasks.json")"
  if [ -n "$cmds" ]; then
    local c
    while IFS= read -r c; do
      [ -n "$c" ] || continue
      bash -c "$c" >/dev/null 2>&1 || return 1
    done <<< "$cmds"
  fi
  # Structural check: all tasks done + spec.md present.
  local remaining; remaining="$(jq -r '[.tasks[]|select(.done|not)]|length' "$cd/tasks.json")"
  [ "${remaining:-1}" -eq 0 ] || return 1
  [ -f "$cd/spec.md" ] || [ -f "$cd/change.md" ] || return 1
  return 0
}

nk_archive() {
  local change="$1" cd date dest
  cd="$(_nk_change_dir "$change")"
  [ -d "$cd" ] || return 1
  date="$(date -u +%Y-%m-%d 2>/dev/null || echo undated)"
  dest=".kbd-orchestrator/changes/archive/$date-$change"
  mkdir -p ".kbd-orchestrator/changes/archive" || return 1
  mv "$cd" "$dest"
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
  # OpenSpec (spec-driven schema) task ids are POSITIONAL and match the
  # `openspec instructions apply --json` ordinal that os_list surfaces: "1" =
  # the first TOP-LEVEL checkbox, "2" = the second, etc. The JSON API does not
  # count indented sub-bullets (e.g. sub-rules nested under a parent task) as
  # separate tasks, so this awk must only count non-indented checkbox lines
  # too — otherwise ordinals drift the moment any task has nested children.
  # If the id is non-numeric, fall back to a text match on the description.
  local tmp; tmp="$(mktemp)"
  if printf '%s' "$id" | grep -qE '^[0-9]+$'; then
    awk -v id="$id" '
      BEGIN { n=0 }
      {
        if ($0 ~ /^-[[:space:]]*\[[ xX]\]/) {
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
        if (!done && $0 ~ /^-[[:space:]]*\[[[:space:]]\]/ && index($0, id)>0) {
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

# BACKEND is the repo-wide guess, used only for the bare `detect` diagnostic
# subcommand (no change id to scope by). Every other subcommand resolves its
# own backend from the change id it was given (see backend_detect), so a
# change is routed to the backend that actually owns it rather than whatever
# backend_detect() would guess for the repo as a whole.
BACKEND="$(backend_detect)"

b_list()      { local be; be="$(backend_detect "${1:-}")"; case "$be" in openspec) os_list "$@";; speckit) sk_list "$@";; native-kbd) nk_list "$@";; *) die "no spec backend detected (cwd=$(pwd))";; esac; }
b_progress()  { local be; be="$(backend_detect "${1:-}")"; case "$be" in openspec) os_progress "$@";; speckit) sk_progress "$@";; native-kbd) nk_progress "$@";; *) die "no spec backend detected";; esac; }
b_mark_done() { local be; be="$(backend_detect "${1:-}")"; case "$be" in openspec) os_mark_done "$@";; speckit) sk_mark_done "$@";; native-kbd) nk_mark_done "$@";; *) die "no spec backend detected";; esac; }
b_verify()    { local be; be="$(backend_detect "${1:-}")"; case "$be" in openspec) os_verify "$@";; native-kbd) nk_verify "$@";; *) return 0;; esac; }   # speckit: /speckit.analyze is model-driven, no CLI gate
b_archive()   { local be; be="$(backend_detect "${1:-}")"; case "$be" in openspec) os_archive "$@";; native-kbd) nk_archive "$@";; *) return 0;; esac; }  # speckit: no archive step

b_remaining_titles() {
  local change="$1"
  b_list "$change" 2>/dev/null | awk -F '\t' '$2=="0"{print $3}' | head -5
}

# ---- progress.json sync ----------------------------------------------------

_phase_dir() {
  # Depth-aware: resolve the active node from the waypoint's path[] (v3), which
  # supports arbitrary nesting (phase → child → grandchild → …). Falls back to
  # the v2 phase/childPointer resolution when the resolver lib is unavailable so
  # the driver still works in a stripped environment.
  [ -f "$WP" ] || return 1
  if command -v kbd_current_node_dir >/dev/null 2>&1; then
    local dir; dir="$(kbd_current_node_dir "$WP" 2>/dev/null || true)"
    [ -n "$dir" ] && { printf '%s' "$dir"; return 0; }
  fi
  # Fallback: v2 one-child-level resolution.
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
    printf 'Starting task %s out of %s:   %s\n' "$i" "$n" "$title" ;;

  end-task)
    change="${1:-}"; id="${2:-}"; i="${3:-1}"; n="${4:-1}"; shift 4 || true; title="$*"
    [ -n "$change" ] && [ -n "$id" ] || die "usage: end-task <change> <id> <i> <n> <title>"
    b_mark_done "$change" "$id"
    # Recompute progress from the backend so the count is authoritative.
    read -r tot comp rem < <(b_progress "$change" 2>/dev/null || echo "$n $i 0")
    sync_progress "$change" "${comp:-$i}" "${tot:-$n}"
    # Re-derive the unified position model so the breadcrumb tracks task
    # completion live (CF-5). Best-effort; never aborts the driver.
    command -v kbd_position_sync >/dev/null 2>&1 && { kbd_position_sync || true; }
    fire task after "$change:$id" "$i" "$n"
    printf 'Completed task %s out of %s:   %s\n' "$i" "$n" "$title"
    if [ "${rem:-0}" -gt 0 ]; then
      local pending_titles pending_preview
      pending_titles="$(b_remaining_titles "$change" | paste -sd ' | ' -)"
      pending_preview="${pending_titles:-unknown}"
      printf 'Remaining tasks after task %s: %s out of %s — %s\n' "$i" "${rem:-0}" "${tot:-$n}" "$pending_preview"
    else
      printf 'Remaining tasks after task %s: 0 out of %s — none\n' "$i" "${tot:-$n}"
    fi ;;

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
