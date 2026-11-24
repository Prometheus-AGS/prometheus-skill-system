"""Isolated production installer and signed journal probes for GREEN-001."""

from __future__ import annotations

import json
import os
from pathlib import Path
import platform
import plistlib
import pwd
import re
import shutil
import socket
import stat
import uuid


HARNESS_COMMANDS = {
    "claude-code": "claude",
    "codex": "codex",
    "opencode": "opencode",
    "kimi": "kimi",
}
HARNESS_CONFIGS = {
    "claude-code": ".claude/settings.json",
    "codex": ".codex/config.toml",
    "opencode": ".config/opencode/opencode.json",
    "kimi": ".kimi-code/config.toml",
}
NATIVE_SERVICES = (
    "surrealdb", "openai-proxy", "liter-llm", "surreal-memory", "pk-cherry",
    "forge-mcp", "surface-bridge", "prometheus-research", "prometheus-exec", "learning-worker",
)
FULL_REQUIRED = {
    "binaries.manifest", "execution.runtime", "services.launch-agents", "learning.worker",
    "learning.snapshots", "hooks.rotation", "learning.surreal-memory",
}
COMMON_REQUIRED = {
    "skills.directory", "skills.installed-agents", "hooks.harness-adapters",
    "hooks.lifecycle", "state.kbd-orchestrator",
}

GENERATE_KEY = r"""
const fs = require('node:fs');
const {generateKeyPairSync, createHash} = require('node:crypto');
const {publicKey, privateKey} = generateKeyPairSync('ed25519');
const publicBytes = publicKey.export({format: 'der', type: 'spki'}).subarray(-32);
const seed = privateKey.export({format: 'der', type: 'pkcs8'}).subarray(-32);
const keyId = 'ed25519:' + createHash('sha256').update(publicBytes).digest('hex');
fs.writeFileSync(process.argv[1], JSON.stringify({
  schemaVersion: '1', keyId, privateKey: seed.toString('base64'),
}), {mode: 0o600, flag: 'wx'});
console.log(JSON.stringify({keyId, publicKey: publicBytes.toString('base64')}));
"""

VERIFY_SIGNATURE = r"""
const fs = require('node:fs');
const {createHash, createPublicKey, verify} = require('node:crypto');
const {events, signer} = JSON.parse(fs.readFileSync(0, 'utf8'));
function canonical(value) {
  if (Array.isArray(value)) return '[' + value.map(canonical).join(',') + ']';
  if (value === null || typeof value !== 'object') return JSON.stringify(value);
  return '{' + Object.keys(value).sort().map(key =>
    JSON.stringify(key) + ':' + canonical(value[key])).join(',') + '}';
}
const key = createPublicKey({
  key: Buffer.concat([Buffer.from('302a300506032b6570032100', 'hex'), Buffer.from(signer.publicKey, 'base64')]),
  type: 'spki', format: 'der',
});
let previous = null;
const checked = events.map(event => {
  const {signature, ...unsigned} = event;
  unsigned.integrityHash = '';
  if (event.schemaVersion !== '2' || event.signerKeyId !== signer.keyId ||
      event.signerPublicKey !== signer.publicKey || event.previousHash !== previous || !signature)
    throw Error('Audit event identity, signature fields or parent hash disagree: ' + event.eventId);
  const bytes = Buffer.from(canonical(unsigned));
  const hash = createHash('sha256').update(bytes).digest('hex');
  if (hash !== event.integrityHash ||
      !verify(null, bytes, key, Buffer.from(signature, 'base64')))
    throw Error('Invalid audit event integrity or signature: ' + event.eventId);
  previous = hash;
  return {eventId: event.eventId, commandId: event.commandId, integrityHash: hash, revision: event.revision};
});
console.log(JSON.stringify(checked));
"""


