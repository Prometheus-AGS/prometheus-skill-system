#!/usr/bin/env bash
# install-mcp-services.sh — idempotent installer for all Prometheus MCP launchd services.
#
# Renders plist templates from shared/launchagents/ into ~/Library/LaunchAgents,
# then bootstraps and enables each service in the current user's GUI launchd domain.
#
# Usage:
#   bash scripts/install-mcp-services.sh [--unload] [--user <username>]
#
# Flags:
#   --unload      Boot out all managed services (does not delete plists)
#   --user <u>    Target a different macOS user (requires sudo or matching uid)
#   --dry-run     Print actions without executing them
#   --help        Show this message

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ACTION="install"
PROMETHEUS_USER="${PROMETHEUS_USER:-$(id -un)}"
DRY_RUN=false

while [ "$#" -gt 0 ]; do
    case "$1" in
        --unload)   ACTION="unload"; shift ;;
        --dry-run)  DRY_RUN=true; shift ;;
        --user)     PROMETHEUS_USER="${2:?missing value for --user}"; shift 2 ;;
        --help|-h)  grep '^#' "$0" | sed 's/^# \?//'; exit 0 ;;
        *)          echo "Unknown argument: $1" >&2; exit 2 ;;
    esac
done

require_macos() {
    [ "$(uname -s)" = "Darwin" ] || { echo "macOS only." >&2; exit 1; }
}

user_home() {
    dscl . -read "/Users/$1" NFSHomeDirectory 2>/dev/null | awk '{print $2}' \
        || eval "printf '%s' ~$1"
}

PROMETHEUS_HOME="$(user_home "$PROMETHEUS_USER")"
PROMETHEUS_UID="$(id -u "$PROMETHEUS_USER" 2>/dev/null)"
LAUNCH_AGENTS_DIR="$PROMETHEUS_HOME/Library/LaunchAgents"
LOG_DIR="$PROMETHEUS_HOME/.prometheus/logs"
KNOWLEDGE_DIR="$PROMETHEUS_HOME/.prometheus/knowledge"
GUI_DOMAIN="gui/$PROMETHEUS_UID"
PROMETHEUS_PATH="/usr/local/bin:/opt/homebrew/bin:$PROMETHEUS_HOME/.cargo/bin:$PROMETHEUS_HOME/.local/bin:/usr/bin:/bin:/usr/sbin:/sbin"

resolve_bin() {
    PATH="$PROMETHEUS_PATH" command -v "$1" 2>/dev/null || true
}

run() {
    if $DRY_RUN; then
        echo "[dry-run] $*"
    else
        "$@"
    fi
}

# All service labels in dependency order (surrealdb before surreal-memory, pk-cherry before forge)
declare -a ALL_LABELS=(
    "ai.prometheus.surrealdb-native"
    "ai.prometheus.surreal-memory-native"
    "ai.prometheus.pk-cherry"
    "ai.prometheus.forge-mcp"
    "ai.prometheus.prometheus-nudge"
)

declare -a ALL_TEMPLATES=(
    "ai.prometheus.surrealdb-native.plist"
    "ai.prometheus.surreal-memory-native.plist"
    "ai.prometheus.pk-cherry.plist"
    "ai.prometheus.forge-mcp.plist"
    "ai.prometheus.prometheus-nudge.plist"
)

