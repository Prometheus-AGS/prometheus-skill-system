#!/usr/bin/env bash
# skills/kbd-apply/kbd-apply.sh
#
# KBD-owned spec-apply driver. Wraps a spec backend and drives it ONE task at
# a time so KBD stays the source of truth: every task boundary fires the KBD
# hooks, emits a plain-text position signal, and syncs progress.json + the
# waypoint.
#
# Supported engines (see docs/guide/spec-engines.md and config/spec-engines.json):
#   openspec  — DEFAULT. @fission-ai/openspec CLI (pinned in package.json),
#               artifacts under openspec/. Adapter: os_* below.
#   speckit   — GitHub Spec Kit v1.x (`specify` CLI, installed via
#               `uv tool install specify-cli`), artifacts under .specify/ and
#               specs/<slug>/{spec.md,plan.md,tasks.md}. Adapter: sk_* below.
#   native-kbd— the KBD-owned change store under .kbd-orchestrator/changes/.
#               Adapter: nk_* below.
# The adapter contract is documented and extensible: implement the five ops
# (list/progress/mark_done/verify/archive) as <prefix>_* functions and add the
# detection shape + pin value to backend_detect.
#
# HARD INVARIANT: this driver never invokes a backend's "implement everything"
# command (bare `/opsx:apply`, `/speckit.implement`). It calls the backend per
# task. That is the entire point of this phase (F1).
#
# Subcommands:
#   detect [<dir>]                 → prints backend id ("openspec"|"speckit"|"native-kbd"|"")
#   list <change>                  → prints tasks as TSV: <id>\t<done 0|1>\t<title>
#   progress <change>              → prints "total complete remaining"
#   begin-task <change> <id> <i> <n> <title>
#                                  → fires task:before + emits "Starting task i out of n:   title"
#   end-task   <change> <id> <i> <n> <title>
#                                  → mark_done + sync progress.json + fires task:after
#                                    + emits "Completed task i out of n:   title"
#   mark-done  <change> <id>       → flip one task to done in the backend AND sync the
#                                    ledger (runtime transition + progress.json); no hooks
#   reconcile [<phase>] [--repair] [--json]
#                                  → compare backend done flags with the canonical ledger;
#                                    exit 1 on drift, 0 when clean; --repair replays each
#                                    drifted task through begin-task/end-task
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

APPLY_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
LOCAL_ORCHESTRATOR_ROOT="$(cd "$APPLY_DIR/../.." && pwd -P)"
if [ ! -f "$LOCAL_ORCHESTRATOR_ROOT/shared/openspec/cli.mjs" ]; then
  LOCAL_ORCHESTRATOR_ROOT="$APPLY_DIR/../kbd-process-orchestrator"
fi
KBD_ORCHESTRATOR_ROOT="${KBD_ORCHESTRATOR_ROOT:-$LOCAL_ORCHESTRATOR_ROOT}"
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
if [ -f "$KBD_ORCHESTRATOR_ROOT/shared/lib/runtime-authority.sh" ]; then
  # shellcheck source=/dev/null
  . "$KBD_ORCHESTRATOR_ROOT/shared/lib/runtime-authority.sh" 2>/dev/null || true
fi
if [ -f "$KBD_ORCHESTRATOR_ROOT/shared/lib/bottleneck-guard.sh" ]; then
  # shellcheck source=/dev/null
  . "$KBD_ORCHESTRATOR_ROOT/shared/lib/bottleneck-guard.sh" 2>/dev/null || true
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
    if [ -f "openspec/changes/$change/proposal.md" ] || [ -f "openspec/changes/$change/tasks.md" ]; then
      printf 'openspec'; return 0
    fi
    # Spec Kit v1 feature dirs carry spec.md and plan.md alongside tasks.md;
    # a change may be driven before tasks.md is generated, so accept any of
    # the three artifacts as the change-scoped speckit shape.
    if [ -f "specs/$change/tasks.md" ] \
       || [ -f "specs/$change/spec.md" ] \
       || [ -f "specs/$change/plan.md" ]; then
      printf 'speckit'; return 0
    fi
    # Change id didn't match a known shape under any backend — fall through
    # to the repo-wide heuristic below rather than failing outright.
  fi

  if [ -d openspec ]; then
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

_os_run() { node "$KBD_ORCHESTRATOR_ROOT/shared/openspec/cli.mjs" run --project "$PWD" -- "$@"; }
_os_apply_json() { _os_run instructions apply --change "$1" --json; }

