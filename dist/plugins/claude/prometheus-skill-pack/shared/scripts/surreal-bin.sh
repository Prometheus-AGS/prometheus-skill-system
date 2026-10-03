#!/usr/bin/env bash
# surreal-bin.sh — resolve the SurrealDB server binary for the native services.
#
# Sourced by scripts/prometheus-services.sh and scripts/install-mcp-services.sh.
# No side effects on source; defines one function:
#
#   resolve_surreal_bin <prometheus_home> <search_path>
#       prints the binary to put in the rendered service, or fails.
#
# The managed install (~/.prometheus/bin/surreal) wins over anything on PATH.
# Resolving a bare `surreal` from PATH alone once picked a stale Homebrew build
# and rendered a plist that would downgrade the live database on next start.
# Any binary whose `surreal version` is not $SURREALDB_VERSION is rejected.

resolve_surreal_bin() {
    local home="$1" search_path="$2" candidate version
    candidate="$home/.prometheus/bin/surreal"
    if [ ! -x "$candidate" ]; then
        candidate="$(PATH="$search_path" command -v surreal 2>/dev/null || true)"
    fi
    if [ -z "$candidate" ]; then
        # Not installed yet: point at the managed location the installer owns.
        printf '%s\n' "$home/.prometheus/bin/surreal"
        return 0
    fi
    version="$("$candidate" version 2>/dev/null | awk '{print $1}')"
    if [ "${version%%+*}" != "$SURREALDB_VERSION" ]; then
        echo "SurrealDB at $candidate is '${version:-unknown}', expected $SURREALDB_VERSION." >&2
        echo "Install surreal $SURREALDB_VERSION to $home/.prometheus/bin/surreal before rendering services." >&2
        return 1
    fi
    printf '%s\n' "$candidate"
}
