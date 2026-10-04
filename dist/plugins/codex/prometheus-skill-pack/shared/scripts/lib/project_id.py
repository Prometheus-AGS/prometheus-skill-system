#!/usr/bin/env python3
"""Single project-id resolver for every memory and learning path.

Precedence (first non-empty wins):
  1. PROMETHEUS_PROJECT_ID
  2. .prometheus/project.json  -> projectId   (searched upward from --cwd)
  3. the runtime-registered project UUID (`prometheus kbd status --json`)
  4. project:<sha256(git common dir)>        (all worktrees share one id)

Also resolves the user scope: PROMETHEUS_USER_ID, else
user:<sha256(global git user.email)[:16]>.

Usage:
  project_id.py [--cwd DIR]          print the project id
  project_id.py [--cwd DIR] --json   print {"projectId","source","userScope"}

Never raises to the caller: on total failure it prints project:unknown and
exits 0, so hooks that use it stay silent and non-blocking.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
from pathlib import Path

RUNTIME_TIMEOUT_SECONDS = 2


def _from_project_json(start: Path) -> str:
    for directory in (start, *start.parents):
        candidate = directory / ".prometheus" / "project.json"
        if candidate.is_file():
            try:
                value = json.loads(candidate.read_text(encoding="utf-8")).get("projectId")
            except (OSError, ValueError, AttributeError):
                return ""
            return value.strip() if isinstance(value, str) else ""
    return ""


def _from_runtime(cwd: Path) -> str:
    binary = shutil.which("prometheus") or str(Path.home() / ".local/bin/prometheus")
    if not Path(binary).is_file():
        return ""
    try:
        completed = subprocess.run(
            [binary, "kbd", "status", "--json"],
            cwd=cwd,
            capture_output=True,
            text=True,
            timeout=RUNTIME_TIMEOUT_SECONDS,
            check=False,
        )
    except (OSError, subprocess.TimeoutExpired):
        return ""
    if completed.returncode != 0:
        return ""
    try:
        status = json.loads(completed.stdout)
    except ValueError:
        return ""
    for key in ("projectId", "project_id"):
        value = status.get(key) if isinstance(status, dict) else None
        if isinstance(value, str) and value.strip():
            return value.strip()
    project = status.get("project") if isinstance(status, dict) else None
    if isinstance(project, dict):
        value = project.get("id") or project.get("projectId")
        if isinstance(value, str) and value.strip():
            return value.strip()
    return ""


def _git(cwd: Path, *args: str) -> str:
    try:
        completed = subprocess.run(
            ["git", *args], cwd=cwd, capture_output=True, text=True, timeout=2, check=False
        )
    except (OSError, subprocess.TimeoutExpired):
        return ""
    return completed.stdout.strip() if completed.returncode == 0 else ""


def _from_git(cwd: Path) -> str:
    common = _git(cwd, "rev-parse", "--path-format=absolute", "--git-common-dir")
    if not common:
        return ""
    real = os.path.realpath(common)
    return "project:" + hashlib.sha256(real.encode("utf-8")).hexdigest()


def resolve_project(cwd: Path) -> tuple[str, str]:
    value = os.environ.get("PROMETHEUS_PROJECT_ID", "").strip()
    if value:
        return value, "env"
    value = _from_project_json(cwd)
    if value:
        return value, "project.json"
    if os.environ.get("PROMETHEUS_PROJECT_ID_SKIP_RUNTIME") != "1":
        value = _from_runtime(cwd)
        if value:
            return value, "runtime"
    value = _from_git(cwd)
    if value:
        return value, "git"
    return "project:unknown", "none"


def resolve_user_scope(cwd: Path) -> str:
    value = os.environ.get("PROMETHEUS_USER_ID", "").strip()
    if value:
        return value if value.startswith("@user:") or value.startswith("user:") else f"user:{value}"
    email = _git(cwd, "config", "--global", "user.email")
    if not email:
        return "user:unknown"
    return "user:" + hashlib.sha256(email.lower().encode("utf-8")).hexdigest()[:16]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--cwd", default=os.getcwd())
    parser.add_argument("--json", action="store_true")
    options = parser.parse_args()
    cwd = Path(options.cwd).expanduser()
    if not cwd.is_dir():
        cwd = Path.cwd()
    project, source = resolve_project(cwd)
    if options.json:
        print(json.dumps({"projectId": project, "source": source, "userScope": resolve_user_scope(cwd)}))
    else:
        print(project)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
