#!/usr/bin/env python3
"""Durably enqueue one metadata-only learning job without network or inference."""

from __future__ import annotations

import datetime
import hashlib
import json
import os
from pathlib import Path
import sys

MAX_INPUT_BYTES = 1_048_576
FINAL_STATES = ("pending", "processing", "completed", "rejected", "retry", "dead-letter")


def find_project_root(requested: object) -> Path | None:
    candidate = Path(str(requested)).expanduser() if requested else Path.cwd()
    try:
        cursor = candidate.resolve(strict=True)
    except (OSError, RuntimeError):
        return None
    if not cursor.is_dir():
        cursor = cursor.parent
    for directory in (cursor, *cursor.parents):
        if any(
            (directory / marker).exists()
            for marker in (".git", ".prometheus/project.json", "Cargo.toml", "package.json")
        ):
            return directory
    return None


def first(payload: dict[str, object], *keys: str) -> str:
    for key in keys:
        value = payload.get(key)
        if value is not None and str(value):
            return str(value)
    return ""


def sync_directory(directory: Path) -> None:
    descriptor = os.open(directory, os.O_RDONLY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def main() -> int:
    event = sys.argv[1] if len(sys.argv) > 1 else "unknown"
    harness = sys.argv[2] if len(sys.argv) > 2 else os.environ.get("PROMETHEUS_HARNESS", "unknown")
    raw = sys.stdin.buffer.read(MAX_INPUT_BYTES + 1)
    if len(raw) > MAX_INPUT_BYTES:
        return 0
    try:
        payload = json.loads(raw or b"{}")
    except (json.JSONDecodeError, UnicodeDecodeError):
        payload = {}
    if not isinstance(payload, dict):
        payload = {}

    project_root = find_project_root(
        first(payload, "cwd", "working_directory", "workingDirectory")
    )
    if project_root is None:
        return 0
    session_id = first(
        payload, "session_id", "sessionId", "conversation_id", "conversationId"
    ) or os.environ.get("CLAUDE_SESSION_ID") or os.environ.get("CODEX_THREAD_ID") or "unknown"
    transcript_path = first(payload, "transcript_path", "transcriptPath")
    payload_digest = hashlib.sha256(raw).hexdigest()
    identity = "\0".join(
        (harness, event, session_id, str(project_root), transcript_path, payload_digest)
    ).encode()
    event_id = hashlib.sha256(identity).hexdigest()

    queue_root = Path(
        os.environ.get(
            "PROMETHEUS_LEARNING_QUEUE",
            str(Path.home() / ".prometheus" / "learning-queue"),
        )
    ).expanduser()
    pending = queue_root / "pending"
    pending.mkdir(mode=0o700, parents=True, exist_ok=True)
    filename = f"{event_id}.json"
    if any((queue_root / state / filename).exists() for state in FINAL_STATES):
        return 0

    job = {
        "schemaVersion": 2,
        "eventId": event_id,
        "eventType": event,
        "harness": harness,
        "sessionId": session_id,
        "projectRoot": str(project_root),
        "transcriptPath": transcript_path or None,
        "capturedAt": datetime.datetime.now(datetime.timezone.utc).isoformat().replace("+00:00", "Z"),
        "payloadDigest": payload_digest,
        "scope": "project",
    }
    target = pending / filename
    temporary = pending / f".{event_id}.{os.getpid()}.tmp"
    descriptor = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        with os.fdopen(descriptor, "wb", closefd=False) as output:
            output.write(json.dumps(job, separators=(",", ":"), sort_keys=True).encode() + b"\n")
            output.flush()
            os.fsync(output.fileno())
    except Exception:
        try:
            temporary.unlink()
        except FileNotFoundError:
            pass
        raise
    finally:
        os.close(descriptor)
    try:
        try:
            os.link(temporary, target)
        except FileExistsError:
            return 0
        sync_directory(pending)
    finally:
        try:
            temporary.unlink()
        except FileNotFoundError:
            pass
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception:
        # Hook paths are fail-open. The absence of a durable file is observable
        # and is never reclassified as a successful delivery receipt.
        raise SystemExit(0)