def run_checkout(probe, origin: Path, clone: Path, env):
    """Copy frozen tracked and in-scope dirty sources to a private Git checkout."""
    before = probe.data["observations"]["checkouts"]["full"]
    result = probe.command("clone-install-source", [
        "git", "clone", "--quiet", "--shared", "--no-checkout", "--", origin, clone,
    ], cwd=clone.parent, env=env, timeout=300)
    if result.returncode != 0:
        raise probe.blocked("Could not prepare a local disposable installer checkout")
    probe.success("checkout-install-source", [
        "git", "-c", "advice.detachedHead=false", "checkout", "--quiet", "--detach", before["head"],
    ], cwd=clone, env=env, timeout=300)
    for name, properties in before["dirty"].items():
        path = Path(name)
        if path.parts[0] in {".git", ".kbd-orchestrator", ".prometheus"}:
            continue
        src, dst = origin / path, clone / path
        if properties.get("directory"):
            continue
        if dst.is_symlink() or dst.is_file():
            dst.unlink()
        if "link" in properties:
            dst.parent.mkdir(parents=True, exist_ok=True)
            dst.symlink_to(properties["link"])
        elif "sha256" in properties:
            probe.expect(probe.sha(src) == properties["sha256"], f"Frozen installer input agrees: {name}")
            dst.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(src, dst)
    return clone


def copy_executable(probe, source: Path, destination: Path):
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, destination)
    expected = probe.sha(source)
    probe.expect(os.access(destination, os.X_OK) and probe.sha(destination) == expected,
                 f"Isolated installed executable matches supplied candidate: {destination.name}")


def phase_identity(probe, cli, work, env, label):
    probe.success(f"{label}-init-consumer", ["git", "init", "--quiet"], cwd=work, env=env)
    before = probe.success(f"{label}-contract", [cli, "contract", "show", "--json"], cwd=work, env=env)
    contract = probe.json_result(before, f"{label} contract")
    probe.expect(contract.get("endpoint") is None and contract.get("endpoint_source") == "absent",
                 f"{label} consumer has no Companion control endpoint")
    key_path = Path(env["HOME"]) / ".private-signing-key.json"
    env["PROMETHEUS_DEVICE_KEY_FILE"] = str(key_path)
    made = probe.success(f"{label}-fixture-public-key", ["node", "-e", GENERATE_KEY, key_path], cwd=work, env=env)
    signer = probe.json_result(made, f"{label} public signer identity")
    probe.success(f"{label}-register-consumer", [cli, "kbd", "register", "."], cwd=work, env=env)
    nonce = uuid.uuid4().hex
    command = f"green-health-{nonce}"
    phase = f"green-{nonce}"
    result = probe.success(f"{label}-phase-create", [
        cli, "kbd", "phase", "create", "--id", phase, "--command-id", command,
        "--title", "Local signed health verification",
    ], cwd=work, env=env)
    if label == "skills":
        created = probe.json_result(result, f"{label} phase creation")
        probe.expect(created.get("committedLocally") is True, "Standalone command committed locally")
    fresh = probe.success(f"{label}-fresh-status", [cli, "kbd", "status", "--json"], cwd=work, env=env)
    status = probe.json_result(fresh, f"{label} status")
    revision = status["revision"]
    probe.expect(isinstance(revision, int) and revision > 0 and status["commandRevisions"].get(command) == revision,
                 f"{label} fresh status contains the exact command at authoritative revision")
    registry = probe.success(f"{label}-private-project-registry", [
        cli, "kbd", "projects", "--json",
    ], cwd=work, env=env)
    replicas = probe.json_result(registry, f"{label} registry")["replicas"]
    probe.expect(set(replicas) == {str(work)} and replicas[str(work)]["projectId"] == status["projectId"],
                 f"{label} private registry contains exactly the signed fixture project")
    audit = probe.success(f"{label}-fresh-audit", [cli, "kbd", "audit", "--json"], cwd=work, env=env)
    events = probe.json_result(audit, f"{label} audit")
    matches = [event for event in events if event.get("commandId") == command]
    probe.expect(len(matches) == 1 and matches[0]["projectId"] == status["projectId"]
                 and matches[0]["eventId"] == status["lastEventId"]
                 and matches[0]["integrityHash"] == status["lastEventHash"],
                 f"{label} audit and status agree on a single signed command")
    signed = probe.success(f"{label}-verify-ed25519", ["node", "-e", VERIFY_SIGNATURE], cwd=work, env=env,
                           stdin=json.dumps({"events": events, "signer": signer}))
    checked = probe.json_result(signed, f"{label} signatures")
    probe.expect(len(checked) == len(events) and checked[-1]["commandId"] == command,
                 f"{label} verified every signed audit event against the disposable public key")
    probe.observe(f"{label}SignedReadback", {
        "projectId": status["projectId"], "revision": revision, "commandId": command,
        "eventId": matches[0]["eventId"], "frontier": status["frontier"], "signer": signer,
        "verifiedEvents": checked,
    })
    return revision


