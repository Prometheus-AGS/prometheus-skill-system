#!/usr/bin/env python3
"""Integration acceptance for the progress-memory recorder and its process boundaries."""

from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time


ROOT = Path(__file__).resolve().parents[4]
RECORDER = ROOT / "skills/process/karpathy-progress-memory/scripts/record-progress.py"


def run(command: list[str], cwd: Path, env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    merged = os.environ.copy()
    if env:
        merged.update(env)
    return subprocess.run(
        command,
        cwd=cwd,
        env=merged,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )


def require(result: subprocess.CompletedProcess[str], operation: str) -> None:
    if result.returncode != 0:
        raise AssertionError(
            f"{operation} failed with {result.returncode}\nstdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        )


def tree_digest(root: Path) -> str:
    digest = hashlib.sha256()
    for path in sorted(item for item in root.rglob("*") if item.is_file()):
        digest.update(path.relative_to(root).as_posix().encode())
        digest.update(b"\0")
        digest.update(path.read_bytes())
        digest.update(b"\0")
    return digest.hexdigest()


def write_executable(path: Path, content: str) -> None:
    path.write_text(content, encoding="utf-8")
    path.chmod(0o755)


def event(event_id: str, boundary: str, commit: str) -> dict[str, object]:
    return {
        "schemaVersion": 1,
        "eventId": event_id,
        "observedAt": "2026-09-19T19:00:00Z",
        "runId": "run-1",
        "boundary": boundary,
        "status": "complete",
        "phaseId": "phase-1",
        "changeId": "change-1" if boundary in {"task", "change"} else None,
        "taskId": "task-1" if boundary == "task" else None,
        "taskClass": "integration",
        "elapsedHours": 1.25,
        "touchedFiles": ["src/example.txt"],
        "verification": [
            {"command": "integration-check", "exitCode": 0, "summary": "1 scenario passed"}
        ],
        "commitSha": commit,
        "blocker": None,
        "exactNextWork": "change-1/task-2",
    }


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="karpathy-progress-memory-") as raw:
        fixture = Path(raw)
        project = fixture / "project"
        bin_dir = fixture / "bin"
        queue = fixture / "queue"
        project.joinpath(".prometheus").mkdir(parents=True)
        project.joinpath(".kbd-orchestrator").mkdir()
        project.joinpath("src").mkdir()
        project.joinpath(".prometheus/project.json").write_text(
            '{"id":"progress-memory-integration"}\n', encoding="utf-8"
        )
        project.joinpath(".kbd-orchestrator/current-waypoint.json").write_text(
            '{"generatedBy":"kbd-runtime","revision":9}\n', encoding="utf-8"
        )
        project.joinpath(".kbd-orchestrator/progress.json").write_text(
            '{"generatedBy":"kbd-runtime","nextTask":"task-2"}\n', encoding="utf-8"
        )
        project.joinpath("src/example.txt").write_text("fixture\n", encoding="utf-8")

        require(run(["git", "init", "-q"], project), "initialize git fixture")
        require(run(["git", "config", "user.name", "Progress Memory Test"], project), "configure git name")
        require(run(["git", "config", "user.email", "progress-memory@example.invalid"], project), "configure git email")
        require(run(["git", "add", "."], project), "stage git fixture")
        require(
            run(["git", "-c", "commit.gpgsign=false", "commit", "-qm", "test: initialize fixture"], project),
            "commit git fixture",
        )
        commit = run(["git", "rev-parse", "HEAD"], project).stdout.strip()

        state = {
            "schemaVersion": 4,
            "projectId": "progress-memory-integration",
            "runId": "run-1",
            "revision": 9,
            "exactNextWork": "change-1/task-2",
            "activePath": {
                "phasePath": ["phase-1"],
                "phaseId": "phase-1",
                "changeId": "change-1",
                "taskId": "task-2",
            },
            "phases": {
                "phase-1": {
                    "id": "phase-1",
                    "status": "complete",
                    "changes": {
                        "change-1": {
                            "id": "change-1",
                            "status": "complete",
                            "implementationStatus": "complete",
                            "tasks": {
                                "task-1": {"id": "task-1", "status": "complete"},
                                "task-2": {"id": "task-2", "status": "pending"},
                            },
                        }
                    },
                }
            },
        }
        state_path = fixture / "status.json"
        state_path.write_text(json.dumps(state), encoding="utf-8")
        bin_dir.mkdir()
        fake_prometheus = bin_dir / "prometheus"
        write_executable(fake_prometheus, "#!/bin/sh\ncat \"$KPM_TEST_STATE\"\n")
        fake_pk = bin_dir / "pk"
        write_executable(
            fake_pk,
            """#!/bin/sh
payload=$(cat)
printf '%s\t%s\n' "$*" "$payload" >> "$KPM_TEST_PK_LOG"
[ "${KPM_TEST_PK_HANG:-0}" = 1 ] && sleep 5
[ "${KPM_TEST_PK_FAIL:-0}" = 1 ] && { echo 'memory unavailable' >&2; exit 23; }
printf '{"accepted":true}\n'
""",
        )
        pk_log = fixture / "pk.log"
        common_env = {
            "PROMETHEUS_BIN": str(fake_prometheus),
            "PK_BIN": str(fake_pk),
            "KPM_TEST_STATE": str(state_path),
            "KPM_TEST_PK_LOG": str(pk_log),
            "PROMETHEUS_LEARNING_QUEUE": str(queue),
        }
        projections_before = tree_digest(project / ".kbd-orchestrator")

        hook_env = {
            **common_env,
            "KBD_HOOK_NAME": "change-1:task-1",
            "KBD_TASK_CLASS": "integration",
            "KBD_TASK_ELAPSED_HOURS": "1.25",
        }
        first = run(
            [sys.executable, str(RECORDER), "--project-root", str(project), "--from-hook", "--boundary", "task"],
            project,
            hook_env,
        )
        require(first, "record successful task hook")
        first_result = json.loads(first.stdout)
        assert first_result["status"] == "recorded", first_result
        assert first_result["memory"]["transport"] == "pk", first_result

        state["revision"] = 10
        state["exactNextWork"] = "change-2/task-9"
        project.joinpath("src/unrelated.txt").write_text("unrelated commit\n", encoding="utf-8")
        require(run(["git", "add", "src/unrelated.txt"], project), "stage unrelated replay commit")
        require(
            run(
                ["git", "-c", "commit.gpgsign=false", "commit", "-qm", "test: advance unrelated head"],
                project,
            ),
            "commit unrelated replay state",
        )
        state_path.write_text(json.dumps(state), encoding="utf-8")
        replay = run(
            [sys.executable, str(RECORDER), "--project-root", str(project), "--from-hook", "--boundary", "task"],
            project,
            hook_env,
        )
        require(replay, "replay task hook after process restart")
        assert json.loads(replay.stdout)["status"] == "duplicate", replay.stdout
        log_text = project.joinpath(".prometheus/session-log.md").read_text(encoding="utf-8")
        assert log_text.count(first_result["eventId"]) == 2, "event ID should appear once in marker and once in body"
        assert (
            pk_log.read_text(encoding="utf-8").count("--scope project --source karpathy-progress-memory:")
            == 1
        ), "duplicate replay reached pk"

        state["runId"] = "run-2"
        state["revision"] = 1
        state_path.write_text(json.dumps(state), encoding="utf-8")
        successor = run(
            [sys.executable, str(RECORDER), "--project-root", str(project), "--from-hook", "--boundary", "task"],
            project,
            hook_env,
        )
        require(successor, "record identical task identity in successor run")
        successor_result = json.loads(successor.stdout)
        assert successor_result["status"] == "recorded", successor_result
        assert successor_result["eventId"] != first_result["eventId"], successor_result
        assert pk_log.read_text(encoding="utf-8").count(
            "--scope project --source karpathy-progress-memory:"
        ) == 2
        state["runId"] = "run-1"
        state["revision"] = 11
        state_path.write_text(json.dumps(state), encoding="utf-8")

        outage_event = fixture / "change-event.json"
        outage_event.write_text(json.dumps(event("progress-outage-0001", "change", commit)), encoding="utf-8")
        outage = run(
            [sys.executable, str(RECORDER), "--project-root", str(project), "--input", str(outage_event)],
            project,
            {**common_env, "KPM_TEST_PK_FAIL": "1"},
        )
        require(outage, "queue progress record during pk outage")
        outage_result = json.loads(outage.stdout)
        assert outage_result["status"] == "queued", outage_result
        pending = list((queue / "memory/pending").glob("*.json"))
        assert len(pending) == 1, f"expected one durable outbox item, found {len(pending)}"
        pending_payload = json.loads(pending[0].read_text(encoding="utf-8"))
        assert pending_payload["arguments"]["user_id"] == "progress-memory-integration"

        outage_replay = run(
            [sys.executable, str(RECORDER), "--project-root", str(project), "--input", str(outage_event)],
            project,
            {**common_env, "KPM_TEST_PK_FAIL": "1"},
        )
        require(outage_replay, "replay queued event after restart")
        assert json.loads(outage_replay.stdout)["status"] == "duplicate"
        assert len(list((queue / "memory/pending").glob("*.json"))) == 1

        altered_event = event("progress-outage-0001", "change", commit)
        altered_event["exactNextWork"] = "different-work"
        altered_path = fixture / "altered-event.json"
        altered_path.write_text(json.dumps(altered_event), encoding="utf-8")
        altered = run(
            [sys.executable, str(RECORDER), "--project-root", str(project), "--input", str(altered_path)],
            project,
            common_env,
        )
        assert altered.returncode == 2, altered
        assert "already belongs to a different payload" in altered.stderr

        concurrent_event = fixture / "concurrent-event.json"
        concurrent_event.write_text(
            json.dumps(event("progress-concurrent-0001", "phase", commit)), encoding="utf-8"
        )
        processes = [
            subprocess.Popen(
                [sys.executable, str(RECORDER), "--project-root", str(project), "--input", str(concurrent_event)],
                cwd=project,
                env={**os.environ, **common_env},
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )
            for _ in range(2)
        ]
        concurrent_results = [process.communicate(timeout=10) + (process.returncode,) for process in processes]
        assert all(result[2] == 0 for result in concurrent_results), concurrent_results
        assert sorted(json.loads(result[0])["status"] for result in concurrent_results) == [
            "duplicate",
            "recorded",
        ]
        pk_text = pk_log.read_text(encoding="utf-8")
        assert pk_text.count("karpathy-progress-memory:progress-concurrent-0001") == 1

        pre_submit_event = fixture / "crash-before-memory-event.json"
        pre_submit_event.write_text(
            json.dumps(event("progress-crash-before-memory-0001", "phase", commit)), encoding="utf-8"
        )
        pre_submit_crash = run(
            [sys.executable, str(RECORDER), "--project-root", str(project), "--input", str(pre_submit_event)],
            project,
            {**common_env, "KPM_TEST_CRASH_BEFORE_MEMORY": "1"},
        )
        assert pre_submit_crash.returncode == 74, pre_submit_crash
        pre_submit_replay = run(
            [sys.executable, str(RECORDER), "--project-root", str(project), "--input", str(pre_submit_event)],
            project,
            common_env,
        )
        require(pre_submit_replay, "recover pending delivery after pre-submit crash")
        pre_submit_result = json.loads(pre_submit_replay.stdout)
        assert pre_submit_result["status"] == "recorded", pre_submit_result
        assert pre_submit_result["recoveredPendingDelivery"] is True, pre_submit_result
        pk_text = pk_log.read_text(encoding="utf-8")
        assert pk_text.count("karpathy-progress-memory:progress-crash-before-memory-0001") == 1

        crash_event = fixture / "crash-event.json"
        crash_event.write_text(
            json.dumps(event("progress-crash-after-pk-0001", "phase", commit)), encoding="utf-8"
        )
        crashed = run(
            [sys.executable, str(RECORDER), "--project-root", str(project), "--input", str(crash_event)],
            project,
            {**common_env, "KPM_TEST_CRASH_AFTER_PK": "1"},
        )
        assert crashed.returncode == 75, crashed
        crash_replay = run(
            [sys.executable, str(RECORDER), "--project-root", str(project), "--input", str(crash_event)],
            project,
            common_env,
        )
        require(crash_replay, "replay event after injected post-pk crash")
        crash_result = json.loads(crash_replay.stdout)
        assert crash_result["status"] == "duplicate" and crash_result["deliveryComplete"] is True
        pk_text = pk_log.read_text(encoding="utf-8")
        assert pk_text.count("karpathy-progress-memory:progress-crash-after-pk-0001") == 1

        hang_event = fixture / "hang-event.json"
        hang_event.write_text(
            json.dumps(event("progress-hung-pk-0001", "phase", commit)), encoding="utf-8"
        )
        started = time.monotonic()
        hung = run(
            [sys.executable, str(RECORDER), "--project-root", str(project), "--input", str(hang_event)],
            project,
            {
                **common_env,
                "KPM_TEST_PK_HANG": "1",
                "KPM_PK_TIMEOUT_SECONDS": "0.1",
            },
        )
        elapsed = time.monotonic() - started
        require(hung, "queue progress after bounded pk timeout")
        assert json.loads(hung.stdout)["status"] == "queued", hung.stdout
        assert elapsed < 2, f"hung pk exceeded fixed bound: {elapsed:.2f}s"
        assert len(list((queue / "memory/pending").glob("*.json"))) == 2

        secret_tokens = [
            "gh" + "p_abcdefghijklmnopqrstuvwxyz123456",
            "xo" + "xb-1234567890-abcdefghijklmnop",
            "AK" + "IA1234567890ABCDEF",
            "sk-" + "proj-abcdefghijklmnopqrstuvwxyz",
        ]
        secret_fields = ["phaseId", "changeId", "taskId", "blocker", "exactNextWork"]
        for index, field in enumerate(secret_fields):
            secret_event = event(f"progress-secret-field-{index:04d}", "task", commit)
            secret_event[field] = secret_tokens[index % len(secret_tokens)]
            secret_path = fixture / f"secret-field-{index}.json"
            secret_path.write_text(json.dumps(secret_event), encoding="utf-8")
            rejected = run(
                [sys.executable, str(RECORDER), "--project-root", str(project), "--input", str(secret_path)],
                project,
                common_env,
            )
            assert rejected.returncode == 2 and "contain a secret" in rejected.stderr
        for index, (field, token) in enumerate(
            [
                ("touchedFiles", secret_tokens[0]),
                ("command", secret_tokens[1]),
                ("summary", secret_tokens[2]),
                ("blocker", secret_tokens[3]),
            ]
        ):
            secret_event = event(f"progress-secret-text-{index:04d}", "task", commit)
            if field == "touchedFiles":
                secret_event[field] = [token]
            elif field in {"command", "summary"}:
                secret_event["verification"][0][field] = token
            else:
                secret_event[field] = token
            secret_path = fixture / f"secret-text-{index}.json"
            secret_path.write_text(json.dumps(secret_event), encoding="utf-8")
            rejected = run(
                [sys.executable, str(RECORDER), "--project-root", str(project), "--input", str(secret_path)],
                project,
                common_env,
            )
            assert rejected.returncode == 2 and "contain a secret" in rejected.stderr

        mismatch_event = event("progress-mismatch-0001", "task", commit)
        mismatch_event["taskId"] = "task-2"
        mismatch_path = fixture / "mismatch-event.json"
        mismatch_path.write_text(json.dumps(mismatch_event), encoding="utf-8")
        mismatch = run(
            [sys.executable, str(RECORDER), "--project-root", str(project), "--input", str(mismatch_path)],
            project,
            common_env,
        )
        assert mismatch.returncode == 2, mismatch
        assert "canonical task status is 'pending'" in mismatch.stderr, mismatch.stderr
        assert "progress-mismatch-0001" not in project.joinpath(".prometheus/session-log.md").read_text(encoding="utf-8")

        projections_after = tree_digest(project / ".kbd-orchestrator")
        assert projections_after == projections_before, "recorder mutated a generated KBD projection"
        receipts = list(project.joinpath(".prometheus/progress-memory-receipts").glob("*.json"))
        assert len(receipts) == 7, f"expected seven durable receipts, found {len(receipts)}"

        print("progress-memory integration: 12 scenarios passed")
        print("covered: run-scoped identity, dynamic-state replay, concurrency, pre/post-pk crash, bounded timeout, secret rejection, outage, restart, mismatch, projection non-mutation")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
