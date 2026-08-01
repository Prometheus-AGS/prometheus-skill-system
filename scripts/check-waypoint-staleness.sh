#!/usr/bin/env bash
# Detect a stale or self-referential KBD waypoint.
#
# WHY THIS LIVES HERE AND NOT IN THE SKILL
#
# The two defects it checks for are in *installed* skills under
# `~/.claude/skills/`. Editing those is worse than useless: the next install
# overwrites them, the change is invisible to git, and someone later "fixes" the
# same bug again. So this repo gets a DETECTOR, and the fix goes where the skill
# is authored.
#
# WHAT IT CATCHES
#
#   1. `.phase` disagrees with the active phase directory.
#      `kbd-reflect` never writes `.phase`, so after a couple of transitions the
#      waypoint confidently names a phase from two moves ago. Every tool that
#      trusts it — including the position reminder an agent reads first — then
#      reports the wrong position.
#
#   2. `next` is self-referential (`/kbd-next-phase <the phase you are in>`).
#      A next-command that points at the command that produced it is a loop: an
#      agent following it re-runs the transition instead of advancing.
#
# Exit codes: 0 clean · 1 stale/self-referential · 2 cannot check (missing files)
#
# Deliberately NOT bash 4+: macOS ships bash 3.2 and launchd runs /bin/bash.

set -uo pipefail

ROOT="${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
KBD="${ROOT}/.kbd-orchestrator"
WAYPOINT="${KBD}/current-waypoint.json"

fail=0

note() { printf '  %s\n' "$*"; }
bad()  { printf '  ✗ %s\n' "$*"; fail=1; }
ok()   { printf '  ✓ %s\n' "$*"; }

if [ ! -f "${WAYPOINT}" ]; then
    printf 'check-waypoint-staleness: no waypoint at %s\n' "${WAYPOINT}" >&2
    exit 2
fi

if ! command -v jq >/dev/null 2>&1; then
    printf 'check-waypoint-staleness: jq is required\n' >&2
    exit 2
fi

phase=$(jq -r '.phase // ""' "${WAYPOINT}")
next=$(jq -r '.next // ""' "${WAYPOINT}")
exact=$(jq -r '.exactNextCommand // ""' "${WAYPOINT}")

printf 'Waypoint: %s\n' "${WAYPOINT}"
note "phase:            ${phase:-<unset>}"
note "next:             ${next:-<unset>}"
note "exactNextCommand: ${exact:-<unset>}"
printf '\n'

# --- Check 1: does `.phase` name a directory that exists and is active? -----
if [ -z "${phase}" ]; then
    bad "phase is unset — no tool can report position from this waypoint"
elif [ ! -d "${KBD}/phases/${phase}" ]; then
    bad "phase '${phase}' has no directory under .kbd-orchestrator/phases/"
else
    # A phase whose reflection.md exists is CLOSED. A waypoint still pointing at
    # a closed phase is the exact `kbd-reflect` defect: it never rewrote .phase.
    if [ -f "${KBD}/phases/${phase}/reflection.md" ]; then
        newer=""
        for d in "${KBD}"/phases/*/; do
            p=$(basename "${d}")
            [ "${p}" = "${phase}" ] && continue
            if [ -f "${d}/progress.json" ] && [ ! -f "${d}/reflection.md" ]; then
                newer="${p}"
            fi
        done
        if [ -n "${newer}" ]; then
            bad "phase '${phase}' is CLOSED (has reflection.md) while '${newer}' is open"
            note "  kbd-reflect never writes .phase, so the waypoint names a stale phase."
            note "  FIX (where kbd-reflect is authored): after writing reflection.md,"
            note "  update current-waypoint.json's .phase to the next active phase —"
            note "  the same jq assignment kbd-next-phase.sh already performs."
        else
            ok "phase '${phase}' is closed, and no other phase is open"
        fi
    else
        ok "phase '${phase}' exists and is open"
    fi
fi

# --- Check 2: is `next` self-referential? -----------------------------------
case "${next}" in
    */kbd-next-phase*"${phase}"*|"/kbd-next-phase ${phase}")
        bad "next is SELF-REFERENTIAL: '${next}' points at the phase it is already in"
        note "  An agent following this re-runs the transition instead of advancing."
        note "  FIX: write the stage-appropriate command (/kbd-assess <phase>),"
        note "  which exactNextCommand in the same file already gets right."
        ;;
    *)
        [ -n "${next}" ] && ok "next is not self-referential"
        ;;
esac

# --- Check 3: do the two next-command fields agree? -------------------------
if [ -n "${next}" ] && [ -n "${exact}" ] && [ "${next}" != "${exact}" ]; then
    bad "next and exactNextCommand DISAGREE"
    note "  next:             ${next}"
    note "  exactNextCommand: ${exact}"
    note "  Two fields answering the same question differently means half the"
    note "  tools read one and half read the other."
fi

printf '\n'
if [ "${fail}" -eq 0 ]; then
    printf 'check-waypoint-staleness: OK\n'
else
    printf 'check-waypoint-staleness: STALE — see above\n'
fi
exit "${fail}"
