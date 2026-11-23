"""Produce native Full-pack binaries and release archives. Never executes a built tool."""
import argparse
import gzip
import hashlib
import io
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import stat
import struct
import subprocess
import tarfile
import tempfile
import urllib.request

ROOT = Path(__file__).resolve().parents[2]


def config(name):
    return json.loads((ROOT / name).read_text())


CATALOG = config("config/release-binaries.json")
PLATFORMS = CATALOG["platforms"]
VERSION = config("skill-system.json")["releaseVersion"]
TAG = f"v{VERSION}"


def require(ok, message):
    if not ok:
        raise SystemExit(message)


def git(*args):
    return subprocess.check_output(["git", *args], cwd=ROOT, text=True).strip()


def components(platform):
    return [c for c in CATALOG["components"] if platform in c.get("platforms", PLATFORMS)]


def groups(platform):
    return sorted({c["group"] for c in components(platform)})


def digest(path):
    with path.open("rb") as stream:
        h = hashlib.sha256()
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(chunk)
        return h.hexdigest()


def basename(platform):
    return f"prometheus-binaries-{TAG}-{platform}"


def exec_release_record(platform):
    manifest = config("config/prometheus-exec-binary.json")
    target = PLATFORMS[platform]["target"]
    record = manifest.get("releaseBuildsByPackVersion", {}).get(VERSION, {}).get(target)
    if record:
        paths = [".cargo", "crates/prometheus-exec", "scripts/build-prometheus-exec.sh",
                 "scripts/prometheus-exec-pin.sh", "wit"]
        paths.extend(p.relative_to(ROOT).as_posix() for p in (ROOT / "substrate").glob("exec-*") if p.is_dir())
        tree = subprocess.check_output(["git", "ls-tree", "-z", "HEAD", "--", *paths], cwd=ROOT)
        require(hashlib.sha256(tree).hexdigest() == record["sourceInputsSha256"],
                f"Release exec build inputs changed for {TAG} {target}")
    return record


def pin(platform):
    if record := exec_release_record(platform):
        return record["sha256"]
    return config("config/prometheus-exec-binary.json")["expectedBuildSha256ByTarget"].get(PLATFORMS[platform]["target"])


def cpu_check(path, platform):
    os_name, arch = platform.split("-")
    with path.open("rb") as stream:
        header = stream.read(64)
        if os_name == "windows" and header[:2] == b"MZ" and len(header) == 64:
            stream.seek(struct.unpack_from("<I", header, 60)[0])
            pe = stream.read(6)
            if pe == b"PE\0\0" + struct.pack("<H", {"amd64": 0x8664, "arm64": 0xaa64}[arch]):
                return
        if os_name == "linux" and header[:6] == b"\x7fELF\x02\x01" and len(header) >= 20:
            if struct.unpack_from("<H", header, 18)[0] == {"amd64": 62, "arm64": 183}[arch]:
                return
        if os_name == "darwin" and header[:4] == b"\xcf\xfa\xed\xfe" and len(header) >= 8:
            if struct.unpack_from("<I", header, 4)[0] == {"amd64": 0x1000007, "arm64": 0x100000c}[arch]:
                return
    raise SystemExit(f"{path} is not a {platform} native executable")


def build_pinned_xcode_exec(record, platform, output):
    target = PLATFORMS[platform]["target"]
    require(PLATFORMS[platform]["runner"] == record["runner"], "Wrong macOS release runner selection")
    cwd = ROOT / "crates/prometheus-exec"
    rust = subprocess.check_output(["rustc", "-vV"], cwd=cwd, text=True).splitlines()
    for field, value in (("release", record["rustc"]), ("commit-hash", record["rustcCommit"]), ("host", target)):
        require(f"{field}: {value}" in rust, f"Expected native {field}: {value}")
    dev = Path("/Applications") / f"Xcode_{record['xcode']}.app/Contents/Developer"
    env = {**os.environ, "DEVELOPER_DIR": str(dev)}
    version = subprocess.check_output(["xcodebuild", "-version"], env=env, text=True).splitlines()
    require(version == ["Xcode " + record["xcode"], "Build version " + record["xcodeBuild"]],
            f"Expected Xcode {record['xcode']} build {record['xcodeBuild']}, got {version}")
    xcrun = ["xcrun", "--sdk", "macosx" + record["sdk"]]
    sdk = subprocess.check_output([*xcrun, "--show-sdk-version"], env=env, text=True).strip()
    require(sdk == record["sdk"], f"Unexpected release SDK {sdk}")
    env["SDKROOT"] = subprocess.check_output([*xcrun, "--show-sdk-path"], env=env, text=True).strip()
    ld = subprocess.run([*xcrun, "ld", "-v"], env=env, text=True, capture_output=True)
    identity = re.search(r"PROJECT:ld-([^\s]+)", ld.stdout + ld.stderr)
    require(identity, "Could not identify the Apple linker")
    print(f"Building {target} with rustc {record['rustc']} ({record['rustcCommit']}), "
          f"{' / '.join(version)}, SDK {sdk}, ld {identity[1]}", flush=True)
    subprocess.run(["bash", str(ROOT / "scripts/build-prometheus-exec.sh")], cwd=ROOT, env=env, check=True)
    unsigned = ROOT / config("config/prometheus-exec-binary.json")["sourcePath"]
    subprocess.run([*xcrun, "vtool", "-show-build", str(unsigned)], env=env, check=True)
    actual = digest(unsigned)
    require(actual == record["sha256"], f"Official {TAG} {target} exec pin mismatch: produced {actual}")
    print(f"Native release exec SHA-256 {actual}", flush=True)
    shutil.copyfile(unsigned, output)


