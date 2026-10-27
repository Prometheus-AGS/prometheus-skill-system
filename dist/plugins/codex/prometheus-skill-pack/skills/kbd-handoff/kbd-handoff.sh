#!/usr/bin/env bash
# skills/kbd-handoff/kbd-handoff.sh — lease-free cross-harness handoff packet.
#
# Usage:
#   kbd-handoff.sh --to <harness> [--phase <name>] [--repo <path>]... [--dry-run]
#
# Reads canonical KBD state and writes ONE handoff packet for the target
# harness:
#   .kbd-orchestrator/phases/<phase>/harness-handoffs/<harness>-<UTC>.md
#   .kbd-orchestrator/phases/<phase>/harness-handoffs/<harness>-<UTC>.json
# The JSON sidecar follows references/schemas/harness-handoff.schema.json.
#
# The script is read-only on KBD state: it never edits progress.json, the
# waypoint, position files or tasks, and it never calls a lease command (KBD
# has no writer lease; the journal lock serializes each command). Stage
# handoffs under handoffs/ belong to stage-gate.sh and are not touched.
#
# Exit codes: 0 packet written (or dry run printed), 1 refused (half-closed
# boundary, no phase, no .kbd-orchestrator) or write failure, 2 usage error or
# unknown harness.
#
# Environment: KBD_HANDOFF_TIMEOUT (seconds per external command, default 30),
# PROMETHEUS_HARNESS (the sending harness, recorded in the packet).
# bash 3.2 compatible.

set -u
PROG="kbd-handoff"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" 2>/dev/null && pwd -P)"
TIMEOUT_SECS="${KBD_HANDOFF_TIMEOUT:-30}"
SUPPORTED_HARNESSES="claude-code codex kimi-code opencode"
NL='
'

usage() {
  printf '%s\n' \
    'usage: kbd-handoff.sh --to <harness> [--phase <name>] [--repo <path>]... [--dry-run]' \
    '  --to       target harness: claude-code codex kimi-code opencode (aliases: claude, kimi)' \
    '  --phase    phase to hand off (default: the active phase from the waypoint)' \
    '  --repo     extra repository to report branch/clean/pushed state for (repeatable)' \
    '  --dry-run  print the packet; write nothing' >&2
}
die_usage() { printf '%s: %s\n' "$PROG" "$*" >&2; usage; exit 2; }
refuse() { printf '%s: REFUSED: %s\n' "$PROG" "$*" >&2; exit 1; }

WARNINGS=""
warn() {
  printf '%s: warn: %s\n' "$PROG" "$1" >&2
  WARNINGS="${WARNINGS}${1}${NL}"
}

run_bounded() {
  if command -v timeout >/dev/null 2>&1; then
    timeout "$TIMEOUT_SECS" "$@"
  elif command -v gtimeout >/dev/null 2>&1; then
    gtimeout "$TIMEOUT_SECS" "$@"
  else
    "$@"
  fi
}

# ---------------------------------------------------------------- arguments
target=""
opt_phase=""
dry_run=0
extra_repos=""
while [ $# -gt 0 ]; do
  case "$1" in
    --to) [ $# -ge 2 ] || die_usage "--to needs a harness"; target="$2"; shift 2 ;;
    --to=*) target="${1#--to=}"; shift ;;
    --phase) [ $# -ge 2 ] || die_usage "--phase needs a name"; opt_phase="$2"; shift 2 ;;
    --phase=*) opt_phase="${1#--phase=}"; shift ;;
    --repo) [ $# -ge 2 ] || die_usage "--repo needs a path"; extra_repos="${extra_repos}${2}${NL}"; shift 2 ;;
    --repo=*) extra_repos="${extra_repos}${1#--repo=}${NL}"; shift ;;
    --dry-run) dry_run=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) die_usage "unknown argument: $1" ;;
  esac
done

[ -n "$target" ] || die_usage "--to <harness> is required"
case "$target" in
  claude) target="claude-code" ;;
  kimi) target="kimi-code" ;;
esac
known=0
for h in $SUPPORTED_HARNESSES; do [ "$h" = "$target" ] && known=1; done
[ "$known" = 1 ] || die_usage "unknown harness '$target' (supported: $SUPPORTED_HARNESSES)"
command -v jq >/dev/null 2>&1 || { printf '%s: jq is required\n' "$PROG" >&2; exit 2; }

