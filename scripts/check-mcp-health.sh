#!/usr/bin/env bash
# check-mcp-health.sh — health table for all Prometheus MCP services.
#
# Cross-platform: shows the service-manager state (launchd on macOS, systemd --user
# on Linux) plus an HTTP probe for each HTTP-reachable daemon. Stdio-only servers
# (sycophancy-correction, liter-llm, sequential-thinking, tavily) are managed by the
# AI client and listed as "stdio — no HTTP probe".
#
# Usage: bash scripts/check-mcp-health.sh [--json] [--exclude <service> ...]

set -euo pipefail

JSON_MODE=false
EXCLUDED_SERVICES=""
while [ "$#" -gt 0 ]; do
    case "$1" in
        --json) JSON_MODE=true; shift ;;
        --exclude)
            [ "$#" -ge 2 ] || { echo "Missing value for --exclude" >&2; exit 2; }
            EXCLUDED_SERVICES="$EXCLUDED_SERVICES${2#service:}
"
            shift 2
            ;;
        *) echo "Unknown argument: $1" >&2; exit 2 ;;
    esac
done

service_is_excluded() {
    printf '%s' "$EXCLUDED_SERVICES" | grep -qx "$1"
}

PROMETHEUS_USER="${PROMETHEUS_USER:-$(id -un)}"
PROMETHEUS_UID="$(id -u "$PROMETHEUS_USER" 2>/dev/null || id -u)"
GUI_DOMAIN="gui/$PROMETHEUS_UID"

case "$(uname -s 2>/dev/null)" in
    Darwin*) OS="macos"; NUDGE_HEALTH_LABEL="ai.prometheus.prometheus-nudge" ;;
    Linux*)  OS="linux"; NUDGE_HEALTH_LABEL="ai.prometheus.prometheus-nudge.timer"; export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$PROMETHEUS_UID}" ;;
    *)       OS="other" ;;
esac
NUDGE_HEALTH_LABEL="${NUDGE_HEALTH_LABEL:-n/a}"

# Reports the service-manager state for a label ("n/a" if the label is n/a).
service_state() {
    local label="$1"
    [ "$label" = "n/a" ] && { printf 'n/a'; return; }
    if [ "$OS" = "macos" ]; then
        if launchctl print "$GUI_DOMAIN/$label" >/dev/null 2>&1; then
            local state pid
            state=$(launchctl print "$GUI_DOMAIN/$label" 2>/dev/null | awk -F'= ' '/[[:space:]]state =/{print $2; exit}')
            pid=$(launchctl print "$GUI_DOMAIN/$label" 2>/dev/null | awk -F'= ' '/[[:space:]]pid =/{print $2; exit}')
            printf '%s (pid %s)' "${state:-unknown}" "${pid:-n/a}"
        else
            printf 'not loaded'
        fi
    elif [ "$OS" = "linux" ]; then
        local unit="$label.service"; case "$label" in *.timer) unit="$label" ;; esac
        if systemctl --user cat "$unit" >/dev/null 2>&1; then
            local active pid
            active=$(systemctl --user is-active "$unit" 2>/dev/null || echo "inactive")
            pid=$(systemctl --user show "$unit" -p MainPID --value 2>/dev/null || echo 0)
            [ "${pid:-0}" = "0" ] && printf '%s' "$active" || printf '%s (pid %s)' "$active" "$pid"
        else
            printf 'not installed'
        fi
    else
        printf 'n/a'
    fi
}

http_probe() {
    local url="$1" code
    code=$(curl -s -o /dev/null -w '%{http_code}' --connect-timeout 2 --max-time 4 "$url" 2>/dev/null) || code="000"
    [ -n "$code" ] || code="000"
    printf '%s' "$code"
}

