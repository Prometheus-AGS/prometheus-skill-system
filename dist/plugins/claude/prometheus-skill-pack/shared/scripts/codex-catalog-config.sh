#!/usr/bin/env bash
# Keep Codex's skill catalog inside its fixed size budget without deleting a file.
#
# Codex lists every SKILL.md it finds under every skill root with no de-duplication,
# and names plus paths use most of a ~29.6K-character budget; descriptions get the
# remainder. This helper asks Codex itself what it lists (`codex debug prompt-input`),
# decides which entries are redundant or not selected by config/codex-catalog.txt,
# and writes `[[skills.config]] path = ... enabled = false` entries for them inside
# ONE marked block of $CODEX_HOME/config.toml. Everything outside the block is
# preserved byte for byte, and the result is validated with stdlib tomllib.
#
# What is disabled:
#   - pack skills not selected by config/codex-catalog.txt (all copies); they stay
#     invokable as slash commands, and the helper refuses if a prompt is missing;
#   - every `*.foreign-<date>` backup directory;
#   - redundant duplicate copies: one canonical copy per pack skill, and one copy
#     per byte-identical third-party skill. Different skills that merely share a
#     name are never touched.
#
# Usage: codex-catalog-config.sh [--check | --create | --uninstall]
#                                [--repo-root DIR] [--catalog FILE]
# --check is read-only and prints JSON (ok=false means the block would change).
# --create applies. --uninstall removes the managed block.
# Exit: 0 success (or Codex absent), 1 failure with the original untouched, 2 usage.
set -uo pipefail

MODE=apply
CHECK=0
UNINSTALL=0
REPO_ROOT=""
CATALOG=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --check) CHECK=1; MODE=check ;;
    --create) MODE=apply ;;
    --uninstall) UNINSTALL=1; MODE=uninstall ;;
    --repo-root) REPO_ROOT="${2:-}"; shift ;;
    --catalog) CATALOG="${2:-}"; shift ;;
    *) echo "codex-catalog-config: unknown argument: $1" >&2; exit 2 ;;
  esac
  shift
done
if [ -z "$REPO_ROOT" ]; then
  REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
fi
[ -n "$CATALOG" ] || CATALOG="$REPO_ROOT/config/codex-catalog.txt"
if ! command -v python3 >/dev/null 2>&1; then
  echo 'codex-catalog-config: Python 3 with stdlib tomllib is required; original unchanged' >&2
  exit 1
fi

export PYTHONDONTWRITEBYTECODE=1
python3 -B - "${CODEX_HOME:-$HOME/.codex}" "$MODE" "$REPO_ROOT" "$CATALOG" "${PROMETHEUS_CODEX_BIN:-codex}" <<'PY'
import copy
import datetime
import fnmatch
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile

home = Path(os.path.abspath(sys.argv[1]))
mode, repo, catalog_file, codex_bin = sys.argv[2], Path(sys.argv[3]), Path(sys.argv[4]), sys.argv[5]
config = home / "config.toml"
BEGIN = "# >>> prometheus-codex-catalog (managed by shared/scripts/codex-catalog-config.sh; do not edit) >>>"
END = "# <<< prometheus-codex-catalog <<<"
MARKERS = (".prometheus-generation", ".prometheus-pack")
try:
    import tomllib
except ImportError:
    tomllib = None


class Refused(Exception):
    """The original config is left untouched."""


def split_block(text):
    start = text.find(BEGIN)
    if start < 0:
        return text, []
    end = text.find(END, start)
    if end < 0:
        raise Refused("managed block has no end marker; original unchanged")
    stop = end + len(END)
    if text[stop:stop + 1] == "\n":
        stop += 1
    before = text[:start]
    # The block is written after exactly one separating blank line; undo exactly that.
    rest = before[:-1] if before.endswith("\n\n") else before
    old = [json.loads('"' + m + '"') for m in re.findall(r'^path = "((?:[^"\\]|\\.)*)"$', text[start:end], re.M)]
    return rest + text[stop:], old


THIRD_PARTY_VERBS = ("third-party", "keep-path", "keep-name")


def third_party_policy():
    """('exclude'|'keep', keep-path globs, keep-name globs) from the catalog file."""
    default, paths, names = "keep", [], []
    if catalog_file.exists():
        for line in catalog_file.read_text().splitlines():
            line = line.split("#", 1)[0].strip()
            verb, _, value = line.partition(" ")
            value = value.strip()
            if verb == "third-party" and value in ("keep", "exclude"):
                default = value
            elif verb == "keep-path" and value:
                paths.append(os.path.expanduser(value))
            elif verb == "keep-name" and value:
                names.append(value)
    return default, paths, names


