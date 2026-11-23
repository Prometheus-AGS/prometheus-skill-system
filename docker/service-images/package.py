#!/usr/bin/env python3
"""Wrap published native release assets in OCI images. Run from the pack checkout."""
import argparse
import hashlib
import json
import re
import shutil
import struct
import subprocess
import tarfile
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
CATALOG = json.loads((HERE / "catalog.json").read_text())
REPO = CATALOG["repository"]
SERVICES = CATALOG["services"]
TARGETS = {"amd64": "x86_64-unknown-linux-gnu", "arm64": "aarch64-unknown-linux-gnu"}
DIGEST = r"sha256:[0-9a-f]{64}"


def require(condition, message):
    if not condition:
        raise SystemExit(message)


def output(*args):
    return subprocess.check_output(args, text=True).strip()


def save(path, data):
    path.write_text(json.dumps(data, sort_keys=True, indent=2) + "\n")


def checksum(path):
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def repository(service):
    return "ghcr.io/" + REPO.lower() + "-" + service


def release(tag):
    require(re.fullmatch(r"v\d+\.\d+\.\d+(?:-[\w.-]+)?", tag), "Expected a version tag")
    commit = output("git", "rev-parse", "--verify", f"refs/tags/{tag}^{{commit}}")
    remote = output("gh", "api", f"repos/{REPO}/commits/{tag}", "--jq", ".sha")
    require(remote == commit, f"Remote tag {tag} differs from local tag")
    record = json.loads(output("gh", "api", f"repos/{REPO}/releases/tags/{tag}"))
    require(record["tag_name"] == tag and not record["draft"], f"Missing published release {tag}")
    return commit, {asset["name"]: asset for asset in record["assets"]}


def source_for(commit, service):
    directory = service["directory"]
    mode, kind, oid, path = output("git", "ls-tree", commit, "--", directory).split()
    require(path == directory, f"Unknown component directory {directory}")
    if kind == "tree" and mode == "040000":
        return {"repository": REPO, "commit": commit, "path": path, "tree": oid}
    require((mode, kind) == ("160000", "commit"), f"Expected a gitlink or tree at {path}")
    url = output("git", "config", "--blob", f"{commit}:.gitmodules", "--get", f"submodule.{path}.url")
    require(url.startswith("https://github.com/"), f"Unexpected source repository {url}")
    repo = url.removeprefix("https://github.com/").removesuffix(".git")
    require(re.fullmatch(r"[\w.-]+/[\w.-]+", repo), f"Unexpected source repository {url}")
    return {"repository": repo, "commit": oid, "path": "."}


def get_asset(tag, assets, folder, name, sums=None):
    public = assets.get(name, {}).get("digest") or ""
    require(re.fullmatch(DIGEST, public), f"Missing GitHub asset digest for {name}")
    path = folder / name
    if not path.exists():
        subprocess.run(["gh", "release", "download", tag, "--repo", REPO, "--pattern", name,
                        "--dir", str(folder)], check=True)
    actual = checksum(path)
    require("sha256:" + actual == public, f"GitHub release digest mismatch: {name}")
    if sums is not None:
        require(sums.get(name) == actual, f"SHA256SUMS mismatch: {name}")
    return path


def full_payloads(tag, arch, commit, get, dest):
    definition = json.loads(output("git", "show", f"{commit}:config/release-binaries.json"))
    compiled = {name: group["directory"] for group in definition["components"] for name in group["binaries"]}
    names = {spec["binary"] for spec in SERVICES.values()}
    require(set(compiled) == names | set(CATALOG["otherBinaries"]), "Unclassified native binary in release definition")
    destinations = {}
    for key, spec in SERVICES.items():
        binary = spec["binary"]
        require(compiled[binary] == spec["directory"], f"Source directory changed for {key}")
        destinations.setdefault(binary, []).append(dest / key / "payload")
        for companion in spec.get("companions", []):
            require(companion in compiled, f"Missing runtime companion {companion}")
            destinations.setdefault(companion, []).append(dest / key / companion)
    base = f"prometheus-binaries-{tag}-linux-{arch}"
    sidecar = get(base + ".manifest.json").read_bytes()
    manifest = json.loads(sidecar)
    require((manifest["tag"], manifest["platform"], manifest["sourceCommit"]) ==
            (tag, f"linux-{arch}", commit), "Native manifest does not match requested tag, platform and commit")
    entries = {row["path"]: row for row in manifest["files"]}
    seen = set()
    with tarfile.open(get(base + ".tar.gz"), mode="r|gz") as archive:
        for member in archive:
            if member.name == "manifest.json":
                require("manifest.json" not in seen and member.isfile() and member.size == len(sidecar)
                        and archive.extractfile(member).read() == sidecar, "Embedded manifest mismatch")
                seen.add(member.name)
                continue
            binary = member.name.removeprefix("bin/")
            targets = destinations.get(binary)
            if not targets:
                continue
            row = entries.get(member.name, {})
            require(member.isfile() and member.name == f"bin/{binary}" and binary not in seen and
                    row.get("size") == member.size and row.get("name") == binary,
                    f"Malformed binary in archive: {member.name}")
            with targets[0].open("wb") as handle:
                shutil.copyfileobj(archive.extractfile(member), handle)
            require(checksum(targets[0]) == row["sha256"], f"Binary digest mismatch: {binary}")
            for target in targets[1:]:
                shutil.copyfile(targets[0], target)
            seen.add(binary)
    require(seen == set(destinations) | {"manifest.json"}, "Incomplete release archive")



