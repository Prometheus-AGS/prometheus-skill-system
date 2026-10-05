#!/usr/bin/env bash
# Disable new Codex memory pipelines and prompt use; existing sessions can retain
# their old configuration. Edit only the three policy values, preserving other
# TOML bytes, and validate with Python's stdlib tomllib before atomic replacement.
# Known v1/v2 summaries are archived; MEMORY.md, raw memories and DBs stay intact.
# Usage: codex-memories-config.sh [--check | --create]
# --check is read-only. --create allows selected installers to create Codex config.
# Exit: 0 success (or absent Codex without --create), 1 explicit apply failure,
# 2 unsupported arguments. Check health is carried by JSON's ok field.
set -uo pipefail

CHECK=0
CREATE=0
for arg in "$@"; do
  case "$arg" in
    --check) CHECK=1 ;;
    --create) CREATE=1 ;;
    *) echo "codex-memories-config: unknown argument: $arg" >&2; exit 2 ;;
  esac
done
if [ "$CHECK" -eq 1 ] && [ "$CREATE" -eq 1 ]; then
  echo 'codex-memories-config: --check and --create are mutually exclusive' >&2
  exit 2
fi
if ! command -v python3 >/dev/null 2>&1; then
  echo 'codex-memories-config: Python 3 with stdlib tomllib is required; original unchanged' >&2
  exit 1
fi

# -B also protects the editor itself; exported suppression covers descendants.
export PYTHONDONTWRITEBYTECODE=1
python3 -B - "${CODEX_HOME:-$HOME/.codex}" "$CHECK" "$CREATE" <<'PY'
import copy
import datetime
import json
import math
import os
from pathlib import Path
import shutil
import sys
import tempfile

home = Path(os.path.abspath(sys.argv[1]))
os.environ["CODEX_HOME"] = str(home)
check, create = sys.argv[2] == "1", sys.argv[3] == "1"
config = home / "config.toml"
POLICY = (("features", "memories"), ("memories", "generate_memories"), ("memories", "use_memories"))
SUMMARIES = {"v1": home / "memories" / "memory_summary.md",
             "v2": home / "memories_v2" / "memory_summary.md"}
try:
    import tomllib
except ImportError:
    tomllib = None


def setting(data, section, key):
    table = data.get(section, {})
    if not isinstance(table, dict):
        return "invalid"
    value = table.get(key)
    return "unset" if key not in table else "false" if value is False else "true" if value is True else "invalid"


def read_config():
    if tomllib is None:
        raise ValueError("Python 3.11+ with stdlib tomllib is required")
    raw = config.read_bytes() if config.exists() else b""
    text = raw.decode("utf-8")
    return raw, text, tomllib.loads(text)


def report():
    error = None
    try:
        _, _, data = read_config()
        states = [setting(data, *key) for key in POLICY]
    except (OSError, UnicodeError, ValueError) as exc:
        states, error = ["invalid"] * len(POLICY), str(exc)
    present = {version: os.path.lexists(source) for version, source in SUMMARIES.items()}
    installed = home.is_dir()
    result = {
        "codex_home": str(home), "installed": installed,
        "generate_memories": states[1], "summary_present": present["v1"],
        "feature_memories": states[0], "use_memories": states[2],
        "summary_v1_present": present["v1"], "summary_v2_present": present["v2"],
        "parser_available": tomllib is not None,
        "ok": error is None and (not installed or (all(s == "false" for s in states) and not any(present.values()))),
    }
    if error:
        result["error"] = error
    print(json.dumps(result, sort_keys=True))


# This scanner locates spans only. tomllib validates both input and candidate;
# it also decodes quoted/dotted keys. Never interpret a multiline string as TOML.
def string_end(text, start):
    quote = text[start]
    triple = text.startswith(quote * 3, start)
    index = start + (3 if triple else 1)
    while index < len(text):
        if quote == '"' and text[index] == "\\":
            index += 2
            continue
        if text[index] == quote:
            end = index + 1
            if triple:
                while end < len(text) and text[end] == quote:
                    end += 1
                if end - index >= 3:
                    return end
            else:
                return end
            index = end
        else:
            index += 1
    raise ValueError("unsupported unterminated quoted token")


def key_path(key):
    node = tomllib.loads(key + " = false")
    parts = []
    while isinstance(node, dict) and len(node) == 1:
        name, node = next(iter(node.items()))
        parts.append(name)
    if node is not False:
        raise ValueError("unsupported key syntax")
    return tuple(parts)


def statements(text):
    index, context, in_array_table = 0, (), False
    while index < len(text):
        if text[index].isspace():
            index += 1
            continue
        if text[index] == "#":
            newline = text.find("\n", index)
            index = len(text) if newline < 0 else newline + 1
            continue
        if text[index] == "[":
            start = index
            in_array_table = text.startswith("[[", start)
            index += 2 if in_array_table else 1
            key_start = index
            while index < len(text) and text[index] != "]":
                if text[index] in "\"'":
                    index = string_end(text, index)
                else:
                    index += 1
            context = key_path(text[key_start:index].strip())
            index += 2 if in_array_table else 1
            newline = text.find("\n", index)
            index = len(text) if newline < 0 else newline + 1
            yield ("header", context, start, index, in_array_table)
            continue
        key_start = index
        while index < len(text) and text[index] != "=":
            if text[index] in "\"'":
                index = string_end(text, index)
            else:
                index += 1
        if index == len(text):
            raise ValueError("unsupported assignment syntax")
        key = key_path(text[key_start:index].strip())
        index += 1
        while index < len(text) and text[index] in " \t":
            index += 1
        value_start, depth = index, 0
        while index < len(text):
            char = text[index]
            if char in "\"'":
                index = string_end(text, index)
                continue
            if char == "#":
                if not depth:
                    break
                newline = text.find("\n", index)
                index = len(text) if newline < 0 else newline + 1
                continue
            if char in "[{":
                depth += 1
            elif char in "]}":
                depth -= 1
            elif char in "\r\n" and not depth:
                break
            index += 1
        value_end = index
        while value_end > value_start and text[value_end - 1].isspace():
            value_end -= 1
        yield ("value", context + key, value_start, value_end, in_array_table)