def frontmatter_name(path):
    try:
        head = Path(path).read_text(encoding="utf-8", errors="replace")[:4096]
    except OSError:
        return None
    block = re.match(r"---\r?\n(.*?)\r?\n---", head, re.S)
    found = re.search(r"^name:\s*['\"]?([^'\"\r\n]+?)['\"]?\s*$", block.group(1) if block else "", re.M)
    return found.group(1) if found else None


def repo_skills():
    """(top-level selected names, all pack names, top-level dir names)."""
    skill_dirs = set()
    for root, dirs, files in os.walk(repo / "skills"):
        dirs[:] = [d for d in dirs if d not in (".git", "node_modules")]
        if "SKILL.md" in files:
            skill_dirs.add(os.path.relpath(root, repo))
    skill_dirs = {d for d in skill_dirs if not d.startswith("skills/imported")}

    def is_top(d):
        parts = d.split("/")
        return not any("/".join(parts[:i]) in skill_dirs for i in range(1, len(parts)))

    rules = []
    if catalog_file.exists():
        for line in catalog_file.read_text().splitlines():
            line = line.split("#", 1)[0].strip()
            if not line:
                continue
            verb, _, pattern = line.partition(" ")
            if verb in THIRD_PARTY_VERBS:
                continue  # handled by third_party_policy()
            rules.append((verb, pattern.strip()) if verb in ("include", "exclude") else ("include", line))

    def selected(d):
        keep = not rules
        for verb, pattern in rules:
            if fnmatch.fnmatch(d, pattern) or fnmatch.fnmatch(d, pattern.rstrip("/*") + "/*") or d == pattern.rstrip("/*"):
                keep = verb == "include"
        return keep

    included, pack, tops, top_names = set(), set(), set(), set()
    for d in sorted(skill_dirs):
        name = frontmatter_name(repo / d / "SKILL.md") or os.path.basename(d)
        pack.add(name)
        if is_top(d):
            tops.add(os.path.basename(d))
            top_names.add(name)
            if selected(d):
                included.add(name)
    return included, pack, tops, top_names


def discover():
    """(entries Codex prints, the roots it scans). Its listing is truncated at a
    fixed character budget, so it is only a lower bound on what exists."""
    env = dict(os.environ, CODEX_HOME=str(home))
    result = subprocess.run([codex_bin, "debug", "prompt-input", "x"], cwd=repo, env=env,
                            capture_output=True, text=True, timeout=180)
    if result.returncode != 0:
        raise Refused(f"`{codex_bin} debug prompt-input` failed: {result.stderr.strip()[:200]}")
    text = ""
    for item in json.loads(result.stdout):
        for chunk in item.get("content", []):
            body = chunk.get("text", "")
            if "### Available skills" in body:
                text = body
    if not text:
        return [], []
    roots = dict(re.findall(r"- `(r\d+)` = `([^`]+)`", text))
    entries = []
    for line in text[text.index("### Available skills"):].splitlines():
        found = re.match(r"- (\S+?):\s+(.*?)\s*\(file: ([^)]+)\)\s*$", line)
        if not found:
            continue
        name, location = found.group(1), found.group(3)
        relative = re.match(r"(r\d+)/(.*)", location)
        path = str(Path(roots[relative.group(1)]) / relative.group(2)) if relative and relative.group(1) in roots else location
        entries.append((name.rpartition(":")[2], path))
    return entries, list(roots.values())


def walk_skills(roots):
    """Every SKILL.md under the roots Codex scans, following directory symlinks."""
    found, seen = [], set()
    for root in roots:
        for current, dirs, files in os.walk(root, followlinks=True):
            real = os.path.realpath(current)
            if real in seen:
                dirs[:] = []
                continue
            seen.add(real)
            dirs[:] = [d for d in dirs if d not in (".git", "node_modules", "target")]
            if "SKILL.md" in files:
                found.append(os.path.join(current, "SKILL.md"))
    return found


def has_marker(directory):
    return any(os.path.lexists(os.path.join(directory, marker)) for marker in MARKERS)


