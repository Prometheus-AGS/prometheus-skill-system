#!/usr/bin/env python3
"""One attributed write path for lessons: surreal-memory, pk and the learning log.

Design: docs/design/team-aware-learning-memory.md §2 (envelope and scope keys)
and §6 (attributed writes).

Every lesson carries a learning envelope (shared/schemas/learning-envelope.schema.json)
built from the resolved identity of the writing agent. It is written to:

  * surreal-memory, through the durable learning-queue outbox (the supervised
    worker delivers it; a hook never waits on the memory service). Scope keys
    follow the design table: one `agent_id` per visibility level, never null.
    Each `audience` entry produces one additional copy keyed to that recipient.
  * the learning log (`~/.prometheus/learning-log/lessons.jsonl`), always — the
    file-tier backup that either store can be rebuilt from.
  * pk, only when asked (`--pk` or PROMETHEUS_LEARNING_PK=1), as a detached
    `pk ingest --type <Kind> --tag team:/role:/vis:` so the hook never waits on
    the model call.

Idempotence: the operation id is derived from (method, canonical arguments), so
writing the same lesson to the same scope twice queues one operation.

Stored memory content is the lesson text followed by one trailer line,
`<!-- prometheus-envelope {json} -->` (the agent-team runtime writes the same), so recall can show the text and still
read the envelope.

Usage:
  learning_write.py --text "LESSON …" [--kind lesson] [--visibility agent]
                    [--audience role:ui-dev …] [--paths src/api/x.ts …]
                    [--stage execute] [--importance 0.6] [--pk]
                    [--payload FILE | --payload-stdin] [--cwd DIR]
                    [--user-id ID]   # explicit project scope (memory bridge)
Prints one JSON summary line. Exit 0 always unless arguments are invalid.
"""
from __future__ import annotations

import argparse
import datetime
import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path

LIB = Path(__file__).resolve().parent
sys.path.insert(0, str(LIB))
from agent_identity import resolve as resolve_identity  # noqa: E402
from project_id import resolve_user_scope  # noqa: E402
import learning_route  # noqa: E402

SCHEMA_PATH = LIB.parent.parent / "schemas" / "learning-envelope.schema.json"
ENQUEUE = LIB.parent / "enqueue-memory-operation.py"
KINDS = ("lesson", "gotcha", "decision", "progress", "candidate")
# Shared with the agent-team runtime (memory.mts ENVELOPE_TRAILER_PREFIX): one
# format for every writer, so recall parses a single trailer.
TRAILER = "<!-- prometheus-envelope "
MAX_PATH_CATEGORIES = 5


def normalise_text(text: str) -> str:
    """NFC, trimmed, internal whitespace collapsed — identical to memory.mts."""
    import unicodedata
    return " ".join(unicodedata.normalize("NFC", text).split())


def content_hash(text: str) -> str:
    return hashlib.sha256(normalise_text(text).encode("utf-8")).hexdigest()


def scope_keys(visibility: str, identity: dict, user_scope: str, recipient: str | None = None) -> tuple[str, str]:
    """(user_id, agent_id) for a visibility level — design §2 table."""
    project = identity["projectId"]
    team = identity["teamId"]
    if recipient:  # an addressed copy: role:<r> or lead
        if recipient == "lead":
            return project, f"{team}/@lead"
        return project, f"{team}/{recipient.split(':', 1)[1]}"
    if visibility == "agent":
        return project, f"{team}/{identity['roleId']}"
    if visibility.startswith("role:"):
        return project, f"{team}/{visibility.split(':', 1)[1]}"
    if visibility == "lead":
        return project, f"{team}/@lead"
    if visibility == "team":
        return project, f"{team}/@team"
    if visibility == "user":
        return f"@{user_scope}" if not user_scope.startswith("@") else user_scope, "@user"
    if visibility == "global":
        return "@global", "@global"
    return project, "@project"


