"""Select a Full-pack release, then publish complete immutable native archives."""
import json
import os
from pathlib import Path
import re
import subprocess
import sys
from tempfile import TemporaryDirectory

from binaries import PLATFORMS, ROOT, TAG, VERSION, basename, digest, git, groups, require


def gh(*args, missing=False):
    proc = subprocess.run(["gh", *args], capture_output=True)
    if missing and proc.returncode and (b"HTTP 404" in proc.stderr or b"release not found" in proc.stderr.lower()):
        return None
    if proc.returncode:
        raise SystemExit(proc.stderr.decode(errors="replace"))
    return proc.stdout


def repo():
    return os.environ["GITHUB_REPOSITORY"]


def release():
    raw = gh("release", "view", TAG, "--repo", repo(), "--json", "tagName,isDraft,targetCommitish,assets", missing=True)
    return None if raw is None else json.loads(raw)


def remote_commit():
    api = f"repos/{repo()}/git/"
    raw = gh("api", api + "ref/tags/" + TAG, missing=True)
    if raw is None:
        return None
    obj = json.loads(raw)["object"]
    while obj["type"] == "tag":
        obj = json.loads(gh("api", api + "tags/" + obj["sha"]))["object"]
    require(obj["type"] == "commit", "Release tag must resolve to a commit")
    return obj["sha"]


def assets():
    return [basename(p) + ext for p in PLATFORMS for ext in (".tar.gz", ".manifest.json")]


def read_asset(name, directory):
    path = directory / name
    gh("release", "download", TAG, "--repo", repo(), "--pattern", name, "--dir", str(directory))
    return path


def exact_manifest(path, platform, commit):
    manifest = json.loads(path.read_bytes())
    require(manifest["platform"] == platform and manifest["sourceCommit"] == commit and
            manifest["version"] == VERSION and manifest["tag"] == TAG and manifest["schemaVersion"] == 1,
            f"Wrong release identity in {path.name}")


def public_complete(info, commit):
    listed = {a["name"]: a["digest"] for a in info["assets"]}
    require(all(name in listed for name in [*assets(), "SHA256SUMS"]),
            "A published release is missing binary assets. Refusing to edit it in place.")
    with TemporaryDirectory() as tmp:
        stage = Path(tmp)
        sums = read_asset("SHA256SUMS", stage)
        require(listed["SHA256SUMS"] == "sha256:" + digest(sums), "Uploaded checksum file digest mismatch")
        expected = "".join(f"{listed[name].removeprefix('sha256:')}  {name}\n" for name in sorted(assets()))
        require(all(re.fullmatch("sha256:[0-9a-f]{64}", listed[n]) for n in assets()) and sums.read_text() == expected,
                "Public release checksum list disagrees with its assets")
        for platform in PLATFORMS:
            sidecar = read_asset(basename(platform) + ".manifest.json", stage)
            require(listed[sidecar.name] == "sha256:" + digest(sidecar), "Uploaded manifest digest mismatch")
            exact_manifest(sidecar, platform, commit)


def identity():
    require(re.fullmatch(r"v\d+\.\d+\.\d+(?:-[a-zA-Z0-9.-]+)?", TAG), "Invalid pack release version")
    require(json.loads((ROOT / "package.json").read_bytes())["version"] == VERSION, "Pack versions disagree")
    return git("rev-parse", "HEAD")


def output(**items):
    with Path(os.environ["GITHUB_OUTPUT"]).open("a") as stream:
        for key, val in items.items():
            print(f"{key}={val}", file=stream)


def prepare():
    commit = identity()
    event = os.environ["GITHUB_EVENT_NAME"]
    ref = os.environ["GITHUB_REF"]
    requested = os.environ.get("REQUESTED_TAG", "")
    if event == "workflow_dispatch":
        require(not requested or requested == TAG, f"Requested tag does not match checked out pack {TAG}")
    elif ref.startswith("refs/tags/"):
        require(ref == "refs/tags/" + TAG, f"Pushed tag must match pack {TAG}")
    else:
        require(ref == "refs/heads/main", "Release creation requires main or a version tag")
        before = os.environ["BEFORE_SHA"]
        if before != "0" * 40:
            old = json.loads(git("show", before + ":skill-system.json"))
            if old["releaseVersion"] == VERSION:
                output(active="false")
                return
    existing = remote_commit()
    require(existing in (None, commit), f"Existing tag {TAG} belongs to another commit")
    info = release()
    if info and not info["isDraft"]:
        require(existing == commit, "Public release is missing its source tag")
        public_complete(info, commit)
        output(active="false")
        return
    matrix = [{"platform": p, "runner": data["runner"], "group": group}
              for p, data in PLATFORMS.items() for group in groups(p)]
    output(active="true", sha=commit, tag=TAG, version=VERSION, builds=json.dumps({"include": matrix}),
           platforms=json.dumps(list(PLATFORMS)))


def publish(directory):
    commit = identity()
    existing = remote_commit()
    require(existing in (None, commit), f"Refusing to move tag {TAG}")
    for platform in PLATFORMS:
        exact_manifest(directory / (basename(platform) + ".manifest.json"), platform, commit)
    files = [directory / name for name in sorted(assets())]
    hashes = {f.name: digest(f) for f in files}
    sums = directory / "SHA256SUMS"
    sums.write_bytes("".join(f"{hashes[name]}  {name}\n" for name in hashes).encode())
    hashes[sums.name] = digest(sums)
    info = release()
    if info and not info["isDraft"]:
        require(existing == commit, "Public release lost its source tag")
        public_complete(info, commit)
        return
    if not info:
        gh("release", "create", TAG, "--draft", "--target", commit, "--title", TAG,
           "--notes", "Native Full skill pack binaries, per-platform manifests and SHA-256 checksums.", "--repo", repo())
        info = release()
    require(info and info["isDraft"], "Could not stage release")
    require(info["targetCommitish"] == commit or existing == commit, "Existing draft targets a different commit")
    staged = {a["name"]: a["digest"] for a in info["assets"]}
    for f in [*files, sums]:
        if staged.get(f.name) != "sha256:" + hashes[f.name]:
            gh("release", "upload", TAG, str(f), "--clobber", "--repo", repo())
    info = release()
    staged = {a["name"]: a["digest"] for a in info["assets"]}
    require(info["isDraft"] and all(staged.get(k) == "sha256:" + v for k, v in hashes.items()),
            "Draft does not contain the complete release bytes")
    # Resolve again after uploads. Never replace an existing tag. GitHub creates
    # a missing tag at the draft's exact commit when making the release public.
    require(remote_commit() in (None, commit), "Source tag changed during upload")
    gh("release", "edit", TAG, "--draft=false", "--target", commit, "--repo", repo())
    require(remote_commit() == commit, "Public release tag differs from its package source")


if __name__ == "__main__":
    if sys.argv[1:] == ["prepare"]:
        prepare()
    elif len(sys.argv) == 3 and sys.argv[1] == "publish":
        publish(Path(sys.argv[2]))
    else:
        raise SystemExit("Usage: publish_binaries.py prepare | publish ASSET_DIRECTORY")
