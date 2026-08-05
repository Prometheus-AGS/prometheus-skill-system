#!/usr/bin/env python3
"""Execute and archive redacted Prometheus Exec release use cases locally."""

from __future__ import annotations

import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import signal
import socket
import subprocess
import tempfile
import time
from typing import Any


def run(command: list[str], *, check: bool = True) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(command, text=True, capture_output=True, check=False)
    if check and result.returncode != 0:
        raise RuntimeError(
            f"command failed ({result.returncode}): {' '.join(command)}\n{result.stderr}"
        )
    return result


def write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def indexed(path: Path, relative: str) -> dict[str, Any]:
    payload = path.read_bytes()
    return {
        "path": relative,
        "hash": f"sha256:{hashlib.sha256(payload).hexdigest()}",
        "sizeBytes": len(payload),
    }


def wait_ready(socket_path: Path, process: subprocess.Popen[str], timeout: float = 15.0) -> None:
    deadline = time.monotonic() + timeout
    last_response = b""
    while time.monotonic() < deadline:
        if process.poll() is not None:
            raise RuntimeError(f"daemon exited before readiness: {process.returncode}")
        try:
            with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as client:
                client.settimeout(1.0)
                client.connect(str(socket_path))
                client.sendall(
                    b"GET /ready HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"
                )
                chunks: list[bytes] = []
                while True:
                    chunk = client.recv(65536)
                    if not chunk:
                        break
                    chunks.append(chunk)
                last_response = b"".join(chunks)
                if last_response.startswith(b"HTTP/1.1 200"):
                    return
        except (FileNotFoundError, ConnectionRefusedError, socket.timeout, OSError):
            pass
        time.sleep(0.05)
    raise RuntimeError(f"daemon did not become ready: {last_response[-1000:]!r}")


def stop(process: subprocess.Popen[str], *, kill: bool = False) -> None:
    if process.poll() is not None:
        return
    process.send_signal(signal.SIGKILL if kill else signal.SIGTERM)
    try:
        process.wait(timeout=5)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait(timeout=5)


class McpClient:
    def __init__(self, command: list[str]) -> None:
        self.process = subprocess.Popen(
            command,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,
        )
        self.next_id = 1

    def send(self, payload: dict[str, Any]) -> None:
        assert self.process.stdin is not None
        self.process.stdin.write(json.dumps(payload, separators=(",", ":")) + "\n")
        self.process.stdin.flush()

    def request(self, method: str, params: dict[str, Any]) -> dict[str, Any]:
        request_id = self.next_id
        self.next_id += 1
        self.send({"jsonrpc": "2.0", "id": request_id, "method": method, "params": params})
        assert self.process.stdout is not None
        while True:
            line = self.process.stdout.readline()
            if not line:
                stderr = self.process.stderr.read() if self.process.stderr else ""
                raise RuntimeError(f"MCP server exited before {method}: {stderr}")
            response = json.loads(line)
            if response.get("id") == request_id:
                if "error" in response:
                    raise RuntimeError(f"MCP error for {method}: {response['error']}")
                return response["result"]

    def tool(self, name: str, arguments: dict[str, Any]) -> dict[str, Any]:
        result = self.request("tools/call", {"name": name, "arguments": arguments})
        content = result.get("content", [])
        if not content or content[0].get("type") != "text":
            raise RuntimeError(f"MCP tool {name} returned no text envelope")
        envelope = json.loads(content[0]["text"])
        if not envelope.get("ok"):
            raise RuntimeError(f"MCP tool {name} failed: {envelope}")
        return envelope["result"]

    def close(self) -> None:
        if self.process.stdin:
            self.process.stdin.close()
        stop(self.process)


