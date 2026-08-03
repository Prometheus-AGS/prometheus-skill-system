#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
TEST_HOME="$(mktemp -d)"
trap 'rm -rf "$TEST_HOME"' EXIT

for parent in \
    .claude .opencode .kimi-code .minimax .cursor .codex .gemini .roo \
    .windsurf .codeium/windsurf .agents .config/zed .zed .cline; do
    mkdir -p "$TEST_HOME/$parent"
done

HOME="$TEST_HOME" \
CODEX_HOME="$TEST_HOME/.codex" \
PROMETHEUS_INSTALL_SKILLS_ONLY=1 \
    /bin/bash "$REPO_ROOT/scripts/install-skills-flat.sh" >/dev/null

TARGETS=(
    ".claude/skills"
    ".opencode/skills"
    ".kimi-code/skills"
    ".minimax/skills"
    ".cursor/skills"
    ".codex/skills"
    ".gemini/skills"
    ".roo/skills"
    ".windsurf/skills"
    ".codeium/windsurf/skills"
    ".agents/skills"
    ".config/zed/skills"
    ".zed/skills"
    ".cline/skills"
)

EXPECTED="$(find "$REPO_ROOT/skills" -name SKILL.md \
    -not -path '*/imported/*' -not -path '*/tests/*' -not -path '*/fixtures/*' \
    | wc -l | tr -d ' ')"
[[ "$EXPECTED" == "145" ]]

for relative in "${TARGETS[@]}"; do
    target="$TEST_HOME/$relative"
    installed="$(python3 - "$target" <<'PY'
import os
import pathlib
import re
import sys

names = set()
for directory, _, files in os.walk(sys.argv[1], followlinks=True):
    relative = pathlib.Path(directory).relative_to(sys.argv[1])
    if "tests" in relative.parts or "fixtures" in relative.parts:
        continue
    if "SKILL.md" in files:
        text = pathlib.Path(directory, "SKILL.md").read_text()
        frontmatter = re.search(r"^---\n([\s\S]*?)\n---", text)
        match = re.search(r"^name:\s*['\"]?([^'\"\n]+)", frontmatter.group(1), re.M)
        names.add(match.group(1).strip())
print(len(names))
PY
)"
    if [[ "$installed" != "$EXPECTED" ]]; then
        echo "$relative has $installed/$EXPECTED unique skill payloads" >&2
        exit 1
    fi
done

if grep -RIl --include=SKILL.md '/Users/gqadonis/Projects/prometheus' "${TARGETS[@]/#/$TEST_HOME/}" | grep -q .; then
    echo "installed payloads contain a machine-specific repository path" >&2
    exit 1
fi

printf 'Clean install parity: %s/%s payloads across %s targets\n' \
    "$EXPECTED" "$EXPECTED" "${#TARGETS[@]}"