def darwin_exec(stage, platform):
    output = stage / "prometheus-exec"
    if record := exec_release_record(platform):
        build_pinned_xcode_exec(record, platform, output)
        return
    expected = pin(platform)
    require(expected, f"Missing independent {platform} exec pin")
    # gh can access drafts. Never pass release credentials to a configurable URL.
    url = os.environ.get(f"PROMETHEUS_EXEC_{platform.upper().replace('-', '_')}_URL")
    if url and platform == "darwin-arm64":
        require(url.startswith("https://"), "Certified exec URL requires HTTPS")
        with urllib.request.urlopen(url, timeout=180) as body, output.open("xb") as dest:
            require(body.geturl().startswith("https://"), "Insecure certified exec redirect")
            shutil.copyfileobj(body, dest)
    elif platform == "darwin-arm64" and os.environ.get("GITHUB_ACTIONS"):
        asset = "prometheus-exec-" + expected
        result = subprocess.run(["gh", "release", "download", TAG, "--repo", os.environ["GITHUB_REPOSITORY"],
                                 "--pattern", asset, "--dir", str(stage)])
        if result.returncode == 0:
            (stage / asset).rename(output)
    if not output.exists():
        target = PLATFORMS[platform]["target"]
        apple = config("config/prometheus-exec-binary.json").get("appleBuildToolsByTarget", {}).get(target)
        envs = [os.environ.copy()]
        seen = set()
        developers = [Path("/Library/Developer/CommandLineTools"), *sorted(Path("/Applications").glob("Xcode*.app/Contents/Developer"))]
        for dev in developers:
            if not dev.is_dir():
                continue
            probe = {**os.environ, "DEVELOPER_DIR": str(dev)}
            for sdk in sorted([*dev.glob("SDKs/MacOSX*.sdk"), *dev.glob("Platforms/MacOSX.platform/Developer/SDKs/MacOSX*.sdk")], reverse=True):
                if sdk.is_symlink() or not sdk.is_dir():
                    continue
                native = ["xcrun", "--sdk", str(sdk)]
                version = subprocess.run([*native, "--show-sdk-version"], env=probe, text=True, capture_output=True)
                ld = subprocess.run([*native, "ld", "-v"], env=probe, text=True, capture_output=True)
                identity = re.search(r"PROJECT:ld-([^\s]+)", ld.stderr + ld.stdout)
                if not identity or version.returncode:
                    continue
                key = (version.stdout.strip(), identity[1])
                if key in seen or (apple and key != (apple["sdk"], apple["ld"])):
                    continue
                seen.add(key)
                envs.append({**probe, "SDKROOT": str(sdk)})
        unsigned = ROOT / config("config/prometheus-exec-binary.json")["sourcePath"]
        mirrored_main = ROOT / "crates/prometheus-exec/target/reproducible-build/crates/prometheus-exec/src/main.rs"
        for env in envs:
            # Cargo does not include the selected Apple linker/SDK in its input fingerprint.
            # The canonical builder mirrors source mtimes. Touch the input to force
            # one fresh final link on this disposable runner.
            if mirrored_main.exists():
                (ROOT / "crates/prometheus-exec/src/main.rs").touch()
            result = subprocess.run(["bash", str(ROOT / "scripts/build-prometheus-exec.sh")], cwd=ROOT, env=env)
            if result.returncode:
                if apple and "SDKROOT" not in env:
                    continue
                raise SystemExit("Native prometheus-exec build failed")
            if digest(unsigned) == expected:
                shutil.copyfile(unsigned, output)
                break
            print(f"Native linker produced {digest(unsigned)} with {env.get('SDKROOT', 'default SDK')}", flush=True)
        require(output.exists(), f"No installed native Apple SDK/linker reproduced {target} pin {expected}")
    require(digest(output) == expected, f"Independent {platform} prometheus-exec pin mismatch")