def pack_copy(path, name, pack_names, tops):
    real = os.path.realpath(path)
    plugins = os.path.join(os.path.expanduser("~"), ".prometheus", "plugins", "prometheus-skill-pack")
    if real.startswith(str(repo / "skills") + os.sep) or real.startswith(plugins + os.sep):
        return True
    if "/plugins/cache/prometheus-skill-pack/" in path:
        return True
    # Walk up to the skill root. A bundle holds nested `skills/` folders of its own,
    # so stop at a known root, never at the first directory merely named `skills`.
    skill_roots = {str(home / "skills"), os.path.join(os.path.expanduser("~"), ".agents", "skills"),
                   str(repo / ".agents" / "skills"), str(repo / ".codex" / "skills")}
    directory = os.path.dirname(path)
    while directory and directory != os.path.dirname(directory) and directory not in skill_roots:
        if has_marker(directory):
            return True
        directory = os.path.dirname(directory)
    for part in Path(path).parts:
        base = re.sub(r"\.foreign-\d+$", "", part)
        if base != part and (base in tops or base.removeprefix("prometheus-") in tops):
            return True
    return any(path.startswith(str(repo / root) + os.sep) for root in (".agents/skills", ".codex/skills")) and name in pack_names


def priority(path, name):
    codex_skills = str(home / "skills") + os.sep
    agents_skills = os.path.join(os.path.expanduser("~"), ".agents", "skills") + os.sep
    direct = os.path.dirname(path) == str(home / "skills" / name)
    if path.startswith(codex_skills):
        rank = 0 if direct and has_marker(os.path.dirname(path)) else 1 if direct else 2
    elif path.startswith(agents_skills):
        rank = 3
    elif path.startswith(str(repo) + os.sep):
        rank = 4
    else:
        rank = 5
    return (rank, len(path), path)


def decide(entries, roots, old_paths):
    included, pack_names, tops, top_names = repo_skills()
    third_default, keep_paths, keep_names = third_party_policy()
    # Codex matches `enabled = false` on the resolved file, so work per real file:
    # a symlink and its target are one skill, and a path is never disabled while
    # another alias of the same file is the copy being kept.
    aliases = {}
    # Static roots first: Codex drops a root from its table once every skill in it
    # is disabled, so the table alone would make the universe shrink between runs.
    static_roots = [str(home / "skills"), os.path.join(os.path.expanduser("~"), ".agents", "skills"),
                    str(repo / ".agents" / "skills"), str(repo / ".codex" / "skills")]
    for path in [p for _, p in entries] + old_paths + walk_skills(static_roots + roots):
        if os.path.exists(path):
            aliases.setdefault(os.path.realpath(path), set()).add(path)
    disabled, groups, excluded_names, third_disabled = set(), {}, set(), 0
    for real, paths in aliases.items():
        name = frontmatter_name(real) or os.path.basename(os.path.dirname(real))
        path = min(paths, key=lambda p: priority(p, name))
        if any(".foreign-" in p for p in paths) and all(".foreign-" in p for p in paths):
            disabled.add(real)
        elif name in pack_names and any(pack_copy(p, name, pack_names, tops) for p in paths):
            if name not in included:
                disabled.add(real)
                excluded_names.add(name)
            else:
                groups.setdefault(("pack", name), []).append((path, real))
        elif third_default == "exclude" and not any(
                fnmatch.fnmatch(p, g) for g in keep_paths for p in list(paths) + [real]) and not any(
                fnmatch.fnmatch(name, g) for g in keep_names):
            disabled.add(real)
            third_disabled += 1
        else:
            try:
                digest = hashlib.sha256(Path(real).read_bytes()).hexdigest()
            except OSError:
                continue
            groups.setdefault(("copy", name, digest), []).append((path, real))
    for key, members in groups.items():
        members.sort(key=lambda m: priority(m[0], key[1]))
        disabled.update(real for _, real in members[1:])
    # Disable by resolved file path only: a `name` selector would miss skills that
    # Codex lists under a plugin-namespaced name (`bundle:child`).
    return sorted(disabled), excluded_names & top_names, len(included), len(aliases), third_disabled


def missing_prompts(names):
    prompts = home / "prompts"
    return sorted(n for n in names if not (prompts / f"{n}.md").exists() and not (prompts / f"prometheus-{n}.md").exists())


def render(paths):
    if not paths:
        return ""
    lines = [BEGIN, f"# {len(paths)} redundant or unselected skill files; regenerated on every install."]
    for path in paths:
        lines += ["[[skills.config]]", f"path = {json.dumps(path)}", "enabled = false", ""]
    lines[-1] = END
    return "\n".join(lines) + "\n"