def build_envelope(text: str, identity: dict, kind: str, visibility: str, audience: list[str],
                   paths: list[str], stage: str | None, importance: float | None) -> dict:
    envelope = {
        "schemaVersion": 1,
        "projectId": identity["projectId"],
        "visibility": visibility,
        "kind": kind,
        "author": {"harness": identity.get("harness") or "other"},
        "contentHash": content_hash(text),
        "ts": datetime.datetime.now(datetime.timezone.utc).isoformat().replace("+00:00", "Z"),
    }
    if identity["roleId"] != "unresolved" and not identity["teamId"].startswith("@"):
        envelope["teamId"] = identity["teamId"]
        envelope["roleId"] = identity["roleId"]
    for key, source in (("agentId", "agentId"), ("agentType", "agentType"), ("sessionId", "sessionId")):
        if identity.get(source):
            envelope["author"][key] = identity[source]
    if envelope["author"]["harness"] not in ("claude-code", "codex", "opencode", "kimi", "other"):
        envelope["author"]["harness"] = "other"
    if audience:
        envelope["audience"] = audience
    if stage:
        envelope["stage"] = stage
    if paths:
        envelope["paths"] = paths
    if importance is not None:
        envelope["importance"] = max(0.0, min(1.0, importance))
    return envelope


def validate(envelope: dict) -> list[str]:
    """Schema errors (jsonschema when importable, structural checks otherwise)."""
    try:
        import jsonschema  # type: ignore
    except ImportError:
        problems = [key for key in ("schemaVersion", "projectId", "visibility", "kind", "author", "contentHash", "ts")
                    if key not in envelope]
        if envelope.get("visibility") == "agent" and "roleId" not in envelope:
            problems.append("roleId required for visibility agent")
        return problems
    schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))
    validator = jsonschema.Draft202012Validator(schema)
    return [error.message for error in validator.iter_errors(envelope)]


def categories(envelope: dict, recipient: str | None) -> list[str]:
    cats = ["env:1", f"vis:{envelope['visibility']}", f"kind:{envelope['kind']}", f"h:{envelope['contentHash'][:16]}"]
    if envelope.get("stage"):
        cats.append(f"stage:{envelope['stage']}")
    if envelope.get("teamId"):
        cats.append(f"author:{envelope['teamId']}/{envelope['roleId']}")
    for path in envelope.get("paths", [])[:MAX_PATH_CATEGORIES]:
        cats.append(f"path:{path}")
    if "importance" in envelope:
        cats.append(f"imp:{min(9, int(envelope['importance'] * 10))}")
    if recipient:
        cats.append(f"aud:{recipient}")
    return cats


def enqueue(arguments: dict) -> str:
    """Queue one add_memory operation; returns the operation id (or '' on failure)."""
    if not ENQUEUE.is_file():
        return ""
    try:
        completed = subprocess.run(
            [sys.executable, str(ENQUEUE), "add_memory", json.dumps(arguments, sort_keys=True), "[]"],
            capture_output=True, text=True, timeout=5, check=False,
        )
    except (OSError, subprocess.TimeoutExpired):
        return ""
    return completed.stdout.strip().splitlines()[-1] if completed.returncode == 0 and completed.stdout.strip() else ""


def _index_path() -> Path:
    return Path(os.environ.get("PROMETHEUS_LEARNING_INDEX_DIR", str(Path.home() / ".prometheus" / "learning-index"))) / "written.tsv"


def already_written(user_id: str, agent_id: str, digest: str) -> bool:
    """True when this lesson (by content hash) was already queued for this scope.

    The envelope carries a timestamp, so content-derived operation ids alone do
    not collapse a repeated lesson; this index does."""
    key = f"{user_id}\t{agent_id}\t{digest}\n"
    try:
        with open(_index_path(), encoding="utf-8") as handle:
            return key in handle
    except OSError:
        return False


def remember_written(user_id: str, agent_id: str, digest: str) -> None:
    try:
        path = _index_path()
        path.parent.mkdir(parents=True, exist_ok=True)
        with open(path, "a", encoding="utf-8") as handle:
            handle.write(f"{user_id}\t{agent_id}\t{digest}\n")
    except OSError:
        pass


