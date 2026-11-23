"""Exercise the strict installer using real staged v1.17.0 macOS release files.

Run: python3 scripts/tests/exec-release-pin.integration.py STAGE RELEASE_DIR
STAGE contains manifest.json and bin/. RELEASE_DIR contains the published
SHA256SUMS. Run on the matching architecture. On Apple Silicon, the alternate
pin cases use a temporary policy fixture and the actual certified ARM image.
"""
from copy import deepcopy
from functools import partial
from hashlib import sha256
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
from tempfile import TemporaryDirectory
from threading import Thread

ROOT = Path(__file__).resolve().parents[2]
VERSION = "1.17.0"
INTEL = "x86_64-apple-darwin"
ARM = "aarch64-apple-darwin"
INTEL_SOURCE = "fe2f2c5e66d68f5179609afded64650f5e7cd038787648b617991a324a84c95e"
INTEL_RELEASE = "00718e7e89f17909d6b47d1d3f5cf61aa68cc0e8a3a1219bdc9f73ee6bb12354"
ARM_SOURCE = "ca7ea8dec23cf21efffcbd5d86d4df1087714feb0ccd1700f2dfd7d2165aa065"
ARM_RELEASE = "cd8cb3d7bb806d6eff314f974fff0aec0c002ddca4d32e0b87275e29bec0c6f9"


def main(stage, release_dir, work):
    manifest = json.loads((stage / "manifest.json").read_bytes())
    assert manifest["version"] == VERSION
    target = {"darwin-arm64": ARM, "darwin-amd64": INTEL}[manifest["platform"]]
    exe = stage / "bin/prometheus-exec"
    build_hash = sha256(exe.read_bytes()).hexdigest()
    fixture = work / "checkout"
    for name in ["skill-system.json", "config/release-binaries.json", "config/prometheus-exec-binary.json",
                 "scripts/prometheus-exec-pin.sh", "scripts/binary-release.mjs", "scripts/install-prometheus-exec.sh"]:
        (fixture / name).parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(ROOT / name, fixture / name)
    policy_file = fixture / "config/prometheus-exec-binary.json"
    policy = json.loads(policy_file.read_bytes())
    assert policy["expectedBuildSha256ByTarget"][INTEL] == INTEL_SOURCE
    assert policy["expectedBuildSha256ByTarget"][ARM] == ARM_SOURCE
    assert policy["releaseBuildsByPackVersion"][VERSION][INTEL]["sha256"] == INTEL_RELEASE
    assert policy["releaseBuildsByPackVersion"][VERSION][ARM]["sha256"] == ARM_RELEASE
    source_pin = policy["expectedBuildSha256ByTarget"][target]
    hosted_pin = policy["releaseBuildsByPackVersion"][VERSION].get(target, {}).get("sha256")
    assert build_hash in (source_pin, hosted_pin)

    tag = manifest["tag"]
    mirror = work / "mirror"
    (mirror / tag).mkdir(parents=True)
    shutil.copyfile(release_dir / "SHA256SUMS", mirror / tag / "SHA256SUMS")
    with ThreadingHTTPServer(("127.0.0.1", 0), partial(SimpleHTTPRequestHandler, directory=mirror)) as server:
        Thread(target=server.serve_forever, daemon=True).start()
        home = work / "home"
        bindir = home / ".local/bin"
        installer = ["bash", str(fixture / "scripts/install-prometheus-exec.sh")]
        resolver = ["node", str(fixture / "scripts/binary-release.mjs")]
        env = {k: v for k, v in os.environ.items() if not k.startswith("PROMETHEUS_")}
        env.update(HOME=str(home), PROMETHEUS_BINARY_RELEASE_BASE_URL=f"http://127.0.0.1:{server.server_port}")

        def run(argv, extra=None, fail_with=None):
            result = subprocess.run(argv, cwd=fixture, env={**env, **(extra or {})}, text=True, capture_output=True)
            if fail_with:
                assert result.returncode != 0 and fail_with in result.stderr, result.stderr
            else:
                assert result.returncode == 0, result.stderr
            return result.stdout

        released = {"PROMETHEUS_EXEC_RELEASE_MANIFEST": str(stage / "manifest.json")}
        run(installer, released)  # As-shipped host policy.
        for name in ["prometheus", "pk", "pk-cherry", "prometheus-learning-worker", "surreal-memory-server"]:
            run([*resolver, "copy", bindir / name, stage / "bin" / name])
        doctor = [bindir / "prometheus", "doctor", "--check", "binaries.manifest", "--json"]
        receipt_path = home / ".prometheus/install/manifests/prometheus-exec.json"
        if build_hash == source_pin:
            run(installer, {"PROMETHEUS_EXEC_SOURCE_BIN": str(exe)})
            source_receipt = json.loads(receipt_path.read_bytes())
            assert source_receipt["buildSha256"] == source_pin and "releaseTag" not in source_receipt
            source_check, = json.loads(run(doctor))["checks"]
            assert source_check["status"] == "pass" and any(
                row.startswith("prometheus-exec: ") and "source sha256" in row for row in source_check["details"]
            ), source_check
            # Keep the delivered config intact. In an isolated checkout exercise
            # the same two-pin decision with the native image and observed digests.
            policy["expectedBuildSha256ByTarget"][target] = INTEL_SOURCE if target == ARM else ARM_SOURCE
            policy["releaseBuildsByPackVersion"][VERSION][target] = {"sha256": build_hash}
            policy_file.write_text(json.dumps(policy, indent=2))
            run(installer, released)

        installed = bindir / "prometheus-exec"
        digest = sha256(installed.read_bytes()).hexdigest()
        receipt = json.loads(receipt_path.read_bytes())
        assert receipt["expectedBuildSha256"] == receipt["buildSha256"] == build_hash
        assert receipt["installedSha256"] == digest and receipt["releaseTag"] == tag
        assert receipt["releaseManifestSha256"] == sha256((stage / "manifest.json").read_bytes()).hexdigest()

        check = json.loads(run(doctor))
        result, = check["checks"]
        assert result["status"] == "pass" and any(
            f"prometheus-exec: installed sha256 {digest}; GitHub release {tag}" in row for row in result["details"]
        ), result

        run(installer, {"PROMETHEUS_EXEC_SOURCE_BIN": str(exe)}, "source hash mismatch")
        for version, other_target in [("1.17.1", target), (VERSION, INTEL if target == ARM else ARM)]:
            scoped = deepcopy(policy)
            scoped["releaseBuildsByPackVersion"] = {version: {other_target: {"sha256": build_hash}}}
            policy_file.write_text(json.dumps(scoped, indent=2))
            run(installer, released, "violates the independent prometheus-exec pin")
        assert sha256(installed.read_bytes()).hexdigest() == digest
        print(f"{manifest['platform']} passed release install, receipt/doctor, source rejection, "
              f"wrong version/target rejection and preservation of the installed image.")
        server.shutdown()


if __name__ == "__main__":
    stage, release_dir = map(lambda arg: Path(arg).resolve(), sys.argv[1:])
    with TemporaryDirectory(prefix="exec-release-pin-") as tmp:
        main(stage, release_dir, Path(tmp))