def individual_payloads(tag, arch, commit, get, sums_file, sums, dest):
    suffix = "-" + tag + "-" + TARGETS[arch]
    classified = set(CATALOG["otherBinaries"]) | {s["binary"] for s in SERVICES.values()}
    require({name.removesuffix(suffix) for name in sums if name.endswith(suffix)} == classified,
            "Unclassified or missing native binary in Mini release")
    prefix = "# mini-native-source-v1 "
    rows = [json.loads(line[len(prefix):]) for line in sums_file.read_text().splitlines() if line.startswith(prefix)]
    if tag == "v1.20.0":
        require(commit == "24f2f8f2cf08a6b36f7961f84a42a4781764ca18" and
                checksum(sums_file) == "f616c6bef77b683326d2adcabb37bc7d0ace8d38d9972138e4104f93b3702024",
                "Legacy Mini checksum or tagged source mismatch")
    for key, service in SERVICES.items():
        binary = service["binary"]
        if tag != "v1.20.0":
            source = source_for(commit, service)
            matching = [row for row in rows if row[:2] == [tag, binary]]
            require(len(matching) == 1 and matching[0][2:4] == [source["repository"], source["commit"]]
                    and matching[0][5] == source["path"], f"Mini release source mismatch for {key}")
        shutil.copyfile(get(binary + suffix), dest / key / "payload")


def render(key, tag, arch, commit, directory):
    service = SERVICES[key]
    binary, payload = service["binary"], directory / "payload"
    with payload.open("rb") as handle:
        header = handle.read(20)
    require(header[:6] == b"\x7fELF\x02\x01" and struct.unpack("<H", header[18:20])[0] ==
            {"amd64": 62, "arm64": 183}[arch], f"{key}: payload is not Linux/{arch}")
    digest = checksum(payload)
    entrypoint = ["/usr/bin/tini", "-g", "--"]
    lines = [f'LABEL org.opencontainers.image.source="https://github.com/{REPO}"',
             f'LABEL org.opencontainers.image.version="{tag}" org.opencontainers.image.revision="{commit}"',
             f'LABEL org.prometheus.binary.sha256="{digest}" org.prometheus.service="{key}"']
    companions = {}
    if service.get("companions"):
        config = json.loads(output("git", "show", f"{commit}:config/release-binaries.json"))
        for name in service["companions"]:
            group = next(group for group in config["components"] if name in group["binaries"])
            companions[name] = {"binarySha256": checksum(directory / name), "source": source_for(commit, group)}
            lines.append(f"COPY --chmod=0755 {name} /usr/local/bin/{name}")
    for name, value in service.get("env", {}).items():
        lines.append(f"ENV {name}={json.dumps(value)}")
    if "port" in service:
        lines.append(f"EXPOSE {service['port']}")
    if "relayTo" in service:
        shutil.copyfile(HERE / "relay.sh", directory / "relay.sh")
        lines.append("COPY --chmod=0755 relay.sh /usr/local/bin/relay")
        entrypoint.extend(["/usr/local/bin/relay", str(service["port"]), str(service["relayTo"])])
    entrypoint.append("/usr/local/bin/" + binary)
    lines.append("ENTRYPOINT " + json.dumps(entrypoint))
    if service.get("args"):
        lines.append("CMD " + json.dumps(service["args"]))
    template = (HERE / "Dockerfile.in").read_text()
    packages = service.get("runtimePackages", [])
    require(all(re.fullmatch(r"[a-z0-9][a-z0-9+.-]*", name) for name in packages), f"Invalid runtime package for {key}")
    template = template.replace("@RUNTIME_PACKAGES@", "".join(" " + name for name in packages))
    recipe = template.replace("@BINARY@", binary).replace("@SERVICE@", "\n".join(lines))
    identity = hashlib.sha256(recipe.encode())
    # Companion and relay bytes are also inputs to COPY. Base image is digest-pinned.
    if "relayTo" in service:
        identity.update((directory / "relay.sh").read_bytes())
    for name in sorted(companions):
        identity.update(companions[name]["binarySha256"].encode())
    recipe_hash = identity.hexdigest()
    (directory / "Dockerfile").write_text(recipe + f'LABEL org.prometheus.recipe.sha256="{recipe_hash}"\n')
    return {"binary": binary, "binarySha256": digest, "recipeSha256": recipe_hash, "source": source_for(commit, service),
            "repository": repository(key), **({"companions": companions} if companions else {})}