def install_generation(probe, profile, source, home, env, options):
    base = [source / "install.sh", "--profile", profile, "--targets", "agents", "--non-interactive", *options]
    probe.success(f"{profile}-install", [*base, "--yes"], cwd=source, env=env, timeout=3600)
    probe.success(f"{profile}-install-verify", [*base, "--verify"], cwd=source, env=env, timeout=600)
    store = home / ".prometheus/plugins/prometheus-skill-pack"
    active = (store / "current").resolve(strict=True)
    manifest_path = active / "manifest.json"
    manifest = probe.load(manifest_path)
    generation = manifest["generation"]
    probe.expect(active.parent == store / "generations" and active.name == generation,
                 f"{profile} activated real content-addressed generation")
    selection_file = store / "receipts" / generation / "install-profiles" / f"{profile}-agents.json"
    selection = probe.load(selection_file)
    probe.expect(selection["generation"] == generation and selection["profile"] == profile
                 and selection["targets"] == ["agents"] and Path(selection["sourceRoot"]) == source,
                 f"{profile} installer recorded matching profile, source and target")
    payload = next(item for item in manifest["targetPayloads"] if item["target"] == ".agents/skills")
    index = active / "indexes/skills.json"
    index_data = probe.load(index)
    entries = index_data["entries"]
    installed = home / ".agents/skills"
    expected = {entry["id"] for entry in entries}
    actual = {path.parent.name for path in installed.glob("*/SKILL.md") if path.is_file()}
    probe.expect(len(expected) == payload["skillCount"] >= 1 and expected <= actual,
                 f"{profile} consumer installation contains every manifest-listed skill")
    probe.expect(index_data["sha256"] == manifest["skillIndex"]["sha256"]
                 and manifest["skillIndex"]["entryCount"] == len(expected),
                 f"{profile} generation records its catalog digest")
    release_manifest = active / "shared/harnesses/generated/release-manifest.json"
    probe.observe(f"{profile}InstalledGeneration", {
        "selection": selection, "generation": generation, "bundleId": manifest["bundleId"],
        "sourceVersion": manifest["sourceVersion"], "manifestSha256": probe.sha(manifest_path),
        "releaseManifestSha256": probe.sha(release_manifest), "indexSha256": probe.sha(index),
        "installedCatalogCount": len(actual), "requiredCatalogCount": len(expected),
    })
    return base, selection


def discovery(probe, source, home, cli, env, profile, selection):
    configured = set(selection["harnesses"])
    configured.update(harness for harness, relative in HARNESS_CONFIGS.items() if (home / relative).is_file())
    receipt_dir = home / ".prometheus/skill-activation/harness-budgets"
    receipts = {}
    probe.observe(f"{profile}Discovery", {"configured": sorted(configured), "receipts": receipts})
    for harness in sorted(configured):
        executable = HARNESS_COMMANDS[harness]
        if not shutil.which(executable, path=env["PATH"]):
            raise probe.blocked(f"{profile} selects {harness}, but native {executable} is not available")
        measured = probe.command(f"{profile}-discovery-{harness}", [
            "python3", source / "scripts/certification/measure-discovery.py", "--harness", harness,
            "--prometheus", cli, "--receipt-dir", receipt_dir,
        ], cwd=source, env=env, timeout=900)
        if measured.returncode != 0 or not (receipt_dir / f"{harness}.json").is_file():
            raise probe.blocked(f"{profile} has no genuine discovery capture for configured {harness}")
        path = receipt_dir / f"{harness}.json"
        receipt = probe.load(path)
        probe.expect(receipt["harness"] == harness and receipt["inventorySkills"] > 0
                     and bool(receipt["samples"]) and all(sample["promptSha256"] for sample in receipt["samples"]),
                     f"{profile} discovery capture includes native samples for {harness}")
        receipts[harness] = {"sha256": probe.sha(path), "capture": receipt}
    return receipt_dir, configured


