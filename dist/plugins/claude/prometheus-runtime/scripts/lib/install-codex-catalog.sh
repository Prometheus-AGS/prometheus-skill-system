#!/usr/bin/env bash
# Apply the Codex catalog policy: make sure a slash-command prompt exists for every
# pack skill (so a skill left out of the catalog stays invokable), then disable the
# redundant and unselected skill copies Codex would otherwise list. Sourced by
# install-skills-flat.sh and refresh-native-plugin-installs.sh, or run directly:
#   install-codex-catalog.sh [repo-root]
# Absent Codex (no CODEX_HOME directory or no `codex` binary) is silent success.

install_codex_catalog() {
    local root="${1:-${REPO_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}}"
    local output
    if ! output="$(bash "$root/scripts/register-slash-commands.sh" --codex-only 2>&1)"; then
        printf '%s\n' "$output" >&2
        echo "  ⚠️  codex catalog: could not register slash-command prompts" >&2
        return 1
    fi
    bash "$root/shared/scripts/codex-catalog-config.sh" --create --repo-root "$root"
}

if [ "${BASH_SOURCE[0]}" = "$0" ]; then
    install_codex_catalog "${1:-}"
fi
