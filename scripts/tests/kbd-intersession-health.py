#!/usr/bin/env python3
"""Run local GREEN-001 and GREEN-002 with independent scratch fixtures."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import signal
import subprocess
import sys
import tempfile
import time

sys.dont_write_bytecode = True
from kbd_intersession_install_health import verify_installation


ROOT = Path(__file__).resolve().parents[2]
CHILD = ROOT / ".kbd-orchestrator/phases/vlbv-r4-portability/children/intersession-communication"
BASELINE = CHILD / "failure-inventory.json"
LEDGER = CHILD / "evidence/strict-recovery-ledger.json"
MANAGED = ROOT / "skills/process/kbd-process-orchestrator/shared/openspec"
STRICT = ["validate", "--all", "--strict", "--json", "--no-interactive"]


class Blocked(Exception):
    """A required local fixture or executable is unavailable."""


class Failed(Exception):
    """A local observation contradicted the verification contract."""


def sha(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def load(path: Path):
    return json.loads(path.read_text())


def executable(path: str | Path) -> Path:
    name = os.fspath(path)
    found = shutil.which(name)
    if not found:
        raise Blocked(f"Required executable is unavailable: {name}")
    # Preserve the real command name for wrapper and multicall entry points.
    return Path(found).absolute()


def fixture_env(home: Path, temporary: Path) -> dict[str, str]:
    home.mkdir(mode=0o700, parents=True, exist_ok=True)
    temporary.mkdir(mode=0o700, parents=True, exist_ok=True)
    return {
        "HOME": str(home),
        "PATH": os.pathsep.join((str(home / ".local/bin"), os.environ["PATH"])),
        "TMPDIR": str(temporary),
        "LANG": "en_US.UTF-8",
        "USER": os.environ.get("USER", ""),
        "LOGNAME": os.environ.get("LOGNAME", ""),
        "XDG_CONFIG_HOME": str(home / ".config"),
        "XDG_DATA_HOME": str(home / ".local/share"),
        "XDG_CACHE_HOME": str(home / ".cache"),
        "XDG_STATE_HOME": str(home / ".local/state"),
        "CODEX_HOME": str(home / ".codex"),
        "CLAUDE_CONFIG_DIR": str(home / ".claude"),
        "KIMI_CODE_HOME": str(home / ".kimi-code"),
        "PROMETHEUS_DATA_DIR": str(home / ".local/share/prometheus-data"),
        "GIT_CONFIG_NOSYSTEM": "1",
        "GIT_CONFIG_GLOBAL": os.devnull,
        "GIT_TERMINAL_PROMPT": "0",
        "GIT_OPTIONAL_LOCKS": "0",
        "NPM_CONFIG_CACHE": str(home / ".cache/npm"),
        "OPEN_SPEC_INTERACTIVE": "0",
        "OPENSPEC_NO_UPDATE_CHECK": "1",
        "OPENSPEC_TELEMETRY": "0",
        "DO_NOT_TRACK": "1",
        "CI": "1",
        "NO_COLOR": "1",
    }


def redact(text: str) -> str:
    text = re.sub(r"(?im)(authorization:\s*bearer\s+)\S+", r"\1[redacted]", text)
    return re.sub(
        r"(?im)((?:[\"']?[-\w]*(?:token|password|secret|api[_-]?key|private[_-]?key)[\"']?)\s*[:=]\s*)"
        r"[\"']?[^\s,\"'}]+[\"']?",
        r"\1[redacted]",
        text,
    )


class Probe:
    """Collect each process receipt before checking observations."""

    blocked = Blocked
    failed = Failed
    fixture_env = staticmethod(fixture_env)
    load = staticmethod(load)
    sha = staticmethod(sha)
    executable = staticmethod(executable)

    def __init__(self, report: Path, scenario: str, negative: bool):
        self.report = report
        self.log_dir = report.with_name(f"{report.name}.logs")
        self.log_dir.mkdir(mode=0o700, parents=True)
        self.data = {
            "case": scenario,
            "negativeOnly": negative,
            "commands": [],
            "assertions": [],
            "observations": {},
            "cleanup": {},
        }

    def command(self, name, args, *, cwd, env, stdin=None, timeout=180):
        args = list(map(os.fspath, args))
        started = time.monotonic()
        process = subprocess.Popen(
            args,
            cwd=cwd,
            env=env,
            text=True,
            stdin=subprocess.PIPE if stdin is not None else subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            start_new_session=True,
        )
        timed_out = False
        try:
            stdout, stderr = process.communicate(stdin, timeout=timeout)
        except subprocess.TimeoutExpired:
            timed_out = True
            os.killpg(process.pid, signal.SIGKILL)
            stdout, stderr = process.communicate()
        except BaseException:
            os.killpg(process.pid, signal.SIGKILL)
            process.communicate()
            raise
        observation = {
            "name": name, "argv": args, "cwd": str(cwd),
            "exitCode": process.returncode, "elapsedSeconds": round(time.monotonic() - started, 3),
            "pid": process.pid, "timeout": timed_out,
        }
        number = len(self.data["commands"])
        for channel, output in (("stdout", stdout), ("stderr", stderr)):
            log = self.log_dir / f"{number:03d}-{name}.{channel}"
            log.write_text(redact(output))
            observation[channel] = {"path": str(log), "sha256": sha(log)}
        self.data["commands"].append(observation)
        if timed_out:
            raise Failed(f"{name} exceeded its {timeout}s deadline")
        return subprocess.CompletedProcess(args, process.returncode, stdout, stderr)

    def expect(self, condition, message):
        self.data["assertions"].append({"name": message, "passed": bool(condition)})
        if not condition:
            raise Failed(message)

    def success(self, name, args, **kwargs):
        result = self.command(name, args, **kwargs)
        self.expect(result.returncode == 0, f"{name} exited zero")
        return result

    def json_result(self, result, label):
        try:
            return json.loads(result.stdout)
        except json.JSONDecodeError:
            raise Failed(f"{label} returned invalid JSON") from None

    def observe(self, name, value):
        self.data["observations"][name] = value

    def finish(self, status, detail=None):
        self.data.update(
            result=status, detail=detail, assertionCount=len(self.data["assertions"]),
        )
        temp = self.report.with_suffix(self.report.suffix + ".pending")
        with temp.open("x") as stream:
            json.dump(self.data, stream, indent=2, ensure_ascii=False)
            stream.write("\n")
        temp.replace(self.report)


def checkout_manifest(root: Path) -> dict:
    def git(*args):
        return subprocess.check_output(
            ["git", "--no-optional-locks", "-C", str(root), *args],
            env={**os.environ, "GIT_OPTIONAL_LOCKS": "0"},
        )

    changes = git("ls-files", "-z", "--modified", "--deleted", "--others", "--exclude-standard")
    dirty = {}
    for name in sorted(set(os.fsdecode(changes).split("\0")) - {""}):
        path = root / name
        if path.is_symlink():
            dirty[name] = {"link": os.readlink(path)}
        elif path.is_file():
            dirty[name] = {"sha256": sha(path)}
        elif path.is_dir():
            dirty[name] = {"directory": True}
        else:
            dirty[name] = {"absent": True}
    locks = {}
    for name in ("Cargo.lock", "package-lock.json", "tools/prometheus-cli/Cargo.lock"):
        if (root / name).is_file():
            locks[name] = sha(root / name)
    return {
        "head": git("rev-parse", "HEAD").decode().strip(),
        "dirty": dirty,
        "trackedDiffSha256": hashlib.sha256(git("diff", "--no-ext-diff", "--binary", "HEAD")).hexdigest(),
        "lockfiles": locks,
    }


def read_baselines(probe, scratch, inventory, names):
    received = {}
    historical = Path(os.environ.get("KBD_HEALTH_BASELINE_ROOT", CHILD / "evidence/baseline-inputs"))
    for name in names:
        receipt = inventory["sourceReceipts"][name]
        original = historical / receipt["path"]
        if not original.is_file():
            raise Blocked(f"Immutable baseline {name} is unavailable at {original}")
        copy = scratch / f"baseline-{name}.json"
        shutil.copyfile(original, copy)
        copy.chmod(0o400)
        digest = sha(copy)
        probe.expect(digest == receipt["sha256"], f"{name} matches its frozen SHA-256")
        received[name] = load(copy)
        probe.observe(name + "Input", {"path": str(original), "sha256": digest})
    return received


def typed_rows(rows):
    items = {}
    for row in rows:
        key = (row["type"], row["id"])
        if key in items:
            raise Failed(f"Duplicate result for {identity(key)}")
        items[key] = row
    return items


def identity(key):
    return "/".join(key)


def check_inventory(report, expected):
    actual = typed_rows(report["items"])
    missing = expected.keys() - actual.keys()
    if missing:
        raise Failed("Missing baseline identity: " + ", ".join(map(identity, sorted(missing))))
    failures = [identity(key) for key, row in actual.items() if not row["valid"] or row["issues"]]
    if failures:
        raise Failed("Strict inventory contains diagnostics: " + ", ".join(failures))
    totals = report["summary"]["totals"]
    if not actual or totals != {"items": len(actual), "passed": len(actual), "failed": 0}:
        raise Failed("Strict summary disagrees with item results")
    return actual


def check_locked_ledger(probe, roots, raw, inventory):
    ledger = load(LEDGER)
    probe.expect(ledger["inventory"]["sha256"] == sha(BASELINE), "Recovery ledger binds immutable inventory")
    probe.observe("recoveryLedger", {"path": str(LEDGER), "sha256": sha(LEDGER)})
    for name, historical, source in (
        ("full", "fullStrict", ROOT), ("companion", "companionStrict", roots["companion"]),
    ):
        expected = typed_rows(inventory[f"{name}OpenSpec"]["items"])
        original = typed_rows(raw[historical]["items"])
        source_rows = typed_rows(ledger[name]["items"])
        probe.expect(expected.keys() == original.keys() == source_rows.keys(), f"{name} baseline identities agree")
        for key, row in source_rows.items():
            recorded = original[key]
            probe.expect(
                recorded["issues"] == expected[key]["issues"] == row["originalDiagnostics"]
                and recorded["valid"] == expected[key]["valid"] == row["originalValid"],
                f"{name} {identity(key)} original diagnostics retained",
            )
            probe.expect(bool(row["sources"] and row["requirements"] and row["artifacts"]),
                         f"{name} {identity(key)} has source and requirement/scenario lineage")
            for authority in row["sources"]:
                path = source / authority["path"]
                if revision := authority.get("revision"):
                    before = subprocess.check_output(
                        ["git", "--no-optional-locks", "-C", str(source), "show",
                         f"{revision}:{authority['path']}"],
                    )
                    digest = hashlib.sha256(before).hexdigest()
                else:
                    digest = sha(path)
                probe.expect(digest == authority["sha256"],
                             f"{name} requirement source matches hash: {authority['path']}")
            for artifact in row["artifacts"]:
                path = source / artifact["path"]
                probe.expect(path.is_file() and sha(path) == artifact["sha256"],
                             f"{name} recovery artifact matches hash: {artifact['path']}")
    return ledger


def workspace_copy(probe, source: Path, destination: Path):
    destination.mkdir()
    copied = destination / "openspec"
    shutil.copytree(source / "openspec", copied, symlinks=True)
    for link in copied.rglob("*"):
        if link.is_symlink():
            probe.expect(link.exists() and link.resolve().is_relative_to(copied.resolve()),
                         f"OpenSpec symlink stays inside disposable store: {link.relative_to(copied)}")
    return destination


def baseline_version(probe, cli, env, cwd):
    result = probe.success("openspec-version", [cli, "--version"], cwd=cwd, env=env)
    if result.stdout.strip() != "2.1.1":
        raise Blocked(f"Direct OpenSpec CLI {cli} is not 2.1.1; supply KBD_HEALTH_OPENSPEC_211")
    probe.expect(True, "Direct OpenSpec CLI is the baseline 2.1.1")
    probe.observe("openSpecExecutable", {"path": str(cli.resolve()), "sha256": sha(cli)})


def strict_report(probe, name, command, cwd, env):
    result = probe.command(name, [*command, *STRICT], cwd=cwd, env=env, timeout=240)
    return result.returncode, probe.json_result(result, name)


def retained(probe, report, expected, label):
    actual = check_inventory(report, expected)
    probe.expect(True, f"{label} reports every baseline identity and zero strict diagnostics")
    probe.observe(label, [
        {"type": key[0], "id": key[1], "original": key in expected, "issues": row["issues"]}
        for key, row in sorted(actual.items())
    ])


def managed_strict(probe, full, env, ledger):
    runner = MANAGED / "cli.mjs"
    source_pin = re.search(r"PINNED_VERSION = ['\"](\d+\.\d+\.\d+)['\"]", (MANAGED / "state.mjs").read_text())
    probe.expect(bool(source_pin), "Full source names a managed OpenSpec pin")
    expected = typed_rows(ledger["full"]["items"])
    for version in sorted({source_pin[1], "2.1.0"}):
        managed_env = dict(env)
        if version != source_pin[1]:
            managed_env["PROMETHEUS_OPENSPEC_VERSION"] = version
        command = ["node", runner, "run", "--project", full, "--timeout-ms", "220000", "--"]
        probe_args = dict(cwd=full, env=managed_env, timeout=240)
        actual = probe.success(f"managed-{version}-version", [*command, "--version"], **probe_args)
        probe.expect(actual.stdout.strip() == version, f"Managed OpenSpec selected {version}")
        package = (Path(env["HOME"]) / ".prometheus/openspec/versions" / version /
                   "node_modules/@fission-ai/openspec/package.json")
        if package.is_file():
            definition = load(package)
            entry = definition["bin"]
            entry = entry["openspec"] if isinstance(entry, dict) else entry
            real = package.parent / entry
            probe.observe(f"managed-{version}-executable", {"path": str(real), "sha256": sha(real)})
        else:
            raise Blocked(f"Managed CLI receipt for {version} has no inspectable package at {package}")
        code, result = strict_report(probe, f"managed-{version}-strict", command, full, managed_env)
        probe.expect(code == 0, f"Managed strict {version} exited zero")
        retained(probe, result, expected, f"managed-{version}")


def negative_strict(probe, cli, scratch, env, ledger):
    positives = typed_rows(ledger["full"]["items"])
    for mutation, key, matcher in (
        ("delta", ("change", "change-cowork-005-codex-toml-config"), r"no deltas found|at least one delta"),
        ("purpose", ("spec", "adversarial-review-gating"), r"purpose section is still a placeholder"),
    ):
        store = workspace_copy(probe, ROOT, scratch / mutation)
        if mutation == "delta":
            target = store / "openspec/changes" / key[1] / "specs"
            changed = scratch / "hidden-deltas"
            probe.expect(target.is_dir() and bool(list(target.rglob("spec.md"))), "Delta control removes existing requirements")
            target.rename(changed)
            restore = lambda: changed.rename(target)
        else:
            target = store / "openspec/specs" / key[1] / "spec.md"
            original = target.read_text()
            replaced, count = re.subn(
                r"(?ms)(^## Purpose\s*\n).*?(?=^## )",
                r"\1\nTBD - created by archiving change change-arc-001-ratify-goal-wording. Update Purpose after archive.\n\n",
                original,
            )
            probe.expect(count == 1 and original != replaced, "Purpose control reinstates one placeholder")
            target.write_text(replaced)
            restore = lambda: target.write_text(original)
        try:
            code, report = strict_report(probe, f"negative-{mutation}-strict", [cli], store, env)
            rows = typed_rows(report["items"])
            invalid = {item for item, row in rows.items() if not row["valid"] or row["issues"]}
            probe.expect(code != 0 and invalid == {key}, f"Strict rejects only the affected {identity(key)}")
            probe.expect(any(re.search(matcher, item["message"], re.I) for item in rows[key]["issues"]),
                         f"Strict reports the intentional {mutation} diagnostic")
        finally:
            restore()
        probe.expect(all((store / row["path"]).is_file() and sha(store / row["path"]) == row["sha256"]
                         for row in positives[key]["artifacts"]), f"Restore the {mutation} input byte-for-byte")

    store = workspace_copy(probe, ROOT, scratch / "omit-id")
    key = ("change", "change-cowork-005-codex-toml-config")
    target = store / "openspec/changes" / key[1]
    hidden = scratch / "hidden-identity"
    target.rename(hidden)
    try:
        code, report = strict_report(probe, "negative-omitted-id-strict", [cli], store, env)
        probe.expect(code == 0, "Strict alone accepts the deliberately incomplete store")
        try:
            check_inventory(report, positives)
        except Failed as error:
            probe.expect(str(error) == f"Missing baseline identity: {identity(key)}",
                         "Independent reconciliation rejects the exact omitted identity")
        else:
            probe.expect(False, "Independent reconciliation must reject an omitted identity")
    finally:
        hidden.rename(target)
    probe.expect(all(sha(store / row["path"]) == row["sha256"] for row in positives[key]["artifacts"]),
                 "Restore the removed identity byte-for-byte")


def verify_specs(probe, args, scratch, roots, inventory):
    raw = read_baselines(probe, scratch, inventory, ["fullStrict", "companionStrict"])
    ledger = check_locked_ledger(probe, roots, raw, inventory)
    cli = executable(os.environ.get("KBD_HEALTH_OPENSPEC_211", "openspec"))
    env = fixture_env(scratch / "validator-home", scratch / "validator-tmp")
    baseline_version(probe, cli, env, ROOT)
    if args.negative_only:
        negative_strict(probe, cli, scratch, env, ledger)
        return

    full = workspace_copy(probe, ROOT, scratch / "full")
    companion = workspace_copy(probe, roots["companion"], scratch / "companion")
    for name, store in (("full", full), ("companion", companion)):
        code, report = strict_report(probe, f"{name}-strict", [cli], store, env)
        probe.expect(code == 0, f"{name} direct strict validation exited zero")
        retained(probe, report, typed_rows(ledger[name]["items"]), name)

    managed_strict(probe, full, env, ledger)
    cli_dir = scratch / "selected-openspec"
    cli_dir.mkdir()
    (cli_dir / "openspec").symlink_to(cli.resolve())
    test_env = {
        **env, "PROMETHEUS_CLI_TEST_BINARY": str(args.pack_cli),
        "OPENSPEC_TEST_BIN_DIR": str(cli_dir),
    }
    applied = probe.success("actual-openspec-apply", [
        "node", "--test", "--test-reporter=tap", "--test-name-pattern=actual OpenSpec apply",
        "scripts/tests/kbd-phase-consistency.integration.test.mjs",
    ], cwd=ROOT, env=test_env, timeout=240)
    probe.expect(
        bool(re.search(r"(?m)^ok \d+ - actual OpenSpec apply retains remaining work", applied.stdout))
        and bool(re.search(r"(?m)^# pass 1$", applied.stdout))
        and not re.search(r"(?m)^ok \d+ - actual OpenSpec apply .*# SKIP", applied.stdout),
        "The named actual OpenSpec apply integration ran and passed without a skip",
    )
    probe.success("validate-codex-distribution", ["npm", "run", "validate:codex"], cwd=ROOT, env=env, timeout=300)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", required=True, choices=["GREEN-001", "GREEN-002"])
    parser.add_argument("--negative-only", action="store_true")
    for binary in ("pack-cli", "headless", "sync-cli"):
        parser.add_argument(f"--{binary}", type=Path, required=True)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    probe = Probe(args.report.resolve(), args.case, args.negative_only)
    outcome, reason = "PASS", None
    scratch = None
    try:
        companion = Path(os.environ.get("COMPANION_ROOT", str(ROOT.parent / "prometheus-companion"))).resolve()
        if not (companion / "Cargo.lock").is_file():
            raise Blocked(f"Registered Companion checkout is unavailable at {companion}")
        roots = {"full": ROOT, "companion": companion}
        checkouts = {name: checkout_manifest(root) for name, root in roots.items()}
        probe.observe("checkouts", checkouts)
        inputs = [Path(__file__).resolve(), Path(__file__).with_name("kbd_intersession_install_health.py")]
        probe.observe("verifierSources", [{"path": str(path), "sha256": sha(path)} for path in inputs])
        for field in ("pack_cli", "headless", "sync_cli"):
            path = executable(getattr(args, field).resolve())
            setattr(args, field, path)
            probe.observe(field, {"path": str(path), "sha256": sha(path)})
        with tempfile.TemporaryDirectory(prefix="kh-", dir="/tmp") as temp:
            scratch = Path(temp).resolve()
            inventory = load(BASELINE)
            probe.observe("failureInventory", {"path": str(BASELINE), "sha256": sha(BASELINE)})
            if args.case == "GREEN-001":
                read_baselines(probe, scratch, inventory, ["installedDoctor"])
                verify_installation(probe, args, scratch, roots, inventory)
            else:
                verify_specs(probe, args, scratch, roots, inventory)
    except Blocked as error:
        outcome, reason = "BLOCKED", str(error)
    except (Failed, ValueError, KeyError, TypeError, OSError, subprocess.SubprocessError) as error:
        outcome, reason = "FAIL", str(error)
    finally:
        probe.data["cleanup"]["scratchRemoved"] = scratch is None or not scratch.exists()
        if not probe.data["cleanup"]["scratchRemoved"]:
            outcome, reason = "FAIL", f"Scratch root still exists: {scratch}"
        probe.finish(outcome, reason)
    if reason:
        print(f"{args.case} {outcome}: {reason}", file=sys.stderr)
    return {"PASS": 0, "FAIL": 1, "BLOCKED": 2}[outcome]


if __name__ == "__main__":
    sys.exit(main())