def doctor(probe, cli, work, env, name):
    result = probe.command(name, [cli, "doctor", "--json"], cwd=work, env=env, timeout=240)
    report = probe.json_result(result, name)
    rows = {check["id"]: check for check in report["checks"]}
    probe.expect(rows and len(rows) == len(report["checks"]) and report["mode"] == "diagnose"
                 and report["selection"] == {"check": None, "excluded": []},
                 f"{name} produced an unfiltered, unique and nonempty check inventory")
    return result.returncode, rows, report["summary"]


def reconcile_doctor(probe, observation, baseline, profile, selection, configured, name):
    code, rows, summary = observation
    original = {row["id"]: row for row in baseline}
    probe.expect(len(original) == 24 and set(original) <= rows.keys(),
                 f"{name} retains every recorded doctor ID")
    dispositions = {}
    probe.observe(name + "Reconciliation", dispositions)
    for key, row in sorted(rows.items()):
        dispositions[key] = {
            "prior": original.get(key, {}).get("status"),
            "status": row["status"], "optional": row["optional"],
            "reason": row["summary"], "details": row["details"],
        }
    probe.expect(any(row["status"] == "pass" and not row["optional"] for row in rows.values()),
                 f"{name} exercised applicable mandatory checks")
    required = COMMON_REQUIRED | (FULL_REQUIRED if profile == "full" else set())
    if configured:
        required = required | {"skills.discovery-budget"}
    probe.expect(required <= rows.keys(), f"{name} includes required profile checks")
    for key in sorted(required):
        probe.expect(rows[key]["status"] == "pass", f"{name} required check passed: {key}")
    for key, row in sorted(rows.items()):
        if row["status"] == "skip":
            probe.expect(row["optional"] is True and bool(row["summary"]), f"{name} optional skip has a reason: {key}")
            if key in FULL_REQUIRED:
                probe.expect(profile == "skills" and selection["services"] == []
                             and selection["generation"] in " ".join(row["details"]),
                             f"{name} service exclusion follows actual installer selection: {key}")
    probe.expect(code == 0 and summary["failed"] == summary["warned"] == 0
                 and all(row["status"] in {"pass", "skip"} for row in rows.values()),
                 f"{name} confirmed zero failures, zero unresolved warnings and no required skips")


def isolated_gui(probe, work, env):
    record_path = os.environ.get("KBD_HEALTH_DISPOSABLE_GUI")
    if platform.system() != "Darwin" or not record_path:
        raise probe.blocked("Full install and supervision require a separately provisioned disposable macOS GUI account or VM (KBD_HEALTH_DISPOSABLE_GUI)")
    path = Path(record_path)
    if not path.is_file() or path.is_symlink():
        raise probe.blocked("KBD_HEALTH_DISPOSABLE_GUI must name a local disposable GUI fixture record")
    record = probe.load(path)
    uid = os.getuid()
    if uid == 0 or path.stat().st_uid != uid or path.stat().st_mode & (stat.S_IRWXG | stat.S_IRWXO):
        raise probe.blocked("Disposable GUI fixture record must be owned by the executing non-root login and private")
    account = pwd.getpwuid(uid)
    mode = record.get("kind")
    if record.get("guiUid") != uid or record.get("hostName") != socket.gethostname() or mode not in {"separate-user", "disposable-vm"}:
        raise probe.blocked("Current login identity and host do not match the disposable GUI fixture record")
    if mode == "separate-user":
        operator_uid = record.get("operatorUid")
        if not isinstance(operator_uid, int) or operator_uid in {0, uid}:
            raise probe.blocked("Disposable GUI account must differ from the recorded operator UID")
        if any(os.access(candidate, os.W_OK) for candidate in ("/usr/local/bin", "/opt/homebrew/bin", "/usr/local/sbin")):
            raise probe.blocked("Current account can write shared binaries; use a disposable macOS VM")
    else:
        model = probe.success("disposable-vm-model", [
            "/usr/sbin/sysctl", "-n", "hw.model",
        ], cwd=work, env=env).stdout.strip()
        if not model.startswith("VirtualMac"):
            raise probe.blocked("Native GUI VM route requires macOS VirtualMac hardware")
    domain = f"gui/{uid}"
    active = probe.command("disposable-gui-domain", ["/bin/launchctl", "print", domain], cwd=work, env=env)
    if active.returncode != 0:
        raise probe.blocked(f"The disposable login {domain} has no active GUI launchd domain")
    before = parse_gui_inventory(probe, active.stdout)
    existing_home = Path(account.pw_dir)
    account_state = (".agents/skills", ".codex/config.toml", ".prometheus", ".claude/settings.json")
    if any((existing_home / relative).exists() for relative in account_state) or before:
        raise probe.blocked(f"The disposable GUI account already has agent or Prometheus state: {domain}")
    probe.observe("disposableGui", {"uid": uid, "kind": mode, "hostName": record["hostName"], "domain": domain})
    return domain, before


