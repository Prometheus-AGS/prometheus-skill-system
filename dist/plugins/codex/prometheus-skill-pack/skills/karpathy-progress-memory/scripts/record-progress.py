#!/usr/bin/env python3
"""Record one canonical KBD boundary in project history and durable memory."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import os
from pathlib import Path, PureWindowsPath
import re
import subprocess
import sys
import tempfile
import time
from typing import Any


EVENT_ID = re.compile(r"^[A-Za-z0-9._:-]{8,128}$")
COMMIT = re.compile(r"^[0-9a-fA-F]{7,64}$")
SECRET = re.compile(
    r"(?i)(?:api[_-]?key|access[_-]?token|password|secret)\s*[:=]\s*\S+"
    r"|bearer\s+[A-Za-z0-9._~+/=-]{12,}|-----BEGIN [A-Z ]*PRIVATE KEY-----"
    r"|gh[pousr]_[A-Za-z0-9]{20,}|xox[baprs]-[A-Za-z0-9-]{10,}"
    r"|AKIA[0-9A-Z]{16}|sk-(?:proj-)?[A-Za-z0-9_-]{16,}"
)
BOUNDARIES = {"task", "change", "phase"}
STATUSES = {"in_progress", "complete", "blocked", "cancelled"}
TASK_CLASSES = {"product", "research", "evidence", "integration", "release"}


class ProgressError(Exception):
    pass


def run(
    command: list[str],
    cwd: Path,
    *,
    input_text: str | None = None,
    timeout: float | None = None,
) -> subprocess.CompletedProcess[str]:
    try:
        return subprocess.run(
            command,
            cwd=cwd,
            input=input_text,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
            timeout=timeout,
        )
    except subprocess.TimeoutExpired:
        return subprocess.CompletedProcess(command, 124, "", "command timed out")


def receipt_path(root: Path, event_id: str) -> Path:
    digest = hashlib.sha256(event_id.encode()).hexdigest()
    return root / ".prometheus" / "progress-memory-receipts" / f"{digest}.json"


def project_root(start: Path) -> Path:
    cursor = start.resolve()
    for candidate in (cursor, *cursor.parents):
        if (candidate / ".prometheus" / "project.json").is_file() and (
            candidate / ".kbd-orchestrator"
        ).is_dir():
            return candidate
    raise ProgressError("project root with .prometheus/project.json and .kbd-orchestrator was not found")


def kbd_status(root: Path) -> dict[str, Any]:
    prometheus = os.environ.get("PROMETHEUS_BIN", "prometheus")
    result = run([prometheus, "kbd", "--path", str(root), "status", "--json"], root)
    if result.returncode != 0:
        raise ProgressError(f"prometheus kbd status failed: {result.stderr.strip()}")
    try:
        value = json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise ProgressError(f"prometheus kbd status returned invalid JSON: {error}") from error
    if not isinstance(value, dict):
        raise ProgressError("prometheus kbd status did not return an object")
    return value


def git_value(root: Path, *args: str) -> str | None:
    result = run(["git", *args], root)
    return result.stdout.strip() if result.returncode == 0 and result.stdout.strip() else None


def touched_files(root: Path) -> list[str]:
    paths: set[str] = set()
    for command in (
        ["git", "diff", "--name-only", "-z", "HEAD"],
        ["git", "ls-files", "--others", "--exclude-standard", "-z"],
    ):
        result = subprocess.run(command, cwd=root, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
        if result.returncode == 0:
            paths.update(
                item.decode("utf-8", "surrogateescape")
                for item in result.stdout.split(b"\0")
                if item
            )
    recorder_owned = (
        ".prometheus/progress-memory-receipts/",
        ".prometheus/memory-outbox/",
    )
    return sorted(
        path
        for path in paths
        if path != ".prometheus/session-log.md"
        and not path.startswith(recorder_owned)
    )[:500]


def parse_hook_subject(boundary: str, name: str, state: dict[str, Any]) -> tuple[str | None, str | None]:
    active = state.get("activePath") or {}
    change_id = active.get("changeId")
    task_id = active.get("taskId")
    if "/" in name:
        named_change, named_task = name.split("/", 1)
    elif ":" in name:
        named_change, named_task = name.split(":", 1)
    else:
        named_change, named_task = name, ""
    if boundary == "task":
        return named_change or change_id, named_task or task_id
    if boundary == "change":
        return named_change or change_id, None
    return None, None


def event_from_hook(root: Path, boundary: str) -> tuple[dict[str, Any], dict[str, Any]]:
    if boundary not in BOUNDARIES:
        raise ProgressError(f"unsupported hook boundary: {boundary}")
    state = kbd_status(root)
    active = state.get("activePath") or {}
    phase_id = active.get("phaseId")
    if not isinstance(phase_id, str) or not phase_id:
        raise ProgressError("canonical status has no active phase")
    name = os.environ.get("KBD_HOOK_NAME", "")
    change_id, task_id = parse_hook_subject(boundary, name, state)
    run_id = state.get("runId")
    if not isinstance(run_id, str) or not run_id:
        raise ProgressError("canonical status has no runId")
    identity = "\0".join(
        str(value or "")
        for value in (
            state.get("projectId"),
            run_id,
            boundary,
            phase_id,
            change_id,
            task_id,
            "complete",
        )
    )
    event_id = "kpm-" + hashlib.sha256(identity.encode()).hexdigest()[:32]
    now = dt.datetime.now(dt.timezone.utc).isoformat().replace("+00:00", "Z")
    task_class = os.environ.get("KBD_TASK_CLASS", "product").lower()
    try:
        elapsed = float(os.environ.get("KBD_TASK_ELAPSED_HOURS", "0"))
    except ValueError as error:
        raise ProgressError("KBD_TASK_ELAPSED_HOURS must be numeric") from error
    event = {
        "schemaVersion": 1,
        "eventId": event_id,
        "observedAt": now,
        "runId": run_id,
        "boundary": boundary,
        "status": "complete",
        "phaseId": phase_id,
        "changeId": change_id,
        "taskId": task_id,
        "taskClass": task_class,
        "elapsedHours": elapsed,
        "touchedFiles": touched_files(root),
        "verification": [],
        "commitSha": git_value(root, "rev-parse", "HEAD"),
        "blocker": None,
        "exactNextWork": state.get("exactNextWork"),
    }
    return event, state


def load_event(path: str) -> dict[str, Any]:
    text = sys.stdin.read() if path == "-" else Path(path).read_text(encoding="utf-8")
    try:
        value = json.loads(text)
    except json.JSONDecodeError as error:
        raise ProgressError(f"progress event is invalid JSON: {error}") from error
    if not isinstance(value, dict):
        raise ProgressError("progress event must be a JSON object")
    return value


def require_string(event: dict[str, Any], key: str, *, nullable: bool = False, limit: int = 4000) -> None:
    value = event.get(key)
    if nullable and value is None:
        return
    if not isinstance(value, str) or not value or len(value) > limit:
        raise ProgressError(f"{key} must be a non-empty string no longer than {limit} characters")


def validate_event(event: dict[str, Any]) -> None:
    allowed = {
        "schemaVersion", "eventId", "observedAt", "runId", "boundary", "status", "phaseId",
        "changeId", "taskId", "taskClass", "elapsedHours", "touchedFiles",
        "verification", "commitSha", "blocker", "exactNextWork",
    }
    unknown = sorted(set(event) - allowed)
    if unknown:
        raise ProgressError(f"unknown progress event fields: {', '.join(unknown)}")
    if event.get("schemaVersion") != 1:
        raise ProgressError("schemaVersion must be 1")
    require_string(event, "eventId", limit=128)
    if not EVENT_ID.fullmatch(event["eventId"]):
        raise ProgressError("eventId has an invalid format")
    require_string(event, "observedAt", limit=64)
    try:
        dt.datetime.fromisoformat(event["observedAt"].replace("Z", "+00:00"))
    except ValueError as error:
        raise ProgressError("observedAt must be an ISO-8601 timestamp") from error
    require_string(event, "runId", limit=200)
    if event.get("boundary") not in BOUNDARIES:
        raise ProgressError("boundary must be task, change, or phase")
    if event.get("status") not in STATUSES:
        raise ProgressError("status is invalid")
    require_string(event, "phaseId", limit=160)
    require_string(event, "changeId", nullable=True, limit=200)
    require_string(event, "taskId", nullable=True, limit=200)
    if event["boundary"] in {"task", "change"} and not event.get("changeId"):
        raise ProgressError(f"{event['boundary']} boundary requires changeId")
    if event["boundary"] == "task" and not event.get("taskId"):
        raise ProgressError("task boundary requires taskId")
    if event.get("taskClass") not in TASK_CLASSES:
        raise ProgressError("taskClass is invalid")
    elapsed = event.get("elapsedHours")
    if not isinstance(elapsed, (int, float)) or isinstance(elapsed, bool) or not 0 <= elapsed <= 1000:
        raise ProgressError("elapsedHours must be between 0 and 1000")
    files = event.get("touchedFiles")
    if not isinstance(files, list) or len(files) > 500 or len(set(files)) != len(files):
        raise ProgressError("touchedFiles must be a unique array of at most 500 paths")
    for path in files:
        if (
            not isinstance(path, str)
            or not path
            or len(path) > 1000
            or Path(path).is_absolute()
            or PureWindowsPath(path).is_absolute()
            or ".." in Path(path).parts
            or ".." in PureWindowsPath(path).parts
        ):
            raise ProgressError(f"unsafe touchedFiles entry: {path!r}")
    verification = event.get("verification")
    if not isinstance(verification, list) or len(verification) > 100:
        raise ProgressError("verification must be an array of at most 100 records")
    for record in verification:
        if not isinstance(record, dict) or set(record) != {"command", "exitCode", "summary"}:
            raise ProgressError("each verification record requires command, exitCode, and summary")
        if not isinstance(record["command"], str) or not record["command"] or len(record["command"]) > 2000:
            raise ProgressError("verification command is invalid")
        if not isinstance(record["exitCode"], int) or not 0 <= record["exitCode"] <= 255:
            raise ProgressError("verification exitCode is invalid")
        if not isinstance(record["summary"], str) or len(record["summary"]) > 4000:
            raise ProgressError("verification summary is invalid")
    commit = event.get("commitSha")
    if commit is not None and (not isinstance(commit, str) or not COMMIT.fullmatch(commit)):
        raise ProgressError("commitSha must be null or a 7-64 character hexadecimal Git object ID")
    for key in ("blocker", "exactNextWork"):
        value = event.get(key)
        if value is not None and (not isinstance(value, str) or len(value) > 4000):
            raise ProgressError(f"{key} must be null or a string no longer than 4000 characters")
    serialized = json.dumps(event, ensure_ascii=False, sort_keys=True)
    if len(serialized.encode()) > 256_000:
        raise ProgressError("progress event exceeds 256 KiB")
    if SECRET.search(serialized):
        raise ProgressError("progress event appears to contain a secret")


def canonical_validate(event: dict[str, Any], state: dict[str, Any]) -> None:
    if event["runId"] != state.get("runId"):
        raise ProgressError(
            f"canonical run disagreement: event={event['runId']} state={state.get('runId')}"
        )
    phase_id = event["phaseId"]
    phase = (state.get("phases") or {}).get(phase_id)
    if not isinstance(phase, dict):
        raise ProgressError(f"canonical phase does not exist: {phase_id}")
    boundary = event["boundary"]
    expected_status = event["status"]
    if boundary == "phase":
        actual = phase.get("status")
    else:
        change_id = event.get("changeId")
        change = (phase.get("changes") or {}).get(change_id)
        if not isinstance(change, dict):
            raise ProgressError(f"canonical change does not exist in {phase_id}: {change_id}")
        if boundary == "change":
            actual = change.get("implementationStatus") or change.get("status")
        else:
            task_id = event.get("taskId")
            task = (change.get("tasks") or {}).get(task_id)
            if not isinstance(task, dict):
                raise ProgressError(f"canonical task does not exist in {change_id}: {task_id}")
            actual = task.get("status")
    if actual != expected_status:
        raise ProgressError(
            f"canonical {boundary} status is {actual!r}, event reports {expected_status!r}"
        )


def markdown_record(event: dict[str, Any]) -> str:
    files = ", ".join(f"`{path}`" for path in event["touchedFiles"]) or "none"
    verification = event["verification"]
    checks = (
        "\n".join(
            f"  - `{item['command']}` → exit {item['exitCode']}: {item['summary']}"
            for item in verification
        )
        if verification
        else "  - none recorded"
    )
    return (
        f"\n<!-- karpathy-progress-event:{event['eventId']} -->\n"
        f"## Progress boundary — {event['observedAt']}\n\n"
        f"- Event: `{event['eventId']}`\n"
        f"- Boundary: `{event['boundary']}` / `{event['status']}`\n"
        f"- Position: `{event['phaseId']}` / `{event.get('changeId') or '-'}` / `{event.get('taskId') or '-'}`\n"
        f"- Class and elapsed time: `{event['taskClass']}` / `{event['elapsedHours']}` hours\n"
        f"- Commit: `{event.get('commitSha') or 'uncommitted'}`\n"
        f"- Files: {files}\n"
        f"- Blocker: {event.get('blocker') or 'none'}\n"
        f"- Exact next work: {event.get('exactNextWork') or 'none recorded'}\n"
        f"- Verification:\n{checks}\n"
    )


def append_session_log(root: Path, event: dict[str, Any], record: str) -> bool:
    log = root / ".prometheus" / "session-log.md"
    log.parent.mkdir(parents=True, exist_ok=True)
    marker = f"<!-- karpathy-progress-event:{event['eventId']} -->"
    lock = log.with_name(f"{log.name}.karpathy-progress.lock")
    descriptor = acquire_lock(lock)
    try:
        with log.open("a+", encoding="utf-8") as output:
            output.seek(0)
            if marker in output.read():
                return False
            output.seek(0, os.SEEK_END)
            output.write(record)
            output.flush()
            os.fsync(output.fileno())
        return True
    finally:
        release_lock(lock, descriptor)


def acquire_lock(lock: Path) -> int:
    lock.parent.mkdir(parents=True, exist_ok=True)
    descriptor: int | None = None
    for _ in range(100):
        try:
            descriptor = os.open(lock, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
            os.write(descriptor, f"{os.getpid()}\n".encode())
            os.fsync(descriptor)
            break
        except FileExistsError:
            try:
                owner_text = lock.read_text(encoding="utf-8").strip()
                owner_alive = True
                try:
                    os.kill(int(owner_text), 0)
                except (ProcessLookupError, ValueError):
                    owner_alive = False
                except PermissionError:
                    owner_alive = True
                if not owner_alive or time.time() - lock.stat().st_mtime > 60:
                    lock.unlink()
                    continue
            except FileNotFoundError:
                continue
            time.sleep(0.05)
    if descriptor is None:
        raise ProgressError(f"timed out acquiring progress lock: {lock}")
    return descriptor


def release_lock(lock: Path, descriptor: int) -> None:
    os.close(descriptor)
    try:
        lock.unlink()
    except FileNotFoundError:
        pass


def memory_write(
    root: Path, event: dict[str, Any], record: str, project_id: str
) -> dict[str, Any]:
    pk = os.environ.get("PK_BIN", "pk")
    try:
        pk_timeout = float(os.environ.get("KPM_PK_TIMEOUT_SECONDS", "5"))
    except ValueError as error:
        raise ProgressError("KPM_PK_TIMEOUT_SECONDS must be numeric") from error
    if not 0.1 <= pk_timeout <= 10:
        raise ProgressError("KPM_PK_TIMEOUT_SECONDS must be between 0.1 and 10")
    try:
        result = run(
            [pk, "ingest", "--scope", "project", "--source", f"karpathy-progress-memory:{event['eventId']}"],
            root,
            input_text=record,
            timeout=pk_timeout,
        )
    except OSError as error:
        result = subprocess.CompletedProcess([pk, "ingest"], 127, "", str(error))
    if result.returncode == 0:
        receipt = result.stdout.strip()
        return {
            "transport": "pk",
            "status": "accepted",
            "receiptSha256": hashlib.sha256(receipt.encode()).hexdigest() if receipt else None,
        }

    pk_error = "pk executable unavailable" if result.returncode == 127 else f"pk exited {result.returncode}"

    enqueue = os.environ.get("KPM_MEMORY_ENQUEUE")
    if enqueue is None:
        enqueue = str(Path(__file__).resolve().parents[4] / "shared" / "scripts" / "enqueue-memory-operation.py")
    enqueue_path = Path(enqueue)
    if enqueue_path.is_file():
        arguments = json.dumps(
            {"content": record, "user_id": project_id},
            ensure_ascii=False,
            separators=(",", ":"),
            sort_keys=True,
        )
        fallback = run(
            [sys.executable, str(enqueue_path), "add_memory", arguments, "[]"],
            root,
            timeout=5,
        )
        if fallback.returncode == 0:
            operation_id = fallback.stdout.strip()
            return {
                "transport": "outbox",
                "status": "queued",
                "operationId": operation_id if re.fullmatch(r"[0-9a-f]{64}", operation_id) else None,
                "pkError": pk_error,
            }
        fallback_error = f"outbox enqueue exited {fallback.returncode}"
    else:
        fallback_error = f"enqueue script unavailable: {enqueue_path}"
    return {
        "transport": "none",
        "status": "degraded",
        "pkError": pk_error,
        "fallbackError": fallback_error,
    }


def event_sha256(event: dict[str, Any]) -> str:
    semantic_event = dict(event)
    # A hook replay observes a later wall-clock instant. The boundary identity
    # and all substantive fields must still match, while observedAt is merely
    # receipt metadata and cannot make a stable event look like a collision.
    semantic_event.pop("observedAt", None)
    return hashlib.sha256(
        json.dumps(semantic_event, ensure_ascii=False, separators=(",", ":"), sort_keys=True).encode()
    ).hexdigest()


def event_identity_sha256(event: dict[str, Any]) -> str:
    identity = {
        key: event.get(key)
        for key in (
            "schemaVersion",
            "eventId",
            "runId",
            "boundary",
            "status",
            "phaseId",
            "changeId",
            "taskId",
        )
    }
    return hashlib.sha256(
        json.dumps(identity, ensure_ascii=False, separators=(",", ":"), sort_keys=True).encode()
    ).hexdigest()


def write_receipt(
    root: Path,
    event: dict[str, Any],
    appended: bool,
    memory: dict[str, Any],
    *,
    complete: bool,
) -> Path:
    directory = root / ".prometheus" / "progress-memory-receipts"
    directory.mkdir(parents=True, exist_ok=True)
    target = receipt_path(root, event["eventId"])
    payload = {
        "schemaVersion": 1,
        "eventId": event["eventId"],
        "eventSha256": event_sha256(event),
        "eventIdentitySha256": event_identity_sha256(event),
        "event": event,
        "sessionLogAppended": appended,
        "memory": memory,
        "complete": complete,
        "recordedAt": dt.datetime.now(dt.timezone.utc).isoformat().replace("+00:00", "Z"),
    }
    with tempfile.NamedTemporaryFile("w", encoding="utf-8", dir=directory, delete=False) as output:
        json.dump(payload, output, indent=2)
        output.write("\n")
        output.flush()
        os.fsync(output.fileno())
        temporary = Path(output.name)
    os.replace(temporary, target)
    return target


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--project-root", default=".")
    parser.add_argument("--input", default="-", help="JSON event file, or - for stdin")
    parser.add_argument("--from-hook", action="store_true")
    parser.add_argument("--boundary", choices=sorted(BOUNDARIES))
    args = parser.parse_args()
    try:
        root = project_root(Path(args.project_root))
        if args.from_hook:
            if not args.boundary:
                raise ProgressError("--from-hook requires --boundary")
            event, state = event_from_hook(root, args.boundary)
        else:
            event = load_event(args.input)
            state = kbd_status(root)
        validate_event(event)
        canonical_validate(event, state)
        project_id = state.get("projectId")
        if not isinstance(project_id, str) or not project_id:
            raise ProgressError("canonical status has no projectId for project-scoped memory")
        existing_receipt = receipt_path(root, event["eventId"])
        event_lock = existing_receipt.with_suffix(".lock")
        lock_descriptor = acquire_lock(event_lock)
        try:
            if existing_receipt.is_file():
                try:
                    prior = json.loads(existing_receipt.read_text(encoding="utf-8"))
                except (OSError, json.JSONDecodeError) as error:
                    raise ProgressError(f"existing progress receipt is unreadable: {error}") from error
                expected_hash = (
                    event_identity_sha256(event) if args.from_hook else event_sha256(event)
                )
                receipt_hash = (
                    prior.get("eventIdentitySha256") if args.from_hook else prior.get("eventSha256")
                )
                if receipt_hash != expected_hash:
                    raise ProgressError(
                        f"eventId {event['eventId']} already belongs to a different payload"
                    )
                prior_event = prior.get("event")
                if not isinstance(prior_event, dict):
                    raise ProgressError(
                        f"existing progress receipt for {event['eventId']} has no durable event snapshot"
                    )
                if not prior.get("complete"):
                    record = markdown_record(prior_event)
                    memory = memory_write(root, prior_event, record, project_id)
                    receipt = write_receipt(
                        root,
                        prior_event,
                        bool(prior.get("sessionLogAppended")),
                        memory,
                        complete=True,
                    )
                    status = "recorded" if memory["status"] == "accepted" else memory["status"]
                    print(
                        json.dumps(
                            {
                                "status": status,
                                "eventId": prior_event["eventId"],
                                "recoveredPendingDelivery": True,
                                "memory": memory,
                                "receipt": str(receipt),
                            }
                        )
                    )
                    return 0
                print(
                    json.dumps(
                        {
                            "status": "duplicate",
                            "eventId": event["eventId"],
                            "deliveryComplete": bool(prior.get("complete")),
                            "memory": prior.get("memory"),
                            "receipt": str(existing_receipt),
                        }
                    )
                )
                return 0
            record = markdown_record(event)
            appended = append_session_log(root, event, record)
            write_receipt(
                root,
                event,
                appended,
                {"transport": "pending", "status": "pending"},
                complete=False,
            )
            if os.environ.get("KPM_TEST_CRASH_BEFORE_MEMORY") == "1":
                os._exit(74)
            memory = memory_write(root, event, record, project_id)
            receipt = write_receipt(root, event, appended, memory, complete=True)
            if os.environ.get("KPM_TEST_CRASH_AFTER_PK") == "1":
                os._exit(75)
            status = "recorded" if memory["status"] == "accepted" else memory["status"]
            print(
                json.dumps(
                    {
                        "status": status,
                        "eventId": event["eventId"],
                        "sessionLogAppended": appended,
                        "memory": memory,
                        "receipt": str(receipt),
                    },
                    ensure_ascii=False,
                )
            )
            return 0
        finally:
            release_lock(event_lock, lock_descriptor)
    except (OSError, ProgressError) as error:
        print(f"karpathy-progress-memory: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
