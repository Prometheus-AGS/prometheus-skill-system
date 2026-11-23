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


def source_exec_pin(platform):
    target = PLATFORMS[platform]["target"]
    return config("config/prometheus-exec-binary.json")["expectedBuildSha256ByTarget"].get(target)


def exec_pins(platform, hosted):
    source = source_exec_pin(platform)
    record = exec_release_record(platform)
    if record:
        return {record["sha256"]} if hosted else {record["sha256"], source}
    return {source} if source else set()


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
    expected_xcode = ["Xcode " + record["xcode"], "Build version " + record["xcodeBuild"]]
    for app_name in (f"Xcode_{record['xcode']}.app", "Xcode.app"):
        dev = Path("/Applications") / app_name / "Contents/Developer"
        if not dev.is_dir():
            continue
        env = {**os.environ, "DEVELOPER_DIR": str(dev)}
        version = subprocess.check_output(["xcodebuild", "-version"], env=env, text=True).splitlines()
        if version == expected_xcode:
            break
    else:
        raise SystemExit(f"Required native release toolchain unavailable: {', '.join(expected_xcode)}")
    xcrun = ["xcrun", "--sdk", "macosx" + record["sdk"]]
    sdk = subprocess.check_output([*xcrun, "--show-sdk-version"], env=env, text=True).strip()
    require(sdk == record["sdk"], f"Unexpected release SDK {sdk}")
    env["SDKROOT"] = subprocess.check_output([*xcrun, "--show-sdk-path"], env=env, text=True).strip()
    ld = subprocess.run([*xcrun, "ld", "-v"], env=env, text=True, capture_output=True)
    identity = re.search(r"PROJECT:ld-([^\s]+)", ld.stdout + ld.stderr)
    require(identity, "Could not identify the Apple linker")
    require(not record.get("ld") or identity[1] == record["ld"], f"Unexpected Apple linker {identity[1]}")
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
    expected = source_exec_pin(platform)
    require(expected, f"Missing independent {platform} exec pin")
    subprocess.run(["bash", str(ROOT / "scripts/build-prometheus-exec.sh")], cwd=ROOT, check=True)
    unsigned = ROOT / config("config/prometheus-exec-binary.json")["sourcePath"]
    require(digest(unsigned) == expected, f"Native {TAG} {platform} exec source pin mismatch")
    shutil.copyfile(unsigned, output)


def build_rust(c, platform, stage, scratch):
    cwd = ROOT / c["directory"]
    if platform.startswith("windows") and c["directory"] == "tools/cowork-skills/cli":
        staged = scratch / "cowork"
        subprocess.run(["node", str(ROOT / "scripts/release/stage-windows-cowork.mjs"), str(staged)], check=True)
        cwd = staged / "cli"
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


def sign_native_apple(binary):
    if subprocess.run(["codesign", "--verify", "--strict", binary], capture_output=True).returncode:
        subprocess.run(["codesign", "--force", "--sign", "-", "--timestamp=none", binary], check=True)
    subprocess.run(["codesign", "--verify", "--strict", binary], check=True)


def build_mlx(c, stage, scratch):
    cwd = ROOT / c["directory"]
    product, = c["binaries"]
    macro = c["trustedMacro"]
    resolved = json.loads((cwd / "Package.resolved").read_bytes())["pins"]
    require(any(p["identity"] == macro["packageIdentity"] and p["state"]["revision"] == macro["fingerprint"]
                for p in resolved), "Swift macro trust no longer matches the locked package revision")
    # Xcode consumes SwiftPM's revision-scoped macro fingerprint store. Isolate it
    # from the user's preferences and retain normal checks for all other macros.
    home = scratch / "home"
    security = home / "Library/org.swift.swiftpm/security"
    security.mkdir(parents=True)
    (home / ".swiftpm").mkdir()
    (home / ".swiftpm/security").symlink_to(security, target_is_directory=True)
    (security / "macros.json").write_text(json.dumps([macro]))
    env = {**os.environ, "HOME": str(home), "CFFIXED_USER_HOME": str(home)}
    derived = scratch / "derived"
    # Only the executable requires a signature. Ad-hoc signing a resource-only
    # bundle creates a zero-byte CodeSignature placeholder in the payload.
    # Xcode compiles the Metal shaders; swift build alone cannot produce default.metallib.
    subprocess.run(["xcodebuild", "build", "-scheme", product, "-configuration", "Release",
                    "-destination", "platform=macOS,arch=arm64", "-derivedDataPath", str(derived),
                    "-onlyUsePackageVersionsFromResolvedFile", "ENABLE_CODE_COVERAGE=NO",
                    "CLANG_COVERAGE_MAPPING=NO", "ARCHS=arm64", "ONLY_ACTIVE_ARCH=YES", "CODE_SIGNING_ALLOWED=NO"],
                   cwd=cwd, env=env, check=True)
    products = derived / "Build/Products/Release"
    shutil.copyfile(products / product, stage / product)
    (stage / product).chmod(0o755)
    sign_native_apple(stage / product)
    shutil.copytree(products / c["resourceTree"], stage / c["resourceTree"])


def source_mlx(output):
    output.mkdir(parents=True, exist_ok=True)
    require(not any(output.iterdir()), "MLX build destination must be empty")
    component, = [c for c in components("darwin-arm64") if c["group"] == "mlx"]
    with tempfile.TemporaryDirectory() as work:
        build_mlx(component, output.resolve(), Path(work).resolve())


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
        if platform.startswith("darwin") and group != "mlx":
            for c in targets:
                for name in c["binaries"]:
                    # Keep the independent build pin intact. Its installer signs a verified copy.
                    if name == "prometheus-exec":
                        continue
                    sign_native_apple(stage / name)
        with tarfile.open(out / f"{platform}-{group}.tar", "w") as archive:
            for path in regular_files(stage):
                name = path.relative_to(stage).as_posix()
                with path.open("rb") as source:
                    add_file(archive, name, source, path.stat().st_size, 0o755 if "/" not in name else 0o644)


def pack(platform, stage, out, hosted=False):
    suffix = ".exe" if platform.startswith("windows") else ""
    entries = {}
    for c in components(platform):
        for name in c["binaries"]:
            path = stage / (name + suffix)
            require(path.is_file() and not path.is_symlink(), f"Missing native {path}")
            cpu_check(path, platform)
            if name == "prometheus-exec":
                allowed = exec_pins(platform, hosted)
                require(not allowed or digest(path) in allowed, f"Independent {platform} prometheus-exec pin mismatch")
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


def assemble(platform, inputs, out, hosted=False):
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
        pack(platform, stage, out, hosted)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    for name in ("build", "assemble", "pack"):
        cmd = sub.add_parser(name)
        cmd.add_argument("platform", choices=PLATFORMS)
        cmd.add_argument("source", type=str if name == "build" else Path)
        cmd.add_argument("output", type=Path)
        if name in ("pack", "assemble"):
            cmd.add_argument("--hosted", action="store_true", help="Require the exact hosted digest for this pack version")
    sub.add_parser("mlx-build").add_argument("output", type=Path)
    args = parser.parse_args()
    if args.command == "mlx-build":
        source_mlx(args.output)
    elif args.command == "build":
        build(args.platform, args.source, args.output)
    else:
        {"assemble": assemble, "pack": pack}[args.command](args.platform, args.source, args.output, args.hosted)
