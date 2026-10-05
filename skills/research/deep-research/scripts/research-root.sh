#!/usr/bin/env bash
# Sourced by research entry points. Locate a payload, never scan other checkouts.
# Existing root environment contracts also support a separately copied skill.

research_pack_root() {
    local here="$1" candidate parent hops=0
    for candidate in "${PROMETHEUS_PLUGIN_ROOT:-}" "${CLAUDE_PLUGIN_ROOT:-}"; do
        if [ -n "$candidate" ] && [ -d "$candidate/shared/scripts/lib" ]; then
            (cd "$candidate" && pwd -P)
            return 0
        fi
    done
    candidate="$here"
    # scripts -> skill -> skills -> flattened payload, or one extra category
    # directory in the authored source tree. No recursive/global discovery.
    while [ "$hops" -le 4 ]; do
        if [ -d "$candidate/shared/scripts/lib" ]; then
            printf '%s\n' "$candidate"
            return 0
        fi
        parent="$(dirname "$candidate")"
        [ "$parent" != "$candidate" ] || break
        candidate="$parent"
        hops=$((hops + 1))
    done
    return 1
}
