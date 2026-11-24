#!/usr/bin/env python3
"""Dispatch the nine real-process cases and retain a separate receipt per attempt."""
import argparse
import datetime
import fcntl
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time

FULL = Path(__file__).resolve().parents[2]
PHASE = FULL / ".kbd-orchestrator/phases/vlbv-r4-portability/children/intersession-communication"
CASES = {
    "local": ["LOCAL-001", "LOCAL-002", "LOCAL-003"],
    "peer": ["PEER-001", "PEER-002", "PEER-003", "PEER-004"],
    "health": ["GREEN-001", "GREEN-002"],
}


class Blocked(Exception):
    """A required input or resource is unavailable."""


def read(path):
    return json.loads(path.read_bytes())


def save(path, data):
    with tempfile.NamedTemporaryFile(dir=path.parent, mode="w", delete=False) as writer:
        json.dump(data, writer, indent=2, ensure_ascii=False)
        writer.write("\n")
    os.replace(writer.name, path)


def digest(path):
    hasher = hashlib.sha256()
    with path.open("rb") as src:
        for block in iter(lambda: src.read(1024 * 1024), b""):
            hasher.update(block)
    return hasher.hexdigest()


def git(root, *args):
    return subprocess.check_output(["git", "-C", str(root), *args])


def checkout(root, paths):
    head = git(root, "rev-parse", "HEAD").decode().strip()
    changed = git(root, "diff", "HEAD", "--name-only", "-z", "--", *paths)
    untracked = git(root, "ls-files", "--others", "--exclude-standard", "-z", "--", *paths)
    files = {}
    for name in sorted(set((changed + untracked).split(b"\0")) - {b""}):
        relative = os.fsdecode(name)
        path = root / relative
        if path.is_symlink():
            files[relative] = {"link": os.readlink(path)}
        elif path.is_file():
            files[relative] = digest(path)
        elif not path.exists():
            files[relative] = "deleted"
        else:
            raise Blocked(f"unresolved changed dependency directory: {relative}")
    return {"root": str(root), "commit": head, "dirtyFiles": files}


def command(exe, *arguments):
    return subprocess.check_output([exe, *arguments], text=True).strip()


def build(workspace, flags, expected, logfile):
    if subprocess.run(["pgrep", "-x", "cargo|rustc"], stdout=subprocess.DEVNULL).returncode == 0:
        raise Blocked("another Cargo or rustc process is active on this machine")
    argv = ["cargo", "build", "--locked", "--message-format=json-render-diagnostics", *flags]
    executables = {}
    with logfile.open("wb") as log:
        with subprocess.Popen(argv, cwd=workspace, stdout=subprocess.PIPE, stderr=log) as process:
            for line in process.stdout:
                log.write(line)
                if line.startswith(b"{"):
                    item = json.loads(line)
                    if item.get("reason") == "compiler-artifact" and item.get("executable"):
                        executables[item["target"]["name"]] = item["executable"]
            if process.wait():
                raise RuntimeError(f"locked candidate build failed; see {logfile}")
    outputs = {}
    for name in expected:
        if name not in executables:
            raise RuntimeError(f"Cargo omitted candidate {name}; see {logfile}")
        binary = Path(executables[name]).resolve(strict=True)
        outputs[name] = {"path": str(binary), "sha256": digest(binary)}
    return outputs


def verify_calibration():
    calibration = read(PHASE / "calibration.json")
    recorded_root = Path(read(PHASE / "failure-inventory.json")["sourceRoots"]["receiptRoot"])
    historical = Path(os.environ.get("KBD_HEALTH_BASELINE_ROOT", PHASE / "evidence/baseline-inputs"))
    for source, expected in {**calibration["sourceIdentities"], **calibration["nativeRequirements"]}.items():
        path = Path(source)
        if path.is_absolute():
            # Keep signed source identities intact while reopening the same frozen bytes.
            path = historical / path.relative_to(recorded_root)
        else:
            path = FULL / path
        if digest(path) != expected:
            raise Blocked(f"calibrated input changed: {path}")


