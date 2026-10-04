#!/usr/bin/env python3
"""Cross-agent awareness: path-overlap routing and the team digest.

Design: docs/design/team-aware-learning-memory.md §7.

Routing (`route`): a role-private lesson that carries `paths` is addressed to
the roles whose `owns` globs match those paths, other than its author
(`role:<r>`, at most MAX_ADDRESSEES). More matches than that, or no role owning
any of the paths, address it to `lead` instead. A lesson without paths is
never routed, and a lesson whose paths only the author owns stays private.

Digest: every role-private lesson also leaves one line of at most
DIGEST_LINE_MAX characters (author, paths, contentHash, time) in
`~/.prometheus/team-digest/<projectId>/<team>.jsonl`, rotated at
DIGEST_ROTATE_LINES lines and replaced atomically (temp file + rename). The
same line is mirrored to surreal-memory under `<team>/@team` by
learning_write; learning_recall reads the last DIGEST_RECALL_LINES lines back.
The digest never carries lesson text.

Usage (debugging aid):
  learning_route.py [--cwd DIR] --author ROLE --paths P ...   -> JSON audience
"""
from __future__ import annotations

import argparse
import fnmatch
import json
import os
import re
import sys
import tempfile
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from agent_identity import active_team, find_root, load_teams, role_ids  # noqa: E402

MAX_ADDRESSEES = 3
DIGEST_LINE_MAX = 160
DIGEST_ROTATE_LINES = 1000
DIGEST_RECALL_LINES = 50
LEAD_AUDIENCE = "lead"


def resolve_team(payload: dict | None, cwd: Path) -> tuple[str, dict | None]:
    """(team id, manifest) the same way agent_identity resolves the team."""
    payload = payload or {}
    working = payload.get("cwd") if isinstance(payload.get("cwd"), str) else None
    base = Path(working) if working and Path(working).is_dir() else cwd
    root = find_root(base)
    if not root:
        return "@solo", None
    return active_team(root, load_teams(root))


def owners(team: dict | None, paths: list[str]) -> list[str]:
    """Every role (manifest order) whose `owns` globs match at least one path."""
    if not team or not paths:
        return []
    matched: list[str] = []
    for role in team.get("roles", []):
        if not isinstance(role, dict) or not isinstance(role.get("id"), str):
            continue
        globs = [g for g in role.get("owns", []) or [] if isinstance(g, str)]
        if any(fnmatch.fnmatch(path, glob) for path in paths for glob in globs):
            matched.append(role["id"])
    return matched


def route(team: dict | None, paths: list[str], author_role: str) -> list[str]:
    """Audience entries (`role:<r>` / `lead`) for a lesson touching `paths`."""
    if not team or not paths:
        return []
    matched = owners(team, paths)
    others = [role for role in matched if role != author_role]
    if len(others) > MAX_ADDRESSEES:
        return [LEAD_AUDIENCE]
    if others:
        return [f"role:{role}" for role in others]
    if matched:
        return []  # only the author owns these paths: nothing to tell anyone else
    return [LEAD_AUDIENCE] if role_ids(team) else []


# --------------------------------------------------------------------------- digest
def digest_dir() -> Path:
    return Path(os.environ.get("PROMETHEUS_TEAM_DIGEST_DIR", str(Path.home() / ".prometheus" / "team-digest")))


def digest_path(project_id: str, team_id: str) -> Path:
    def safe(value: str) -> str:
        return re.sub(r"[^A-Za-z0-9._:@-]", "_", value) or "_"
    return digest_dir() / safe(project_id) / f"{safe(team_id)}.jsonl"


def digest_line(role: str, paths: list[str], digest: str, now: float | None = None) -> str:
    """One compact JSON line, at most DIGEST_LINE_MAX characters: the paths are
    trimmed (and counted as `+N`) until it fits."""
    stamp = int(now if now is not None else time.time())
    for keep in range(len(paths), -1, -1):
        shown = ",".join(paths[:keep])
        if keep < len(paths):
            shown += f"+{len(paths) - keep}"
        line = json.dumps({"by": role, "h": digest, "t": stamp, "p": shown}, separators=(",", ":"), ensure_ascii=True)
        if len(line) <= DIGEST_LINE_MAX:
            return line
    # An unusually long role id: drop the paths entirely.
    return json.dumps({"by": role[:40], "h": digest, "t": stamp}, separators=(",", ":"))[:DIGEST_LINE_MAX]


def digest_text(line: str | dict) -> str:
    """Human-readable form of a digest line (what recall delivers and what is
    mirrored to surreal-memory): never the lesson text."""
    record = json.loads(line) if isinstance(line, str) else line
    paths = record.get("p") or "(no paths)"
    return f"{record.get('by', 'an agent')} recorded a lesson on {paths} (h:{str(record.get('h', ''))[:8]})"


def append_digest(path: Path, line: str, rotate: int = DIGEST_ROTATE_LINES) -> bool:
    """Append `line`, keep the newest `rotate` lines, replace the file atomically."""
    try:
        path.parent.mkdir(parents=True, exist_ok=True)
        try:
            import fcntl
            lock = open(path.parent / f".{path.name}.lock", "a")
            fcntl.flock(lock, fcntl.LOCK_EX)
        except (ImportError, OSError):
            lock = None
        try:
            try:
                lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
            except OSError:
                lines = []
            lines.append(line)
            lines = lines[-rotate:]
            descriptor, temporary = tempfile.mkstemp(prefix=f".{path.name}.", suffix=".tmp", dir=str(path.parent))
            try:
                with os.fdopen(descriptor, "w", encoding="utf-8") as handle:
                    handle.write("\n".join(lines) + "\n")
                os.replace(temporary, path)
            except OSError:
                try:
                    os.unlink(temporary)
                except OSError:
                    pass
                return False
        finally:
            if lock is not None:
                lock.close()
        return True
    except OSError:
        return False


def read_digest(project_id: str, team_id: str, limit: int = DIGEST_RECALL_LINES) -> list[dict]:
    """The newest `limit` digest records (oldest first)."""
    try:
        raw = digest_path(project_id, team_id).read_text(encoding="utf-8", errors="replace").splitlines()
    except OSError:
        return []
    out: list[dict] = []
    for line in raw[-limit:]:
        try:
            record = json.loads(line)
        except ValueError:
            continue
        if isinstance(record, dict) and isinstance(record.get("h"), str):
            out.append(record)
    return out


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--cwd", default=os.getcwd())
    parser.add_argument("--author", default="")
    parser.add_argument("--paths", nargs="*", default=[])
    options = parser.parse_args()
    cwd = Path(options.cwd)
    team_id, team = resolve_team({}, cwd if cwd.is_dir() else Path.cwd())
    print(json.dumps({"teamId": team_id, "audience": route(team, options.paths, options.author)}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
