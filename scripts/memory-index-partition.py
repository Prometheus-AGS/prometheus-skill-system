#!/usr/bin/env python3
"""Partition a Claude auto-memory MEMORY.md into a small project index plus queued lessons.

Design: docs/design/team-aware-learning-memory.md section 5 (file tier) and 10.
Guide:  docs/guide/memory-tiers.md

Every Claude subagent loads the whole auto-memory index, untargeted. This tool
shrinks the index to at most 4,096 bytes and moves what it removes into the
learning store, where SubagentStart delivery recalls it for the right role.

Deterministic rules, applied line by line (file order is preserved):

  1. Non-bullet lines (title, blank lines, prose) always stay in the index.
  2. A bullet line carrying an explicit role marker -- `role:<id>`, `agent:<id>`
     or `@role/<id>` -- is REMOVED from the index and written as a lesson with
     visibility `role:<id>`.
  3. A bullet line carrying `team:<id>`, `team:*` or `[team]` is REMOVED and
     written with visibility `team`.
  4. Every other bullet line STAYS in the index, in order, while the byte budget
     lasts. A line that would push the index past the budget (budget = limit
     minus the footer line) and every bullet after it is REMOVED and written as
     a project-scope lesson (visibility `project`).
  5. When anything was removed a one-line footer is appended to the index; the
     footer is counted against the budget.

The marker text itself is stripped from the lesson body. Nothing is removed
without being queued: with --apply the lessons are queued FIRST and the index is
only replaced (atomically, after a timestamped backup) when every queue write
succeeded.

Safety: the tool reads and writes only the path given as MEMORY_MD (plus its
`.bak-*` sibling). Default is a dry run that prints a JSON plan and touches
nothing. Lessons go through shared/scripts/lib/learning_write.py (queue, log and
index locations follow PROMETHEUS_LEARNING_QUEUE / _LOG_DIR / _INDEX_DIR, so a
scratch HOME fully isolates a run).

Usage:
  memory-index-partition.py MEMORY_MD [--limit 4096] [--apply]
                            [--cwd DIR] [--user-id PROJECT_SCOPE]
Exit: 0 ok, 1 error (nothing modified), 2 BLOCKED (learning_write unavailable).
"""
from __future__ import annotations

import argparse
import datetime
import json
import os
import re
import sys
import tempfile
from pathlib import Path

LIB = Path(__file__).resolve().parent.parent / "shared" / "scripts" / "lib"
DEFAULT_LIMIT = 4096
ROLE_MARKER = re.compile(r"(?:\b(?:role|agent):|@role/)([a-z][a-z0-9-]*)\b")
TEAM_MARKER = re.compile(r"(?:\bteam:(?:[a-z][a-z0-9-]*|\*)|\[team\])")
BULLET = re.compile(r"^\s*[-*+]\s+")


def classify(line: str) -> tuple[str, str | None, str]:
    """Return (destination, role, text). destination: keep | role | team | overflow-candidate."""
    if not BULLET.match(line):
        return "structure", None, line
    role = ROLE_MARKER.search(line)
    if role:
        return "role", role.group(1), line
    if TEAM_MARKER.search(line):
        return "team", None, line
    return "bullet", None, line


def lesson_text(line: str) -> str:
    text = BULLET.sub("", line, count=1)
    text = ROLE_MARKER.sub("", text)
    text = TEAM_MARKER.sub("", text)
    cleaned = " ".join(text.split())
    return cleaned or " ".join(BULLET.sub("", line, count=1).split())


def footer(moved: int) -> str:
    return f"- ({moved} entries moved to the learning store; delivered at SubagentStart, not read from this file)"


def size(lines: list[str]) -> int:
    return len("".join(line + "\n" for line in lines).encode("utf-8"))