harness_skills_dir() {
  case "$1" in
    claude-code) echo ".claude/skills" ;;
    codex) echo ".codex/skills" ;;
    kimi-code) echo ".kimi-code/skills" ;;
    opencode) echo ".opencode/skills" ;;
  esac
}
# Project-local native role definitions, where the harness has them.
harness_agents_dir() {
  case "$1" in
    claude-code) echo ".claude/agents" ;;
    codex) echo ".codex/agents" ;;
    kimi-code) echo ".kimi-code/agents" ;;
    opencode) echo ".opencode/agents" ;;
    *) echo "" ;;
  esac
}
harness_agent_file() { # <harness> <role-id>
  if [ "$1" = "codex" ]; then
    printf '%s/%s.toml' "$(harness_agents_dir codex)" "$(printf '%s' "$2" | tr '-' '_')"
  else
    printf '%s/%s.md' "$(harness_agents_dir "$1")" "$2"
  fi
}

# ------------------------------------------------------------- project state
root=""
cursor_dir="$PWD"
while [ -n "$cursor_dir" ] && [ "$cursor_dir" != "/" ]; do
  if [ -d "$cursor_dir/.kbd-orchestrator" ]; then root="$cursor_dir"; break; fi
  cursor_dir="$(dirname "$cursor_dir")"
done
[ -n "$root" ] || refuse "no .kbd-orchestrator/ found above $PWD; run /kbd-init first"
kbd="$root/.kbd-orchestrator"
wp="$kbd/current-waypoint.json"
[ -f "$wp" ] && jq -e 'type == "object"' "$wp" >/dev/null 2>&1 || wp=""

status_json=""
if command -v prometheus >/dev/null 2>&1; then
  status_json="$(run_bounded prometheus kbd --path "$root" status --json </dev/null 2>/dev/null)" || status_json=""
  printf '%s' "$status_json" | jq -e 'type == "object"' >/dev/null 2>&1 || status_json=""
fi
status_ok=1
[ -n "$status_json" ] || { status_ok=0; status_json='{}'; }
sj() { printf '%s' "$status_json" | jq -r "$1" 2>/dev/null; }
wj() { [ -n "$wp" ] && jq -r "$1" "$wp" 2>/dev/null; }

# Phase: --phase, else the deepest existing node of the waypoint path, else the
# runtime's active phase.
phase=""
phase_dir=""
active_phase="$(wj '.activePhaseId // .phase // .activePhase // empty')"
[ -n "$active_phase" ] || active_phase="$(sj '.activePath.phaseId // empty')"
if [ -n "$opt_phase" ]; then
  phase="$opt_phase"
  if [ -d "$kbd/phases/$phase" ]; then
    phase_dir="$kbd/phases/$phase"
  else
    phase_dir="$(find "$kbd/phases" -type d -path "*/children/$phase" 2>/dev/null | head -n 1)"
  fi
else
  if [ -n "$wp" ]; then
    node="$kbd/phases"
    first=1
    for tok in $(jq -r '(.path // []) | map(select(type == "string" and . != "")) | .[]' "$wp" 2>/dev/null); do
      if [ "$first" = 1 ]; then cand="$node/$tok"; else cand="$node/children/$tok"; fi
      [ -d "$cand" ] || break
      node="$cand"; phase="$tok"; first=0
    done
    [ "$first" = 0 ] && phase_dir="$node"
  fi
  if [ -z "$phase" ] && [ -n "$active_phase" ]; then
    phase="$active_phase"
    [ -d "$kbd/phases/$phase" ] && phase_dir="$kbd/phases/$phase"
  fi
fi
[ -n "$phase" ] || refuse "no active phase (no waypoint path, no runtime activePath); pass --phase <name> or start one with /kbd-new-phase"
[ -n "$phase_dir" ] && [ -d "$phase_dir" ] || refuse "phase '$phase' has no directory under .kbd-orchestrator/phases; check the name with /kbd-status"
is_active=0
[ "$phase" = "$active_phase" ] && is_active=1
[ -n "$opt_phase" ] || is_active=1

printf 'Starting kbd-handoff — %s to %s\n' "$phase" "$target"

