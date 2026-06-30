#!/usr/bin/env bash
# service-probe.sh — provenance-agnostic service reachability helpers.
#
# Sourced by scripts/check-prerequisites.sh and scripts/install-mcp-services.sh so
# "is this already running?" detection has a single implementation. No side effects
# on source; defines two functions:
#
#   probe_port <port> [path]            → 0 if something answers, silent
#   check_running_service <label> <port> [path]  → 0 if up (prints a one-line status)
#
# Both prefer an HTTP HEAD probe (works on SSE/streaming endpoints — a 4xx/405 still
# proves a listener) and fall back to a TCP connect via nc.

# Quiet probe: returns 0 if localhost:<port><path> answers, 1 otherwise. Prints nothing.
probe_port() {
    local port="$1" path="${2:-/}"
    if command -v curl >/dev/null 2>&1; then
        local code rc
        code=$(curl -sI -o /dev/null -w '%{http_code}' --connect-timeout 1 --max-time 2 "http://localhost:$port$path" 2>/dev/null)
        rc=$?
        [ "$rc" -eq 0 ] && [ -n "$code" ] && [ "$code" != "000" ] && return 0
    fi
    if command -v nc >/dev/null 2>&1; then
        nc -z -w2 localhost "$port" 2>/dev/null && return 0
    fi
    return 1
}

# Named probe: echoes "    ✅ <label> already running — <how>" and returns 0 when a
# service answers on the port (any provenance: docker, manual, prior unit). Returns 1
# otherwise. Annotates with the Docker container name when the port maps to one.
check_running_service() {
    local label="$1" port="$2" path="${3:-/}"
    local url="http://localhost:$port$path"
    local how=""

    if command -v curl >/dev/null 2>&1; then
        local code rc
        code=$(curl -sI -o /dev/null -w '%{http_code}' --connect-timeout 1 --max-time 2 "$url" 2>/dev/null)
        rc=$?
        if [ "$rc" -eq 0 ] && [ -n "$code" ] && [ "$code" != "000" ]; then
            how="HTTP $code at $url"
        fi
    fi

    if [ -z "$how" ] && command -v nc >/dev/null 2>&1; then
        if nc -z -w2 localhost "$port" 2>/dev/null; then
            how="TCP :$port listening"
        fi
    fi

    [ -z "$how" ] && return 1

    if command -v docker >/dev/null 2>&1; then
        local container
        container=$(docker ps --filter "publish=$port" --format '{{.Names}}' 2>/dev/null | head -1)
        [ -n "$container" ] && how="$how (docker: $container)"
    fi

    echo "    ✅ $label already running — $how"
    return 0
}