render_plist() {
    local template="$1"
    local output="$2"
    local src="$REPO_ROOT/shared/launchagents/$template"

    [ -f "$src" ] || { echo "Template not found: $src" >&2; return 1; }

    local pk_cherry_bin forge_bin docker_bin surreal_bin surreal_memory_bin
    pk_cherry_bin="$(resolve_bin pk-cherry)";       [ -n "$pk_cherry_bin" ]     || pk_cherry_bin="/Users/gqadonis/.local/bin/pk-cherry"
    forge_bin="$(resolve_bin forge)";               [ -n "$forge_bin" ]         || forge_bin="/Users/gqadonis/.local/bin/forge"
    docker_bin="$(resolve_bin docker)";             [ -n "$docker_bin" ]        || docker_bin="/usr/local/bin/docker"
    surreal_bin="$(resolve_bin surreal)";           [ -n "$surreal_bin" ]       || surreal_bin="/opt/homebrew/bin/surreal"
    surreal_memory_bin="$PROMETHEUS_HOME/Projects/prometheus/surreal-memory-server/target/release/surreal-memory-server"

    python3 - "$src" "$output" <<PY
import pathlib, sys
src, dst = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
text = src.read_text()
for k, v in {
    "__PROMETHEUS_USER__":    "$PROMETHEUS_USER",
    "__PROMETHEUS_HOME__":    "$PROMETHEUS_HOME",
    "__PROMETHEUS_ROOT__":    "$REPO_ROOT",
    "__PROMETHEUS_LOG_DIR__": "$LOG_DIR",
    "__PROMETHEUS_PATH__":    "$PROMETHEUS_PATH",
    "__PK_CHERRY_BIN__":      "$pk_cherry_bin",
    "__FORGE_BIN__":          "$forge_bin",
    "__DOCKER_BIN__":         "$docker_bin",
    "__SURREAL_BIN__":        "$surreal_bin",
    "__SURREAL_MEMORY_BIN__": "$surreal_memory_bin",
}.items():
    text = text.replace(k, v)
dst.write_text(text)
PY
}

plist_path() { printf '%s/%s.plist' "$LAUNCH_AGENTS_DIR" "$1"; }

install_all() {
    require_macos
    $DRY_RUN || mkdir -p "$LAUNCH_AGENTS_DIR" "$LOG_DIR" "$KNOWLEDGE_DIR"

    for i in "${!ALL_LABELS[@]}"; do
        local label="${ALL_LABELS[$i]}"
        local template="${ALL_TEMPLATES[$i]}"
        local out; out="$(plist_path "$label")"

        echo "→ rendering $label"
        if $DRY_RUN; then
            echo "  [dry-run] render $REPO_ROOT/shared/launchagents/$template → $out"
        else
            render_plist "$template" "$out"
            plutil -lint "$out" || { echo "  WARN: plist lint failed for $out" >&2; }
        fi

        # Skip bootstrap/enable for nudge (cron-style, RunAtLoad=false)
        if [ "$label" = "ai.prometheus.prometheus-nudge" ]; then
            echo "  ↳ nudge: registered (fires every 4h, not at load)"
            if ! $DRY_RUN; then
                launchctl bootout "$GUI_DOMAIN/$label" >/dev/null 2>&1 || true
                launchctl bootstrap "$GUI_DOMAIN" "$out" 2>/dev/null || true
            fi
            continue
        fi

        echo "  ↳ bootstrapping $label"
        if ! $DRY_RUN; then
            launchctl bootout "$GUI_DOMAIN/$label" >/dev/null 2>&1 || true
            launchctl bootstrap "$GUI_DOMAIN" "$out"
            launchctl enable "$GUI_DOMAIN/$label"
            launchctl kickstart -k "$GUI_DOMAIN/$label"
        else
            echo "  [dry-run] launchctl bootstrap $GUI_DOMAIN $out"
            echo "  [dry-run] launchctl kickstart -k $GUI_DOMAIN/$label"
        fi
        echo "  ✓ $label loaded"
    done

    echo ""
    echo "All services installed. Run 'bash scripts/check-mcp-health.sh' to verify."
}

unload_all() {
    require_macos
    for label in "${ALL_LABELS[@]}"; do
        run launchctl bootout "$GUI_DOMAIN/$label" >/dev/null 2>&1 || true
        echo "unloaded $label"
    done
}

case "$ACTION" in
    install) install_all ;;
    unload)  unload_all ;;
esac
