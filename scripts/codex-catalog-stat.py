#!/usr/bin/env python3
"""Report what Codex's skill catalog actually costs the model.

Codex renders every discoverable skill into one "## Skills" section with a fixed
size budget. Names and paths are mandatory; descriptions get the remainder. Past
a few hundred skills the descriptions truncate to a handful of characters and the
model can no longer tell skills apart -- they load, but never auto-trigger.

Usage:
    codex debug prompt-input | python3 scripts/codex-catalog-stat.py
"""

import json
import re
import sys
from collections import Counter


def main() -> int:
    try:
        payload = json.load(sys.stdin)
    except json.JSONDecodeError as exc:
        print(f"not valid prompt-input JSON: {exc}", file=sys.stderr)
        return 1

    text = ""
    for item in payload:
        for chunk in item.get("content", []):
            body = chunk.get("text", "")
            if "### Available skills" in body:
                text = body
                break

    if not text:
        print("no '## Skills' section — Codex is exposing no skills at all.", file=sys.stderr)
        return 1

    roots = dict(re.findall(r"- `(r\d+)` = `([^`]+)`", text))
    tail = text[text.find("### Available skills"):]
    lines = [l for l in tail.split("\n") if l.startswith("- ")]

    per_root: Counter = Counter()
    desc_lengths = []
    for line in lines:
        # A line is "- <name>: <description> (file: rN/<path>)". Plugin-scoped names
        # embed a colon ("pack:skill"), so split on the trailing file reference first
        # and then on the first colon-space -- never on a bare colon.
        m = re.search(r"\(file: (r\d+)/", line)
        if not m:
            continue
        per_root[m.group(1)] += 1
        head = line[2 : m.start()].rstrip()
        _, sep, desc = head.partition(": ")
        desc_lengths.append(len(desc.strip()) if sep else 0)

    total = len(lines)
    avg = sum(desc_lengths) / len(desc_lengths) if desc_lengths else 0
    empty = sum(1 for d in desc_lengths if d == 0)

    print(f"catalog entries      : {total}")
    print(f"avg description      : {avg:.0f} chars")
    print(f"empty descriptions   : {empty}")
    print()
    print("entries per skill root:")
    for root, count in sorted(per_root.items()):
        print(f"  {root}  {count:4d}  {roots.get(root, '?')}")
    print()

    if avg >= 120:
        print("verdict: healthy — descriptions are intact, auto-triggering works.")
    elif avg >= 55:
        print("verdict: usable — descriptions are trimmed but still meaningful.")
    else:
        print("verdict: DEGRADED — descriptions are too short for the model to")
        print("         distinguish skills. Trim the catalog in config/codex-catalog.txt;")
        print("         excluded skills stay reachable as slash commands.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