def is_fixture_label(name):
    return re.fullmatch(r"(?:ai|com)\.prometheus\.[\w.-]+|cpc007\.fixture\.\d+", name) is not None


def parse_gui_inventory(probe, output):
    services = re.search(r"(?ms)^[ \t]*services = \{\n(.*?)^[ \t]*\}", output)
    probe.expect(services is not None, "launchctl returned an inspectable GUI service inventory")
    return {row.split()[-1] for row in services[1].splitlines()
            if row.split() and is_fixture_label(row.split()[-1])}


def loaded_gui_labels(probe, domain, work, env, name):
    result = probe.success(name, ["/bin/launchctl", "print", domain], cwd=work, env=env)
    return parse_gui_inventory(probe, result.stdout)


def service_ports():
    """Select distinct currently free loopback ports for real native listeners."""
    listeners = [socket.socket() for _ in NATIVE_SERVICES[:-2]]
    try:
        for listener in listeners:
            listener.bind(("127.0.0.1", 0))
        return {service: listener.getsockname()[1] for service, listener in zip(NATIVE_SERVICES, listeners)}
    finally:
        for listener in listeners:
            listener.close()


def full_options(env, home):
    ports = service_ports()
    options = [value for service in NATIVE_SERVICES for value in ("--service", f"{service}=native")]
    options += [value for service, port in ports.items() for value in ("--port", f"{service}={port}")]
    env.update({
        "SURREAL_MEMORY_URL": f"http://127.0.0.1:{ports['surreal-memory']}",
        "LITER_LLM_BASE_URL": f"http://127.0.0.1:{ports['liter-llm']}/v1",
        "PROMETHEUS_EXEC_SOCKET": str(home / ".prometheus/run/e.sock"),
    })
    return options, ports


def full_diagnostics(probe, source, home, domain, env, selection, ports):
    state_file = home / ".prometheus/service-providers/full.json"
    state = probe.load(state_file)["services"]
    probe.expect(set(selection["services"]) == set(NATIVE_SERVICES)
                 and all(state.get(service, {}).get("provider") == "native"
                         and state[service]["owner"] == "pack" and state[service]["phase"] == "ready"
                         for service in NATIVE_SERVICES)
                 and all(state[service]["endpoint"] == f"http://127.0.0.1:{port}"
                         for service, port in ports.items()),
                 "Full installer selected all native service collaborators")
    status = probe.success("installed-full-service-status", [
        "node", source / "scripts/service-providers/cli.mjs", "status", "--home", home, "--json",
    ], cwd=source, env=env, timeout=300)
    services = probe.json_result(status, "native service status")["services"]
    probe.expect({row["id"] for row in services if row["healthy"] is True} == set(NATIVE_SERVICES),
                 "Installed service provider probed every selected native service successfully")
    installed = home / "Library/LaunchAgents"
    records = {}
    for file in sorted(installed.glob("*.plist")):
        with file.open("rb") as data:
            definition = plistlib.load(data)
        name = definition["Label"]
        if not is_fixture_label(name):
            continue
        check = probe.success(f"launchd-{name}", [
            "/bin/launchctl", "print", f"{domain}/{name}",
        ], cwd=source, env=env)
        probe.expect(str(file) in check.stdout, f"Owned launchd job resolves to fixture plist: {name}")
        command = definition.get("ProgramArguments") or [definition.get("Program")]
        records[name] = {"plistSha256": probe.sha(file), "program": command[0], "arguments": command[1:]}
    probe.expect(bool(records), "Full install exercised owned launchd registrations")
    for label in ("ai.prometheus.hooks-logrotate", "ai.prometheus.prometheus-nudge"):
        if label in records:
            probe.expect(records[label]["program"] == "/bin/bash",
                         f"Installed maintenance agent declares the tested Bash interpreter: {label}")
    probe.observe("fullNativeRegistrations", records)