# HTTP probe over a Unix socket: "unix:<socket-path>[:<http-path>]".
# Several 1.7.0 services (sovereign-sync, prometheus-exec) serve HTTP on a
# same-user Unix socket and bind NO TCP port unless explicitly given --tcp.
# Probing a TCP port for those reports a healthy service as UNREACHABLE.
unix_http_probe() {
    local spec="${1#unix:}" sock path code
    # Split trailing :/http/path off the socket path (paths contain no colon).
    case "$spec" in
        *:/*) sock="${spec%:*}"; path="${spec##*:}" ;;
        *)    sock="$spec";      path="" ;;
    esac
    [ -S "$sock" ] || { printf '000'; return; }
    # Socket exists but no HTTP path to probe — existence is all we can assert.
    [ -n "$path" ] || { printf 'socket'; return; }
    code=$(curl -s -o /dev/null -w '%{http_code}' --unix-socket "$sock" \
        --connect-timeout 2 --max-time 4 "http://localhost${path}" 2>/dev/null) || code="000"
    [ -n "$code" ] || code="000"
    printf '%s' "$code"
}

mcp_probe() {
    local url="$1" reply code body
    # A well-formed JSON-RPC 2.0 `initialize`. The `jsonrpc` field and the
    # protocolVersion/capabilities/clientInfo params are REQUIRED by the MCP
    # spec: a strict server (forge-mcp) rejects anything less with HTTP 422,
    # which reads as a broken service when the service is in fact fine. A
    # lenient server (pk-cherry) accepts a partial body, so a malformed probe
    # fails inconsistently across servers and hides real outages.
    # The Accept header is likewise required by streamable-HTTP MCP servers.
    reply=$(curl -sS --connect-timeout 2 --max-time 5 \
        -H 'Content-Type: application/json' \
        -H 'Accept: application/json, text/event-stream' \
        --data '{"jsonrpc":"2.0","id":"prometheus-health","method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"prometheus-health","version":"1.0.0"}}}' \
        -w $'\n%{http_code}' "$url" 2>/dev/null) || reply=$'\n000'
    code="${reply##*$'\n'}"
    body="${reply%$'\n'*}"
    if [ "$code" = "200" ] && printf '%s' "$body" | grep -q '"jsonrpc":"2.0"' \
        && printf '%s' "$body" | grep -q '"result"'; then
        printf 'MCP OK (200)'
    elif [ "$code" = "401" ] || [ "$code" = "403" ]; then
        printf 'AUTH REQUIRED (%s)' "$code"
    elif [ "$code" = "000" ]; then
        printf 'UNREACHABLE'
    else
        printf 'MCP ERROR (%s)' "$code"
    fi
}

print_row() {
    local name="$1" label="$2" url="$3" desc="$4"
    service_is_excluded "$name" && return 0
    local svc code status
    svc="$(service_state "$label" 2>/dev/null || echo 'n/a')"

    if [ "$url" = "stdio" ]; then
        status="stdio — no HTTP probe"; code="n/a"
    elif [[ "$url" == unix:* ]]; then
        # Unix-socket service. Probes HTTP over the socket when the spec names
        # a path, else asserts only that the socket node exists.
        code="$(unix_http_probe "$url")"
        case "$code" in
            200|201|204|404|405) status="OK ($code)" ;;
            socket)              status="OK (socket)"; code="200" ;;
            000)                 status="UNREACHABLE" ;;
            *)                   status="HTTP $code" ;;
        esac
    elif [[ "$url" == */mcp ]]; then
        status="$(mcp_probe "$url")"
        code="${status##*\(}"
        code="${code%\)}"
    else
        code="$(http_probe "$url")"
        if [ "$code" = "200" ] || [ "$code" = "201" ] || [ "$code" = "404" ] || [ "$code" = "405" ]; then
            status="OK ($code)"
        elif [ "$code" = "000" ]; then
            status="UNREACHABLE"
        else
            status="HTTP $code"
        fi
    fi

    if $JSON_MODE; then
        printf '{"name":"%s","label":"%s","service":"%s","url":"%s","status":"%s"}\n' \
            "$name" "$label" "$svc" "$url" "$status"
    else
        printf '%-30s  %-32s  %-20s  %s\n' "$name" "$svc" "$status" "$desc"
    fi
}

print_optional_sovereign_row() {
    local name="sovereign-sync" label="ai.prometheus.sovereign-sync"
    local legacy_label="com.prometheusags.sovereign-sync"
    local url="$1" desc="Optional P2P sharing service (disabled by default)" svc legacy_svc enabled_state
    local disabled_registry legacy_enabled_state
    service_is_excluded "$name" && return 0
    svc="$(service_state "$label" 2>/dev/null || echo 'n/a')"
    legacy_svc="$(service_state "$legacy_label" 2>/dev/null || echo 'n/a')"
    case "$svc" in
        "not loaded"|"not installed"|inactive) ;;
        *) legacy_svc="not loaded" ;;
    esac
    case "$legacy_svc" in
        "not loaded"|"not installed"|inactive) ;;
        *) svc="$legacy_svc (legacy label)" ;;
    esac

    # Inactive is optional only when the operator actually disabled the unit or
    # no managed definition exists. A failed or enabled-but-inactive sharing
    # service is unhealthy and must never be laundered as an intentional opt-out.
    if [ "$OS" = "macos" ]; then
        disabled_registry="$(launchctl print-disabled "$GUI_DOMAIN" 2>/dev/null || true)"
        if printf '%s' "$disabled_registry" | grep -Fq '"ai.prometheus.sovereign-sync" => disabled' \
            && printf '%s' "$disabled_registry" | grep -Fq '"com.prometheusags.sovereign-sync" => disabled'; then
            enabled_state="disabled"
        elif [ -f "${HOME}/Library/LaunchAgents/${label}.plist" ] \
            || [ -f "${HOME}/Library/LaunchAgents/${legacy_label}.plist" ]; then
            enabled_state="enabled"
        else
            enabled_state="not installed"
        fi
    elif [ "$OS" = "linux" ]; then
        enabled_state="$(systemctl --user is-enabled "${label}.service" 2>/dev/null || true)"
        legacy_enabled_state="$(systemctl --user is-enabled "${legacy_label}.service" 2>/dev/null || true)"
        case "$legacy_enabled_state" in
            enabled|enabled-runtime|linked|linked-runtime) enabled_state="$legacy_enabled_state (legacy label)" ;;
        esac
        [ -n "$enabled_state" ] || enabled_state="not installed"
    else
        enabled_state="unknown"
    fi

    case "$svc" in
        failed|failed\ *)
            if $JSON_MODE; then
                printf '{"name":"%s","label":"%s","service":"%s","url":"%s","status":"%s"}\n' \
                    "$name" "$label" "$svc" "$url" "FAILED"
            else
                printf '%-30s  %-32s  %-20s  %s\n' \
                    "$name" "$svc" "FAILED" "$desc"
            fi
            ;;
        "not loaded"|"not installed"|inactive)
            case "$enabled_state" in
                disabled|masked|"not installed"|not-found)
                    if $JSON_MODE; then
                        printf '{"name":"%s","label":"%s","service":"%s","url":"%s","status":"%s"}\n' \
                            "$name" "$label" "$svc" "$url" "DISABLED (optional)"
                    else
                        printf '%-30s  %-32s  %-20s  %s\n' \
                            "$name" "$svc" "DISABLED (optional)" "$desc"
                    fi
                    ;;
                *)
                    if $JSON_MODE; then
                        printf '{"name":"%s","label":"%s","service":"%s","url":"%s","status":"%s"}\n' \
                            "$name" "$label" "$svc" "$url" "UNAVAILABLE (enabled sharing service)"
                    else
                        printf '%-30s  %-32s  %-20s  %s\n' \
                            "$name" "$svc" "UNAVAILABLE (enabled)" "$desc"
                    fi
                    ;;
            esac
            ;;
        *) print_row "$name" "$label" "$url" "$desc" ;;
    esac
}