def equivalent(left, right):
    # TOML allows nan; ordinary equality would reject unchanged nan values.
    if type(left) is not type(right):
        return False
    if isinstance(left, dict):
        return left.keys() == right.keys() and all(equivalent(left[k], right[k]) for k in left)
    if isinstance(left, list):
        return len(left) == len(right) and all(equivalent(a, b) for a, b in zip(left, right))
    if isinstance(left, float) and math.isnan(left):
        return math.isnan(right)
    return left == right


def edit(text, data):
    for section, _ in POLICY:
        if section in data and not isinstance(data[section], dict):
            raise ValueError(f"unsupported non-table {section}; original unchanged")
    records = list(statements(text))
    headers = {key: end for kind, key, _, end, array in records if kind == "header" and not array}
    changes, seen, dotted = [], set(), set()
    for kind, key, start, end, array in records:
        if kind != "value" or array:
            continue
        if key in (("features",), ("memories",)):
            raise ValueError(f"unsupported inline-table assignment for {key[0]}; original unchanged")
        if len(key) > 1 and key[0] in ("features", "memories") and (key[0],) not in headers:
            # Root dotted assignments define a table that cannot be redeclared.
            dotted.add(key[0])
        if key in POLICY:
            seen.add(key)
            if text[start:end] != "false":
                changes.append((start, end, "false"))
    newline = "\r\n" if "\r\n" in text else "\n"
    insertions, appended = {}, []
    for section in ("features", "memories"):
        missing = [key for table, key in POLICY if table == section and (table, key) not in seen]
        if not missing:
            continue
        header = headers.get((section,))
        if header is not None:
            prefix = newline if header and text[header - 1] != "\n" else ""
            insertions[header] = insertions.get(header, "") + prefix + "".join(f"{key} = false{newline}" for key in missing)
        elif section in dotted:
            insertions[0] = insertions.get(0, "") + "".join(f"{section}.{key} = false{newline}" for key in missing)
        else:
            appended.append(f"[{section}]{newline}" + "".join(f"{key} = false{newline}" for key in missing))
    changes += [(offset, offset, value) for offset, value in insertions.items()]
    candidate = text
    for start, end, value in sorted(changes, reverse=True):
        candidate = candidate[:start] + value + candidate[end:]
    if appended:
        candidate += (newline if candidate and not candidate.endswith("\n") else "") + newline.join(appended)
    parsed = tomllib.loads(candidate)
    expected = copy.deepcopy(data)
    for section, key in POLICY:
        expected.setdefault(section, {})[key] = False
    if not equivalent(parsed, expected):
        raise ValueError("candidate changed unrelated TOML data; original unchanged")
    return candidate.encode("utf-8")


def exclusive_copy(source, directory, stem, suffix):
    count = 0
    while True:
        name = stem + (f"-{count}" if count else "") + suffix
        destination = directory / name
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


def apply():
    if not home.is_dir() and not create:
        return
    if config.is_symlink() or (os.path.lexists(config) and not config.is_file()):
        raise ValueError("unsupported config path; original unchanged")
    original, text, data = read_config()
    candidate = edit(text, data)  # parse and semantic validation precede any write
    stamp = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    if candidate != original:
        home.mkdir(parents=True, exist_ok=True)
        backup = exclusive_copy(config, home, config.name + ".bak-" + stamp, "") if config.exists() else None
        temp = None
        replaced = False
        try:
            fd, name = tempfile.mkstemp(prefix=".config.toml-", dir=home)
            temp = Path(name)
            with os.fdopen(fd, "wb") as output:
                output.write(candidate)
                output.flush()
                os.fsync(output.fileno())
            if backup:
                shutil.copystat(config, temp)
            current = config.read_bytes() if config.exists() else b""
            if current != original:
                raise ValueError("config changed during edit; original not replaced")
            os.replace(temp, config)
            replaced = True
            if not equivalent(tomllib.loads(config.read_text(encoding="utf-8")), tomllib.loads(candidate.decode("utf-8"))):
                raise ValueError("installed config validation failed")
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
    for version, source in SUMMARIES.items():
        if not os.path.lexists(source):
            continue
        if source.is_symlink() or not source.is_file():
            raise ValueError(f"unsupported {version} summary path; not archived")
        archive = home / "memories-archive"
        archive.mkdir(exist_ok=True)
        before = source.stat()
        exclusive_copy(source, archive, f"memory_summary-{version}-{stamp}", ".md")
        after = source.stat()
        if (before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns) != (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns):
            raise ValueError(f"{version} summary changed during archive; live summary preserved")
        source.unlink()


if check:
    report()
else:
    try:
        apply()
    except (OSError, UnicodeError, ValueError) as exc:
        print(f"codex-memories-config: {exc}", file=sys.stderr)
        sys.exit(1)
PY