os_list() {
  local change="$1" js
  js="$(_os_apply_json "$change")" || return 1
  [ -n "$js" ] || return 1
  printf '%s' "$js" | jq -r '.tasks[]? | [.id, (if .done then "1" else "0" end), .description] | @tsv'
}

os_progress() {
  local change="$1" js
  js="$(_os_apply_json "$change")" || return 1
  printf '%s' "$js" | jq -er '.progress |
    if type == "object" and
       ([.total, .complete, .remaining] | all(type == "number" and . >= 0 and floor == .)) and
       .complete + .remaining == .total
    then "\(.total) \(.complete) \(.remaining)"
    else error("invalid backend progress; no completion may be inferred") end'
}

os_mark_done() {
  local change="$1" id="$2"
  local tasks_file="openspec/changes/$change/tasks.md"
  [ -f "$tasks_file" ] || { warn "no tasks.md at $tasks_file"; return 1; }
  # Task IDs are whatever `openspec instructions apply --json` (os_list) says
  # they are, and OpenSpec's counting rule has changed across releases: older
  # versions counted only column-0 checkboxes, 1.10.0 counts every checkbox
  # line (nested sub-tasks and `*` bullets included). Re-deriving the count
  # here drifts whenever the rule changes, and a drift checks off the wrong
  # task. So resolve the ID through OpenSpec's own list: its description, and
  # which occurrence of that description it is, then flip that line.
  # Checkbox lines are matched with OpenSpec 1.10's TASK_LINE_PATTERN
  # (^\s*[-*]\s*\[([\sxX])\]). Without JSON, fall back to that pattern's
  # ordinal. A non-numeric id is a text match on the first open task line.
  local js="" resolved="" tmp
  js="$(_os_apply_json "$change")" || js=""
  if [ -n "$js" ]; then
    resolved="$(printf '%s' "$js" | jq -r --arg id "$id" '
      (.tasks // []) as $t
      | ($t | map(.id | tostring) | index($id)) as $i
      | if $i == null then empty
        else $t[$i].description as $d
          | "\([$t[0:$i+1][] | select(.description == $d)] | length)\t\($d)"
        end' 2>/dev/null)" || resolved=""
  fi
  tmp="$(mktemp)"
  if [ -n "$resolved" ]; then
    KBD_OCC="${resolved%%$'\t'*}" KBD_DESC="${resolved#*$'\t'}" awk '
      BEGIN { want = ENVIRON["KBD_DESC"]; occ = ENVIRON["KBD_OCC"] + 0; n = 0; hit = 0 }
      {
        line = $0; sub(/\r$/, "", line)
        if (!hit && match(line, /^[[:space:]]*[-*][[:space:]]*\[[[:space:]xX]\]/)) {
          desc = substr(line, RLENGTH + 1); gsub(/^[[:space:]]+|[[:space:]]+$/, "", desc)
          if (desc == want && ++n == occ) {
            prefix = substr($0, 1, RLENGTH); rest = substr($0, RLENGTH + 1)
            sub(/\[[[:space:]xX]\]$/, "[x]", prefix); $0 = prefix rest; hit = 1
          }
        }
        print
      }
      END { exit hit ? 0 : 3 }
    ' "$tasks_file" > "$tmp"
    if [ $? -ne 0 ]; then
      rm -f "$tmp"
      warn "task $id (\"${resolved#*$'\t'}\") from openspec has no matching line in $tasks_file"
      return 1
    fi
  elif printf '%s' "$id" | grep -qE '^[0-9]+$'; then
    awk -v id="$id" '
      BEGIN { n = 0 }
      {
        if (match($0, /^[[:space:]]*[-*][[:space:]]*\[[[:space:]xX]\]/)) {
          if (++n == id) {
            prefix = substr($0, 1, RLENGTH); rest = substr($0, RLENGTH + 1)
            sub(/\[[[:space:]xX]\]$/, "[x]", prefix); $0 = prefix rest
          }
        }
        print
      }
    ' "$tasks_file" > "$tmp"
  else
    KBD_ID="$id" awk '
      BEGIN { want = ENVIRON["KBD_ID"]; done = 0 }
      {
        if (!done && match($0, /^[[:space:]]*[-*][[:space:]]*\[[[:space:]]\]/) && index($0, want) > 0) {
          prefix = substr($0, 1, RLENGTH); rest = substr($0, RLENGTH + 1)
          sub(/\[[[:space:]]\]$/, "[x]", prefix); $0 = prefix rest; done = 1
        }
        print
      }
    ' "$tasks_file" > "$tmp"
  fi
  mv "$tmp" "$tasks_file"
}

os_verify()  { _os_run validate "$1" >/dev/null; }
# `--yes` is REQUIRED, not cosmetic: without it `openspec archive` waits on an
# interactive confirmation. Driven from a script there is no stdin to answer it,
# so the command fails — and because the old form also swallowed stderr, the
# function still returned 0. The result was a silent no-op reported as success:
# the change stayed in openspec/changes/, its spec was never promoted, and the
# only way to notice was to list the directory afterwards.
#
# stderr is deliberately NOT swallowed now, so a real archive failure is visible
# rather than inferred.
os_archive() { _os_run archive "$1" --yes >/dev/null; }

# ---- Spec Kit (GitHub) adapter --------------------------------------------
# A "change" for Spec Kit is a feature dir name under specs/. tasks.md uses a
# Markdown checklist: "- [ ] T001 description". Artifacts live under .specify/
# (templates, scripts) and specs/<slug>/{spec.md,plan.md,tasks.md}.

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

sk_verify() {
  # Structural gate. Spec Kit's own check (`/speckit.analyze`) is
  # model-driven — it dispatches an LLM, not a CLI — so it cannot run inside
  # this driver's non-interactive loop. Instead we enforce the structural
  # equivalent: every checkbox in specs/<change>/tasks.md is checked AND
  # specs/<change>/spec.md exists.
  local change="$1" tf unchecked
  tf="$(_sk_tasks_file "$change")"
  [ -n "$tf" ] && [ -f "$tf" ] || { warn "no tasks.md for change $change under specs/"; return 1; }
  [ -f "specs/$change/spec.md" ] || { warn "specs/$change/spec.md missing"; return 1; }
  unchecked="$(grep -cE '^[[:space:]]*[-*][[:space:]]*\[[[:space:]]\]' "$tf")"
  [ "${unchecked:-1}" -eq 0 ] || { warn "$unchecked unchecked task(s) in $tf"; return 1; }
  return 0
}

sk_archive() {
  # Best-effort archive mirroring native-kbd's: move specs/<change> to
  # specs/archive/<date>-<change>. Spec Kit has no native archive command, so
  # this is the documented pack-side convention, not upstream behavior.
  local change="$1" src date dest
  src="specs/$change"
  [ -d "$src" ] || return 1
  date="$(date -u +%Y-%m-%d 2>/dev/null || echo undated)"
  dest="specs/archive/$date-$change"
  mkdir -p specs/archive || return 1
  [ -e "$dest" ] && { warn "archive destination exists: $dest"; return 1; }
  mv "$src" "$dest"
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
# speckit's /speckit.analyze is model-driven (no CLI gate), so sk_verify is a
# structural gate instead; unknown backends stay a safe no-op pass.
b_verify()    { local be; be="$(backend_detect "${1:-}")"; case "$be" in openspec) os_verify "$@";; speckit) sk_verify "$@";; native-kbd) nk_verify "$@";; *) return 0;; esac; }
# speckit has no native archive; sk_archive is the documented pack-side move
# to specs/archive/. Unknown backends stay a safe no-op pass.
b_archive()   { local be; be="$(backend_detect "${1:-}")"; case "$be" in openspec) os_archive "$@";; speckit) sk_archive "$@";; native-kbd) nk_archive "$@";; *) return 0;; esac; }

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
  # KbdStateV2 derives change/task counters from committed task events. The
  # compatibility ledger is a projection and must not be edited in place.
  if command -v kbd_runtime_authoritative >/dev/null 2>&1 \
     && kbd_runtime_authoritative "."; then
    return 0
  fi
  pdir="$(_phase_dir)" || return 0
  pj="$pdir/progress.json"
  [ -f "$pj" ] || return 0
  tmp="$(mktemp)"
  jq --arg c "$change" --argjson done "$complete" --argjson tot "$total" '
    (.changes[]? | select(.id==$c) | .tasks_done) = $done
    | (.changes[]? | select(.id==$c) | .tasks_total) = $tot
  ' "$pj" > "$tmp" 2>/dev/null && mv "$tmp" "$pj" || rm -f "$tmp"
}

