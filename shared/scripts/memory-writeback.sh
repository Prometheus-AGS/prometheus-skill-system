#!/usr/bin/env bash
# memory-writeback.sh — persist an accepted phase reflection as attributed lessons.
#
# Dual use:
#   1. As the orchestrator reflect:after builtin action (no stdin) — resolves the
#      active phase from the waypoint and persists its reflection.md.
#   2. As a Claude Code PostToolUse(Write|Edit) hook (stdin JSON) — fires only
#      when the written file is a reflection.md whose phase progress.json has no
#      reflect_gate=rejected, so a rejected reflection is never persisted.
#
# Design §6 (KBD reflect write-back). Every record goes through
# shared/scripts/lib/learning_write.py, so it carries a learning envelope and
# the design-table scope keys:
#   * Delta + Root Cause + Corrective Actions -> one `project` lesson (stage reflect)
#   * each Lessons Learned bullet -> one lesson: `[GLOBAL] …` -> global,
#     `[USER] …` -> user, everything else -> project
#   * Next Phase Seed (legacy: Next Phase Focus) -> one `project` progress record
#   * Codify as Skill? is never written back.
# A per-phase hash ledger (<phase>/.memory-writeback.tsv) skips records this
# phase already wrote, on top of learning_write's per-scope dedupe. Project
# lessons are also ingested into the project pk KB (global/user ones into the
# shared KB), one entry per lesson, by a detached process when pk is installed
# (PROMETHEUS_LEARNING_PK=0 disables it; =1 leaves it to learning_write). Always exits 0
# and prints nothing to stdout.
set -uo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
HOOK_LOG_LIB="$HERE/lib/hook-log.sh"
if [ -f "$HOOK_LOG_LIB" ]; then
  # shellcheck source=/dev/null
  source "$HOOK_LOG_LIB"
else
  hook_log_start() { :; }
  hook_log_end() { :; }
fi
hook_log_start "PostToolUse" "memory-writeback.sh"
finish() { hook_log_end 0; exit 0; }
command -v python3 >/dev/null 2>&1 || finish
LW="$HERE/lib/learning_write.py"
[ -f "$LW" ] || finish

# Resolve the reflection.md to persist.
REFLECTION=""
if [ ! -t 0 ]; then
  INPUT="$(cat 2>/dev/null || true)"
  if [ -n "$INPUT" ]; then
    FP="$(printf '%s' "$INPUT" | python3 -c "
import sys, json
try: d = json.load(sys.stdin)
except Exception: print(''); raise SystemExit
ti = d.get('tool_input', {}) or {}
print(ti.get('file_path') or ti.get('path') or '')
" 2>/dev/null || true)"
    case "$FP" in
      */reflection.md|reflection.md) REFLECTION="$FP" ;;
      *) finish ;;   # PostToolUse on a non-reflection file → nothing to do
    esac
  fi
fi

# Locate orchestrator root.
_root() {
  local dir="$PWD"
  while [ -n "$dir" ] && [ "$dir" != "/" ]; do
    [ -d "$dir/.kbd-orchestrator" ] && { printf '%s' "$dir"; return 0; }
    dir="$(dirname "$dir")"
  done
  return 1
}
ROOT="$(_root)" || finish

# Orchestrator-action path: resolve the active phase's reflection.md.
if [ -z "$REFLECTION" ]; then
  WP="$ROOT/.kbd-orchestrator/current-waypoint.json"
  [ -f "$WP" ] || finish
  PHASE="$(python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); print(d.get("phase") or d.get("activePhaseId") or "")' "$WP" 2>/dev/null || true)"
  [ -n "$PHASE" ] || finish
  REFLECTION="$ROOT/.kbd-orchestrator/phases/$PHASE/reflection.md"
