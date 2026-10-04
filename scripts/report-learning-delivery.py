#!/usr/bin/env python3
"""Report what each agent is handed at start, per channel, against the old baseline.

Design: docs/design/team-aware-learning-memory.md section 10 (measurement).

Reads `delivery.jsonl` (written by the SubagentStart hook, one line per agent
start: harness, agent, `bytesByChannel`) and adds the file tier every agent
loads regardless of the hook: Claude Code's project auto-memory `MEMORY.md`
and Codex's `memories/memory_summary.md`. It prints bytes per agent per
channel and a total per agent.

With --require-reduction the exit status is 1 unless EVERY agent's total is
below the baseline of its harness (default 14,336 bytes for Claude Code and
11,059 bytes for Codex, the untargeted payload this design replaces). When a
harness has no recorded agents, a subagent that receives nothing from the hook
still loads the file tier, so the file tier alone is checked against that
harness's baseline: an oversized MEMORY.md fails the check even before the hook
has ever run.

Usage:
  report-learning-delivery.py [--index-dir DIR] [--cwd DIR]
        [--claude-memory FILE] [--codex-memory FILE]
        [--baseline-claude BYTES] [--baseline-codex BYTES]
        [--exclude-agent-prefix P ...] [--require-reduction] [--json]
Exit: 0 ok, 1 a total is not below its baseline (only with --require-reduction).
"""
from __future__ import annotations

import argparse
import json
import os
import re
import sys
from pathlib import Path

DEFAULT_BASELINE_CLAUDE = 14336
DEFAULT_BASELINE_CODEX = 11059
HARNESSES = ("claude-code", "codex")


def file_size(path: Path) -> int:
    try:
        return path.stat().st_size
    except OSError:
        return 0


def claude_memory_path(cwd: Path) -> Path:
    slug = re.sub(r"[^A-Za-z0-9]", "-", str(cwd.resolve()))
    return Path.home() / ".claude" / "projects" / slug / "memory" / "MEMORY.md"


def codex_memory_path() -> Path:
    return Path(os.environ.get("CODEX_HOME", str(Path.home() / ".codex"))) / "memories" / "memory_summary.md"


def load_starts(index_dir: Path, excluded: tuple[str, ...]) -> list[dict]:
    records = []
    try:
        lines = (index_dir / "delivery.jsonl").read_text(encoding="utf-8", errors="replace").splitlines()
    except OSError:
        return []
    for line in lines:
        try:
            record = json.loads(line)
        except ValueError:
            continue
        if not isinstance(record, dict) or record.get("event") != "SubagentStart" or record.get("timedOut"):
            continue
        if not isinstance(record.get("bytesByChannel"), dict):
            continue
        if any(str(record.get("agentId") or "").startswith(prefix) for prefix in excluded):
            continue
        records.append(record)
    return records


def build_report(records: list[dict], tiers: dict[str, int], baselines: dict[str, int]) -> list[dict]:
    agents: dict[tuple[str, str], dict] = {}
    for record in records:
        harness = record.get("harness") if record.get("harness") in HARNESSES else "claude-code"
        agent = str(record.get("agentId") or record.get("agentType") or "unknown")
        entry = agents.setdefault((harness, agent), {"harness": harness, "agent": agent, "role": record.get("roleId"), "channels": {}})
        for channel, size in record["bytesByChannel"].items():
            if isinstance(size, (int, float)):
                entry["channels"][channel] = entry["channels"].get(channel, 0) + int(size)
    rows = list(agents.values())
    for harness in HARNESSES:
        if not any(row["harness"] == harness for row in rows):
            rows.append({"harness": harness, "agent": "(any subagent: file tier only)", "role": None, "channels": {}})
    for row in rows:
        row["channels"]["file-tier"] = tiers[row["harness"]]
        row["total"] = sum(row["channels"].values())
        row["baseline"] = baselines[row["harness"]]
        row["belowBaseline"] = row["total"] < row["baseline"]
    return sorted(rows, key=lambda r: (r["harness"], r["agent"]))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--index-dir", default=os.environ.get("PROMETHEUS_LEARNING_INDEX_DIR", str(Path.home() / ".prometheus" / "learning-index")))
    parser.add_argument("--cwd", default=os.getcwd())
    parser.add_argument("--claude-memory", default=None)
    parser.add_argument("--codex-memory", default=None)
    parser.add_argument("--baseline-claude", type=int, default=DEFAULT_BASELINE_CLAUDE)
    parser.add_argument("--baseline-codex", type=int, default=DEFAULT_BASELINE_CODEX)
    parser.add_argument("--exclude-agent-prefix", action="append", default=[])
    parser.add_argument("--require-reduction", action="store_true")
    parser.add_argument("--json", action="store_true")
    options = parser.parse_args()
    cwd = Path(options.cwd)
    claude_file = Path(options.claude_memory) if options.claude_memory else claude_memory_path(cwd)
    codex_file = Path(options.codex_memory) if options.codex_memory else codex_memory_path()
    tiers = {"claude-code": file_size(claude_file), "codex": file_size(codex_file)}
    baselines = {"claude-code": options.baseline_claude, "codex": options.baseline_codex}
    rows = build_report(load_starts(Path(options.index_dir), tuple(options.exclude_agent_prefix)), tiers, baselines)
    if options.json:
        print(json.dumps({"rows": rows, "fileTier": {"claude-code": str(claude_file), "codex": str(codex_file)}}, sort_keys=True))
    else:
        print(f"file tier: claude-code {tiers['claude-code']} B ({claude_file}); codex {tiers['codex']} B ({codex_file})")
        for row in rows:
            label = f"{row['harness']} {row['agent']}" + (f" [{row['role']}]" if row.get("role") else "")
            print(f"{label}")
            for channel in sorted(row["channels"]):
                print(f"  {channel:<14} {row['channels'][channel]:>7} B")
            verdict = "below" if row["belowBaseline"] else "NOT below"
            print(f"  {'total':<14} {row['total']:>7} B  ({verdict} baseline {row['baseline']} B)")
    if options.require_reduction:
        failing = [row for row in rows if not row["belowBaseline"]]
        if failing:
            for row in failing:
                print(f"FAIL: {row['harness']} {row['agent']}: {row['total']} B is not below the {row['baseline']} B baseline", file=sys.stderr)
            return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