resolve_runtime_task_id() {
  # resolve_runtime_task_id <registered-tasks-json> <backend-id> <sequence> <title>
  # Prints the runtime task ID kbd-apply must use for this backend task.
  # /kbd-plan may register a change's tasks before apply runs, under IDs that
  # differ from the backend ordinal (for example "<change>-t1" vs "1").
  # Registering the backend ID beside them creates duplicates: the planned
  # tasks stay pending and the change can never complete. So: an exact ID
  # wins; with no registered tasks the backend ID is new; otherwise reuse the
  # task with the same normalized title, then the unique task with the same
  # sequence. Anything else is refused rather than duplicated.
  local tasks="${1:-"{}"}" id="$2" seq="$3" title="$4" resolved
  resolved="$(printf '%s' "$tasks" | jq -r --arg id "$id" --arg seq "$seq" --arg title "$title" '
    def norm: (. // "") | ascii_downcase
      | sub("^\\s*[0-9]+(\\.[0-9]+)*\\s+"; "") | gsub("\\s+"; " ")
      | sub("^ "; "") | sub(" $"; "");
    if has($id) then $id
    elif length == 0 then $id
    else
      [to_entries[] | select((.value.title | norm) == ($title | norm)) | .key] as $by_title
      | if ($by_title | length) == 1 then $by_title[0]
        else
          [to_entries[] | select((.value.sequence | tostring) == $seq) | .key] as $by_seq
          | if ($by_seq | length) == 1 then $by_seq[0] else empty end
        end
    end')" || return 1
  if [ -z "$resolved" ]; then
    printf '%s: change has registered runtime tasks but none matches backend task %s (sequence %s, "%s"); refusing to register a duplicate. Register it with this ID or reconcile the plan.\n' \
      "$SELF" "$id" "$seq" "$title"
    return 1
  fi
  [ "$resolved" != "$id" ] && warn "backend task $id maps to registered runtime task $resolved"
  printf '%s\n' "$resolved"
}

runtime_task_transition() {
  # runtime_task_transition <change> <task-id> <title> <sequence> <status>
  local change="$1" task_id="$2" title="$3" sequence="$4" status="$5"
  command -v kbd_runtime_authoritative >/dev/null 2>&1 || return 0
  kbd_runtime_authoritative "." || return 0

  local state phase command_id mutation revision
  state="$(kbd_runtime_status_json ".")" || return 1
  phase="$(printf '%s' "$state" | jq -r '.activePath.phaseId // empty')"
  [ -n "$phase" ] || {
    warn "canonical runtime has no active phase"
    return 1
  }

  # Register missing change/task definitions before transitioning them. Each
  # operation refreshes revision so optimistic concurrency remains exact.
  if ! printf '%s' "$state" | jq -e \
      --arg phase "$phase" --arg change "$change" \
      '.phases[$phase].changes[$change]' >/dev/null 2>&1; then
    command_id="apply:change-register:${phase}:${change}"
    prometheus kbd --path . change register \
      --command-id "$command_id" \
      --phase "$phase" --id "$change" --title "$change" >/dev/null || return 1
    state="$(kbd_runtime_status_json ".")" || return 1
  fi

  # Reuse a task /kbd-plan already registered for this change instead of
  # registering a duplicate under the backend ordinal.
  local registered
  registered="$(printf '%s' "$state" | jq -c --arg phase "$phase" --arg change "$change" \
    '.phases[$phase].changes[$change].tasks // {}')" || return 1
  task_id="$(resolve_runtime_task_id "$registered" "$task_id" "$sequence" "$title")" || {
    warn "$task_id"
    return 1
  }

  if ! printf '%s' "$state" | jq -e \
      --arg phase "$phase" --arg change "$change" --arg task "$task_id" \
      '.phases[$phase].changes[$change].tasks[$task]' >/dev/null 2>&1; then
    command_id="apply:task-register:${phase}:${change}:${task_id}"
    prometheus kbd --path . task register \
      --command-id "$command_id" \
      --phase "$phase" --change "$change" --id "$task_id" \
      --title "$title" --sequence "$sequence" >/dev/null || return 1
  fi

  [ "$status" = "register-only" ] && return 0

  command_id="apply:task-${status}:${phase}:${change}:${task_id}"
  prometheus kbd --path . task transition \
    --command-id "$command_id" \
    --phase "$phase" --change "$change" --id "$task_id" \
    --status "$status" --summary "$title" >/dev/null
}

runtime_task_complete() {
  # runtime_task_complete <change> <task-id> <title> <sequence>
  # Closes one task in the canonical ledger from ANY prior state. The runtime
  # only accepts pending -> in-progress -> complete, so a task flipped done in
  # the backend without ever being begun needs both transitions. No-op when the
  # runtime is not authoritative or the task is already complete.
  local change="$1" task_id="$2" title="$3" sequence="$4"
  command -v kbd_runtime_authoritative >/dev/null 2>&1 || return 0
  kbd_runtime_authoritative "." || return 0
  runtime_task_transition "$change" "$task_id" "$title" "$sequence" "register-only" || return 1
  local state phase registered rid status
  state="$(kbd_runtime_status_json ".")" || return 1
  phase="$(printf '%s' "$state" | jq -r '.activePath.phaseId // empty')"
  registered="$(printf '%s' "$state" | jq -c --arg phase "$phase" --arg change "$change" \
    '.phases[$phase].changes[$change].tasks // {}')" || return 1
  rid="$(resolve_runtime_task_id "$registered" "$task_id" "$sequence" "$title" 2>/dev/null)" || return 1
  status="$(printf '%s' "$state" | jq -r --arg phase "$phase" --arg change "$change" --arg t "$rid" \
    '.phases[$phase].changes[$change].tasks[$t].status // "pending"')"
  case "$status" in
    complete|completed|done) return 0 ;;
    pending) runtime_task_transition "$change" "$task_id" "$title" "$sequence" "in-progress" || return 1 ;;
  esac
  runtime_task_transition "$change" "$task_id" "$title" "$sequence" "complete"
}

