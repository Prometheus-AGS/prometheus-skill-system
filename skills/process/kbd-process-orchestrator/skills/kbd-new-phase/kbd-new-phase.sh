#!/usr/bin/env bash
# skills/kbd-new-phase/kbd-new-phase.sh — Manually create a new top-level KBD phase.
#
# Usage:
#   kbd-new-phase.sh <name> [goal-1] [goal-2] …
#
# Validates the name, refuses collisions, writes goals.md and progress.json
# atomically, flips current-waypoint.json + project.json activePhase, fires
# phase:before via shared/lib/hooks.sh, and prints a confirmation banner.
#
# POSIX bash 3.2 compatible (macOS default). Requires jq.

set -euo pipefail

die() { printf 'kbd-new-phase: %s\n' "$*" >&2; exit 1; }
warn() { printf 'kbd-new-phase: warn: %s\n' "$*" >&2; }

name="${1:-}"
[[ -n "$name" ]] || die "usage: kbd-new-phase.sh <name> [goal-1] [goal-2] …"
shift
goals=("$@")

# ---------- Validation ----------
case "$name" in
  *..*) die "invalid name: parent traversal not allowed" ;;
  */*)  die "invalid name: slashes not allowed" ;;
  .|..) die "invalid name: '$name'" ;;
esac
[[ "$name" =~ ^[a-z0-9][a-z0-9._-]*$ ]] \
  || die "invalid name '$name': must match ^[a-z0-9][a-z0-9._-]*$"

command -v jq >/dev/null 2>&1 || die "jq is required (already a documented orchestrator dependency)"

wp=".kbd-orchestrator/current-waypoint.json"
pj=".kbd-orchestrator/project.json"
phase_dir=".kbd-orchestrator/phases/$name"

# Validate waypoint BEFORE creating any on-disk state (design D7).
if [[ -f "$wp" ]]; then
  jq -e . "$wp" >/dev/null 2>&1 \
    || die "malformed waypoint at $wp — fix by hand before retrying (no files were modified)"
fi

# Refuse name collisions BEFORE creating any on-disk state.
[[ -e "$phase_dir" ]] && die "phase already exists: $phase_dir (try /kbd-next-phase or pick another name)"

now="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

# ---------- 1. Phase directory + goals.md ----------
mkdir -p "$phase_dir"

{
  printf '# Goals\n\n'
  if [[ ${#goals[@]} -gt 0 ]]; then
    for g in "${goals[@]}"; do printf -- '- %s\n' "$g"; done
  else
    printf -- '<!-- TBD: enumerate goals before /kbd-assess -->\n'
  fi
} > "$phase_dir/goals.md.tmp"
mv -f "$phase_dir/goals.md.tmp" "$phase_dir/goals.md"

# Runtime-authority mode records phase creation and activation as typed events.
# Compatibility JSON is rendered by kbd-runtime and must never be edited here.
KBD_ORCHESTRATOR_ROOT="${KBD_ORCHESTRATOR_ROOT:-$HOME/.claude/skills/kbd-process-orchestrator}"
runtime_lib="$KBD_ORCHESTRATOR_ROOT/shared/lib/runtime-authority.sh"
if [[ -f "$runtime_lib" ]]; then
  # shellcheck source=/dev/null
  . "$runtime_lib"
fi
if command -v kbd_runtime_authoritative >/dev/null 2>&1 && kbd_runtime_authoritative "."; then
  mutation="$(kbd_runtime_mutation_args "." "phase-create:${name}")" || die "writer lease required"
  revision="$(printf '%s\n' "$mutation" | sed -n '1p')"
  lease_id="$(printf '%s\n' "$mutation" | sed -n '3p')"
  fencing_token="$(printf '%s\n' "$mutation" | sed -n '4p')"
  prometheus kbd --path . phase create \
    --expected-revision "$revision" --command-id "phase-create:${name}" \
    --lease-id "$lease_id" --fencing-token "$fencing_token" \
    --id "$name" --title "$name" >/dev/null
  mutation="$(kbd_runtime_mutation_args "." "phase-activate:${name}")" || die "writer lease required"
  revision="$(printf '%s\n' "$mutation" | sed -n '1p')"
  lease_id="$(printf '%s\n' "$mutation" | sed -n '3p')"
  fencing_token="$(printf '%s\n' "$mutation" | sed -n '4p')"
  prometheus kbd --path . phase activate \
    --expected-revision "$revision" --command-id "phase-activate:${name}" \
    --lease-id "$lease_id" --fencing-token "$fencing_token" \
    --id "$name" --exact-next-work "/kbd-assess $name" >/dev/null
  printf '\nCompleted kbd-new-phase — %s ready for /kbd-assess\n' "$name"
  printf '  phase:  %s\n' "$name"
  printf '  goals:  %s\n' "$phase_dir/goals.md"
  printf '  Next:   /kbd-assess %s\n' "$name"
  exit 0
fi

# ---------- 2. progress.json ----------
source_tool=""
if [[ -f "$wp" ]]; then
  source_tool="$(jq -r '.sourceTool // ""' "$wp" 2>/dev/null || true)"
fi
[[ -n "$source_tool" ]] || source_tool="unknown"

jq -n \
  --arg phase "$name" --arg src "$source_tool" --arg now "$now" '
{
  schemaVersion: "2",
  phase: $phase,
  parentPhase: null,
  childPhases: [],
  childPointer: null,
  assessment_complete: false,
  plan_complete: false,
  execute_complete: false,
  reflect_complete: false,
  implementation_total: 0,
  implementation_completed: 0,
  changes_total: 0,
  changes_completed: 0,
  completion: {
    primaryCounter: "implementation",
    implementation: { completed: 0, total: 0, status: "PENDING" },
    evidence: { status: "NOT_TRACKED", summary: null, blockers: [] },
    certification: { status: "NOT_TRACKED", summary: null, blockers: [] },
    publication: { status: "NOT_TRACKED", summary: null, blockers: [] }
  },
  completed_changes: [],
  active_change: null,
  blocked_changes: [],
  changes: [],
  last_updated: $now,
  last_updated_by: "kbd-new-phase",
  sourceTool: $src,
  createdBy: "kbd-new-phase",
  updatedAt: $now
}' > "$phase_dir/progress.json.tmp"
mv -f "$phase_dir/progress.json.tmp" "$phase_dir/progress.json"

# ---------- 3. Waypoint flip ----------
mkdir -p "$(dirname "$wp")"
if [[ -f "$wp" ]]; then
  prior_phase="$(jq -r '.phase // ""' "$wp" 2>/dev/null || printf '')"
  jq --arg phase "$name" --arg prev "$prior_phase" --arg now "$now" '
    del(
      .stage, .previous_phase, .last_updated, .last_updated_by,
      .exact_next_command, .fallback_command, .active_change,
      .next_pending_change, .changes_total, .changes_completed,
      .changesCompleted, .changesTotal
    ) |
    .schemaVersion   = "5" |
    .previousPhase    = (if $prev == "" then null else $prev end) |
    .phase            = $phase |
    .change           = null |
    .status           = "assessment_ready" |
    .currentTask      = ("run kbd-assess for " + $phase) |
    .nextPendingChange= null |
    .exactNextCommand = ("/kbd-assess " + $phase) |
    .parentPhase      = null |
    .childPhases      = [] |
    .childPointer     = null |
    .completionMetric = "implementation" |
    .implementationCompleted = 0 |
    .implementationTotal = 0 |
    .certificationStatus = "NOT_TRACKED" |
    .publicationStatus = "NOT_TRACKED" |
    .planRevision    = 1 |
    .revision        = ((.revision // 0) + 1) |
    .updatedAt        = $now
  ' "$wp" > "$wp.tmp"
else
  jq -n --arg phase "$name" --arg now "$now" '
    {
      schemaVersion: "5",
      phase: $phase,
      previousPhase: null,
      change: null,
      status: "assessment_ready",
      currentTask: ("run kbd-assess for " + $phase),
      nextPendingChange: null,
      sourceTool: "unknown",
      exactNextCommand: ("/kbd-assess " + $phase),
      parentPhase: null,
      childPhases: [],
      childPointer: null,
      completionMetric: "implementation",
      implementationCompleted: 0,
      implementationTotal: 0,
      certificationStatus: "NOT_TRACKED",
      publicationStatus: "NOT_TRACKED",
      planRevision: 1,
      revision: 0,
      updatedAt: $now
    }' > "$wp.tmp"
fi
mv -f "$wp.tmp" "$wp"

# ---------- 4. project.json activePhase flip ----------
if [[ -f "$pj" ]]; then
  jq --arg phase "$name" --arg now "$now" '.activePhase = $phase | .updatedAt = $now' "$pj" > "$pj.tmp"
  mv -f "$pj.tmp" "$pj"
else
  warn "$pj missing — writing a minimal project identity so KBD can keep project state isolated"
  root_pwd="$(pwd -P)"
  project_name="$(basename "$root_pwd")"
  jq -n \
    --arg name "$project_name" \
    --arg phase "$name" \
    --arg root "$root_pwd" \
    --arg now "$now" \
    '{
      name: $name,
      activePhase: $phase,
      focus_project_path: $root,
      updatedAt: $now,
      bootstrappedBy: "kbd-new-phase"
    }' > "$pj.tmp"
  mv -f "$pj.tmp" "$pj"
fi

# ---------- 5. Hook fire (best-effort) ----------
export KBD_ORCHESTRATOR_ROOT
hooks_lib="$KBD_ORCHESTRATOR_ROOT/shared/lib/hooks.sh"
waypoint_lib="$KBD_ORCHESTRATOR_ROOT/shared/lib/waypoint.sh"
if [[ -f "$hooks_lib" && -f "$waypoint_lib" ]]; then
  # shellcheck source=/dev/null
  . "$waypoint_lib"
  # shellcheck source=/dev/null
  . "$hooks_lib"
  if ! kbd_hooks_fire phase before "$name" 1 1; then
    warn "phase:before hook fire failed (phase still created)"
  fi
else
  warn "hooks subsystem unavailable at $KBD_ORCHESTRATOR_ROOT/shared/lib/ (phase still created)"
fi

# ---------- 6. Banner ----------
printf '\nCompleted kbd-new-phase — %s ready for /kbd-assess\n' "$name"
printf '  phase:  %s\n' "$name"
printf '  goals:  %s\n' "$phase_dir/goals.md"
printf '  Next:   /kbd-assess %s\n' "$name"