def ensure_cargo_idle(probe, work, env, name):
    active = probe.command(name, ["pgrep", "-x", "cargo|rustc"], cwd=work, env=env)
    if active.returncode == 0:
        raise probe.blocked("Another Cargo or rustc process owns the local build slot")
    probe.expect(active.returncode == 1, "Local Cargo process inspection succeeded")


def regressions(probe, args, roots, home, cli, env):
    legacy_daemon = home / ".local/bin/sovereign-sync"
    if not legacy_daemon.is_file():
        legacy_path = os.environ.get("KBD_HEALTH_LEGACY_PACK_DAEMON")
        if not legacy_path:
            raise probe.blocked("Continuity requires the independent legacy Full pack daemon (KBD_HEALTH_LEGACY_PACK_DAEMON)")
        copy_executable(probe, probe.executable(legacy_path), legacy_daemon)
    daemon_digest = probe.sha(legacy_daemon)
    probe.expect(daemon_digest not in {probe.sha(args.sync_cli), probe.sha(args.headless)},
                 "Continuity uses the legacy Full daemon independently of the Companion candidate")
    probe.observe("continuityLegacyDaemon", {"path": str(legacy_daemon), "sha256": daemon_digest})
    lock_digest = probe.sha(roots["companion"] / "Cargo.lock")
    ensure_cargo_idle(probe, roots["companion"], env, "continuity-cargo-slot")
    tests = probe.command("companion-continuity-supervision", [
        "cargo", "test", "--locked", "-p", "prometheus-substrate", "--no-default-features",
        "--features", "headless", "--test", "continuity", "--test", "supervision", "--", "--nocapture",
    ], cwd=roots["companion"], env={**env, "PROMETHEUS_CLI_TEST_BINARY": str(cli)}, timeout=3600)
    output = tests.stdout + tests.stderr
    if re.search(r"(?im)^\s*SKIP:", output):
        raise probe.blocked("A mandatory Companion continuity or real launchd supervision scenario skipped")
    probe.expect(tests.returncode == 0 and
                 "test companion_node_reads_and_extends_state_written_by_the_pack_daemon ... ok" in output and
                 "test a_killed_adopted_service_is_restarted_and_reported_healthy_over_the_rest_surface ... ok" in output,
                 "Continuity and the real launchd restart scenario each executed and passed")
    probe.expect(probe.sha(roots["companion"] / "Cargo.lock") == lock_digest, "Locked Companion tests preserved Cargo.lock")


def negative_binary(probe, work, home, cli, env, baseline, selection, configured):
    required = home / ".local/bin/pk"
    if not required.is_file():
        raise probe.blocked("Disposable full install has no managed pk executable for the missing-binary control")
    digest = probe.sha(required)
    backup = required.with_name(".fixture-hidden-pk")
    required.rename(backup)
    probe.observe("missingBinaryInput", {"id": required.name, "sha256": digest})
    try:
        code, rows, _ = doctor(probe, cli, work, env, "negative-missing-required-binary")
        affected = rows["binaries.manifest"]
        probe.expect(code != 0 and affected["status"] == "fail"
                     and any("pk is missing from" in text for text in affected["details"]),
                     "Unfiltered doctor rejected the exact removed managed executable")
    finally:
        backup.rename(required)
    probe.expect(probe.sha(required) == digest, "Restore the removed managed binary byte-for-byte")
    reconcile_doctor(probe, doctor(probe, cli, work, env, "restored-missing-binary"),
                     baseline, "full", selection, configured, "restored-missing-binary")


