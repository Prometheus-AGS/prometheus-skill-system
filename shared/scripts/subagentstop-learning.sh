#!/bin/bash
# subagentstop-learning.sh — attributed learning when a team-role subagent stops.
#
# Runs behind `team-role-guard.sh --only-team-roles`, so it only sees agents
# that resolve to a role in the active team (design §1, §6). From the payload:
#   1. lines of `last_assistant_message` that start with LESSON:, GOTCHA:,
#      DECISION:, [GLOBAL] or [USER] are written immediately through
#      learning_write.py (role-private by default; [GLOBAL]/[USER] promote);
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
from learning_write import write_lesson  # noqa: E402

cwd = Path(payload.get("cwd") or ".")
if not cwd.is_dir():
    cwd = Path.cwd()
message = payload.get("last_assistant_message") or ""
markers = {"LESSON:": ("lesson", None), "GOTCHA:": ("gotcha", None), "DECISION:": ("decision", None),
           "[GLOBAL]": ("lesson", "global"), "[USER]": ("lesson", "user")}
for raw in str(message).splitlines():
    line = raw.strip().lstrip("-* ").strip()
    for marker, (kind, visibility) in markers.items():
        if line.upper().startswith(marker):
            text = line[len(marker):].strip()
            if text:
                write_lesson(text, payload, cwd=cwd, kind=kind, visibility=visibility, stage="execute")
            break
PY

# Queue the asynchronous extraction job (identity fields added by the enqueuer).
python3 "$HERE/enqueue-learning-job.py" subagent_stop "${PROMETHEUS_HARNESS:-unknown}" < "$payload" >/dev/null 2>&1 || true
exit 0
