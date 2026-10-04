#!/usr/bin/env python3
"""Render `pk context --format json` as hook text and detect a knowledge gap.

Design: docs/design/team-aware-learning-memory.md §4.

Called by karpathy-hook-dispatch.sh on every prompt event, with:
  stdin                    the JSON produced by `pk context --format json`
  PROMPT_GAP_PROMPT        the user prompt
  PROMPT_GAP_PAYLOAD       the hook payload JSON (session_id, cwd, agent_type, ...)
  PROMPT_GAP_MAX_BYTES     the hook byte budget (same value given to pk)

stdout is, in order:
  1. the hook text, byte-identical to `pk context --format hook` for the same
     query (same rendering and the same UTF-8-safe truncation as pk);
  2. when a gap is detected and not yet reported this session, exactly one line
       [prometheus-gap] <topic> — consider /learn-goal <topic> or /feynman-loop

A gap requires ALL of:
  a. no results, or the top score is below PROMETHEUS_GAP_MIN_SCORE
     (default 4.0: pk scores a term 4 per title hit, 2 per description hit and
     1 per content hit, so the default means "at least one title hit or four
     content hits" - a single stray content mention is not coverage);
  b. the prompt is problem-shaped: it contains '?', starts with an interrogative
     or modal-question word, or carries an error marker (error, exception,
     failed, panic, traceback, cannot, unable, "doesn't work", Exxxx, ...), and
     has at least 4 words;
  c. fewer than 2 distinct prompt keywords (lowercased, >= 4 chars, stop words
     removed) occur in the project's CLAUDE.md / AGENTS.md.

State (once per session per topic) lives in ~/.prometheus/knowledge-gaps/:
  seen/<session>.txt   one topic key per line
  gaps.jsonl           append-only records, see below

gaps.jsonl records (one JSON object per line, latest record per topicKey wins):
  open:     {ts, status:"open", sessionId, topic, topicKey, promptHash, projectId,
             teamId?, roleId?}            teamId/roleId only when the hook runs in
                                          a subagent that resolves to a team role
  resolved: {ts, status:"resolved", topicKey, topic, resolvedBy, entryId?, scope?}
            appended by the learn skills after ingesting the final explanation.

Resolving a gap (called by the learn skills after ingesting the explanation):
  prompt_gap.py resolve --topic "<subject>" [--entry-id ID] [--scope project|shared]
                        [--by learn-goal|feynman-loop|learn-kb]
Matches every still-open topic whose key equals the key of <subject>'s keywords
or that shares >= 2 keywords with <subject>, and appends one resolved record per
match. Prints the resolved topicKeys (one per line); no match prints nothing.
Exit 0 always, so a closing step never fails a learning flow.

Never fails a hook: any error yields the plain hook text (or nothing), exit 0.
"""
from __future__ import annotations

import hashlib
import json
import os
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

DEFAULT_MIN_SCORE = 4.0
MIN_PROMPT_WORDS = 4
MAX_TOPIC_KEYWORDS = 6
MAX_DOC_BYTES = 262_144

STOP_WORDS = frozenset(
    "about above after again also another because been before being between both "
    "could does doing done down during each either else even every from have having "
    "here how into just like make many more most much must need only other over "
    "same should some such than that their them then there these they this those "
    "through under until upon very want well were what when where which while will "
    "with within without would your yours please using used use get got why can "
    "cant dont doesnt isnt not".split()
)
QUESTION_STARTS = frozenset(
    "how why what where when which who can could should would does do did is are "
    "am was were will shall has have".split()
)
ERROR_MARKER = re.compile(
    r"\b(error|errors|exception|failed|failing|failure|panic|panicked|traceback|"
    r"segfault|cannot|can't|unable|fatal|crash|crashes|crashed|broken|"
    r"doesn't work|does not work|not working|stack trace)\b|\b[eE]\d{4}\b|"
    r"\bHTTP\s?[45]\d\d\b",
    re.IGNORECASE,
)
WORD = re.compile(r"[A-Za-z0-9_][A-Za-z0-9_\-]*")


