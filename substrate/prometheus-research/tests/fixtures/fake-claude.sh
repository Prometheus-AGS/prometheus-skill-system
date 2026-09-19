#!/bin/bash
# Fake headless harness for tests/job_execution.rs.
#
# Installed as `claude` in a directory that is the ONLY entry on the daemon's
# PATH, so the daemon resolves it exactly as it would the real binary. It
# records what a harness receives (the -p prompt and the environment contract
# from references/headless-execution.md), then does what the prompt asks: it
# runs the driver command the prompt names. With RESEARCH_STAGE_RUNNER set
# (inherited from the test) the real driver runs every stage with the
# deep-research fixture runner and fires the real deep-research hooks.
#
# Absolute shebang on purpose: PATH holds only this script's directory.
#
# Env from the test:
#   FAKE_CLAUDE_PATH  the PATH to restore for the driver (jq, awk, python3, bash)
#   FAKE_CLAUDE_LOG   where to record the prompt and environment
set -euo pipefail
export PATH="${FAKE_CLAUDE_PATH:-/usr/bin:/bin:/usr/local/bin:/opt/homebrew/bin}"
LOG="${FAKE_CLAUDE_LOG:-/dev/null}"

PROMPT=""; MODE_FLAG=""
while [ $# -gt 0 ]; do
  case "$1" in
    -p) MODE_FLAG="-p"; PROMPT="${2:-}"; shift 2 ;;
    --permission-mode) shift 2 ;;
    *) shift ;;
  esac
done

{
  echo "invocation: ${MODE_FLAG:-<no -p flag>}"
  echo "cwd: $(pwd)"
  echo "KBD_HOOKS_DISABLED=${KBD_HOOKS_DISABLED:-unset}"
  echo "RESEARCH_JOB_ID=${RESEARCH_JOB_ID:-unset}"
  echo "RESEARCH_OUTPUT_DIR=${RESEARCH_OUTPUT_DIR:-unset}"
  echo "--- prompt ---"
  printf '%s\n' "$PROMPT"
  echo "--- end prompt ---"
} >> "$LOG"

# A harness whose KBD hooks were not disabled would fire them on session
# events; this fixture models that as a marker file the test asserts is absent.
if [ "${KBD_HOOKS_DISABLED:-}" != "1" ]; then
  touch "$(dirname "$LOG")/kbd-hook.marker"
fi

# Run exactly the driver command the prompt names (the four-space-indented
# `bash <driver> ...` line). In runner mode the driver runs every stage itself
# and exits 0, so the checkpoint-mode resume loop is not needed here.
CMD="$(printf '%s\n' "$PROMPT" | grep -m1 '^    bash ' | sed 's/^    //')" || CMD=""
if [ -z "$CMD" ]; then
  echo "[fake claude] prompt names no driver command" >&2
  exit 2
fi
echo "[fake claude] running: $CMD" >> "$LOG"
eval "$CMD"