# ------------------------------------------------------------ preconditions
# Obligation rule (shared with the mini pack's kbd-handoff):
#   ignore  the active phase path, and the active change while it is not complete;
#   refuse  any open task obligation (begin without end), and a change or phase
#           the runtime marks complete or does not know (an interrupted close);
#   warn    a change or phase outside the active position that was started and
#           not finished (carried work).
if [ "$status_ok" = 1 ]; then
  classified="$(sj '
    def done($s): (($s // "") | ascii_downcase) as $v | ($v == "complete" or $v == "completed" or $v == "done");
    def chg($c): [(.phases // {})[] | (.changes // {}) | if type == "object" then .[$c] else (.[] | select(.id == $c)) end]
                 | map(select(. != null)) | .[0];
    . as $s
    | ($s.activePath.phasePath // [$s.activePath.phaseId] | map(select(. != null))) as $activePhases
    | ($s.activePath.changeId // "") as $activeChange
    | ($s.boundaryObligations // {}) | to_entries[] | .value
    | (.subject // "") as $subj
    | (if .boundary == "task" then "refuse"
       elif .boundary == "phase" then
         (if ($activePhases | index($subj)) != null then "ignore"
          else ($s.phases[$subj] // null) as $p
            | if $p == null or done($p.status) then "refuse" else "warn" end end)
       elif .boundary == "change" then
         ($s | chg($subj)) as $c
         | if $c == null or done($c.status) then "refuse"
           elif $subj == $activeChange then "ignore" else "warn" end
       else "warn" end) as $verdict
    | [$verdict, .boundary, $subj, (.exactSignal // "")] | @tsv')"
  blocking="$(printf '%s\n' "$classified" | awk -F '\t' '$1 == "refuse" { print $2 "\t" $3 "\t" $4 }')"
  carried="$(printf '%s\n' "$classified" | awk -F '\t' '$1 == "warn" { print $2 " " $3 }')"
  if [ -n "$blocking" ]; then
    {
      printf '%s: REFUSED: open boundary obligations must be repaired before a handoff.\n' "$PROG"
      printf 'An open task, or a change or phase that is complete or unknown to the runtime,\n'
      printf 'is a half-closed boundary the receiving harness would inherit. Repair each one:\n'
      printf '%s\n' "$blocking" | while IFS="$(printf '\t')" read -r b s sig; do
        printf '  - %s %s (%s)\n' "$b" "$s" "$sig"
        printf '    timeout %s prometheus kbd --path "%s" guard evaluate --boundary %s --edge after --subject %s --json --repair-projections </dev/null\n' \
          "$TIMEOUT_SECS" "$root" "$b" "$s"
      done
      printf 'Then re-run /kbd-handoff --to %s.\n' "$target"
    } >&2
    exit 1
  fi
  if [ -n "$carried" ]; then
    saved_ifs="$IFS"; IFS="$NL"
    for line in $carried; do
      warn "carried work: $line was started and not finished outside the active position"
    done
    IFS="$saved_ifs"
  fi
else
  warn "prometheus kbd status --json was unavailable; boundary obligations were NOT checked. The receiving harness must run it before any work."
fi

# ----------------------------------------------------------------- position
rel() { case "$1" in "$root"/*) printf '%s' "${1#"$root"/}" ;; *) printf '%s' "$1" ;; esac; }
progress="$phase_dir/progress.json"
[ -f "$progress" ] && jq -e 'type == "object"' "$progress" >/dev/null 2>&1 || progress=""
pj() { [ -n "$progress" ] && jq -r "$1" "$progress" 2>/dev/null; }

impl_done="$(pj '.completion.implementation.completed // .implementation_completed // empty')"
impl_total="$(pj '.completion.implementation.total // .implementation_total // empty')"
[ -n "$progress" ] || warn "no progress.json for phase $phase; implementation counter unknown"
stage="$(sj '.activePath.stageId // empty')"; [ -n "$stage" ] || stage="$(wj '.stageId // empty')"
stage_status="$(wj '.stageStatus // empty')"
lifecycle="$(sj '.lifecycle // empty')"; [ -n "$lifecycle" ] || lifecycle="$(wj '.lifecycle // .status // empty')"
revision="$(sj '.revision // empty')"; [ -n "$revision" ] || revision="$(wj '.revision // .sourceRevision // empty')"
plan_revision="$(sj '.planRevision // empty')"; [ -n "$plan_revision" ] || plan_revision="$(wj '.planRevision // empty')"
proj_done="$(sj '.completion.implementation.completed // empty')"
proj_total="$(sj '.completion.implementation.total // empty')"
exact_next="$(sj '.exactNextWork // empty')"; [ -n "$exact_next" ] || exact_next="$(wj '.exactNextCommand // empty')"
next_change=""; next_task=""
if [ "$is_active" = 1 ]; then
  next_change="$(wj '.nextChange // .change // empty')"; [ -n "$next_change" ] || next_change="$(sj '.activePath.changeId // empty')"
  next_task="$(wj '.nextTask // .currentTask // empty')"; [ -n "$next_task" ] || next_task="$(sj '.activePath.taskId // empty')"
fi
if [ -z "$next_change" ]; then
  next_change="$(pj '[(.changes // [])[] | select(((.status // "") | ascii_upcase) as $s | ($s != "COMPLETE" and $s != "DONE" and $s != "ARCHIVED"))][0].id // empty')"
fi
next_task_text=""
if [ -n "$next_change" ] && [ -n "$progress" ]; then
  next_task_text="$(jq -r --arg c "$next_change" '(.changes // [])[] | select(.id == $c) | .next_task_pending // empty' "$progress" 2>/dev/null | head -n 1)"
fi
reminder_next=""
[ -f "$kbd/position-reminder.txt" ] && reminder_next="$(grep -m 1 '^Next work:' "$kbd/position-reminder.txt" 2>/dev/null)"
execute_started="no"
[ -f "$phase_dir/execution.md" ] && execute_started="yes"

# ------------------------------------------------------------- repositories
REPOS_JSON='[]'
REPOS_MD=""
source_branch=""; source_commit=""
report_repo() { # <path> <is_project_root>
  local p="$1" branch head dirty dirty_list upstream ahead pushed clean
  if ! run_bounded git -C "$p" rev-parse --is-inside-work-tree </dev/null >/dev/null 2>&1; then
    [ "$2" = 1 ] || warn "repository $p is not a git work tree"
    return 0
  fi
  branch="$(run_bounded git -C "$p" rev-parse --abbrev-ref HEAD </dev/null 2>/dev/null)"
  head="$(run_bounded git -C "$p" rev-parse --short HEAD </dev/null 2>/dev/null)"
  dirty_list="$(run_bounded git -C "$p" status --porcelain </dev/null 2>/dev/null)"
  dirty=0; [ -n "$dirty_list" ] && dirty="$(printf '%s\n' "$dirty_list" | wc -l | tr -d ' ')"
  upstream="$(run_bounded git -C "$p" rev-parse --abbrev-ref --symbolic-full-name '@{u}' </dev/null 2>/dev/null)"
  ahead=""
  [ -n "$upstream" ] && ahead="$(run_bounded git -C "$p" rev-list --count '@{u}..HEAD' </dev/null 2>/dev/null)"
  clean=true; [ "$dirty" = 0 ] || clean=false
  pushed=false; [ -n "$upstream" ] && [ "${ahead:-1}" = 0 ] && pushed=true
  if [ "$2" = 1 ]; then source_branch="$branch"; source_commit="$head"; fi
  if [ "$clean" = false ]; then
    warn "dirty worktree in $p ($dirty entries): $(printf '%s\n' "$dirty_list" | head -n 20 | sed 's/^ *//' | tr '\n' ';' | sed 's/;$//')"
  fi
  if [ -z "$upstream" ]; then
    warn "branch $branch in $p has no upstream; the receiving harness may not see these commits"
  elif [ "$pushed" = false ]; then
    warn "branch $branch in $p is ${ahead:-?} commit(s) ahead of $upstream (unpushed)"
  fi
  REPOS_MD="${REPOS_MD}- \`$p\`: branch \`$branch\` at \`$head\`, $([ "$clean" = true ] && echo clean || echo "DIRTY ($dirty entries)"), $([ "$pushed" = true ] && echo "pushed to \`$upstream\`" || { [ -n "$upstream" ] && echo "UNPUSHED (${ahead:-?} ahead of \`$upstream\`)" || echo "NO UPSTREAM"; })${NL}"
  REPOS_JSON="$(printf '%s' "$REPOS_JSON" | jq -c --arg path "$p" --arg branch "$branch" --arg head "$head" \
    --argjson clean "$clean" --argjson dirty "$dirty" --arg upstream "$upstream" --arg ahead "$ahead" --argjson pushed "$pushed" \
    '. + [{path: $path, branch: $branch, head: $head, clean: $clean, dirtyCount: $dirty,
           upstream: (if $upstream == "" then null else $upstream end),
           ahead: (if $ahead == "" then null else ($ahead | tonumber) end), pushed: $pushed}]')"
}
report_repo "$root" 1
if [ -n "$extra_repos" ]; then
  saved_ifs="$IFS"; IFS="$NL"; set -f
  for r in $extra_repos; do
    [ -n "$r" ] || continue
    case "$r" in /*) ;; *) r="$root/$r" ;; esac
    report_repo "$r" 0
  done
  set +f; IFS="$saved_ifs"
fi
[ -n "$REPOS_MD" ] || REPOS_MD="- \`$root\` is not a git work tree.${NL}"

# ---------------------------------------------------- target harness checks
skills_rel="$(harness_skills_dir "$target")"
skills_dir="$HOME/$skills_rel"
BROKEN=""
MISSING_SKILLS=""
if [ -d "$skills_dir" ]; then
  for entry in "$skills_dir"/kbd-*; do
    [ -e "$entry" ] || [ -L "$entry" ] || continue
    name="$(basename "$entry")"
    if [ -L "$entry" ] && [ ! -e "$entry" ]; then
      BROKEN="${BROKEN}${name} -> $(readlink "$entry")${NL}"
    elif [ ! -f "$entry/SKILL.md" ]; then
      BROKEN="${BROKEN}${name} (no SKILL.md)${NL}"
    fi
  done
  for required in kbd-apply kbd-status kbd-process-orchestrator; do
    [ -f "$skills_dir/$required/SKILL.md" ] || MISSING_SKILLS="${MISSING_SKILLS}${required}${NL}"
  done
  if [ -n "$BROKEN" ]; then
    warn "broken kbd-* skills in ~/$skills_rel: $(printf '%s' "$BROKEN" | tr '\n' ';' | sed 's/;$//'); reinstall the pack (install.sh) to repair them"
  fi
  [ -z "$MISSING_SKILLS" ] || warn "skills missing from ~/$skills_rel: $(printf '%s' "$MISSING_SKILLS" | tr '\n' ' ')"
else
  warn "~/$skills_rel does not exist; the pack is not installed for $target"
fi

agents_rel="$(harness_agents_dir "$target")"
team_manifest=""
if [ -f "$root/.agent-team/project-routing.json" ]; then
  team_manifest="$(jq -r '.manifest // empty' "$root/.agent-team/project-routing.json" 2>/dev/null)"
fi
if [ -z "$team_manifest" ] && [ -d "$root/.agent-team" ]; then
  team_manifest="$(cd "$root" && find .agent-team -mindepth 2 -maxdepth 2 -name team.json 2>/dev/null | head -n 1)"
fi
MISSING_ROLES=""
team_detected=false
if [ -n "$team_manifest" ] && [ -f "$root/$team_manifest" ]; then
  team_detected=true
  if [ -z "$agents_rel" ]; then
    warn "the project has an agent team but $target has no project-local role definitions; adopt roles sequentially and say so"
  else
    for role in $(jq -r '(.roles // [])[] | (.id // .name // empty)' "$root/$team_manifest" 2>/dev/null); do
      f="$(harness_agent_file "$target" "$role")"
      [ -f "$root/$f" ] || MISSING_ROLES="${MISSING_ROLES}${f}${NL}"
    done
    [ -z "$MISSING_ROLES" ] || warn "native role definitions missing for $target: $(printf '%s' "$MISSING_ROLES" | tr '\n' ' ')"
  fi
fi

# ------------------------------------------------------------ read in order
READ_LIST=""
add_read() {
  # Only project-relative paths: a reference from a decision must not list files outside the project.
  case "$1" in ""|/*|..|../*|*/..|*/../*) return 0 ;; esac
  [ -e "$root/$1" ] && READ_LIST="${READ_LIST}${1}${NL}"
  return 0
}
add_read "AGENTS.md"
[ -f "$root/AGENTS.md" ] || add_read "CLAUDE.md"
add_read ".prometheus/gotchas.md"
phase_rel="$(rel "$phase_dir")"
add_read "$phase_rel/plan.md"
add_read "$phase_rel/assessment.md"
add_read "$phase_rel/execution.md"
add_read "$phase_rel/adopted-contract.json"
add_read "$phase_rel/method-context.md"
add_read "$phase_rel/method-context.json"
METHOD_CONTEXT=""
method_helper=""
for candidate in "${KBD_ORCHESTRATOR_ROOT:-}/shared/method-context.mjs" "$here/../../shared/method-context.mjs" "$here/../kbd-process-orchestrator/shared/method-context.mjs"; do
  if [ -f "$candidate" ]; then method_helper="$candidate"; break; fi
done
if [ -n "$method_helper" ] && command -v node >/dev/null 2>&1; then
  # Prefer the canonical phase id (a leaf slug or a legacy field does not name a child phase).
  method_phase_id="$(sj '.activePath.phaseId // empty' 2>/dev/null || true)"
  [ -n "$method_phase_id" ] || method_phase_id="$(pj '.phaseId // .phase // empty')"
  [ -n "$method_phase_id" ] || method_phase_id="$phase"
  if METHOD_CONTEXT="$(run_bounded node "$method_helper" --project "$root" --phase "$method_phase_id" --json)"; then
    while IFS= read -r method_reference; do
      [ -n "$method_reference" ] && add_read "$method_reference"
    done < <(printf '%s' "$METHOD_CONTEXT" | jq -r '.references[]? | select(.path | test("[\\u0000-\\u001f]") | not) | .path')
    method_status="$(printf '%s' "$METHOD_CONTEXT" | jq -r '.status // "UNKNOWN"')"
    [ "$method_status" = "RESOLVED_REFERENCES" ] || warn "selected method references unresolved: read method-context/contract and current canonical decisions before dependent work"
  else
    warn "selected method discovery unavailable; no method or stage authority inferred from the handoff"
    METHOD_CONTEXT=""
  fi
else
  warn "method-context helper or Node unavailable; selected-method recovery is unverified"
fi
[ -n "$next_change" ] && add_read "openspec/changes/$next_change"

# ------------------------------------------------- gates, decisions, blockers
GATES=""
if [ -f "$phase_dir/plan.md" ]; then
  GATES="$(awk '
    /^###? / {
      level = length($1)
      if (active && level <= start) active = 0
      if (!active && tolower($0) ~ /(operator gate|gates|decision)/) { active = 1; start = level; print; next }
    }
    active { print }
  ' "$phase_dir/plan.md" | head -n 80)"
fi
BLOCKERS="$(sj '(.blockers // {}) | to_entries[] | .value | select(.resolved != true) | "- \(.summary // .id)"')"
DECISIONS="$(sj '(.decisions // {}) | to_entries[] | .value | select(type == "object") | select(.resolved != true) | "- \(.summary // .title // .id // "decision")"')"

# ---------------------------------------- never-do list from AGENTS.md rules
NEVER=""
if [ -f "$root/AGENTS.md" ]; then
  NEVER="$(awk '
    /^#+ / {
      heading = $0; sub(/^#+ +/, "", heading)
      active = (tolower(heading) ~ /(must not|never|forbid|do not|don.t|hard rule|stability|user agency|safety|prohibit)/)
      printed = 0; lead = ""; lead_printed = 0
      next
    }
    # A short line ending in ":" (e.g. "Do not modify or touch:") qualifies
    # the bullets that follow it; keep it so the quoted items keep their sense.
    active && !/^[-*] / && /:[[:space:]]*$/ && length($0) < 120 { lead = $0; lead_printed = 0; next }
    active && /^- / && count < 24 {
      line = substr($0, 3)
      if (match(line, /\*\*[^*]+\*\*/)) {
        item = substr(line, RSTART + 2, RLENGTH - 4)
      } else {
        item = line
        if (match(item, /\. /)) item = substr(item, 1, RSTART)
      }
      if (length(item) > 220) item = substr(item, 1, 217) "..."
      if (!printed) { if (count) print ""; print "From AGENTS.md, \"" heading "\":"; print ""; printed = 1 }
      if (lead != "" && !lead_printed) { print "- " lead; lead_printed = 1 }
      print (lead != "" ? "  - " : "- ") item
      count++
    }
  ' "$root/AGENTS.md")"
fi

# -------------------------------------------------------------- the packet
ts="$(date -u '+%Y%m%dT%H%M%SZ')"
created_at="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
out_dir="$phase_dir/harness-handoffs"
packet="$out_dir/$target-$ts.md"
sidecar="$out_dir/$target-$ts.json"
packet_rel="$(rel "$packet")"
apply_cmd="~/$skills_rel/kbd-apply/kbd-apply.sh"
from_harness="${PROMETHEUS_HARNESS:-unknown}"
impl_text="${impl_done:-?} of ${impl_total:-?}"
next_text="${next_change:-none recorded}"
[ -n "$next_task" ] && next_text="$next_text, task $next_task"

start_prompt="You are taking over KBD phase \`$phase\` in \`$root\` on $target. Read \`$packet_rel\` first, then every file in its \"Read in order\" list. Run \`timeout $TIMEOUT_SECS prometheus kbd --path \"$root\" status --json </dev/null\` and confirm phase \`$phase\`, stage \`${stage:-unknown}\`, revision ${revision:-unknown}, next work $next_text before you change anything; if they differ, stop and report the difference. Drive every task through \`$apply_cmd begin-task\` and \`end-task\`, under timeout with stdin closed. Do nothing listed under \"Never do\"; ask the operator for any listed gate or open prerequisite, one batched question at a time."
start_prompt="$start_prompt Resolve current selected method/contract/supersession from canonical decisions and method-context references before dependent action. For selected VLBV, preserve the complete value-phase boundary; task/hour/compiler/reviewer triggers grant no generated-code verification authority. Retain original evidence, candidate freshness, consumed budgets and unresolved effects; reconcile uncertain effects before retry. Instruction-only handoff grants no release authority or automatic successor."

render_list() { # <newline list> <empty text>
  if [ -n "$1" ]; then printf '%s\n' "$1" | sed '/^$/d; s/^/- /'; else printf -- '- %s\n' "$2"; fi
}

render_packet() {
  printf '# Handoff to %s: %s\n\n' "$target" "$phase"
  printf 'Written %s by `/kbd-handoff` from %s at commit `%s` (branch `%s`). Read this file first, then the files in "Read in order". This packet records position and rules; it does not override AGENTS.md, the plan, or canonical KBD state, and it grants no lease: KBD has none, and the journal lock serializes each command.\n\n' \
    "$created_at" "$from_harness" "${source_commit:-unknown}" "${source_branch:-unknown}"

  printf '## Position\n\n'
  printf -- '- Project: `%s`\n' "$root"
  printf -- '- Phase `%s` (`%s`): stage `%s`%s, lifecycle `%s`, Execute started: %s.\n' \
    "$phase" "$phase_rel" "${stage:-unknown}" "${stage_status:+ ($stage_status)}" "${lifecycle:-unknown}" "$execute_started"
  printf -- '- Implementation (this phase, `completion.implementation`): %s.\n' "$impl_text"
  [ -n "$proj_total" ] && printf -- '- Project-wide implementation across all phases: %s of %s.\n' "${proj_done:-?}" "$proj_total"
  printf -- '- Next: change `%s`%s.\n' "${next_change:-none recorded}" "${next_task:+, task $next_task}"
  [ -n "$next_task_text" ] && printf -- '- Next pending task text: %s\n' "$next_task_text"
  [ -n "$reminder_next" ] && printf -- '- Position reminder: %s\n' "$reminder_next"
  [ -n "$exact_next" ] && printf -- '- Operator note (intent, not a work selector): `%s`\n' "$exact_next"
  printf -- '- Runtime revision %s, plan revision %s%s.\n\n' "${revision:-unknown}" "${plan_revision:-unknown}" \
    "$([ "$status_ok" = 1 ] || echo '; NOT read from `prometheus kbd status` (unavailable)')"

  printf '## Selected method and authority\n\n'
  if [ -n "$METHOD_CONTEXT" ]; then
    printf '%s' "$METHOD_CONTEXT" | jq '{selectedMethod,phaseId,sourceRevision,status,blockers,policyConflicts,contract:{path:.contract.path,contractDigest:.contract.contractDigest},methodDecision:{id:.methodDecision.id,supersedes:.methodDecision.supersedes},references,stageAuthority,preserve,control}'
  else
    printf 'UNKNOWN — resolve current canonical method and contract before dependent action.\n'
  fi
  printf '\nThis is an observed/derived recovery view, not execution permission. Preserve original evidence, budget consumption and unresolved effects; do not revive a retired method from old memory.\n\n'

  printf '## Repositories\n\n%s\n' "$REPOS_MD"

  printf '## Read in order\n\n'
  printf '1. `%s` (this file)\n' "$packet_rel"
  printf '%s' "$READ_LIST" | awk 'NF { printf "%d. `%s`\n", NR + 1, $0 }'
  printf '\n'

  printf '## Pending operator gates and decisions\n\n'
  if [ -n "$GATES" ]; then
    printf 'Excerpt from `%s/plan.md` (the plan is authoritative):\n\n' "$phase_rel"
    printf '%s\n' "$GATES" | sed 's/^#/###/'
    printf '\n'
  else
    printf -- '- The plan lists no operator gates or decisions.\n'
  fi
  [ -n "$DECISIONS" ] && printf '\nOpen runtime decisions:\n\n%s\n' "$DECISIONS"
  [ -n "$BLOCKERS" ] && printf '\nUnresolved runtime blockers:\n\n%s\n' "$BLOCKERS"
  printf '\n'

  printf '## Task execution rules\n\n'
  printf -- '- Drive every task through kbd-apply: `%s begin-task <change> <id> <i> <n> "<title>"`, then the work, then `end-task` with the same arguments. A checkbox ticked by hand does not move the runtime.\n' "$apply_cmd"
  printf -- '- Run every `kbd-apply` and `prometheus kbd` call under `timeout` with stdin closed (`</dev/null`).\n'
  printf -- '- After an interrupted close, read `boundaryObligations` from `prometheus kbd status --json` and repair each with `prometheus kbd guard evaluate --boundary <change|task> --edge after --subject <change|change/id> --json --repair-projections`.\n'
  printf -- '- Never hand-edit `progress.json`, `current-waypoint.json`, `position.json`, or `position-reminder.txt`; they are runtime projections. Move position only with typed `prometheus kbd` commands.\n'
  printf -- '- There is no writer lease to claim or release. Do not look for `prometheus kbd handoff` or heartbeat commands; they were removed.\n'
  if [ "$team_detected" = true ]; then
    printf -- '- The project has an agent team (`%s`): edit only paths your role owns, and run reviewers in a separate context after the implementation phase.\n' "$team_manifest"
  fi
  printf '\n'

  printf '## Never do\n\n'
  if [ -n "$NEVER" ]; then
    printf '%s\n' "$NEVER"
    printf '\nThese lines are quoted from AGENTS.md; read the full rules there.\n\n'
  else
    printf -- '- AGENTS.md lists no hard rules; follow CLAUDE.md and the plan.\n\n'
  fi

  printf '## Target harness readiness\n\n'
  printf -- '- Skills directory: `~/%s` %s.\n' "$skills_rel" "$([ -d "$skills_dir" ] && echo present || echo MISSING)"
  if [ -n "$BROKEN" ]; then
    printf -- '- Broken kbd-* skills:\n'
    printf '%s\n' "$BROKEN" | sed '/^$/d; s/^/  - /'
  else
    printf -- '- No broken kbd-* skills found.\n'
  fi
  [ -n "$MISSING_SKILLS" ] && printf -- '- Missing required skills: %s\n' "$(printf '%s' "$MISSING_SKILLS" | tr '\n' ' ')"
  if [ "$team_detected" = true ]; then
    if [ -z "$agents_rel" ]; then
      printf -- '- %s has no project-local role definitions; roles run sequentially.\n' "$target"
    elif [ -n "$MISSING_ROLES" ]; then
      printf -- '- Missing native role definitions:\n'
      printf '%s\n' "$MISSING_ROLES" | sed '/^$/d; s/^/  - `/; s/$/`/'
    else
      printf -- '- Native role definitions present in `%s/`.\n' "$agents_rel"
    fi
  else
    printf -- '- No agent team detected in this project.\n'
  fi
  printf '\n'

  printf '## Open prerequisites\n\n'
  render_list "$WARNINGS" "None found by kbd-handoff."
  printf '\n'

  printf '## Start prompt\n\nPaste this into %s:\n\n```text\n%s\n```\n' "$target" "$start_prompt"
}

packet_text="$(render_packet)"

sidecar_text="$(jq -n \
  --arg harness "$target" --arg fromHarness "$from_harness" --arg phase "$phase" --arg phaseDir "$phase_rel" \
  --arg createdAt "$created_at" --arg packet "$packet_rel" --arg project "$root" \
  --arg stage "$stage" --arg lifecycle "$lifecycle" --arg revision "$revision" --arg planRevision "$plan_revision" \
  --arg implDone "$impl_done" --arg implTotal "$impl_total" \
  --arg nextChange "$next_change" --arg nextTask "$next_task" --arg exactNext "$exact_next" \
  --argjson statusVerified "$([ "$status_ok" = 1 ] && echo true || echo false)" \
  --argjson repos "$REPOS_JSON" --arg readList "$READ_LIST" --arg warnings "$WARNINGS" \
  --arg skillsDir "~/$skills_rel" --argjson skillsDirExists "$([ -d "$skills_dir" ] && echo true || echo false)" \
  --arg broken "$BROKEN" --arg missingSkills "$MISSING_SKILLS" \
  --argjson teamDetected "$team_detected" --arg agentsDir "$agents_rel" --arg missingRoles "$MISSING_ROLES" \
  --arg startPrompt "$start_prompt" '
  def lines($s): $s | split("\n") | map(select(length > 0));
  def nullable($s): if $s == "" then null else $s end;
  def num($s): if $s == "" then null else ($s | tonumber? // null) end;
  {
    schemaVersion: "1",
    kind: "kbd-harness-handoff",
    harness: $harness,
    fromHarness: $fromHarness,
    project: $project,
    phase: $phase,
    phaseDir: $phaseDir,
    createdAt: $createdAt,
    packet: $packet,
    statusVerified: $statusVerified,
    position: {
      stage: nullable($stage),
      lifecycle: nullable($lifecycle),
      revision: num($revision),
      planRevision: num($planRevision),
      implementation: {completed: num($implDone), total: num($implTotal)},
      nextChange: nullable($nextChange),
      nextTask: nullable($nextTask),
      exactNextWork: nullable($exactNext)
    },
    repos: $repos,
    readInOrder: ([$packet] + lines($readList)),
    targetChecks: {
      skillsDir: $skillsDir,
      skillsDirExists: $skillsDirExists,
      brokenSkills: lines($broken),
      missingSkills: lines($missingSkills),
      teamDetected: $teamDetected,
      agentsDir: nullable($agentsDir),
      missingRoleDefinitions: lines($missingRoles)
    },
    warnings: lines($warnings),
    startPrompt: $startPrompt
  }')" || { printf '%s: failed to render the JSON sidecar\n' "$PROG" >&2; exit 1; }

if [ "$dry_run" = 1 ]; then
  printf '%s\n' "$packet_text"
  printf '\n(dry run: would write %s and %s)\n' "$packet_rel" "$(rel "$sidecar")"
  printf 'Completed kbd-handoff — %s dry run (no packet written)\n' "$phase"
  exit 0
fi

mkdir -p "$out_dir" || { printf '%s: cannot create %s\n' "$PROG" "$out_dir" >&2; exit 1; }
tmp_md="$(mktemp "$out_dir/.handoff.XXXXXX")" || exit 1
tmp_json="$(mktemp "$out_dir/.handoff.XXXXXX")" || { rm -f "$tmp_md"; exit 1; }
printf '%s\n' "$packet_text" >"$tmp_md" && printf '%s\n' "$sidecar_text" >"$tmp_json" \
  && mv -f "$tmp_md" "$packet" && mv -f "$tmp_json" "$sidecar" \
  || { rm -f "$tmp_md" "$tmp_json"; printf '%s: failed to write the packet\n' "$PROG" >&2; exit 1; }

printf 'Completed kbd-handoff — %s packet %s\n\n' "$phase" "$packet_rel"
printf 'Start prompt for %s:\n\n%s\n' "$target" "$start_prompt"
exit 0