def append_learning_log(record: dict) -> None:
    log = Path(os.environ.get("PROMETHEUS_LEARNING_LOG_DIR", str(Path.home() / ".prometheus" / "learning-log")))
    try:
        log.mkdir(parents=True, exist_ok=True)
        with open(log / "lessons.jsonl", "a", encoding="utf-8") as handle:
            handle.write(json.dumps(record, sort_keys=True) + "\n")
    except OSError:
        pass


def ingest_pk(text: str, envelope: dict, cwd: Path) -> bool:
    """Detached `pk ingest`; never waits for the model call."""
    from shutil import which
    pk = which("pk")
    if not pk:
        return False
    kind_type = {"lesson": "Lesson", "gotcha": "Gotcha", "decision": "Decision", "progress": "Progress", "candidate": "Candidate"}
    args = [pk, "ingest", "--type", kind_type[envelope["kind"]], "--tag", f"vis:{envelope['visibility']}",
            "--source", f"learning:{envelope['contentHash'][:16]}"]
    if envelope.get("teamId"):
        args += ["--tag", f"team:{envelope['teamId']}", "--tag", f"role:{envelope['roleId']}"]
    scope = envelope["visibility"]
    if scope in ("user", "global"):
        args += ["--scope", "shared", "--yes"]
    try:
        process = subprocess.Popen(args, cwd=cwd, stdin=subprocess.PIPE, stdout=subprocess.DEVNULL,
                                   stderr=subprocess.DEVNULL, start_new_session=True)
        assert process.stdin is not None
        process.stdin.write(text.encode("utf-8"))
        process.stdin.close()
        return True
    except OSError:
        return False


def route_audience(payload: dict, cwd: Path, identity: dict, visibility: str, paths: list[str]) -> list[str]:
    """Path-overlap routing (design §7.1): only role-private lessons that carry
    paths, and only inside an active team. PROMETHEUS_LEARNING_ROUTE=0 disables."""
    if visibility != "agent" or not paths or os.environ.get("PROMETHEUS_LEARNING_ROUTE") == "0":
        return []
    _, team = learning_route.resolve_team(payload, cwd)
    return learning_route.route(team, paths, identity["roleId"])


def merge_audience(explicit: list[str], routed: list[str]) -> list[str]:
    merged: list[str] = []
    for entry in list(explicit) + list(routed):
        if entry not in merged:
            merged.append(entry)
    return merged


def write_digest(envelope: dict, identity: dict, user_scope: str) -> dict:
    """Team digest (design §7.2): one line in the digest file and one mirror
    memory under `<team>/@team`, for role-private lessons of a team role only.
    The digest never carries the lesson text."""
    if envelope["visibility"] != "agent" or not envelope.get("teamId") or os.environ.get("PROMETHEUS_TEAM_DIGEST") == "0":
        return {"written": False}
    role, digest = envelope["roleId"], envelope["contentHash"]
    paths = envelope.get("paths", [])
    line = learning_route.digest_line(role, paths, digest)
    file_ok = learning_route.append_digest(learning_route.digest_path(identity["projectId"], envelope["teamId"]), line)
    text = learning_route.digest_text(line)
    mirror = build_envelope(text, identity, "progress", "team", [], paths, envelope.get("stage"), None)
    problems = validate(mirror)
    operation_id = ""
    if not problems:
        uid, aid = scope_keys("team", identity, user_scope)
        if already_written(uid, aid, mirror["contentHash"]):
            operation_id = "duplicate"
        else:
            categories_ = [c for c in categories(mirror, None) if not c.startswith("h:")]
            categories_.append(f"h:{digest[:16]}")  # same key as the lesson: recall delivers it once
            arguments = {
                "content": f"{text}\n\n{TRAILER}{json.dumps(mirror, sort_keys=True, separators=(',', ':'))} -->",
                "user_id": uid, "agent_id": aid, "categories": categories_,
            }
            if identity.get("sessionId"):
                arguments["session_id"] = identity["sessionId"]
            operation_id = enqueue(arguments)
            if operation_id:
                remember_written(uid, aid, mirror["contentHash"])
    return {"written": file_ok, "line": line, "mirror": operation_id, "mirrorErrors": problems}