def candidate_inputs(companion, identity):
    return {
        "contract": identity,
        "full": checkout(FULL, ["tools/prometheus-cli", "substrate", ".cargo", "package.json", "skill-system.json",
                                "scripts", "shared/scripts", "config", "openspec",
                                "skills/process/agent-team-creator", "skills/process/agent-team-handoff"]),
        "companion": checkout(companion, ["crates", "Cargo.toml", "Cargo.lock", ".cargo", "scripts", "openspec"]),
        "calibration": digest(PHASE / "calibration.json"),
        "failureInventory": digest(PHASE / "failure-inventory.json"),
        "recoveryLedger": digest(PHASE / "evidence/strict-recovery-ledger.json"),
        "toolchain": command("rustc", "-vV"),
        "cargo": command("cargo", "-V"),
        "rustflags": os.environ.get("RUSTFLAGS", ""),
        "locks": {str(p): digest(p) for p in [FULL / "tools/prometheus-cli/Cargo.lock", companion / "Cargo.lock"]},
    }


def prepare(companion, boundary_dir, manifest, attempt):
    record = boundary_dir / "candidate.json"
    if record.exists():
        candidate = read(record)
        if candidate["inputs"] != manifest:
            raise Blocked("candidate inputs changed since the first command of this boundary")
        for name, artifact in candidate["binaries"].items():
            if digest(Path(artifact["path"])) != artifact["sha256"]:
                raise Blocked(f"candidate binary changed: {name}")
        return candidate
    outputs = build(FULL / "tools/prometheus-cli", ["-p", "prometheus-cli", "--bin", "prometheus"],
                    ["prometheus"], attempt / "full-build.log")
    outputs.update(build(companion, ["--no-default-features", "--features", "prometheus-substrate/headless",
                                    "-p", "prometheus-substrate", "-p", "sovereign-sync",
                                    "--bin", "prometheus-companion-headless", "--bin", "sovereign-sync",
                                    "--example", "kbd-peer-fixture", "--example", "kbd-receiver-file-check"],
                         ["prometheus-companion-headless", "sovereign-sync", "kbd-peer-fixture", "kbd-receiver-file-check"],
                         attempt / "companion-build.log"))
    candidate = {"inputs": manifest, "binaries": outputs}
    save(record, candidate)
    return candidate


