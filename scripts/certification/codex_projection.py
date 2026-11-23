"""Compare the installed pack's configured Codex selection with the live native prompt."""
import fnmatch
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile


def selected_files(project):
    rules = []
    for line in (project / "config/codex-catalog.txt").read_text().splitlines():
        item = line.split("#", 1)[0].strip()
        if not item:
            continue
        verb, _, pattern = item.partition(" ")
        if verb in ("third-party", "keep-name", "keep-path"):
            continue
        rules.append((verb, pattern) if verb in ("include", "exclude") else ("include", item))
    selected = {}
    for directory, folders, files in os.walk(project / "skills"):
        folders[:] = [f for f in folders if f not in ("imported", ".git", "node_modules")]
        if "SKILL.md" not in files:
            continue
        folders.clear()  # Only bundle roots are eligible for the installed auto-trigger catalogue.
        source = Path(directory)
        relative = str(source.relative_to(project))
        keep = not rules
        for verb, pattern in rules:
            base = pattern.rstrip("/*")
            if fnmatch.fnmatch(relative, pattern) or fnmatch.fnmatch(relative, base + "/*") or relative == base:
                keep = verb == "include"
        if keep:
            content = (source / "SKILL.md").read_text()
            name = re.search(r"(?m)^name:\s*['\"]?([^'\"\n]+?)['\"]?\s*$", content.split("\n---", 1)[0])
            if not name:
                raise ValueError(f"selected pack skill is missing its declared name: {relative}")
            selected[name[1]] = Path.home() / ".agents/skills" / source.name / "SKILL.md"
    if not selected:
        raise ValueError("the Codex policy selected no pack skills")
    return selected


def render(exe, cwd, env, options=()):
    result = subprocess.run([exe, "debug", "prompt-input", *options, "Reply briefly."],
                            cwd=cwd, env=env, capture_output=True, timeout=120, check=True)
    payload = json.loads(result.stdout)
    entries = {}
    for item in payload:
        for block in item.get("content", []):
            body = block.get("text", "")
            if "### Available skills" not in body:
                continue
            roots = dict(re.findall(r"- `(r\d+)` = `([^`]+)`", body))
            for line in body.split("### Available skills", 1)[1].splitlines():
                m = re.fullmatch(r"- (\S+?):\s*(.*?)\s*\(file: ([^)]+)\)", line)
                if not m:
                    continue
                locator, _, rest = m[3].partition("/")
                path = Path(roots[locator]) / rest if locator in roots else Path(m[3])
                entries[(m[1].rpartition(":")[2], path.resolve())] = m[2].strip()
    return entries, hashlib.sha256(result.stdout).hexdigest()


def assess(project, exe, env_factory, options):
    selected = selected_files(project)
    with tempfile.TemporaryDirectory(prefix="catalogue-live-") as folder:
        # Use the genuine host configuration for this local render. No inference endpoint is invoked.
        env = dict(os.environ, DO_NOT_TRACK="1")
        for key in ["HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "http_proxy", "https_proxy", "all_proxy"]:
            env[key] = "http://127.0.0.1:1"
        advertised, digest = render(exe, folder, env)
    records = []
    items = sorted(selected.items())
    for start in range(0, len(items), 8):
        with tempfile.TemporaryDirectory(prefix="catalogue-reference-") as folder:
            home = Path(folder)
            work = home / "work"
            work.mkdir()
            env = env_factory(home)
            expected_rows = []
            for name, path in items[start:start + 8]:
                dest = home / ".agents/skills" / path.parent.name / "SKILL.md"
                dest.parent.mkdir(parents=True)
                for relative in ("SKILL.md", "SKILL.json", "agents/openai.yaml"):
                    metadata = path.parent / relative
                    if metadata.is_file():
                        copy = dest.parent / relative
                        copy.parent.mkdir(exist_ok=True)
                        shutil.copyfile(metadata, copy)
                expected_rows.append((name, path, dest))
            reference, _ = render(exe, work, env, options)
            for name, path, dest in expected_rows:
                intended = reference.get((name, dest.resolve()))
                if not intended:
                    raise ValueError(f"no low-pressure native reference for selected skill {name}")
                actual = advertised.get((name, path.resolve()))
                prefix = bool(actual) and intended.startswith(actual.removesuffix("...").removesuffix("…"))
                records.append({"name": name, "seen": actual is not None, "validPrefix": prefix,
                                "renderedChars": len(actual or ""), "referenceChars": len(intended),
                                "complete": actual == intended})
    return {"promptSha256": digest, "rows": records,
            "policySha256": hashlib.sha256((project / "config/codex-catalog.txt").read_bytes()).hexdigest()}