def build_rust(c, platform, stage, scratch):
    cwd = ROOT / c["directory"]
    target = PLATFORMS[platform]["target"]
    suffix = ".exe" if platform.startswith("windows") else ""
    obj = scratch / "cargo"
    env = {**os.environ, "CARGO_TARGET_DIR": str(obj), "CARGO_BUILD_BUILD_DIR": str(obj),
           "RUSTC_WRAPPER": "", "CARGO_INCREMENTAL": "0"}
    # The liter-llm workspace config names a cross compiler even on native arm64.
    if platform == "linux-arm64":
        env["CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER"] = "cc"
    subprocess.run(["rustup", "target", "add", target], cwd=cwd, env=env, check=True)
    command = ["cargo", "build", "--release", "--locked", "--target", target]
    for package in c.get("packages", []):
        command += ["--package", package]
    for binary in c["binaries"]:
        command += ["--bin", binary]
    if c.get("noDefaultFeatures"):
        command.append("--no-default-features")
    features = c.get("features", []) + c.get("featuresByPlatform", {}).get(platform, [])
    if features:
        command += ["--features", ",".join(features)]
    subprocess.run(command, cwd=cwd, env=env, check=True)
    for name in c["binaries"]:
        shutil.copyfile(obj / target / "release" / (name + suffix), stage / (name + suffix))


def build_mlx(c, stage, scratch):
    cwd = ROOT / c["directory"]
    product, = c["binaries"]
    # Xcode compiles the Metal shaders; swift build alone cannot produce default.metallib.
    subprocess.run(["xcodebuild", "build", "-scheme", product, "-configuration", "Release",
                    "-destination", "platform=macOS,arch=arm64", "-derivedDataPath", str(scratch),
                    "-onlyUsePackageVersionsFromResolvedFile", "ARCHS=arm64", "ONLY_ACTIVE_ARCH=YES", "CODE_SIGN_IDENTITY=-"], cwd=cwd, check=True)
    products = scratch / "Build/Products/Release"
    shutil.copyfile(products / product, stage / product)
    shutil.copytree(products / c["resourceTree"], stage / c["resourceTree"])


def regular_files(directory):
    files = []
    for path in sorted(directory.rglob("*")):
        reparse = getattr(path.lstat(), "st_file_attributes", 0) & stat.FILE_ATTRIBUTE_REPARSE_POINT
        require(not path.is_symlink() and not reparse, f"Unexpected link or reparse point {path}")
        if path.is_file():
            files.append(path)
        else:
            require(path.is_dir(), f"Unexpected file type {path}")
    return files


def add_file(archive, name, stream, size, mode):
    entry = tarfile.TarInfo(name)
    entry.size = size
    entry.mode = mode
    archive.addfile(entry, stream)


def build(platform, group, out):
    targets = [c for c in components(platform) if c["group"] == group]
    require(targets, f"Unknown group {group} for {platform}")
    out.mkdir(parents=True, exist_ok=True)
    if os.environ.get("GITHUB_ACTIONS"):
        # Fetch only gitlinks containing this build's source. The inventory
        # contains the exact workspace directories; git checks out the tree's
        # recorded gitlink rather than following a submodule branch.
        links = subprocess.check_output(["git", "ls-files", "--stage"], cwd=ROOT, text=True).splitlines()
        paths = [line.split("\t", 1)[1] for line in links if line.startswith("160000 ")]
        needed = [p for p in paths if any(c["directory"] == p or c["directory"].startswith(p + "/") for c in targets)]
        if needed:
            subprocess.run(["git", "submodule", "update", "--init", "--recursive", "--", *needed], cwd=ROOT, check=True)
    with tempfile.TemporaryDirectory() as tmp:
        stage = Path(tmp).resolve()
        for c in targets:
            with tempfile.TemporaryDirectory() as work:
                if group == "mlx":
                    build_mlx(c, stage, Path(work).resolve())
                elif group == "exec" and platform.startswith("darwin"):
                    darwin_exec(stage, platform)
                else:
                    build_rust(c, platform, stage, Path(work).resolve())
        if platform.startswith("darwin"):
            for c in targets:
                for name in c["binaries"]:
                    # Keep the independent build pin intact. Its installer signs a verified copy.
                    if name == "prometheus-exec":
                        continue
                    binary = str(stage / name)
                    if subprocess.run(["codesign", "--verify", "--strict", binary], capture_output=True).returncode:
                        subprocess.run(["codesign", "--force", "--sign", "-", "--timestamp=none", binary], check=True)
                    subprocess.run(["codesign", "--verify", "--strict", binary], check=True)
        with tarfile.open(out / f"{platform}-{group}.tar", "w") as archive:
            for path in regular_files(stage):
                name = path.relative_to(stage).as_posix()
                with path.open("rb") as source:
                    add_file(archive, name, source, path.stat().st_size, 0o755 if "/" not in name else 0o644)