def candidate_for(original_text, paths):
    rest, old = split_block(original_text)
    parsed_rest = tomllib.loads(rest)
    block = render(paths)
    if block:
        # Ensure the last user line is terminated, then one blank line, then the block.
        base = rest if (not rest or rest.endswith("\n")) else rest + "\n"
        candidate = base + ("\n" if base else "") + block
    else:
        candidate = rest
    parsed = tomllib.loads(candidate)
    expected = copy.deepcopy(parsed_rest)
    if paths:
        skills = expected.setdefault("skills", {})
        if not isinstance(skills, dict):
            raise Refused("`skills` in config.toml is not a table; original unchanged")
        skills["config"] = list(skills.get("config", [])) + [{"path": p, "enabled": False} for p in paths]
    if parsed != expected:
        raise Refused("candidate changed unrelated TOML data; original unchanged")
    return candidate, len(old)


def exclusive_copy(source, directory, stem):
    count = 0
    while True:
        destination = directory / (stem + (f"-{count}" if count else ""))
        try:
            fd = os.open(destination, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
            break
        except FileExistsError:
            count += 1
    try:
        with os.fdopen(fd, "wb") as output, source.open("rb") as original:
            shutil.copyfileobj(original, output)
            output.flush()
            os.fsync(output.fileno())
        shutil.copystat(source, destination)
        return destination
    except BaseException:
        destination.unlink(missing_ok=True)
        raise


def write_config(candidate, original):
    stamp = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    home.mkdir(parents=True, exist_ok=True)
    backup = exclusive_copy(config, home, config.name + ".bak-" + stamp) if config.exists() else None
    temp, replaced = None, False
    try:
        fd, name = tempfile.mkstemp(prefix=".config.toml-", dir=home)
        temp = Path(name)
        with os.fdopen(fd, "wb") as output:
            output.write(candidate.encode("utf-8"))
            output.flush()
            os.fsync(output.fileno())
        if backup:
            shutil.copystat(config, temp)
        if (config.read_bytes() if config.exists() else b"") != original:
            raise Refused("config changed during edit; original not replaced")
        os.replace(temp, config)
        replaced = True
        if tomllib.loads(config.read_text(encoding="utf-8")) != tomllib.loads(candidate):
            raise Refused("installed config validation failed")
    except BaseException:
        if replaced:
            if backup:
                os.replace(backup, config)
            else:
                config.unlink(missing_ok=True)
        raise
    finally:
        if temp:
            temp.unlink(missing_ok=True)


def main():
    if tomllib is None:
        raise Refused("Python 3.11+ with stdlib tomllib is required; original unchanged")
    if shutil.which(codex_bin) is None or not home.is_dir():
        if mode == "check":
            print(json.dumps({"codex_home": str(home), "installed": False, "ok": True}))
        return
    if config.is_symlink() or (os.path.lexists(config) and not config.is_file()):
        raise Refused("unsupported config path; original unchanged")
    original = config.read_bytes() if config.exists() else b""
    text = original.decode("utf-8")
    if mode == "uninstall":
        paths, excluded, included, total, third_disabled = [], set(), 0, 0, 0
    else:
        entries, roots = discover()
        paths, excluded, included, total, third_disabled = decide(entries, roots, split_block(text)[1])
        absent = missing_prompts(excluded)
        if absent and mode == "apply":
            raise Refused("excluded skills have no slash-command prompt (run scripts/register-slash-commands.sh): "
                          + ", ".join(absent[:8]) + (" ..." if len(absent) > 8 else ""))
    candidate, old = candidate_for(text, paths)
    if mode == "check":
        print(json.dumps({"codex_home": str(home), "installed": True, "managed_entries": old,
                          "expected_entries": len(paths), "included_pack_skills": included,
                          "discovered_skill_files": total, "enabled_after": total - len(paths),
                          "third_party_disabled": third_disabled,
                          "excluded_without_prompt": missing_prompts(excluded),
                          "ok": candidate.encode("utf-8") == original}, sort_keys=True))
        return
    if candidate.encode("utf-8") != original:
        write_config(candidate, original)


try:
    main()
except (Refused, OSError, UnicodeError, ValueError, subprocess.SubprocessError) as exc:
    print(f"codex-catalog-config: {exc}", file=sys.stderr)
    sys.exit(1)
PY