def negative_discovery(probe, work, cli, env, baseline, selection, discovery_info):
    receipt_dir, configured = discovery_info
    if not configured:
        raise probe.blocked("The discovery negative control requires at least one configured native harness")
    harness = sorted(configured)[0]
    receipt = receipt_dir / f"{harness}.json"
    backup = receipt.with_suffix(".hidden")
    digest = probe.sha(receipt)
    receipt.rename(backup)
    probe.observe("withheldDiscoveryInput", {"harness": harness, "sha256": digest})
    try:
        _, rows, summary = doctor(probe, cli, work, env, "negative-withheld-discovery")
        affected = rows["skills.discovery-budget"]
        other_defects = {name for name, row in rows.items() if row["status"] in {"fail", "warn"}}
        probe.expect(affected["status"] == "warn" and summary["failed"] == 0
                     and other_defects == {"skills.discovery-budget"}
                     and any(harness in text for text in affected["details"]),
                     "Discovery gate refused green for exactly the withheld native harness receipt")
    finally:
        backup.rename(receipt)
    probe.expect(probe.sha(receipt) == digest, "Restore native discovery evidence byte-for-byte")
    reconcile_doctor(probe, doctor(probe, cli, work, env, "restored-discovery"),
                     baseline, "full", selection, configured, "restored-discovery")


def verify_installation(probe, args, scratch, roots, inventory):
    baseline = inventory["installedDoctor"]["checks"]
    skills = scratch / "skills"
    work, home = skills / "consumer", skills / "home"
    work.mkdir(parents=True)
    env = probe.fixture_env(home, skills / "tmp")
    source = run_checkout(probe, roots["full"], skills / "source", env)
    cli = home / ".local/bin/prometheus"
    copy_executable(probe, args.pack_cli, cli)
    probe.observe("skillsCliProvenance", {"path": str(cli), "sha256": probe.sha(cli),
                                         "source": str(args.pack_cli), "method": "isolated candidate binary placement"})
    probe.success("skills-installed-cli-version", [cli, "--version"], cwd=work, env=env)
    env["PROMETHEUS_EXEC_SOCKET"] = str(skills / "absent-execution.sock")
    _, selection = install_generation(probe, "skills", source, home, env, [])
    probe.expect(selection["services"] == [], "Standalone installer selected no native services")
    revision = phase_identity(probe, cli, work, env, "skills")
    doctor_path = home / ".agents/skills/kbd-doctor/scripts/kbd-doctor.sh"
    probe.expect(doctor_path.is_file(), "Installed KBD doctor launcher exists in the selected agent tree")
    inspected = probe.success("installed-kbd-doctor", [
        "bash", doctor_path, "--check", "state.kbd-orchestrator", "--json",
    ], cwd=work, env=env)
    reported = probe.json_result(inspected, "installed KBD doctor")["checks"]
    probe.expect(len(reported) == 1 and reported[0]["status"] == "pass"
                 and re.search(rf"\brevision {revision}\b", reported[0]["summary"]),
                 "Installed KBD doctor confirmed the signed standalone revision")
    _, configured = discovery(probe, source, home, cli, env, "skills", selection)
    reconcile_doctor(probe, doctor(probe, cli, work, env, "skills-unfiltered-doctor"),
                     baseline, "skills", selection, configured, "skills-unfiltered-doctor")

    domain, before = isolated_gui(probe, work, env)
    controls = ("missing-binary", "withheld-discovery") if args.negative_only else ("positive",)
    for control in controls:
        verify_full(probe, args, scratch / control, roots, baseline, domain, before, control)