def truncate_utf8(value: str, max_bytes: int) -> str:
    data = value.encode("utf-8")
    if len(data) <= max_bytes:
        return value
    return data[:max_bytes].decode("utf-8", errors="ignore")


def render_hook(report: dict, max_bytes: int) -> str:
    """Mirror of pk-cli rendered_context + print_context_hook."""
    results = report.get("results") or []
    if not results:
        return ""
    out = "--- prometheus-knowledge context ---\n"
    for item in results:
        out += f"[{item['scope']}:{item['id']}] {item['title']}\n{item['snippet']}\n"
    failures = report.get("failures") or []
    if failures:
        out += f"[context-status] failed_scopes={len(failures)} candidates={report.get('candidate_count', 0)}\n"
    out += "--- end pk context ---\n"
    return truncate_utf8(out, max_bytes)


def min_score() -> float:
    try:
        return float(os.environ.get("PROMETHEUS_GAP_MIN_SCORE", DEFAULT_MIN_SCORE))
    except ValueError:
        return DEFAULT_MIN_SCORE


def top_score(report: dict) -> float:
    scores = [float(item.get("score", 0.0)) for item in report.get("results") or []]
    return max(scores) if scores else 0.0


def is_problem_shaped(prompt: str) -> bool:
    words = WORD.findall(prompt)
    if len(words) < MIN_PROMPT_WORDS:
        return False
    if "?" in prompt:
        return True
    if words[0].lower() in QUESTION_STARTS:
        return True
    return ERROR_MARKER.search(prompt) is not None


def keywords(prompt: str) -> list[str]:
    seen: list[str] = []
    for word in WORD.findall(prompt.lower()):
        word = word.strip("-_")
        if len(word) < 4 or word in STOP_WORDS or word in seen:
            continue
        seen.append(word)
    return seen


def find_start(payload: dict) -> Path:
    cwd = payload.get("cwd")
    if isinstance(cwd, str) and Path(cwd).is_dir():
        return Path(cwd)
    return Path.cwd()


def read_guidance(start: Path) -> str:
    """CLAUDE.md / AGENTS.md from the nearest ancestor that has either."""
    for directory in (start, *start.parents):
        texts = []
        for name in ("CLAUDE.md", "AGENTS.md"):
            candidate = directory / name
            if candidate.is_file():
                try:
                    texts.append(candidate.read_bytes()[:MAX_DOC_BYTES].decode("utf-8", errors="ignore"))
                except OSError:
                    continue
        if texts:
            return "\n".join(texts).lower()
    return ""


def covered_by_guidance(words: list[str], guidance: str) -> int:
    if not guidance:
        return 0
    tokens = set(WORD.findall(guidance))
    return sum(1 for word in words if word in tokens)


def topic_key(topic: str) -> str:
    return hashlib.sha256(topic.encode("utf-8")).hexdigest()[:16]


def gaps_dir() -> Path:
    return Path.home() / ".prometheus" / "knowledge-gaps"


def already_seen(session: str, key: str) -> bool:
    path = gaps_dir() / "seen" / f"{re.sub(r'[^A-Za-z0-9_.-]', '_', session) or 'nosession'}.txt"
    try:
        return key in path.read_text(encoding="utf-8").split()
    except OSError:
        return False


def append_line(path: Path, line: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o600)
    try:
        os.write(descriptor, line.encode("utf-8"))
    finally:
        os.close(descriptor)


def mark_seen(session: str, key: str) -> None:
    name = re.sub(r"[^A-Za-z0-9_.-]", "_", session) or "nosession"
    append_line(gaps_dir() / "seen" / f"{name}.txt", key + "\n")


def build_record(payload: dict, prompt: str, topic: str, key: str, session: str) -> dict:
    record = {
        "ts": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
        "status": "open",
        "sessionId": session,
        "topic": topic,
        "topicKey": key,
        "promptHash": hashlib.sha256(prompt.encode("utf-8")).hexdigest()[:16],
    }
    try:
        from agent_identity import resolve  # noqa: WPS433

        identity = resolve(payload, find_start(payload), [])
    except Exception:
        identity = {}
    if identity.get("projectId"):
        record["projectId"] = identity["projectId"]
    if identity.get("roleId") and identity["roleId"] != "unresolved":
        record["teamId"] = identity["teamId"]
        record["roleId"] = identity["roleId"]
    return record


