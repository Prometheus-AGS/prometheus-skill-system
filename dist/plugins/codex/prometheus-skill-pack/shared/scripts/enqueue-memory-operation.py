#!/usr/bin/env python3
"""Durably enqueue one idempotent Surreal Memory operation."""

from __future__ import annotations

import datetime
import hashlib
import json
import os
from pathlib import Path
import sys

STATES = ("pending", "submitting", "accepted", "completed", "rejected", "retry", "dead-letter")


def sync_directory(directory: Path) -> None:
    descriptor = os.open(directory, os.O_RDONLY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def main() -> int:
    if len(sys.argv) != 4:
        return 1
    method = sys.argv[1]
    arguments = json.loads(sys.argv[2])
    dependencies = json.loads(sys.argv[3])
    if not isinstance(arguments, dict) or not isinstance(dependencies, list) or not all(
        isinstance(item, str) for item in dependencies
    ):
        return 1
    canonical_arguments = json.dumps(
        arguments, ensure_ascii=False, separators=(",", ":"), sort_keys=True
    )
    operation_id = hashlib.sha256(
        method.encode() + b"\0" + canonical_arguments.encode()
    ).hexdigest()
    payload_hash = hashlib.sha256(canonical_arguments.encode()).hexdigest()
    root = Path(
        os.environ.get(
            "PROMETHEUS_LEARNING_QUEUE",
            str(Path.home() / ".prometheus" / "learning-queue"),
        )
    ).expanduser()
    pending = root / "memory" / "pending"
    pending.mkdir(mode=0o700, parents=True, exist_ok=True)
    filename = f"{operation_id}.json"
    if any((root / "memory" / state / filename).exists() for state in STATES):
        print(operation_id)
        return 0

    operation = {
        "schemaVersion": 2,
        "operationId": operation_id,
        "method": method,
        "arguments": arguments,
        "dependencies": dependencies,
        "payloadHash": payload_hash,
        "state": "pending",
        "queuedAt": datetime.datetime.now(datetime.timezone.utc).isoformat().replace("+00:00", "Z"),
        "lastError": None,
        "receipt": None,
    }
    target = pending / filename
    temporary = pending / f".{operation_id}.{os.getpid()}.tmp"
    descriptor = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        content = json.dumps(operation, separators=(",", ":"), sort_keys=True).encode() + b"\n"
        with os.fdopen(descriptor, "wb", closefd=False) as output:
            output.write(content)
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
            print(operation_id)
            return 0
        sync_directory(pending)
    finally:
        try:
            temporary.unlink()
        except FileNotFoundError:
            pass
    print(operation_id)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
