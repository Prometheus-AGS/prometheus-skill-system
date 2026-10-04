#!/bin/bash
# subagentstop-learning.sh — attributed learning when a team-role subagent stops.
#
# Runs behind `team-role-guard.sh --only-team-roles`, so it only sees agents
# that resolve to a role in the active team (design §1, §6). From the payload:
#   1. lines of `last_assistant_message` that start with LESSON:, GOTCHA:,
#      DECISION:, [GLOBAL] or [USER] are written immediately through
#      learning_write.py (role-private by default; [GLOBAL]/[USER] promote).
#      A line may end with `paths: a/b.ts, c/d.ts` (or `(paths: ...)`); those
#      paths drive path-overlap routing (design §7.1). Without one, the lesson
#      takes the files this subagent's transcript shows were written or edited
#      (Write/Edit/MultiEdit `file_path`; the subagent's own transcript only),
#      repo-relative and capped at 20;
#   2. a learning job carrying the agent's project/team/role and transcript is
#      queued for the worker, which extracts the rest asynchronously.
# Silent and exit 0 on every path: a hook must never block the Stop chain.
# bash 3.2 compatible.

set -u
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
command -v python3 >/dev/null 2>&1 || exit 0

payload="$(mktemp "${TMPDIR:-/tmp}/subagentstop-learning.XXXXXX")" || exit 0
trap 'rm -f "$payload"' EXIT
cat > "$payload"

python3 - "$payload" "$HERE" <<'PY' >/dev/null 2>&1 || true
import json, re, subprocess, sys
from pathlib import Path
payload_path, here = sys.argv[1], Path(sys.argv[2])
try:
    payload = json.loads(Path(payload_path).read_text(encoding="utf-8") or "{}")
except (OSError, ValueError):
    sys.exit(0)
if not isinstance(payload, dict):
    sys.exit(0)
sys.path.insert(0, str(here / "lib"))
from learning_write import split_lesson_paths, transcript_written_paths, write_lesson  # noqa: E402
from agent_identity import find_root  # noqa: E402

cwd = Path(payload.get("cwd") or ".")
if not cwd.is_dir():
    cwd = Path.cwd()
message = payload.get("last_assistant_message") or ""
root = find_root(cwd)
transcript_paths = None  # read lazily, once, only when a lesson needs it


def subagent_transcript():
    """The subagent's own transcript. Claude Code's `transcript_path` on
    SubagentStop is the parent's, so only `agent_transcript_path` is trusted
    there; a Codex child's `transcript_path` is its own rollout."""
    own = payload.get("agent_transcript_path")
    if isinstance(own, str) and own:
        return own
    shared = payload.get("transcript_path")
    return shared if "turn_id" in payload and isinstance(shared, str) and shared else None


markers = {"LESSON:": ("lesson", None), "GOTCHA:": ("gotcha", None), "DECISION:": ("decision", None),
           "[GLOBAL]": ("lesson", "global"), "[USER]": ("lesson", "user")}
for raw in str(message).splitlines():
    line = raw.strip().lstrip("-* ").strip()
    for marker, (kind, visibility) in markers.items():
        if line.upper().startswith(marker):
            text = line[len(marker):].strip()
            text, paths = split_lesson_paths(text, root)
            if text:
                if not paths:
                    if transcript_paths is None:
                        transcript_paths = transcript_written_paths(subagent_transcript(), root)
                    paths = transcript_paths
                write_lesson(text, payload, cwd=cwd, kind=kind, visibility=visibility, paths=paths, stage="execute")
            break
PY

# Queue the asynchronous extraction job (identity fields added by the enqueuer).
python3 "$HERE/enqueue-learning-job.py" subagent_stop "${PROMETHEUS_HARNESS:-unknown}" < "$payload" >/dev/null 2>&1 || true
exit 0