def detect_gap(report: dict, prompt: str, payload: dict) -> tuple[str, str] | None:
    if top_score(report) >= min_score() and (report.get("results") or []):
        return None
    if not is_problem_shaped(prompt):
        return None
    words = keywords(prompt)
    if not words:
        return None
    if covered_by_guidance(words, read_guidance(find_start(payload))) >= 2:
        return None
    topic = " ".join(words[:MAX_TOPIC_KEYWORDS])
    return topic, topic_key(topic)


def read_records() -> list[dict]:
    try:
        lines = (gaps_dir() / "gaps.jsonl").read_text(encoding="utf-8").splitlines()
    except OSError:
        return []
    records = []
    for line in lines:
        try:
            value = json.loads(line)
        except ValueError:
            continue
        if isinstance(value, dict):
            records.append(value)
    return records


def resolve_gaps(argv: list[str]) -> int:
    import argparse

    parser = argparse.ArgumentParser(prog="prompt_gap.py resolve")
    parser.add_argument("--topic", required=True)
    parser.add_argument("--entry-id", default=None)
    parser.add_argument("--scope", default=None)
    parser.add_argument("--by", default="learn-goal")
    options = parser.parse_args(argv)
    subject = keywords(options.topic)
    subject_key = topic_key(" ".join(subject[:MAX_TOPIC_KEYWORDS]))
    latest: dict[str, dict] = {}
    for record in read_records():
        key = record.get("topicKey")
        if isinstance(key, str):
            latest[key] = record
    for key, record in latest.items():
        if record.get("status") != "open":
            continue
        shared = set(keywords(str(record.get("topic", "")))) & set(subject)
        if key != subject_key and len(shared) < 2:
            continue
        resolved = {
            "ts": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
            "status": "resolved",
            "topicKey": key,
            "topic": record.get("topic"),
            "resolvedBy": options.by,
        }
        if options.entry_id:
            resolved["entryId"] = options.entry_id
        if options.scope:
            resolved["scope"] = options.scope
        append_line(gaps_dir() / "gaps.jsonl", json.dumps(resolved, sort_keys=True, separators=(",", ":")) + "\n")
        print(key)
    return 0


def main() -> int:
    if len(sys.argv) > 1 and sys.argv[1] == "resolve":
        try:
            return resolve_gaps(sys.argv[2:])
        except (SystemExit, Exception):
            return 0
    try:
        max_bytes = max(256, min(int(os.environ.get("PROMPT_GAP_MAX_BYTES", "6000")), 65_536))
    except ValueError:
        max_bytes = 6000
    try:
        report = json.loads(sys.stdin.buffer.read() or b"{}")
        if not isinstance(report, dict):
            return 0
    except ValueError:
        return 0
    prompt = os.environ.get("PROMPT_GAP_PROMPT", "")
    try:
        payload = json.loads(os.environ.get("PROMPT_GAP_PAYLOAD", "{}") or "{}")
        payload = payload if isinstance(payload, dict) else {}
    except ValueError:
        payload = {}

    out = render_hook(report, max_bytes)
    try:
        gap = detect_gap(report, prompt, payload)
        if gap:
            topic, key = gap
            session = payload.get("session_id") if isinstance(payload.get("session_id"), str) else ""
            session = session or "nosession"
            if not already_seen(session, key):
                mark_seen(session, key)
                record = build_record(payload, prompt, topic, key, session)
                append_line(gaps_dir() / "gaps.jsonl", json.dumps(record, sort_keys=True, separators=(",", ":")) + "\n")
                out += f"[prometheus-gap] {topic} — consider /learn-goal {topic} or /feynman-loop\n"
    except Exception:
        pass
    sys.stdout.buffer.write(out.encode("utf-8"))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