def installed_cli_receipt(probe, home, source, cli):
    matches = []
    receipts = home / ".prometheus/install/releases"
    for path in receipts.glob("*/*/installed/prometheus/*.json"):
        record = probe.load(path)
        if record.get("destination") == str(cli):
            probe.expect(record["installedSha256"] == probe.sha(cli),
                         "Full installed CLI matches its binary release receipt")
            matches.append({"receiptSha256": probe.sha(path), **record})
    if matches:
        return {"method": "versioned binary release", "receipts": matches}
    root = source / "tools/prometheus-cli/target/release"
    for built in (root / "prometheus", source / "target/release/prometheus"):
        if built.is_file():
            return {"method": "installer source build", "sourceBinary": str(built),
                    "sourceBinarySha256": probe.sha(built)}
    raise probe.failed("Full installer provided no binary release or source-build provenance for its CLI")


def verify_full(probe, args, full, roots, baseline, domain, before, control):
    work, home = full / "consumer", full / "home"
    work.mkdir(parents=True)
    env = probe.fixture_env(home, full / "tmp")
    env["PROMETHEUS_USER"] = pwd.getpwuid(os.getuid()).pw_name
    source = run_checkout(probe, roots["full"], full / "source", env)
    prerequisites = probe.command("full-existing-native-prerequisites", [
        "bash", source / "scripts/check-prerequisites.sh", "--core-only", "--skip-docker",
    ], cwd=source, env=env)
    if prerequisites.returncode != 0:
        raise probe.blocked("Disposable full GUI needs preinstalled core native prerequisites")
    options, ports = full_options(env, home)
    probe.observe(f"{control}FullServicePorts", ports)
    ensure_cargo_idle(probe, source, env, f"{control}-full-install-cargo-slot")
    base = [source / "install.sh", "--profile", "full", "--targets", "agents", "--non-interactive", *options]
    try:
        _, selection = install_generation(probe, "full", source, home, env, options)
        cli = home / ".local/bin/prometheus"
        probe.expect(cli.is_file() and os.access(cli, os.X_OK), "Full installer supplied the actual CLI executable")
        probe.observe("fullCliProvenance", {"path": str(cli), "sha256": probe.sha(cli),
                                            "releaseGeneration": selection["generation"],
                                            **installed_cli_receipt(probe, home, source, cli)})
        probe.success("full-installed-cli-version", [cli, "--version"], cwd=work, env=env)
        phase_identity(probe, cli, work, env, "full")
        discovery_info = discovery(probe, source, home, cli, env, "full", selection)
        reconcile_doctor(probe, doctor(probe, cli, work, env, "full-unfiltered-doctor"),
                         baseline, "full", selection, discovery_info[1], "full-unfiltered-doctor")
        full_diagnostics(probe, source, home, domain, env, selection, ports)
        if control == "missing-binary":
            negative_binary(probe, work, home, cli, env, baseline, selection, discovery_info[1])
        elif control == "withheld-discovery":
            negative_discovery(probe, work, cli, env, baseline, selection, discovery_info)
        else:
            regressions(probe, args, roots, home, cli, env)
    finally:
        probe.command("full-fixture-uninstall", [*base, "--uninstall", "--yes"], cwd=source, env=env, timeout=600)
        remaining = loaded_gui_labels(probe, domain, source, env, "full-fixture-cleanup-inventory")
        for label in sorted(remaining - before):
            registration = probe.command(f"inspect-cleanup-service-{label}", [
                "/bin/launchctl", "print", f"{domain}/{label}",
            ], cwd=source, env=env)
            if registration.returncode != 0:
                continue
            plist_line = re.search(r"(?m)^\s*path = (.+)$", registration.stdout)
            probe.expect(plist_line is not None and
                         Path(plist_line[1]).resolve().is_relative_to(full.resolve()),
                         f"Remaining launchd registration is owned by this fixture: {label}")
            probe.success(f"remove-owned-service-{label}", [
                "/bin/launchctl", "bootout", f"{domain}/{label}",
            ], cwd=source, env=env)
        after = loaded_gui_labels(probe, domain, source, env, "full-fixture-post-cleanup-inventory")
        probe.expect(after == before, "Disposable native managed job inventory is restored")
        probe.data["cleanup"]["guiManagedBefore"] = sorted(before)
        probe.data["cleanup"]["guiManagedAfter"] = sorted(after)