def prepare(args):
    commit, assets = release(args.tag)
    args.assets.mkdir(exist_ok=True, parents=True)
    # Keep unrelated files out of the build context.
    args.out.mkdir(parents=True, exist_ok=True)
    require(not any(args.out.iterdir()), f"Use an empty output directory: {args.out}")
    for key in SERVICES:
        (args.out / key).mkdir()
    sums_file = get_asset(args.tag, assets, args.assets, "SHA256SUMS")
    sums = {}
    for line in sums_file.read_text().splitlines():
        if line and not line.startswith("#"):
            hexdigest, name = line.split(None, 1)
            require(name not in sums, f"Duplicate checksum for {name}")
            sums[name] = hexdigest

    def get(name):
        return get_asset(args.tag, assets, args.assets, name, sums)

    if CATALOG["format"] == "bundle":
        full_payloads(args.tag, args.arch, commit, get, args.out)
    else:
        individual_payloads(args.tag, args.arch, commit, get, sums_file, sums, args.out)
    images = {key: render(key, args.tag, args.arch, commit, args.out / key) for key in SERVICES}
    save(args.out / "inputs.json", {"repository": REPO, "tag": args.tag, "arch": args.arch,
                                   "sourceCommit": commit, "images": images})


def registry_digest(ref):
    proc = subprocess.run(["docker", "buildx", "imagetools", "inspect", ref],
                          capture_output=True, text=True)
    if proc.returncode:
        if re.search(r"not found|manifest unknown|name unknown|404 Not Found", proc.stderr, re.I):
            return None
        raise SystemExit(f"Cannot inspect {ref}: {proc.stderr}")
    match = re.search(rf"^Digest:\s+({DIGEST})\s*$", proc.stdout, re.MULTILINE)
    require(match, f"Missing registry digest for {ref}")
    return match.group(1)


def binary_in_image(ref, arch, item, commit, key, tag):
    info = json.loads(output("docker", "image", "inspect", ref))[0]
    labels = info["Config"]["Labels"] or {}
    require(info["Os"] == "linux" and info["Architecture"] == arch and
            labels.get("org.opencontainers.image.revision") == commit and
            labels.get("org.opencontainers.image.version") == tag and
            labels.get("org.opencontainers.image.source") == f"https://github.com/{REPO}" and
            labels.get("org.prometheus.service") == key and
            labels.get("org.prometheus.recipe.sha256") == item["recipeSha256"] and
            labels.get("org.prometheus.binary.sha256") == item["binarySha256"],
            f"Image {ref} does not identify the requested release input")
    container = output("docker", "create", "--platform", "linux/" + arch, ref)
    try:
        with tempfile.TemporaryDirectory() as temp:
            contents = {item["binary"]: item["binarySha256"]}
            contents.update({n: c["binarySha256"] for n, c in item.get("companions", {}).items()})
            for name, digest in contents.items():
                copy = Path(temp) / name
                subprocess.run(["docker", "cp", f"{container}:/usr/local/bin/{name}", str(copy)], check=True)
                require(checksum(copy) == digest, f"Packaged binary {name} mismatch in {ref}")
    finally:
        subprocess.run(["docker", "rm", "-v", container], check=True, stdout=subprocess.DEVNULL)