def certify_mcp(binary: Path, plugin_root: Path, root: Path) -> dict[str, Any]:
    identity = root / "identity.json"
    state = root / "state"
    run([str(binary), "init", "--identity", str(identity)])
    public_identity = json.loads(identity.read_text(encoding="utf-8"))
    client = McpClient(
        [
            str(binary),
            "mcp",
            "--state-dir",
            str(state),
            "--identity",
            str(identity),
            "--plugin-root",
            str(plugin_root),
            "--artifact-budget-mb",
            "64",
        ]
    )
    try:
        initialized = client.request(
            "initialize",
            {
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": {"name": "prometheus-exec-certifier", "version": "1.7.0"},
            },
        )
        client.send({"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}})
        listed = client.request("tools/list", {})
        tool_names = sorted(tool["name"] for tool in listed["tools"])
        expected = [
            "exec-artifact",
            "exec-events",
            "exec-receipt",
            "exec-run",
            "exec-status",
            "exec-verify",
        ]
        if tool_names != expected:
            raise RuntimeError(f"unexpected MCP tool set: {tool_names}")

        code = b"""import json\nimport os\nfrom pathlib import Path\npayload = {\"surface\": \"mcp\", \"answer\": 6 * 7}\nPath(os.environ[\"PROMETHEUS_OUTPUT_DIR\"], \"mcp-result.json\").write_text(json.dumps(payload, sort_keys=True), encoding=\"utf-8\")\nprint(json.dumps(payload, sort_keys=True))\n"""
        encoded = base64.urlsafe_b64encode(code).decode("ascii").rstrip("=")
        submitted = client.tool(
            "exec-run",
            {"runtime": "python3", "codeBase64": encoded, "timeoutMs": 5000, "outputMb": 2},
        )
        run_id = submitted["runId"]
        status = submitted
        deadline = time.monotonic() + 15
        while status["state"] not in {"succeeded", "failed", "cancelled", "expired"}:
            if time.monotonic() >= deadline:
                raise RuntimeError("MCP run did not become terminal")
            time.sleep(0.05)
            status = client.tool("exec-status", {"runId": run_id})
        if status["state"] != "succeeded":
            raise RuntimeError(f"MCP run failed: {status}")
        events = client.tool("exec-events", {"runId": run_id, "after": 0})
        resumed = client.tool("exec-events", {"runId": run_id, "after": events[0]["sequence"]})
        receipt = client.tool("exec-receipt", {"runId": run_id})
        artifact = receipt["outputs"]["artifacts"][0]
        artifact_result = client.tool(
            "exec-artifact", {"digest": artifact["hash"], "inlineCeilingBytes": 262144}
        )
        verified = client.tool(
            "exec-verify",
            {"receipt": receipt, "publicKey": public_identity["publicKey"]},
        )
        transcript = {
            "surface": "real-rmcp-stdio",
            "serverInfo": initialized["serverInfo"],
            "protocolVersion": initialized["protocolVersion"],
            "tools": tool_names,
            "run": status,
            "events": events,
            "resumedEvents": resumed,
            "artifact": artifact_result,
            "verification": verified,
            "privateIdentityArchived": False,
        }
        write_json(root / "mcp-transcript.redacted.json", transcript)
        return transcript
    finally:
        client.close()


def daemon_command(binary: Path, socket_path: Path, state: Path, identity: Path, plugin_root: Path) -> list[str]:
    return [
        str(binary),
        "daemon",
        "--socket",
        str(socket_path),
        "--state-dir",
        str(state),
        "--identity",
        str(identity),
        "--plugin-root",
        str(plugin_root),
        "--artifact-budget-mb",
        "64",
    ]


def certify_tier_w(binary: Path, plugin_root: Path, component: Path, root: Path) -> dict[str, Any]:
    identity = root / "identity.json"
    state = root / "state"
    socket_path = root / "runtime" / "exec.sock"
    run([str(binary), "init", "--identity", str(identity)])
    public_identity = json.loads(identity.read_text(encoding="utf-8"))
    daemon = subprocess.Popen(
        daemon_command(binary, socket_path, state, identity, plugin_root),
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
        text=True,
    )
    wait_ready(socket_path, daemon)
    executed = run(
        [
            str(binary),
            "run",
            "--socket",
            str(socket_path),
            "--state-dir",
            str(state),
            "--identity",
            str(identity),
            "--plugin-root",
            str(plugin_root),
            "--runtime",
            "wasm-component",
            "--code",
            str(component),
            "--timeout-ms",
            "5000",
            "--output-mb",
            "2",
            "--artifact-budget-mb",
            "64",
            "--format",
            "json",
        ]
    )
    terminal = json.loads(executed.stdout)
    if terminal["state"] != "succeeded":
        raise RuntimeError(f"Tier W run failed: {terminal}")
    run_id = terminal["runId"]
    status_before = json.loads(
        run(
            [
                str(binary),
                "status",
                "--socket",
                str(socket_path),
                "--run-id",
                run_id,
                "--format",
                "json",
            ]
        ).stdout
    )
    stop(daemon, kill=True)
    stale_socket_observed = socket_path.exists()
    restarted = subprocess.Popen(
        daemon_command(binary, socket_path, state, identity, plugin_root),
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
        text=True,
    )
    wait_ready(socket_path, restarted)
    status_after = json.loads(
        run(
            [
                str(binary),
                "status",
                "--socket",
                str(socket_path),
                "--run-id",
                run_id,
                "--format",
                "json",
            ]
        ).stdout
    )
    doctor = run(
        [
            str(binary),
            "doctor",
            "--socket",
            str(socket_path),
            "--state-dir",
            str(state),
            "--identity",
            str(identity),
            "--plugin-root",
            str(plugin_root),
            "--exclude",
            "control.kbd-runtime",
            "--exclude",
            "state.kbd-orchestrator",
            "--exclude",
            "control.kbd-rollout",
            "--exclude",
            "service:sovereign-sync",
            "--format",
            "json",
        ]
    )
    stop(restarted, kill=True)
    if status_before != status_after:
        raise RuntimeError("Tier W status changed across SIGKILL restart")
    receipt = terminal["receipt"]
    record_paths = sorted((state / "service" / "ledger" / "runs").glob("*.json"))
    if len(record_paths) != 1:
        raise RuntimeError(f"expected one durable run record, found {len(record_paths)}")
    record_path = record_paths[0]
    record = json.loads(record_path.read_text(encoding="utf-8"))
    request_path = root / "request.json"
    receipt_path = root / "receipt.json"
    write_json(request_path, record["request"])
    write_json(receipt_path, receipt)
    verify = json.loads(
        run(
            [
                str(binary),
                "verify",
                "--receipt",
                str(receipt_path),
                "--public-key",
                public_identity["publicKey"],
                "--request",
                str(request_path),
                "--component",
                str(component),
                "--format",
                "json",
            ]
        ).stdout
    )
    environment_path = root / "environment.json"
    environment_path.write_bytes(b"{}")
    bundle_root = root / "portable-bundle"
    bundle_root.mkdir(parents=True, exist_ok=True)
    bundle_request = bundle_root / "request.json"
    bundle_receipt = bundle_root / "receipt.json"
    bundle_environment = bundle_root / "environment.json"
    bundle_request.write_bytes(request_path.read_bytes())
    bundle_receipt.write_bytes(receipt_path.read_bytes())
    bundle_environment.write_bytes(environment_path.read_bytes())
    artifacts: list[dict[str, Any]] = []
    for number, artifact in enumerate(receipt["outputs"]["artifacts"], start=1):
        digest = artifact["hash"].removeprefix("sha256:")
        source = state / "artifacts" / "blobs" / "sha256" / digest[:2] / digest[2:]
        relative = f"artifacts/{number:02d}-{Path(artifact['path']).name}"
        destination = bundle_root / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(source.read_bytes())
        artifacts.append({"receiptPath": artifact["path"], **indexed(destination, relative)})
    index = {
        "schemaVersion": "1",
        "requirementId": "exec.release-real-use-cases",
        "runId": run_id,
        "environment": "macos-local-release-candidate",
        "receipt": indexed(bundle_receipt, "receipt.json"),
        "request": indexed(bundle_request, "request.json"),
        "verificationIdentity": {
            "sigAlg": "ed25519",
            "keyId": public_identity["keyId"],
            "publicKey": public_identity["publicKey"],
        },
        "artifacts": artifacts,
        "environments": [indexed(bundle_environment, "environment.json")],
    }
    index_path = bundle_root / "index.json"
    write_json(index_path, index)
    bundle_verification = json.loads(
        run(
            [
                str(binary),
                "verify-bundle",
                "--index",
                str(index_path),
                "--root",
                str(bundle_root),
                "--format",
                "json",
            ]
        ).stdout
    )
    result = {
        "run": terminal,
        "statusBeforeRestart": status_before,
        "statusAfterRestart": status_after,
        "staleSocketObservedAfterSigkill": stale_socket_observed,
        "doctor": json.loads(doctor.stdout),
        "offlineVerification": verify,
        "portableBundleVerification": bundle_verification,
        "privateIdentityArchived": False,
    }
    write_json(root / "tier-w-restart-offline.redacted.json", result)
    return result


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--plugin-root", type=Path, required=True)
    parser.add_argument("--component", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--source-commit", required=True)
    args = parser.parse_args()

    binary = args.binary.resolve()
    plugin_root = args.plugin_root.resolve()
    component = args.component.resolve()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    # Keep UDS paths below macOS's sockaddr_un length limit.
    with tempfile.TemporaryDirectory(prefix="pexec-", dir="/tmp") as temporary:
        temporary_root = Path(temporary)
        mcp = certify_mcp(binary, plugin_root, temporary_root / "mcp")
        tier_w = certify_tier_w(
            binary, plugin_root, component, temporary_root / "tier-w"
        )
        for source in [
            temporary_root / "mcp" / "mcp-transcript.redacted.json",
            temporary_root / "tier-w" / "tier-w-restart-offline.redacted.json",
        ]:
            (output / source.name).write_bytes(source.read_bytes())
        bundle_source = temporary_root / "tier-w" / "portable-bundle"
        bundle_target = output / "portable-bundle"
        for source in sorted(bundle_source.rglob("*")):
            if source.is_file():
                destination = bundle_target / source.relative_to(bundle_source)
                destination.parent.mkdir(parents=True, exist_ok=True)
                destination.write_bytes(source.read_bytes())

    summary = {
        "schemaVersion": 1,
        "sourceCommit": args.source_commit,
        "binaryVersion": run([str(binary), "--version"]).stdout.strip(),
        "binarySha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
        "pluginGeneration": (plugin_root / "current").resolve().name,
        "mcp": {
            "protocolVersion": mcp["protocolVersion"],
            "tools": mcp["tools"],
            "runId": mcp["run"]["runId"],
            "state": mcp["run"]["state"],
            "backend": mcp["run"]["receipt"]["backend"],
            "artifactInline": mcp["artifact"]["inline"],
            "verificationValid": mcp["verification"]["valid"],
        },
        "tierW": {
            "runId": tier_w["run"]["runId"],
            "state": tier_w["run"]["state"],
            "backend": tier_w["run"]["receipt"]["backend"],
            "engineVersion": tier_w["run"]["receipt"]["component"]["engineVersion"],
            "restartExact": tier_w["statusBeforeRestart"] == tier_w["statusAfterRestart"],
            "offlineVerificationValid": tier_w["offlineVerification"]["valid"],
            "portableBundleValid": tier_w["portableBundleVerification"]["valid"],
            "doctorHealthy": tier_w["doctor"]["healthy"],
        },
        "externalEvidence": {
            "productionRemoteAdapter": "pending_evidence",
            "physicalMobileDevices": "pending_evidence",
            "installedService": "not evaluated by this disposable certification",
            "githubActions": "not certification evidence",
            "kbdOrSovereignServiceInvoked": False,
        },
    }
    write_json(output / "summary.json", summary)
    print(json.dumps(summary, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
