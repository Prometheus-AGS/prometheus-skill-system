#!/usr/bin/env bash
# check-mcp-health.sh — health table for all Prometheus MCP services.
#
# Prints launchctl state + HTTP probe for each HTTP-reachable service.
# Stdio-only services (sycophancy-correction, liter-llm, sequential-thinking, tavily)
# are listed as "stdio — no HTTP probe".
#
# Usage: bash scripts/check-mcp-health.sh [--json]

set -euo pipefail

JSON_MODE=false
[ "${1:-}" = "--json" ] && JSON_MODE=true

PROMETHEUS_USER="${PROMETHEUS_USER:-$(id -un)}"
PROMETHEUS_UID="$(id -u "$PROMETHEUS_USER" 2>/dev/null)"
GUI_DOMAIN="gui/$PROMETHEUS_UID"

launchctl_state() {
    local label="$1"
    if launchctl print "$GUI_DOMAIN/$label" >/dev/null 2>&1; then
        local state pid
        state=$(launchctl print "$GUI_DOMAIN/$label" 2>/dev/null \
            | awk -F'= ' '/[[:space:]]state =/{print $2; exit}')
        pid=$(launchctl print "$GUI_DOMAIN/$label" 2>/dev/null \
            | awk -F'= ' '/[[:space:]]pid =/{print $2; exit}')
        printf '%s (pid %s)' "${state:-unknown}" "${pid:-n/a}"
    else
        printf 'not loaded'
    fi
}

http_probe() {
    local url="$1"
    local code
    # Use -s without -f so curl doesn't exit non-zero on 4xx/5xx
    code=$(curl -s -o /dev/null -w '%{http_code}' \
        --connect-timeout 2 --max-time 4 "$url" 2>/dev/null) || code="000"
    [ -n "$code" ] || code="000"
    printf '%s' "$code"
}

print_row() {
    local name="$1" label="$2" url="$3" desc="$4"
    local lstate code status

    lstate="$(launchctl_state "$label" 2>/dev/null || echo 'n/a')"

    if [ "$url" = "stdio" ]; then
        status="stdio — no HTTP probe"
        code="n/a"
    else
        code="$(http_probe "$url")"
        if [ "$code" = "200" ] || [ "$code" = "201" ] || [ "$code" = "404" ] || [ "$code" = "405" ]; then
            # 405 = MCP endpoint alive but requires specific method/content-type
            status="OK ($code)"
        elif [ "$code" = "000" ]; then
            status="UNREACHABLE"
        else
            status="HTTP $code"
        fi
    fi

    if $JSON_MODE; then
        printf '{"name":"%s","label":"%s","launchctl":"%s","url":"%s","status":"%s"}\n' \
            "$name" "$label" "$lstate" "$url" "$status"
    else
        printf '%-30s  %-32s  %-20s  %s\n' "$name" "$lstate" "$status" "$desc"
    fi
}

if ! $JSON_MODE; then
    printf '\n%-30s  %-32s  %-20s  %s\n' "SERVICE" "LAUNCHCTL STATE" "HTTP STATUS" "DESCRIPTION"
    printf '%s\n' "$(printf '%.0s-' {1..110})"
fi

# HTTP-reachable services
print_row "surreal-memory"         "ai.prometheus.surreal-memory-native"  "http://localhost:23001/health"  "Knowledge graph + scoped memory (native)"
print_row "prometheus-knowledge"   "ai.prometheus.pk-cherry"              "http://localhost:8942/mcp"      "pk-cherry HTTP MCP (Karpathy KB)"
print_row "forge-rs"               "ai.prometheus.forge-mcp"              "http://localhost:8943/mcp"      "Forge code-enrichment MCP"

# Stdio-only services — managed by the AI client, not launchd
print_row "sycophancy-correction"  "n/a"                                  "stdio"  "Sycophancy gate (/usr/local/bin/sycophancy-correction)"
print_row "liter-llm"              "n/a"                                  "stdio"  "Multi-model routing proxy (/usr/local/bin/liter-llm)"
print_row "sequential-thinking"    "n/a"                                  "stdio"  "Sequential thinking (npx @modelcontextprotocol/server-sequential-thinking)"
print_row "tavily"                 "n/a"                                  "stdio"  "Web search (npx mcp-remote tavily)"

# Cron service
print_row "prometheus-nudge"       "ai.prometheus.prometheus-nudge"       "stdio"  "Periodic nudge every 4h (scripts/scheduled/periodic-nudge.sh)"

if ! $JSON_MODE; then
    printf '\n'
    # Quick Docker status for surreal-memory
    if command -v docker >/dev/null 2>&1; then
        echo "Docker containers:"
        docker ps --format '  {{.Names}}  {{.Status}}  {{.Ports}}' 2>/dev/null \
            | grep -E 'surreal|surrealdb' || echo "  (none matching surreal*)"
    fi
fi
