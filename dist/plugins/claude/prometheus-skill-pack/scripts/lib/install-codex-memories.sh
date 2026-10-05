#!/usr/bin/env bash
# Guarded installer helper: disable Codex memory generation. Sourced by
# install-skills-flat.sh. Never aborts the install: every failure is a warning and
# the function returns 0. Absent Codex (no ~/.codex) is silent.
# CODEX_MEMORIES_SCRIPT overrides the script path (fault injection in tests only).

install_codex_memories() {
    local root script
    root="${REPO_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
    script="${CODEX_MEMORIES_SCRIPT:-$root/shared/scripts/codex-memories-config.sh}"
    if [ ! -f "$script" ]; then
        echo "  ⚠️  codex memories: script not found ($script); skipping" >&2
        return 0
    fi
    if ! bash "$script"; then
        echo "  ⚠️  codex memories: could not set generate_memories=false (install continues)" >&2
    fi
    return 0
}