# ---- reconcile -------------------------------------------------------------
# Drift = the backend (tasks.json / tasks.md) says a task is done while the
# canonical ledger does not. Read-only; the only write path is --repair, which
# replays each drifted task through begin-task/end-task.

reconcile_phase_default() {
  local state="$1" p=""
  p="$(printf '%s' "$state" | jq -r '.activePath.phaseId // .phase // empty' 2>/dev/null)"
  if [ -z "$p" ] && [ -f "$WP" ]; then p="$(jq -r '.phase // empty' "$WP" 2>/dev/null)"; fi
  printf '%s' "$p"
}

# reconcile_scan <phase> <state-json> -> TSV on stdout:
#   <change>\t<task-id|->\t<seq>\t<total>\t<kind>\t<title>
# kind: ledger-missing | ledger-pending | ledger-ahead | unmappable | projection | count
reconcile_scan() {
  local phase="$1" state="$2" authoritative="$3" changes change listing
  local tid done title seq total bcomp registered rid status pj pdone rcomp
  if [ "$authoritative" = "1" ]; then
    changes="$(printf '%s' "$state" | jq -r --arg p "$phase" '(.phases[$p].changes // {}) | keys_unsorted[]' 2>/dev/null)"
  else
    pj=".kbd-orchestrator/phases/$phase/progress.json"
    changes="$(jq -r '.changes[]?.id' "$pj" 2>/dev/null)"
  fi
  while IFS= read -r change; do
    [ -n "$change" ] || continue
    listing="$(b_list "$change" 2>/dev/null)" || { warn "reconcile: no backend task list for $change; skipped"; continue; }
    [ -n "$listing" ] || continue
    total="$(printf '%s\n' "$listing" | wc -l | tr -d ' ')"
    bcomp="$(printf '%s\n' "$listing" | awk -F '\t' '$2=="1"{n++} END{print n+0}')"
    seq=0
    if [ "$authoritative" = "1" ]; then
      registered="$(printf '%s' "$state" | jq -c --arg p "$phase" --arg c "$change" '.phases[$p].changes[$c].tasks // {}')"
      while IFS=$'\t' read -r tid done title; do
        seq=$((seq + 1))
        rid="$(resolve_runtime_task_id "$registered" "$tid" "$seq" "$title" 2>/dev/null)" || rid=""
        if [ -z "$rid" ]; then
          [ "$done" = "1" ] && printf '%s\t%s\t%s\t%s\tunmappable\t%s\n' "$change" "$tid" "$seq" "$total" "$title"
          continue
        fi
        status="$(printf '%s' "$registered" | jq -r --arg t "$rid" '.[$t].status // "absent"')"
        case "$status" in complete|completed|done) status=complete ;; esac
        if [ "$done" = "1" ] && [ "$status" != "complete" ]; then
          if [ "$status" = "absent" ]; then
            printf '%s\t%s\t%s\t%s\tledger-missing\t%s\n' "$change" "$tid" "$seq" "$total" "$title"
          else
            printf '%s\t%s\t%s\t%s\tledger-pending\t%s\n' "$change" "$tid" "$seq" "$total" "$title"
          fi
        elif [ "$done" != "1" ] && [ "$status" = "complete" ]; then
          printf '%s\t%s\t%s\t%s\tledger-ahead\t%s\n' "$change" "$tid" "$seq" "$total" "$title"
        fi
      done <<< "$listing"
      # The progress.json projection must agree with the runtime's own count.
      pj=".kbd-orchestrator/phases/$phase/progress.json"
      if [ -f "$pj" ]; then
        pdone="$(jq -r --arg c "$change" '[.changes[]? | select(.id==$c) | .tasks_done][0] // empty' "$pj" 2>/dev/null)"
        rcomp="$(printf '%s' "$registered" | jq -r '[to_entries[] | select(.value.status=="complete" or .value.status=="completed" or .value.status=="done")] | length')"
        if [ -n "$pdone" ] && [ "$pdone" != "$rcomp" ]; then
          printf '%s\t-\t0\t%s\tprojection\tprogress.json tasks_done=%s but ledger has %s complete\n' "$change" "$total" "$pdone" "$rcomp"
        fi
      fi
    else
      pdone="$(jq -r --arg c "$change" '[.changes[]? | select(.id==$c) | .tasks_done][0] // empty' "$pj" 2>/dev/null)"
      if [ "$pdone" != "$bcomp" ]; then
        printf '%s\t-\t0\t%s\tcount\tprogress.json tasks_done=%s but backend has %s done\n' "$change" "$total" "${pdone:-absent}" "$bcomp"
      fi
    fi
  done <<< "$changes"
}

