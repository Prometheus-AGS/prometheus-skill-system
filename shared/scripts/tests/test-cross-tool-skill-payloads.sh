#!/usr/bin/env bash
# Proves copy-based installers preserve complete skill payloads and executable bits.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
TMP_ROOT="$(mktemp -d)"
trap 'rm -rf "$TMP_ROOT"' EXIT

verify_payloads() {
  local target="$1"
  local expected="$2"
  python3 - "$REPO_ROOT" "$target" "$expected" <<'PY'
import json
import os
import pathlib
import re
import sys

repo = pathlib.Path(sys.argv[1])
target = pathlib.Path(sys.argv[2])
expected = int(sys.argv[3])
skills_root = repo / "skills"
count = 0

for skill_md in skills_root.rglob("SKILL.md"):
    rel = skill_md.relative_to(skills_root)
    if rel.parts[0] == "imported":
        continue
    source = skill_md.parent
    text = skill_md.read_text()
    frontmatter = re.search(r"^---\n([\s\S]*?)\n---", text)
    name_match = re.search(r'^name:\s*[\'\"]?([^\'\"\n]+)', frontmatter.group(1), re.M)
    name = name_match.group(1).strip() if name_match else source.name
    installed = target / name
    assert installed.is_dir(), f"missing installed skill: {name}"
    assert not installed.is_symlink(), f"copy-based install is a symlink: {name}"
    meta = json.loads((installed / "_meta.json").read_text())
    assert meta["platform"] == "minimax", f"invalid metadata marker: {name}"

    for source_file in source.rglob("*"):
        if not source_file.is_file() or any(part in {".git", "node_modules", "target"} for part in source_file.parts):
            continue
        relative_file = source_file.relative_to(source)
        installed_file = installed / relative_file
        assert installed_file.is_file(), f"{name}: missing {relative_file}"
        assert source_file.read_bytes() == installed_file.read_bytes(), f"{name}: content mismatch {relative_file}"
        source_exec = bool(source_file.stat().st_mode & 0o111)
        installed_exec = bool(installed_file.stat().st_mode & 0o111)
        assert source_exec == installed_exec, f"{name}: executable mode mismatch {relative_file}"
    count += 1

assert count == expected, f"expected {expected} skills, verified {count}"
print(f"verified={count}")
PY
}

MINIMAX_TARGET="$TMP_ROOT/direct-minimax"
SUMMARY=$(node "$REPO_ROOT/scripts/install-minimax-skills.js" \
  --repo-root "$REPO_ROOT" --target-dir "$MINIMAX_TARGET" --json)
EXPECTED="$(jq -r '.discovered' <<< "$SUMMARY")"
[[ "$(jq -r '.installed' <<< "$SUMMARY")" == "$EXPECTED" ]]
[[ "$(jq -r '.skipped' <<< "$SUMMARY")" == "0" ]]
verify_payloads "$MINIMAX_TARGET" "$EXPECTED"
echo "[PASS] canonical MiniMax installer preserves all $EXPECTED complete payloads"

PROJECT_ROOT="$TMP_ROOT/project"
mkdir -p "$PROJECT_ROOT"
(
  cd "$PROJECT_ROOT"
  "$REPO_ROOT/node_modules/.bin/tsx" "$REPO_ROOT/scripts/install-platforms.ts" \
    --platform minimax --scope project >/dev/null
)
verify_payloads "$PROJECT_ROOT/.minimax/skills" "$EXPECTED"
echo "[PASS] TypeScript installer delegates to complete-copy MiniMax semantics"

COLLISION_TARGET="$TMP_ROOT/collision-minimax"
mkdir -p "$COLLISION_TARGET/feynman-loop"
printf 'user-owned\n' > "$COLLISION_TARGET/feynman-loop/marker.txt"
node "$REPO_ROOT/scripts/install-minimax-skills.js" \
  --repo-root "$REPO_ROOT" --target-dir "$COLLISION_TARGET" --quiet
[[ "$(cat "$COLLISION_TARGET/feynman-loop/marker.txt")" == "user-owned" ]]
[[ -x "$COLLISION_TARGET/prometheus-feynman-loop/scripts/write-artifact.sh" ]]
[[ "$(jq -r '.platform' "$COLLISION_TARGET/prometheus-feynman-loop/_meta.json")" == "minimax" ]]
echo "[PASS] copy installer preserves collisions and installs a complete namespaced payload"

cmp -s "$REPO_ROOT/shared/scripts/content-grounding.sh" \
  "$REPO_ROOT/skills/learn/learn-goal/scripts/content-grounding.sh"
cmp -s "$REPO_ROOT/shared/scripts/content-grounding-kb.sh" \
  "$REPO_ROOT/skills/learn/learn-goal/scripts/content-grounding-kb.sh"
cmp -s "$REPO_ROOT/shared/scripts/content-grounding-kb.sh" \
  "$REPO_ROOT/skills/learn/learn-kb/scripts/content-grounding-kb.sh"
cmp -s "$REPO_ROOT/shared/scripts/detect-surface-tier.sh" \
  "$REPO_ROOT/skills/learn/ui-surface/scripts/detect-surface-tier.sh"
cmp -s "$REPO_ROOT/docs/learn/meta-corpus/kbd-lifecycle-corpus.json" \
  "$REPO_ROOT/skills/learn/learn-about-system/references/kbd-lifecycle-corpus.json"
cmp -s "$REPO_ROOT/docs/learn/meta-corpus/skill-pack-corpus.json" \
  "$REPO_ROOT/skills/learn/learn-about-system/references/skill-pack-corpus.json"
echo "[PASS] bundled learning resources match their canonical sources"

touch "$MINIMAX_TARGET/user-owned-skill"
node "$REPO_ROOT/scripts/install-minimax-skills.js" \
  --repo-root "$REPO_ROOT" --target-dir "$MINIMAX_TARGET" --uninstall --quiet
[[ -e "$MINIMAX_TARGET/user-owned-skill" ]]
[[ ! -e "$MINIMAX_TARGET/feynman-loop" ]]
echo "[PASS] uninstall removes only pack-owned copies"

node "$REPO_ROOT/scripts/install-minimax-skills.js" \
  --repo-root "$REPO_ROOT" --target-dir "$COLLISION_TARGET" --uninstall --quiet
[[ -e "$COLLISION_TARGET/feynman-loop/marker.txt" ]]
[[ ! -e "$COLLISION_TARGET/prometheus-feynman-loop" ]]
echo "[PASS] uninstall removes namespaced pack copies without touching collisions"