if ! $JSON_MODE; then
    printf '\n  Platform: %s   Service manager: %s\n' "$OS" "$([ "$OS" = macos ] && echo launchd || echo 'systemd --user')"
    printf '%-30s  %-32s  %-20s  %s\n' "SERVICE" "SERVICE STATE" "HTTP STATUS" "DESCRIPTION"
    printf '%s\n' "$(printf '%.0s-' {1..110})"
fi

# HTTP-reachable daemons
print_row "surrealdb"              "ai.prometheus.surrealdb-native"       "http://localhost:28000/health"  "SurrealDB engine (native, :28000)"
print_row "surreal-memory"         "ai.prometheus.surreal-memory-native"  "http://localhost:23001/health"  "Knowledge graph + scoped memory (native)"
print_row "surreal-memory-ready"   "ai.prometheus.surreal-memory-native"  "http://localhost:23001/ready"   "Durable operation-ledger readiness"
print_row "prometheus-knowledge"   "ai.prometheus.pk-cherry"              "http://localhost:8942/mcp"      "pk-cherry HTTP MCP (Karpathy KB)"
print_row "forge-rs"               "ai.prometheus.forge-mcp"              "http://localhost:8943/mcp"      "Forge code-enrichment MCP"
print_row "surface-bridge"         "ai.prometheus.surface-bridge"         "http://localhost:7890/health"   "Tier 2 UI MCP App server (native, :7890)"

# Unix-socket services. As of 1.7.0 sovereign-sync binds NO TCP port unless
# started with --tcp; the managed LaunchAgent does not pass it, so probing
# :7892 reported a perfectly healthy service as UNREACHABLE.
SOVEREIGN_SOCK="${SOVEREIGN_SYNC_SOCKET:-${HOME}/Library/Application Support/prometheus/run/sovereign-sync.sock}"
print_optional_sovereign_row "unix:${SOVEREIGN_SOCK}:/health"
print_row "prometheus-exec"        "ai.prometheus.exec"                   "unix:${HOME}/.prometheus/run/prometheus-exec.sock"  "Code execution engine (socket daemon)"

# Stdio-only services — managed by the AI client, not the service manager
print_row "sycophancy-correction"  "n/a"  "stdio"  "Sycophancy gate (sycophancy-correction)"
print_row "liter-llm"              "n/a"  "stdio"  "Multi-model routing proxy (liter-llm)"
print_row "sequential-thinking"    "n/a"  "stdio"  "Sequential thinking (npx)"
print_row "tavily"                 "n/a"  "stdio"  "Web search (npx)"

# Periodic timer
print_row "prometheus-nudge"       "$NUDGE_HEALTH_LABEL" "stdio"  "Periodic nudge every 4h"
print_row "learning-worker"        "ai.prometheus.learning-worker" "stdio" "Durable Karpathy queue worker"
print_row "hooks-logrotate"        "ai.prometheus.hooks-logrotate" "stdio" "Owner-only hook log rotation"

if ! $JSON_MODE; then
    printf '\n'
    if command -v docker >/dev/null 2>&1; then
        echo "Docker containers (surreal*):"
        docker ps --format '  {{.Names}}  {{.Status}}  {{.Ports}}' 2>/dev/null | grep -E 'surreal' || echo "  (none)"
    fi
fi