def build(args):
    inputs = json.loads((args.out / "inputs.json").read_text())
    arch, tag, commit = inputs["arch"], inputs["tag"], inputs["sourceCommit"]
    require(arch in TARGETS and inputs["repository"] == REPO and set(inputs["images"]) == set(SERVICES),
            "Invalid prepared catalog")
    for key, item in inputs["images"].items():
        repo = repository(key)
        require(item["repository"] == repo and item["binary"] == SERVICES[key]["binary"] and
                checksum(args.out / key / "payload") == item["binarySha256"], f"Changed input for {key}")
        ref = f"{repo}:{tag}-{arch}"
        existing = registry_digest(ref) if args.push else None
        if existing:
            immutable = repo + "@" + existing
            subprocess.run(["docker", "pull", "--platform", "linux/" + arch, immutable], check=True)
            binary_in_image(immutable, arch, item, commit, key, tag)
        else:
            subprocess.run(["docker", "buildx", "build", "--platform", "linux/" + arch,
                            "--load", "--provenance=false", "--sbom=false", "-t", ref, str(args.out / key)], check=True)
            binary_in_image(ref, arch, item, commit, key, tag)
            if args.push:
                subprocess.run(["docker", "push", "--platform", "linux/" + arch, ref], check=True)
                existing = registry_digest(ref)
                require(existing, f"Registry did not return a digest for {ref}")
        if args.push:
            item["digest"] = existing
    if args.push:
        save(args.out / f"platform-{arch}.json", inputs)


def publish(args):
    commit, assets = release(args.tag)
    inventory = {"schemaVersion": 1, "repository": REPO, "tag": args.tag, "sourceCommit": commit, "images": {}}
    for arch in TARGETS:
        record = json.loads((args.records / f"platform-{arch}.json").read_text())
        require((record["tag"], record["arch"], record["repository"], record["sourceCommit"]) ==
                (args.tag, arch, REPO, commit) and set(record["images"]) == set(SERVICES), "Architecture record mismatch")
        for key, service in SERVICES.items():
            item, repo = record["images"][key], repository(key)
            require(item["repository"] == repo and item["binary"] == service["binary"] and
                    item["source"] == source_for(commit, service) and re.fullmatch(DIGEST, item["digest"]) and
                    re.fullmatch(r"[0-9a-f]{64}", item["binarySha256"]), f"Invalid {arch} artifact for {key}")
            row = inventory["images"].setdefault(key, {"binary": service["binary"], "source": item["source"],
                                                       "repository": repo, "platforms": {}})
            row["platforms"]["linux/" + arch] = {"digest": item["digest"], "binarySha256": item["binarySha256"]}
            if service.get("companions"):
                require(set(item["companions"]) == set(service["companions"]), f"Missing runtime companion for {key}")
                row["platforms"]["linux/" + arch]["companions"] = item["companions"]
    for key, row in inventory["images"].items():
        repo = row["repository"]
        ref = f"{repo}:{args.tag}"
        expected = {platform: data["digest"] for platform, data in row["platforms"].items()}
        digest = registry_digest(ref)
        if not digest:
            subprocess.run(["docker", "buildx", "imagetools", "create", "--tag", ref,
                            *[repo + "@" + dg for dg in expected.values()]], check=True)
            digest = registry_digest(ref)
        require(digest, f"Missing published multi-platform tag {ref}")
        manifest = json.loads(output("docker", "buildx", "imagetools", "inspect", "--raw", repo + "@" + digest))
        descriptors = manifest.get("manifests", [])
        actual = {d["platform"]["os"] + "/" + d["platform"]["architecture"]: d["digest"] for d in descriptors}
        require(len(descriptors) == 2 and actual == expected, f"Published platform manifests differ for {ref}")
        row["digest"] = digest
    path = args.records / "service-images.json"
    save(path, inventory)
    if path.name in assets:
        require(assets[path.name]["digest"] == "sha256:" + checksum(path),
                "Published service image inventory differs; version tags are immutable")
    else:
        subprocess.run(["gh", "release", "upload", args.tag, str(path), "--repo", REPO], check=True)
        _, updated = release(args.tag)
        require(updated.get(path.name, {}).get("digest") == "sha256:" + checksum(path),
                "GitHub did not confirm the uploaded inventory digest")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="action", required=True)
    prep = sub.add_parser("prepare", help="Verify release assets and generate one build context per service")
    prep.add_argument("--arch", choices=TARGETS, required=True)
    prep.add_argument("--tag", required=True)
    prep.add_argument("--assets", type=Path, required=True)
    prep.add_argument("--out", type=Path, required=True)
    platform = sub.add_parser("build", help="Package and check the exact binaries in the generated Dockerfiles")
    platform.add_argument("--out", type=Path, required=True)
    platform.add_argument("--push", action="store_true", help="Publish architecture manifests and write their digests")
    index = sub.add_parser("publish", help="Combine the two architecture records and attach the inventory to the release")
    index.add_argument("--tag", required=True)
    index.add_argument("--records", type=Path, required=True)
    args = parser.parse_args()
    {"prepare": prepare, "build": build, "publish": publish}[args.action](args)


if __name__ == "__main__":
    main()