reconcile_print() {
  # reconcile_print <phase> <json 0|1> <scan-tsv>
  local phase="$1" json="$2" scan="$3" change tid seq total kind title n=0
  if [ "$json" = "1" ]; then
    printf '%s' "$scan" | jq -R -s --arg phase "$phase" '
      [split("\n")[] | select(length > 0) | split("\t")
        | {change: .[0], task: .[1], sequence: (.[2]|tonumber), total: (.[3]|tonumber), kind: .[4], title: .[5]}] as $d
      | {phase: $phase, clean: ($d|length == 0), drifted: ($d|length), drift: $d}'
    return 0
  fi
  while IFS=$'\t' read -r change tid seq total kind title; do
    [ -n "$change" ] || continue
    n=$((n + 1))
    printf 'DRIFT %s task %s (%s): %s\n' "$change" "$tid" "$kind" "$title"
  done <<< "$scan"
  if [ "$n" -eq 0 ]; then printf 'reconcile: clean — %s\n' "$phase"
  else printf 'reconcile: %s drifted task(s) in %s — repair with: kbd-apply reconcile %s --repair\n' "$n" "$phase" "$phase"; fi
}

reconcile_main() {
  local phase="" repair=0 json=0 arg state="" authoritative=0 scan rc=0 line
  for arg in "$@"; do
    case "$arg" in
      --repair) repair=1 ;;
      --json) json=1 ;;
      -*) die "reconcile: unknown flag $arg (usage: reconcile [<phase>] [--repair] [--json])" ;;
      *) [ -z "$phase" ] || die "reconcile: only one phase may be given"; phase="$arg" ;;
    esac
  done
  if command -v kbd_runtime_authoritative >/dev/null 2>&1 && kbd_runtime_authoritative "."; then
    authoritative=1
    state="$(kbd_runtime_status_json ".")" || die "reconcile: cannot read canonical runtime status"
  fi
  [ -n "$phase" ] || phase="$(reconcile_phase_default "$state")"
  [ -n "$phase" ] || die "reconcile: no phase given and no active phase found"

  scan="$(reconcile_scan "$phase" "$state" "$authoritative")"
  if [ -n "$scan" ] && [ "$repair" = "1" ]; then
    local change tid seq total kind title
    while IFS=$'\t' read -r change tid seq total kind title; do
      [ -n "$change" ] || continue
      case "$kind" in
        ledger-missing|ledger-pending)
          printf 'repair: %s task %s\n' "$change" "$tid" >&2
          bash "${BASH_SOURCE[0]}" begin-task "$change" "$tid" "$seq" "$total" "$title" >/dev/null || warn "repair: begin-task failed for $change/$tid"
          bash "${BASH_SOURCE[0]}" end-task "$change" "$tid" "$seq" "$total" "$title" >/dev/null || warn "repair: end-task failed for $change/$tid" ;;
        count)
          progress_output="$(b_progress "$change")" && { read -r tot comp rem <<< "$progress_output"; sync_progress "$change" "$comp" "$tot"; } ;;
      esac
    done <<< "$scan"
    # Re-check from fresh state: repair must prove itself, not assume.
    [ "$authoritative" = "1" ] && { state="$(kbd_runtime_status_json ".")" || die "reconcile: cannot re-read runtime status"; }
    scan="$(reconcile_scan "$phase" "$state" "$authoritative")"
  fi
  reconcile_print "$phase" "$json" "$scan"
  [ -z "$scan" ] || rc=1
  return "$rc"
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
    progress_output="$(b_progress "$change")" || die "failed to read backend task progress"
    read -r before_tot before_comp before_rem <<< "$progress_output"
    runtime_task_transition "$change" "$id" "$title" "$i" "register-only" \
      || die "failed to register canonical task boundary"
    change_start=0
    [ "${before_comp:-0}" -eq 0 ] && change_start=1
    guard_enabled=0
    if command -v kbd_bottleneck_active >/dev/null 2>&1 && kbd_bottleneck_active; then
      guard_enabled=1
      change_key="change:$(printf '%s' "$change" | tr '[:upper:]' '[:lower:]')"
      canonical_state="$(kbd_runtime_status_json ".")" \
        || die "failed to read canonical boundary obligations"
      if printf '%s' "$canonical_state" | jq -e --arg key "$change_key" '.boundaryObligations[$key]' >/dev/null 2>&1; then
        change_start=0
      else
        change_start=1
        kbd_bottleneck_evaluate change before "$change" 1 >/dev/null \
          || die "canonical change start precommit evaluation blocked"
      fi
      # OpenSpec task ordinals and standard task titles repeat across changes.
      # Qualify the canonical subject so the guard resolves the exact typed
      # change/task pair and records a distinct boundary obligation.
      kbd_bottleneck_evaluate task before "$change/$id" 1 >/dev/null \
        || die "canonical task start precommit evaluation blocked"
    fi
    runtime_task_transition "$change" "$id" "$title" "$i" "in-progress" \
      || die "failed to commit canonical task start"
    if [ "$guard_enabled" = "1" ]; then
      if [ "$change_start" = "1" ]; then
        change_guard_output="$(kbd_bottleneck_evaluate change before "$change" 0)" \
          || die "canonical change start postcommit evaluation blocked"
      fi
      task_guard_output="$(kbd_bottleneck_evaluate task before "$change/$id" 0)" \
        || die "canonical task start postcommit evaluation blocked"
    fi
    if [ "$change_start" = "1" ]; then
      if [ "$guard_enabled" = "1" ]; then
        kbd_bottleneck_print_signal "$change_guard_output"
      else
        printf 'Starting change 1 out of 1:   %s\n' "$change"
      fi
      fire change before "$change" 1 1
    fi
    fire task before "$change:$id" "$i" "$n"
    if [ "$guard_enabled" = "1" ]; then
      kbd_bottleneck_print_signal "$task_guard_output"
    else
      printf 'Starting task %s out of %s:   %s\n' "$i" "$n" "$title"
    fi ;;

  end-task)
    change="${1:-}"; id="${2:-}"; i="${3:-1}"; n="${4:-1}"; shift 4 || true; title="$*"
    [ -n "$change" ] && [ -n "$id" ] || die "usage: end-task <change> <id> <i> <n> <title>"
    progress_output="$(b_progress "$change")" || die "failed to read backend task progress"
    read -r before_tot before_comp before_rem <<< "$progress_output"
    final_task=0
    [ "${before_rem:-1}" -eq 1 ] && final_task=1
    guard_enabled=0
    if command -v kbd_bottleneck_active >/dev/null 2>&1 && kbd_bottleneck_active; then
      guard_enabled=1
      kbd_bottleneck_evaluate task after "$change/$id" 1 >/dev/null \
        || die "canonical task completion precommit evaluation blocked"
      if [ "$final_task" = "1" ]; then
        kbd_bottleneck_evaluate change after "$change" 1 >/dev/null \
          || die "canonical change completion precommit evaluation blocked"
      fi
    fi
    b_mark_done "$change" "$id"
    runtime_task_transition "$change" "$id" "$title" "$i" "complete" \
      || die "failed to commit canonical task completion"
    # Recompute progress from the backend so the count is authoritative.
    progress_output="$(b_progress "$change")" || die "failed to refresh backend progress after task mutation"
    read -r tot comp rem <<< "$progress_output"
    sync_progress "$change" "${comp:-$i}" "${tot:-$n}"
    # Re-derive the unified position model so the breadcrumb tracks task
    # completion live (CF-5). Best-effort; never aborts the driver.
    command -v kbd_position_sync >/dev/null 2>&1 && { kbd_position_sync || true; }
    if [ "$guard_enabled" = "1" ]; then
      task_guard_output="$(kbd_bottleneck_evaluate task after "$change/$id" 0)" \
        || die "canonical task completion postcommit evaluation blocked"
      if [ "$final_task" = "1" ]; then
        change_guard_output="$(kbd_bottleneck_evaluate change after "$change" 0)" \
          || die "canonical change completion postcommit evaluation blocked"
      fi
      kbd_bottleneck_print_signal "$task_guard_output"
    else
      printf 'Completed task %s out of %s:   %s\n' "$i" "$n" "$title"
    fi
    # Completion hooks run only after both the canonical transition and its
    # signed after-boundary receipt succeed.
    fire task after "$change:$id" "$i" "$n"
    if [ "$final_task" = "1" ]; then
      if [ "$guard_enabled" = "1" ]; then
        kbd_bottleneck_print_signal "$change_guard_output"
      else
        printf 'Completed change 1 out of 1:   %s\n' "$change"
      fi
      fire change after "$change" 1 1
    fi
    if [ "${rem:-0}" -gt 0 ]; then
      pending_titles="$(b_remaining_titles "$change" | paste -sd ' | ' -)"
      pending_preview="${pending_titles:-unknown}"
      printf 'Remaining tasks after task %s: %s out of %s — %s\n' "$i" "${rem:-0}" "${tot:-$n}" "$pending_preview"
    else
      printf 'Remaining tasks after task %s: 0 out of %s — none\n' "$i" "${tot:-$n}"
    fi ;;

  mark-done)
    [ -n "${1:-}" ] && [ -n "${2:-}" ] || die "usage: mark-done <change> <id>"
    change="$1"; id="$2"
    b_mark_done "$change" "$id" || exit 1
    # Same ledger sync as end-task, minus hooks and boundary signals. Without
    # it the backend flag moves and the ledger silently does not (G2c).
    md_seq=0; md_title=""
    while IFS=$'\t' read -r md_id md_done md_t; do
      md_seq=$((md_seq + 1))
      if [ "$md_id" = "$id" ]; then md_title="$md_t"; break; fi
    done <<< "$(b_list "$change" 2>/dev/null)"
    [ -n "$md_title" ] || md_title="$id"
    runtime_task_complete "$change" "$id" "$md_title" "$md_seq" \
      || die "failed to commit canonical task completion for $change/$id"
    progress_output="$(b_progress "$change")" || die "failed to refresh backend progress after mark-done"
    read -r tot comp rem <<< "$progress_output"
    sync_progress "$change" "${comp:-0}" "${tot:-0}"
    command -v kbd_position_sync >/dev/null 2>&1 && { kbd_position_sync || true; }
    printf 'mark-done: %s task %s done; ledger synced (%s of %s). Hooks were NOT fired; use begin-task/end-task for a full boundary.\n' \
      "$change" "$id" "${comp:-0}" "${tot:-0}" ;;

  reconcile)
    reconcile_main "$@" ;;

  verify)
    [ -n "${1:-}" ] || die "usage: verify <change>"
    if b_verify "$1"; then echo "verify: PASS"; else echo "verify: FAIL"; exit 1; fi ;;

  archive)
    [ -n "${1:-}" ] || die "usage: archive <change>"
    b_archive "$1" && echo "archived: $1" ;;

  ""|-h|--help)
    sed -n '2,45p' "$0" ;;

  *)
    die "unknown subcommand: $cmd (try --help)" ;;
esac
