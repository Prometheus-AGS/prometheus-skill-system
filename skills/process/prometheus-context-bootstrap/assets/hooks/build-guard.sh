#!/usr/bin/env bash
# PreToolUse:Bash — A-10 single-writer build discipline.
#
# Blocks a cargo compile while another cargo compile is already running inside this project: two writers
# on one target directory serialize on .cargo-lock at best and corrupt incremental state at worst.
# Separate worktrees are separate project roots and are not affected. Fail-open on every error path.
set -uo pipefail

root="${CLAUDE_PROJECT_DIR:-$PWD}"
command -v jq >/dev/null 2>&1 || exit 0
cmd="$(jq -r '.tool_input.command // empty' 2>/dev/null)" || exit 0
[[ -n "$cmd" ]] || exit 0

# Look at commands, not at text: drop heredoc bodies so a file that merely mentions cargo does not match.
stripped="$(printf '%s\n' "$cmd" | awk '
  skip { if ($0 == tag) skip = 0; next }
  { print }
  match($0, /<<-?[ ]*[\047"]?[A-Za-z_][A-Za-z0-9_]*[\047"]?/) {
    tag = substr($0, RSTART, RLENGTH); gsub(/[<\047" -]/, "", tag); skip = 1
  }')"
printf '%s' "$stripped" | grep -Eq '(^|[;&|(]|&&)[[:space:]]*([A-Z_]+=[^[:space:]]+[[:space:]]+)*cargo[[:space:]]+(build|check|test|clippy|run|doc|bench)\b' || exit 0

command -v pgrep >/dev/null 2>&1 || exit 0
command -v lsof >/dev/null 2>&1 || exit 0

for pid in $(pgrep -f 'cargo(-[a-z]+)?[[:space:]]+(build|check|test|clippy|run|doc|bench)' 2>/dev/null); do
  [[ "$pid" == "$$" || "$pid" == "$PPID" ]] && continue
  cwd="$(lsof -a -p "$pid" -d cwd -Fn 2>/dev/null | sed -n 's/^n//p' | head -1)"
  [[ -n "$cwd" && ( "$cwd" == "$root" || "$cwd" == "$root"/* ) ]] || continue
  cat >&2 <<EOF
BUILD GUARD — another cargo process (pid $pid) is already compiling in this project.

  its cwd: $cwd

One writer per target directory (A-10). Wait for it to finish, or do parallel work in a separate git
worktree with its own CARGO_TARGET_DIR and the shared CARGO_HOME (G-4).
EOF
  exit 2
done
exit 0