fi
case "$REFLECTION" in /*) ;; *) REFLECTION="$PWD/$REFLECTION" ;; esac
[ -f "$REFLECTION" ] || finish

# Gate: never persist a rejected reflection.
PROGRESS="$(dirname "$REFLECTION")/progress.json"
if [ -f "$PROGRESS" ]; then
  GATE="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1])).get("reflect_gate") or "")' "$PROGRESS" 2>/dev/null || true)"
  if [ "$GATE" = "rejected" ]; then
    echo "[memory-writeback] skip: reflect_gate=rejected — not persisting" >&2
    finish
  fi
fi

python3 - "$REFLECTION" "$LW" "$ROOT" <<'PY' >/dev/null 2>"${TMPDIR:-/tmp}/memory-writeback.$$.err" || true
import hashlib, json, os, re, shutil, subprocess, sys
from pathlib import Path

reflection, lw_path, root = Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3])
sys.path.insert(0, str(lw_path.parent))
import learning_write  # noqa: E402

text = reflection.read_text(encoding="utf-8", errors="replace")
phase = reflection.parent.name


def section(*names):
    for name in names:
        match = re.search(r"(?ims)^##\s+" + re.escape(name) + r"\??\s*$(.*?)(?=^##\s|\Z)", text)
        if match and match.group(1).strip():
            return match.group(1).strip()
    return ""


def bullets(body):
    items, current = [], None
    for line in body.splitlines():
        match = re.match(r"^\s*[-*]\s+(.*\S)\s*$", line)
        if match:
            if current:
                items.append(current)
            current = match.group(1)
        elif current and line.strip() and line.startswith((" ", "\t")):
            current += " " + line.strip()
        elif not line.strip() and current:
            items.append(current)
            current = None
    if current:
        items.append(current)
    return items


records = []  # (text, visibility, kind, importance)
drc = [(title, section(title)) for title in ("Delta", "Root Cause", "Corrective Actions")]
if any(body for _, body in drc):
    composed = f"Phase {phase} reflection.\n\n" + "\n\n".join(f"{title}:\n{body}" for title, body in drc if body)
    records.append((composed, "project", "lesson", 0.7))
    # A [GLOBAL]/[USER]-prefixed corrective action is also a lesson in that scope.
    for line in drc[2][1].splitlines():
        tagged = re.match(r"^\s*(?:[-*]|\d+[.)])?\s*\[(GLOBAL|USER)\]\s*(.+\S)\s*$", line, re.I)
        if tagged:
            records.append((tagged.group(2), tagged.group(1).lower(), "lesson", 0.6))
for item in bullets(section("Lessons Learned", "Lessons")):
    visibility = "project"
    tagged = re.match(r"^\[(GLOBAL|USER)\]\s*(.*)$", item, re.I)
    if tagged:
        visibility, item = tagged.group(1).lower(), tagged.group(2)
    if item.strip() and not re.fullmatch(r"\(?none\)?\.?", item.strip(), re.I):
        records.append((item.strip(), visibility, "lesson", 0.6))
seed = section("Next Phase Seed", "Next Phase Focus")
if seed:
    records.append((f"Phase {phase} next-phase seed:\n{seed}", "project", "progress", 0.5))
# `## Codify as Skill?` is deliberately never extracted.

ledger = reflection.parent / ".memory-writeback.tsv"
try:
    done = set(ledger.read_text(encoding="utf-8").split())
except OSError:
    done = set()
written, pk_groups = 0, {"project": [], "global": [], "user": []}
for body, visibility, kind, importance in records:
    digest = hashlib.sha256(f"{visibility}\0{learning_write.normalise_text(body)}".encode("utf-8")).hexdigest()
    if digest in done:
        continue
    result = learning_write.write_lesson(body, {}, cwd=root, kind=kind, visibility=visibility,
                                         stage="reflect", importance=importance)
    if result.get("written") or result.get("duplicate"):
        done.add(digest)
        with open(ledger, "a", encoding="utf-8") as handle:
            handle.write(digest + "\n")
        written += int(bool(result.get("written")))
        if kind == "lesson":
            pk_groups[visibility if visibility in pk_groups else "project"].append(body)

pk = shutil.which("pk")
# PROMETHEUS_LEARNING_PK: unset -> one pk entry per lesson here; "1" ->
# learning_write already ingests every lesson itself; "0" -> no pk at all.
# One entry per lesson keeps each compiled entry (and its context snippet)
# about a single lesson. The ingests run one after another in one detached
# process, so the hook never waits for the model and the KB lock never contends.
jobs = []
if pk and os.environ.get("PROMETHEUS_LEARNING_PK", "") == "":
    for visibility, group in pk_groups.items():
        for body in group:
            args = [pk, "ingest", "--type", "Lesson", "--tag", f"vis:{visibility}", "--tag", "kbd:reflect",
                    "--tag", f"phase:{phase}", "--source", f"learning:{learning_write.content_hash(body)[:16]}"]
            if visibility != "project":
                args += ["--scope", "shared", "--yes"]
            jobs.append({"args": args, "input": f"# Phase {phase} reflection lesson\n\n{body}\n"})
if jobs:
    runner = ("import json,subprocess,sys\n"
              "for job in json.loads(sys.stdin.read()):\n"
              "    try: subprocess.run(job['args'], input=job['input'].encode(), stdout=subprocess.DEVNULL,"
              " stderr=subprocess.DEVNULL, timeout=300, check=False)\n"
              "    except Exception: pass\n")
    try:
        process = subprocess.Popen([sys.executable, "-c", runner], cwd=root, stdin=subprocess.PIPE,
                                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, start_new_session=True)
        process.stdin.write(json.dumps(jobs).encode("utf-8"))
        process.stdin.close()
    except OSError:
        pass
print(f"[memory-writeback] persisted {written} record(s) from {phase}", file=sys.stderr)
PY
ERR="${TMPDIR:-/tmp}/memory-writeback.$$.err"
if [ -s "$ERR" ]; then grep -E '^\[memory-writeback\]' "$ERR" >&2 || true; fi
rm -f "$ERR"
finish
