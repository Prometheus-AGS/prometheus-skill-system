#!/usr/bin/env python3
"""Partition a Claude auto-memory MEMORY.md into a small project index plus queued lessons.

Design: docs/design/team-aware-learning-memory.md section 5 (file tier) and 10.
Guide:  docs/guide/memory-tiers.md

Every Claude subagent loads the whole auto-memory index, untargeted. This tool
shrinks the index to at most 4,096 bytes and moves what it removes into the
learning store, where SubagentStart delivery recalls it for the right role.

Deterministic rules:

  1. Non-bullet lines (title, blank lines, prose) always stay in the index.
  2. A bullet line carrying an explicit role marker -- `role:<id>`, `agent:<id>`
     or `@role/<id>` -- is REMOVED from the index and written as a lesson with
     visibility `role:<id>`.
  3. A bullet line carrying `team:<id>`, `team:*` or `[team]` is REMOVED and
     written with visibility `team`.
  4. Every other bullet is RANKED, then kept in priority order while the byte
     budget lasts (budget = limit minus the footer line). Ranks:
       0  a `project` entry whose slug or title matches the active phase
          (`phase` in .kbd-orchestrator/current-waypoint.json, else the
          `Position:` line of position-reminder.txt, read under --cwd; with no
          phase, rank 0 is empty);
       1  `feedback` entries and any entry whose title contains GLOBAL;
       2  other `project` entries (and unclassified ones), newest first by the
          YYYYMMDD / YYYY-MM-DD in the file name, undated last;
       3  `archive-*` entries.
     Order is rank 0, rank 1 (file order), rank 2 (newest first, ties and undated
     in file order), rank 3 (file order). Bullets are moved from the END of that
     order (archives first, then the oldest rank-2 entries, then rank 1, rank 0
     only if it alone exceeds the budget) until the index fits --limit. Kept
     bullets are written in priority order.
  5. A trailing "N entries moved to the learning store" footer is never ranked
     or moved. When anything was removed it is rewritten last with the running
     total (prior count + newly moved); when nothing was removed an existing
     footer is kept unchanged, so a second run on its own output moves nothing.

--dry-run (the default, no --apply) prints the rank and the keep/move decision of
every line; output is deterministic for identical input.

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


FOOTER_RE = re.compile(r"^\s*[-*+]\s+\((\d+) entries moved to the learning store")
LINK = re.compile(r"\[([^\]]*)\]\(([^)]*)\)")
DATE = re.compile(r"(?<!\d)(\d{4})-?(\d{2})-?(\d{2})(?!\d)")
PHASE_IN_TEXT = re.compile(r"phase-[a-z0-9][a-z0-9-]*")


def footer(moved: int) -> str:
    return f"- ({moved} entries moved to the learning store; delivered at SubagentStart, not read from this file)"


def size(lines: list[str]) -> int:
    return len("".join(line + "\n" for line in lines).encode("utf-8"))


def norm(text: str) -> str:
    return re.sub(r"[^a-z0-9]+", "-", text.lower()).strip("-")


def active_phase(cwd: Path) -> str:
    """Active KBD phase from waypoint json, else position-reminder.txt; '' if unknown."""
    base = cwd / ".kbd-orchestrator"
    try:
        phase = json.loads((base / "current-waypoint.json").read_text(encoding="utf-8")).get("phase")
        if isinstance(phase, str) and phase.strip():
            return phase.strip()
    except (OSError, ValueError, AttributeError):
        pass
    try:
        for line in (base / "position-reminder.txt").read_text(encoding="utf-8").splitlines():
            if line.startswith("Position:"):
                value = line.split(":", 1)[1].strip()
                found = PHASE_IN_TEXT.search(value)
                return found.group(0) if found else value
    except OSError:
        pass
    return ""


def entry_parts(line: str) -> tuple[str, str]:
    """(title, file name) of a bullet; title falls back to the whole text."""
    link = LINK.search(line)
    if link:
        return link.group(1), link.group(2).strip().split("/")[-1]
    return BULLET.sub("", line, count=1), ""


def file_date(name: str) -> int:
    found = DATE.search(name)
    return int("".join(found.groups())) if found else 0


def rank_of(line: str, phase: str) -> tuple[int, int]:
    """Return (rank, date) for a bullet. date is 0 when the file name carries none."""
    title, name = entry_parts(line)
    lower = name.lower()
    date = file_date(name)
    if re.match(r"^archive[-_]", lower):
        return 3, date
    is_project = bool(re.match(r"^project[-_]", lower))
    if is_project and phase:
        key = norm(phase)
        short = key[len("phase-"):] if key.startswith("phase-") else key
        hay = [norm(os.path.splitext(name)[0]), norm(title)]
        if any(key in h or (short and short in h) for h in hay):
            return 0, date
    if re.match(r"^feedback[-_]", lower) or "GLOBAL" in title:
        return 1, date
    return 2, date


def plan(source: str, limit: int, phase: str = "") -> dict:
    lines = source.splitlines()
    moved: list[dict] = []
    prior_moved = 0
    prior_footer = None
    entries = []
    for number, line in enumerate(lines, start=1):
        footer_match = FOOTER_RE.match(line)
        if footer_match:
            prior_moved += int(footer_match.group(1))
            prior_footer = line
            entries.append((number, line, "footer", None))
            continue
        kind, role, _ = classify(line)
        entries.append((number, line, kind, role))
    # Pass 1: role/team removals are decided by marker alone.
    for number, line, kind, role in entries:
        if kind == "role":
            moved.append({"line": number, "visibility": f"role:{role}", "role": role, "reason": "role marker", "text": lesson_text(line)})
        elif kind == "team":
            moved.append({"line": number, "visibility": "team", "role": None, "reason": "team marker", "text": lesson_text(line)})
    removed_numbers = {m["line"] for m in moved}
    # Pass 2: rank the remaining bullets and fill the budget in priority order.
    ranks: dict[int, int] = {}
    dates: dict[int, int] = {}
    for number, line, kind, _ in entries:
        if kind == "bullet":
            ranks[number], dates[number] = rank_of(line, phase)
    order = sorted(ranks, key=lambda n: (ranks[n], -dates[n] if ranks[n] == 2 else 0, n))
    text_of = {n: l for n, l, _, _ in entries}
    structure = [l for _, l, k, _ in entries if k == "structure"]
    structure_bytes = size(structure)
    # The footer is reserved up front (upper bound on the digits of the running total).
    reserve = len((footer(prior_moved + len(entries)) + "\n").encode("utf-8"))
    used = 0
    overflow = False
    kept_bullets: list[int] = []
    decision: dict[int, str] = {}
    for number in order:
        cost = len((text_of[number] + "\n").encode("utf-8"))
        if not overflow and structure_bytes + used + cost + reserve <= limit:
            kept_bullets.append(number)
            used += cost
            decision[number] = "keep"
        else:
            overflow = True
            decision[number] = "move"
            moved.append({"line": number, "visibility": "project", "role": None, "reason": "over budget", "text": lesson_text(text_of[number])})
    moved.sort(key=lambda m: m["line"])
    # Assemble: structure before the first bullet, ranked bullets, remaining structure, footer.
    first_bullet = min((n for n, _, k, _ in entries if k in ("bullet", "role", "team")), default=len(entries) + 1)
    kept: list[str] = [l for n, l, k, _ in entries if k == "structure" and n < first_bullet]
    kept += [text_of[n] for n in kept_bullets]
    kept += [l for n, l, k, _ in entries if k == "structure" and n > first_bullet]
    while kept and not kept[-1].strip():
        kept.pop()
    if moved:
        kept.append(footer(prior_moved + len(moved)))
    elif prior_footer is not None:
        kept.append(prior_footer)
    after = size(kept)
    line_report = []
    for number, line, kind, _ in entries:
        if kind == "structure":
            line_report.append({"line": number, "rank": None, "action": "keep"})
        elif kind == "footer":
            line_report.append({"line": number, "rank": None, "action": "footer"})
        elif kind in ("role", "team"):
            line_report.append({"line": number, "rank": None, "action": "move"})
        else:
            line_report.append({"line": number, "rank": ranks[number], "action": decision[number]})
    return {"limit": limit, "phase": phase, "bytes_before": len(source.encode("utf-8")), "bytes_after": after,
            "within_limit": after <= limit, "kept_lines": len(kept), "moved": moved, "ranks": line_report,
            "index": "\n".join(kept) + "\n"}


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
    base = Path(options.cwd) if options.cwd else Path.cwd()
    result = plan(source, options.limit, active_phase(base))
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
