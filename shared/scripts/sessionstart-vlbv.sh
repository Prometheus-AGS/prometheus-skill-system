#!/usr/bin/env bash
# sessionstart-vlbv — state the verification method at session start.
#
# VLBV is the verification method for KBD processes. The `vlbv` skill carries
# the full text, but a skill loads only when the model picks it, and measured
# trigger rates for it were zero
# (docs/reports/2026-11-17-certification-phase2-pilot.md). This hook puts the
# short boundary rule into the session so the default does not depend on skill
# selection.
#
# Scope (mode), first match wins:
#   1. PROMETHEUS_VLBV_DEFAULT                     environment variable
#   2. <project>/.prometheus/vlbv-default          one word, per project
#   3. ${PROMETHEUS_HOME:-~/.prometheus}/config/vlbv-default   one word, per user
#   4. kbd                                         built-in default
#
#   kbd     print the rule only in a KBD-managed project (a .kbd-orchestrator/
#           directory at or above the project directory). Silent elsewhere.
#   always  print the rule in every session.
#   off     never print it.
#
# It is an instruction, not an interception: nothing is blocked. The operator
# can name another method at any time, and a signed KBD-METHOD decision for the
# phase still wins. Output must not begin with `{` (Codex parses such stdout as
# a structured response). Bash 3.2 compatible; always exits 0.

set -u

PROJECT_DIR="${CLAUDE_PROJECT_DIR:-$PWD}"
PROMETHEUS_HOME="${PROMETHEUS_HOME:-${HOME}/.prometheus}"

# First word of a one-word settings file, lower-cased; empty when unreadable.
read_mode_file() {
  [ -r "$1" ] || return 0
  tr -d '[:space:]' <"$1" 2>/dev/null | tr '[:upper:]' '[:lower:]'
}

mode="$(printf '%s' "${PROMETHEUS_VLBV_DEFAULT:-}" | tr '[:upper:]' '[:lower:]')"
[ -n "$mode" ] || mode="$(read_mode_file "${PROJECT_DIR}/.prometheus/vlbv-default")"
[ -n "$mode" ] || mode="$(read_mode_file "${PROMETHEUS_HOME}/config/vlbv-default")"
[ -n "$mode" ] || mode="kbd"

# An unrecognised value falls back to the built-in default rather than guessing.
case "$mode" in
  kbd|always|off) ;;
  *) mode="kbd" ;;
esac

[ "$mode" = "off" ] && exit 0

# A KBD-managed project has .kbd-orchestrator/ at or above the project directory.
is_kbd_project() {
  dir="$1"
  while [ -n "$dir" ] && [ "$dir" != "/" ]; do
    [ -d "${dir}/.kbd-orchestrator" ] && return 0
    dir="$(dirname "$dir")"
  done
  return 1
}

if is_kbd_project "$PROJECT_DIR"; then
  scope="This is a KBD-managed project, so VLBV (Value-Led Behavioral Verification) is the verification method."
elif [ "$mode" = "always" ]; then
  scope="VLBV (Value-Led Behavioral Verification) is enabled as the verification method for this session by configuration."
else
  exit 0
fi

cat <<RULE
${scope}
Applies whenever you test, build, lint, type-check or otherwise validate code written in this session, unless the operator names a different method.
- Agree the scenarios before generating. Write the whole agreed increment without executing generated code: no tests, builds, compiler checks, lint or type checks along the way. Editor and LSP diagnostics are static reading and stay available.
- At the value-phase boundary (B4), run the boundary packet once. After a FAIL, rerun only the affected scenarios, and only as an operator-authorized repair (B5).
- Report each scenario as exactly one of PASS, FAIL, BLOCKED or NOT_EXERCISED, and claim only what the evidence supports.
- Load the \`vlbv\` skill before any verification step for the full method.
Operator override: if the operator asks for another approach (for example test-first, or running checks after each edit), follow theirs and say that VLBV is not being applied. A signed KBD-METHOD decision for the phase takes precedence.
RULE
exit 0