def execute(args):
    contract = read(PHASE / "adopted-contract.json")
    identity = contract["identity"]
    authorization = PHASE / "boundary-authorization.json"
    if not authorization.exists():
        raise Blocked("the separate local B4 authorization has not been recorded")
    record = read(authorization)
    event = record["events"][-1] if record.get("events") else {}
    if (record.get("phaseId") != identity["phaseId"] or event.get("boundary") not in ("B4", "B5")
            or event.get("contractDigest") != identity["contractDigest"]
            or not contract["status"]["implementationComplete"]):
        raise Blocked("this boundary needs current implementation completion and matching authorization")
    if not event.get("authorizedAt") or not event.get("authorizer"):
        raise Blocked("the boundary authorization is incomplete")
    epoch = hashlib.sha256(json.dumps(event, sort_keys=True).encode()).hexdigest()[:20]
    # Keep the signed historical packet immutable. Each checkout records fresh
    # observations and freezes the first candidate seen under each authorization.
    epoch_dir = FULL / ".scratch/kbd-intersession" / epoch
    epoch_dir.mkdir(mode=0o700, parents=True, exist_ok=True)
    selector = ("negative-" if args.negative_only else "run-") + args.case
    attempt = None
    receipt = {"commandId": selector, "case": args.case, "identity": identity, "authorization": event,
               "argv": ["bash", "scripts/verify-kbd-intersession.sh", *sys.argv[1:]],
               "startedAt": datetime.datetime.now(datetime.timezone.utc).isoformat()}
    start = time.monotonic()
    result = 1
    try:
        companion = Path(os.environ.get("COMPANION_ROOT", FULL.parent / "prometheus-companion")).resolve(strict=True)
        with (epoch_dir / ".lock").open("a") as lock:
            fcntl.flock(lock, fcntl.LOCK_EX)
            verify_calibration()
            inputs = candidate_inputs(companion, identity)
            fingerprint = hashlib.sha256(json.dumps(inputs, sort_keys=True).encode()).hexdigest()
            frozen = epoch_dir / "input-sha256.json"
            if frozen.exists() and read(frozen) != fingerprint:
                raise Blocked("candidate inputs changed since the first command of this boundary")
            if not frozen.exists():
                save(frozen, fingerprint)
            boundary_dir = epoch_dir / fingerprint
            boundary_dir.mkdir(mode=0o700, exist_ok=True)
            attempt = Path(tempfile.mkdtemp(prefix=selector + "-", dir=boundary_dir))
            candidate = prepare(companion, boundary_dir, inputs, attempt)
            binaries = candidate["binaries"]
            group = next(group for group, ids in CASES.items() if args.case in ids)
            script = (companion / "scripts/verify-kbd-peers.py" if group == "peer"
                      else FULL / f"scripts/tests/kbd-intersession-{group}.py")
            report = attempt / "observations.json"
            argv = [sys.executable, str(script), "--case", args.case, "--report", str(report),
                    "--pack-cli", binaries["prometheus"]["path"], "--headless",
                    binaries["prometheus-companion-headless"]["path"], "--sync-cli", binaries["sovereign-sync"]["path"]]
            if group == "peer":
                argv.extend(["--fixture-tool", binaries["kbd-peer-fixture"]["path"]])
                if args.case == "PEER-004" and args.negative_only:
                    argv.extend(["--receiver-file-check", binaries["kbd-receiver-file-check"]["path"]])
            if args.negative_only:
                argv.append("--negative-only")
            receipt.update(runner=str(script), runnerSha256=digest(script), candidateSha256=digest(boundary_dir / "candidate.json"))
            env = dict(os.environ, COMPANION_ROOT=str(companion), PYTHONDONTWRITEBYTECODE="1")
            if shutil.which("prometheus"):
                env["KBD_INTERSESSION_INSTALLED_CLI"] = shutil.which("prometheus")
            result = subprocess.run(argv, cwd=FULL, env=env).returncode
            if report.is_file():
                receipt["observationsSha256"] = digest(report)
                observations = read(report)
                if result == 0 and not (
                    isinstance(observations, dict) and observations.get("case") == args.case
                    and observations.get("negativeOnly") is args.negative_only
                    and observations.get("result") == "PASS"
                    and type(observations.get("assertionCount")) is int
                    and observations["assertionCount"] > 0
                ):
                    raise RuntimeError("case returned success without matching passing assertions")
            elif result == 0:
                raise RuntimeError("successful case produced no observation file")
    except (Blocked, FileNotFoundError) as error:
        result, receipt["diagnostic"] = 2, str(error)
    except Exception as error:
        result, receipt["diagnostic"] = 1, str(error)
    receipt.update(exitCode=result, status={0: "PASS", 2: "BLOCKED"}.get(result, "FAIL"),
                   durationSeconds=round(time.monotonic() - start, 3))
    if attempt is None:
        attempt = Path(tempfile.mkdtemp(prefix=selector + "-", dir=epoch_dir))
    save(attempt / "receipt.json", receipt)
    print(f"{receipt['status']} {selector}: {attempt / 'receipt.json'}", flush=True)
    return result if result in (0, 2) else 1


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", required=True, choices=sum(CASES.values(), []))
    parser.add_argument("--negative-only", action="store_true")
    try:
        sys.exit(execute(parser.parse_args()))
    except (Blocked, FileNotFoundError) as exc:
        print(f"BLOCKED: {exc}", file=sys.stderr)
        sys.exit(2)