def write_lesson(text: str, payload: dict | None = None, *, cwd: Path | None = None, kind: str = "lesson",
                 visibility: str | None = None, audience: list[str] | None = None, paths: list[str] | None = None,
                 stage: str | None = None, importance: float | None = None, pk: bool = False,
                 user_id: str | None = None) -> dict:
    cwd = cwd or Path.cwd()
    payload = payload or {}
    text = text.strip()
    if not text:
        return {"written": 0, "reason": "empty"}
    identity = resolve_identity(payload, cwd, paths or [])
    if user_id:
        identity["projectId"] = user_id
    # Role-private needs a resolved role; otherwise fall back to project scope.
    if visibility is None:
        visibility = "agent" if identity["roleId"] != "unresolved" else "project"
    if visibility == "agent" and identity["roleId"] == "unresolved":
        visibility = "project"
    audience = merge_audience(audience or [], route_audience(payload, cwd, identity, visibility, paths or []))
    envelope = build_envelope(text, identity, kind, visibility, audience, paths or [], stage, importance)
    problems = validate(envelope)
    if problems:
        return {"written": 0, "reason": "invalid envelope", "errors": problems}
    user_scope = resolve_user_scope(cwd)
    stored = f"{text}\n\n{TRAILER}{json.dumps(envelope, sort_keys=True, separators=(',', ':'))} -->"
    copies = [None] + list(audience)
    operations = []
    for recipient in copies:
        uid, aid = scope_keys(visibility, identity, user_scope, recipient)
        if already_written(uid, aid, envelope["contentHash"]):
            operations.append({"user_id": uid, "agent_id": aid, "operation": "", "duplicate": True})
            continue
        arguments = {
            "content": stored,
            "user_id": uid,
            "agent_id": aid,
            "categories": categories(envelope, recipient),
        }
        if identity.get("sessionId"):
            arguments["session_id"] = identity["sessionId"]
        operation_id = enqueue(arguments)
        if operation_id:
            remember_written(uid, aid, envelope["contentHash"])
        operations.append({"user_id": uid, "agent_id": aid, "operation": operation_id})
    written = [o for o in operations if o.get("operation")]
    if not written:
        return {"written": 0, "duplicate": any(o.get("duplicate") for o in operations), "envelope": envelope, "operations": operations}
    append_learning_log({"envelope": envelope, "text": text, "scopes": [(o["user_id"], o["agent_id"]) for o in operations]})
    pk_started = ingest_pk(text, envelope, cwd) if (pk or os.environ.get("PROMETHEUS_LEARNING_PK") == "1") else False
    digest = write_digest(envelope, identity, user_scope)
    return {"written": len(written), "envelope": envelope, "operations": operations, "pk": pk_started, "digest": digest}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--text", required=True)
    parser.add_argument("--kind", default="lesson", choices=KINDS)
    parser.add_argument("--visibility", default=None)
    parser.add_argument("--audience", nargs="*", default=[])
    parser.add_argument("--paths", nargs="*", default=[])
    parser.add_argument("--stage", default=None)
    parser.add_argument("--importance", type=float, default=None)
    parser.add_argument("--pk", action="store_true")
    parser.add_argument("--payload", default=None)
    parser.add_argument("--payload-stdin", action="store_true")
    parser.add_argument("--cwd", default=os.getcwd())
    parser.add_argument("--user-id", default=None)
    options = parser.parse_args()
    payload: dict = {}
    try:
        if options.payload:
            payload = json.loads(Path(options.payload).read_text(encoding="utf-8"))
        elif options.payload_stdin:
            payload = json.loads(sys.stdin.read() or "{}")
    except (OSError, ValueError):
        payload = {}
    if not isinstance(payload, dict):
        payload = {}
    cwd = Path(options.cwd)
    if not cwd.is_dir():
        cwd = Path.cwd()
    result = write_lesson(options.text, payload, cwd=cwd, kind=options.kind, visibility=options.visibility,
                          audience=options.audience, paths=options.paths, stage=options.stage,
                          importance=options.importance, pk=options.pk, user_id=options.user_id)
    print(json.dumps({key: value for key, value in result.items() if key != "envelope"} | {"contentHash": result.get("envelope", {}).get("contentHash")}, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