def pack(platform, stage, out):
    suffix = ".exe" if platform.startswith("windows") else ""
    entries = {}
    for c in components(platform):
        for name in c["binaries"]:
            path = stage / (name + suffix)
            require(path.is_file() and not path.is_symlink(), f"Missing native {path}")
            cpu_check(path, platform)
            if name == "prometheus-exec" and pin(platform):
                require(digest(path) == pin(platform), f"Independent {platform} prometheus-exec pin mismatch")
            entries[name] = path
        if resource := c.get("resourceTree"):
            tree = stage / resource
            require(not tree.is_symlink() and (tree / "Contents/Resources/default.metallib").is_file(),
                    f"Missing {resource} Metal resource")
            for path in regular_files(tree):
                entries[path.relative_to(stage).as_posix()] = path
    files = []
    for name, path in sorted(entries.items()):
        relative = path.relative_to(stage).as_posix()
        require(all(re.fullmatch(r"[A-Za-z0-9_+\-][A-Za-z0-9._+\-]*", p) for p in relative.split("/")),
                f"Unsafe archive member {relative}")
        size = path.stat().st_size
        require(size > 0, f"Empty payload {name}")
        files.append(dict(name=name, path="bin/" + relative, sha256=digest(path), size=size))
    manifest = dict(schemaVersion=1, version=VERSION, tag=TAG, sourceCommit=git("rev-parse", "HEAD"), platform=platform, files=files)
    payload = (json.dumps(manifest, indent=2) + "\n").encode()
    out.mkdir(parents=True, exist_ok=True)
    base = basename(platform)
    with (out / (base + ".tar.gz")).open("wb") as output, gzip.GzipFile(filename="", mode="wb", fileobj=output, mtime=0, compresslevel=6) as compressed:
        with tarfile.open(fileobj=compressed, mode="w|") as archive:
            add_file(archive, "manifest.json", io.BytesIO(payload), len(payload), 0o644)
            for entry in files:
                with entries[entry["name"]].open("rb") as stream:
                    add_file(archive, entry["path"], stream, entry["size"], 0o644 if "/" in entry["name"] else 0o755)
    (out / (base + ".manifest.json")).write_bytes(payload)


def assemble(platform, inputs, out):
    with tempfile.TemporaryDirectory() as tmp:
        stage = Path(tmp)
        for group in groups(platform):
            with tarfile.open(inputs / f"{platform}-{group}.tar", "r:") as archive:
                for member in archive:
                    path = PurePosixPath(member.name)
                    require(member.isfile() and path.parts and not path.is_absolute() and ".." not in path.parts
                            and all(re.fullmatch(r"[A-Za-z0-9_+\-][A-Za-z0-9._+\-]*", part) for part in path.parts),
                            f"Unsafe component archive member {member.name}")
                    target = stage.joinpath(*path.parts)
                    target.parent.mkdir(parents=True, exist_ok=True)
                    with target.open("xb") as dest, archive.extractfile(member) as src:
                        shutil.copyfileobj(src, dest)
        pack(platform, stage, out)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    for name in ("build", "assemble", "pack"):
        cmd = sub.add_parser(name)
        cmd.add_argument("platform", choices=PLATFORMS)
        cmd.add_argument("source", type=str if name == "build" else Path)
        cmd.add_argument("output", type=Path)
    args = parser.parse_args()
    {"build": build, "assemble": assemble, "pack": pack}[args.command](args.platform, args.source, args.output)