def plan(source: str, limit: int) -> dict:
    lines = source.splitlines()
    structure_bytes = 0
    moved: list[dict] = []
    # Pass 1: role/team removals are decided by marker alone.
    entries = []
    for number, line in enumerate(lines, start=1):
        kind, role, _ = classify(line)
        entries.append((number, line, kind, role))
    for number, line, kind, role in entries:
        if kind == "role":
            moved.append({"line": number, "visibility": f"role:{role}", "role": role, "reason": "role marker", "text": lesson_text(line)})
        elif kind == "team":
            moved.append({"line": number, "visibility": "team", "role": None, "reason": "team marker", "text": lesson_text(line)})
    removed_numbers = {m["line"] for m in moved}
    # Pass 2: budget for the remaining lines. The footer is reserved up front.
    # Its count is an upper bound (all bullets), refined once the true count is known.
    kept: list[str] = []
    overflow = False
    structure = [l for n, l, k, _ in entries if k == "structure"]
    structure_bytes = size(structure)
    reserve = len((footer(len(entries)) + "\n").encode("utf-8"))
    used = 0
    for number, line, kind, role in entries:
        if number in removed_numbers:
            continue
        if kind == "structure":
            kept.append(line)
            continue
        cost = len((line + "\n").encode("utf-8"))
        if not overflow and structure_bytes + used + cost + reserve <= limit:
            kept.append(line)
            used += cost
        else:
            overflow = True
            moved.append({"line": number, "visibility": "project", "role": None, "reason": "over budget", "text": lesson_text(line)})
    moved.sort(key=lambda m: m["line"])
    if moved:
        kept.append(footer(len(moved)))
    # Trailing blank structure lines are not worth bytes.
    while kept and not kept[-1].strip():
        kept.pop()
    after = size(kept)
    return {"limit": limit, "bytes_before": len(source.encode("utf-8")), "bytes_after": after,
            "within_limit": after <= limit, "kept_lines": len(kept), "moved": moved, "index": "\n".join(kept) + "\n"}


def load_writer():
    sys.path.insert(0, str(LIB))
    try:
        from learning_write import write_lesson  # type: ignore
    except Exception as error:  # noqa: BLE001 - any import failure means BLOCKED
        print(f"BLOCKED: learning_write unavailable: {error}", file=sys.stderr)
        raise SystemExit(2)
    return write_lesson


def apply(path: Path, result: dict, cwd: Path, user_id: str | None) -> dict:
    write_lesson = load_writer()
    queued = []
    for item in result["moved"]:
        outcome = write_lesson(item["text"], {}, cwd=cwd, kind="lesson", visibility=item["visibility"],
                               user_id=user_id)
        ok = bool(outcome.get("written")) or bool(outcome.get("duplicate"))
        queued.append({"line": item["line"], "visibility": item["visibility"], "queued": ok,
                       "reason": outcome.get("reason")})
        if not ok:
            raise RuntimeError(f"lesson for line {item['line']} was not queued ({outcome.get('reason')}); index untouched")
    stamp = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    backup = path.with_name(f"{path.name}.bak-{stamp}")
    backup.write_bytes(path.read_bytes())
    handle, temp = tempfile.mkstemp(prefix=f".{path.name}.", dir=str(path.parent))
    try:
        with os.fdopen(handle, "w", encoding="utf-8") as stream:
            stream.write(result["index"])
        os.replace(temp, path)
    except BaseException:
        if os.path.exists(temp):
            os.unlink(temp)
        raise
    return {"backup": str(backup), "queued": queued}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("memory_md")
    parser.add_argument("--limit", type=int, default=DEFAULT_LIMIT)
    parser.add_argument("--apply", action="store_true", help="write the new index (with backup) and queue the lessons")
    parser.add_argument("--cwd", default=None, help="directory used to resolve team identity (default: current directory)")
    parser.add_argument("--user-id", default=None, help="explicit project scope for the queued lessons")
    options = parser.parse_args()
    path = Path(options.memory_md)
    if not path.is_file():
        print(f"error: not a file: {path}", file=sys.stderr)
        return 1
    if options.limit < 512:
        print("error: --limit below 512 bytes cannot hold a useful index", file=sys.stderr)
        return 1
    try:
        source = path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError) as error:
        print(f"error: cannot read {path}: {error}", file=sys.stderr)
        return 1
    result = plan(source, options.limit)
    report = {"path": str(path), "applied": False, **{k: v for k, v in result.items() if k != "index"}}
    if not result["within_limit"]:
        print(json.dumps(report, indent=2))
        print("error: structure lines alone exceed the limit; index untouched", file=sys.stderr)
        return 1
    if options.apply:
        cwd = Path(options.cwd) if options.cwd else Path.cwd()
        try:
            report.update(apply(path, result, cwd, options.user_id))
        except RuntimeError as error:
            print(json.dumps(report, indent=2))
            print(f"error: {error}", file=sys.stderr)
            return 1
        report["applied"] = True
    print(json.dumps(report, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
