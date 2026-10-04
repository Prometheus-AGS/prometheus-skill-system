#!/usr/bin/env python3
"""Resolve the agent behind a hook payload to (projectId, teamId, roleId).

Design: docs/design/team-aware-learning-memory.md §1.

Input is a hook payload on stdin (Claude Code or Codex). Role resolution:
  1. normalise agent_type: strip a `<plugin>:` prefix; map Codex `_` to `-`
     (Codex agent names allow only [a-z0-9_], role ids are kebab-case);
  2. if the normalised name is a role id in the active team manifest -> that role;
  3. else the role whose `owns` globs longest-match the touched paths
     (`--paths` or payload `tool_input.file_path`), ties broken by manifest order;
  4. else `unresolved`.
Team: `.agent-team/project-routing.json` activeTeam, else the sole team under
`.agent-team/<id>/team.json`, else `@solo`.

Usage:
  agent_identity.py [--cwd DIR] [--paths P ...]  < payload.json   -> JSON identity
  agent_identity.py [--cwd DIR] --is-team-role AGENT_TYPE          -> exit 0 if it
      resolves to a role in an active team, 1 otherwise (used by team-role-guard.sh)

Never fails a hook: malformed input yields an `unresolved` identity, exit 0.
"""
from __future__ import annotations

import argparse
import fnmatch
import json
import os
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from project_id import resolve_project  # noqa: E402

MAX_PAYLOAD_BYTES = 1_048_576
SOLO_TEAM = "@solo"


def normalise_agent_type(agent_type: str) -> str:
    name = (agent_type or "").strip()
    if ":" in name:
        name = name.rsplit(":", 1)[1]
    return name.replace("_", "-").lower()


def find_root(start: Path) -> Path | None:
    for directory in (start, *start.parents):
        if (directory / ".agent-team").is_dir():
            return directory
    return None


def load_teams(root: Path) -> dict[str, dict]:
    teams: dict[str, dict] = {}
    base = root / ".agent-team"
    for manifest in sorted(base.glob("*/team.json")):
        try:
            data = json.loads(manifest.read_text(encoding="utf-8"))
        except (OSError, ValueError):
            continue
        if isinstance(data, dict) and isinstance(data.get("roles"), list):
            team_id = data.get("id") if isinstance(data.get("id"), str) else manifest.parent.name
            teams[team_id] = data
    return teams


def active_team(root: Path, teams: dict[str, dict]) -> tuple[str, dict | None]:
    routing = root / ".agent-team" / "project-routing.json"
    if routing.is_file():
        try:
            selected = json.loads(routing.read_text(encoding="utf-8")).get("activeTeam")
        except (OSError, ValueError, AttributeError):
            selected = None
        if isinstance(selected, str) and selected in teams:
            return selected, teams[selected]
    if len(teams) == 1:
        team_id = next(iter(teams))
        return team_id, teams[team_id]
    return SOLO_TEAM, None


def role_ids(team: dict | None) -> list[str]:
    if not team:
        return []
    return [role["id"] for role in team.get("roles", []) if isinstance(role, dict) and isinstance(role.get("id"), str)]


def match_owns(team: dict | None, paths: list[str]) -> tuple[str, bool]:
    """Longest matching `owns` glob wins; returns (role, ambiguous)."""
    if not team or not paths:
        return "", False
    best: list[tuple[int, int, str]] = []
    for order, role in enumerate(team.get("roles", [])):
        if not isinstance(role, dict) or not isinstance(role.get("id"), str):
            continue
        for glob in role.get("owns", []) or []:
            if not isinstance(glob, str):
                continue
            if any(fnmatch.fnmatch(path, glob) for path in paths):
                best.append((len(glob), -order, role["id"]))
    if not best:
        return "", False
    best.sort(reverse=True)
    top = best[0]
    ambiguous = len(best) > 1 and best[1][0] == top[0] and best[1][2] != top[2]
    return top[2], ambiguous


def touched_paths(payload: dict, root: Path | None) -> list[str]:
    paths: list[str] = []
    tool_input = payload.get("tool_input")
    if isinstance(tool_input, dict):
        for key in ("file_path", "path", "notebook_path"):
            value = tool_input.get(key)
            if isinstance(value, str) and value:
                paths.append(value)
    normalised = []
    for path in paths:
        candidate = Path(path)
        if root and candidate.is_absolute():
            try:
                candidate = candidate.resolve().relative_to(root.resolve())
            except ValueError:
                pass
        normalised.append(candidate.as_posix())
    return normalised


def harness_of(payload: dict) -> str:
    explicit = os.environ.get("PROMETHEUS_HARNESS", "").strip()
    if explicit:
        return explicit
    if "turn_id" in payload:
        return "codex"
    return "claude-code" if payload else "other"


def resolve(payload: dict, cwd: Path, extra_paths: list[str]) -> dict:
    working = payload.get("cwd") if isinstance(payload.get("cwd"), str) else None
    base = Path(working) if working and Path(working).is_dir() else cwd
    project_id, source = resolve_project(base)
    root = find_root(base)
    teams = load_teams(root) if root else {}
    team_id, team = active_team(root, teams) if root else (SOLO_TEAM, None)
    agent_type = payload.get("agent_type") if isinstance(payload.get("agent_type"), str) else ""
    name = normalise_agent_type(agent_type)
    role, how, ambiguous = "unresolved", "none", False
    if name and name in role_ids(team):
        role, how = name, "agent_type"
    else:
        owner, ambiguous = match_owns(team, extra_paths or touched_paths(payload, root))
        if owner:
            role, how = owner, "owns"
    return {
        "projectId": project_id,
        "projectIdSource": source,
        "teamId": team_id,
        "roleId": role,
        "roleSource": how,
        "ambiguous": ambiguous,
        "agentType": agent_type or None,
        "agentId": payload.get("agent_id") if isinstance(payload.get("agent_id"), str) else None,
        "sessionId": payload.get("session_id") if isinstance(payload.get("session_id"), str) else None,
        "harness": harness_of(payload),
    }


def is_team_role(agent_type: str, cwd: Path) -> bool:
    root = find_root(cwd)
    if not root:
        return False
    teams = load_teams(root)
    team_id, team = active_team(root, teams)
    return team is not None and normalise_agent_type(agent_type) in role_ids(team)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--cwd", default=os.getcwd())
    parser.add_argument("--paths", nargs="*", default=[])
    parser.add_argument("--is-team-role", dest="team_role", default=None)
    options = parser.parse_args()
    cwd = Path(options.cwd).expanduser()
    if not cwd.is_dir():
        cwd = Path.cwd()
    if options.team_role is not None:
        return 0 if is_team_role(options.team_role, cwd) else 1
    raw = sys.stdin.buffer.read(MAX_PAYLOAD_BYTES + 1) if not sys.stdin.isatty() else b""
    try:
        payload = json.loads(raw[:MAX_PAYLOAD_BYTES] or b"{}")
    except (ValueError, UnicodeDecodeError):
        payload = {}
    if not isinstance(payload, dict):
        payload = {}
    print(json.dumps(resolve(payload, cwd, options.paths), sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
