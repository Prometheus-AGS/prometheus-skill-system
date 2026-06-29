#!/usr/bin/env bash
# install-mcp-services.sh — idempotent installer for all Prometheus MCP daemon services.
#
# Cross-platform: macOS (launchd LaunchAgents) and Linux (systemd --user units).
# Renders the service templates with the same __PLACEHOLDER__ substitution, then
# loads/starts each daemon via the platform service manager. Already-running
# services (any provenance: docker, manual, a prior install) are detected and
# REUSED — never double-started.
#
#   macOS → shared/launchagents/*.plist  → ~/Library/LaunchAgents      → launchctl
#   Linux → shared/systemd/*.service     → ~/.config/systemd/user/     → systemctl --user
#
# Daemons (dependency order): surrealdb-native(:28000) → surreal-memory-native(:23001)
#                             → pk-cherry(:8942) → forge-mcp(:8943); plus a nudge timer.
# The bundled SurrealDB binds :28000 and never touches an external instance on :8000.
#
# Usage:
#   bash scripts/install-mcp-services.sh [--unload] [--user <username>] [--dry-run]
#
# Flags:
#   --unload      Stop/boot out all managed services (does not delete unit files)
#   --user <u>    Target a different user (requires matching uid / privileges)
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

# Shared provenance-agnostic reachability helpers (probe_port, check_running_service).
# shellcheck source=../shared/scripts/service-probe.sh
. "$REPO_ROOT/shared/scripts/service-probe.sh"

# ── Platform detection ───────────────────────────────────────────────────────
case "$(uname -s 2>/dev/null)" in
    Darwin*) OS="macos" ;;
    Linux*)  OS="linux" ;;
    *)       echo "Unsupported OS: $(uname -s). This installer supports macOS (launchd) and Linux (systemd)." >&2; exit 1 ;;
esac

# ── Per-OS paths and binary search ───────────────────────────────────────────
if [ "$OS" = "macos" ]; then
    user_home() {
        dscl . -read "/Users/$1" NFSHomeDirectory 2>/dev/null | awk '{print $2}' \
            || eval "printf '%s' ~$1"
    }
    PROMETHEUS_HOME="$(user_home "$PROMETHEUS_USER")"
    PROMETHEUS_PATH="/usr/local/bin:/opt/homebrew/bin:$PROMETHEUS_HOME/.cargo/bin:$PROMETHEUS_HOME/.local/bin:/usr/bin:/bin:/usr/sbin:/sbin"
    SURREAL_FALLBACK="/opt/homebrew/bin/surreal"
    BIN_FALLBACK_DIR="$PROMETHEUS_HOME/.local/bin"
else
    PROMETHEUS_HOME="$(getent passwd "$PROMETHEUS_USER" 2>/dev/null | cut -d: -f6)"
    [ -n "$PROMETHEUS_HOME" ] || PROMETHEUS_HOME="$HOME"
    PROMETHEUS_PATH="$PROMETHEUS_HOME/.cargo/bin:$PROMETHEUS_HOME/.local/bin:/usr/local/bin:/usr/bin:/bin"
    SURREAL_FALLBACK="/usr/local/bin/surreal"
    BIN_FALLBACK_DIR="$PROMETHEUS_HOME/.local/bin"
    SYSTEMD_USER_DIR="$PROMETHEUS_HOME/.config/systemd/user"
    export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u "$PROMETHEUS_USER" 2>/dev/null || id -u)}"
fi

LOG_DIR="$PROMETHEUS_HOME/.prometheus/logs"
KNOWLEDGE_DIR="$PROMETHEUS_HOME/.prometheus/knowledge"
LAUNCH_AGENTS_DIR="$PROMETHEUS_HOME/Library/LaunchAgents"
PROMETHEUS_UID="$(id -u "$PROMETHEUS_USER" 2>/dev/null || id -u)"
GUI_DOMAIN="gui/$PROMETHEUS_UID"

resolve_bin() {
    PATH="$PROMETHEUS_PATH" command -v "$1" 2>/dev/null || true
}

run() { if $DRY_RUN; then echo "[dry-run] $*"; else "$@"; fi; }

# ── Daemons in dependency order: label | probe-port | probe-path ─────────────
declare -a DAEMON_LABELS=(
    "ai.prometheus.surrealdb-native"
    "ai.prometheus.surreal-memory-native"
    "ai.prometheus.pk-cherry"
    "ai.prometheus.forge-mcp"
)
declare -A DAEMON_PORT=(
    [ai.prometheus.surrealdb-native]=28000
    [ai.prometheus.surreal-memory-native]=23001
    [ai.prometheus.pk-cherry]=8942
    [ai.prometheus.forge-mcp]=8943
)
declare -A DAEMON_PATH=(
    [ai.prometheus.surrealdb-native]=/health
    [ai.prometheus.surreal-memory-native]=/health
    [ai.prometheus.pk-cherry]=/mcp
    [ai.prometheus.forge-mcp]=/mcp
)
NUDGE_LABEL="ai.prometheus.prometheus-nudge"

# ── Shared template rendering (identical __PLACEHOLDER__ map for both OSes) ───
render_template() {
    local src="$1" output="$2"
    [ -f "$src" ] || { echo "Template not found: $src" >&2; return 1; }

    local pk_cherry_bin forge_bin docker_bin surreal_bin surreal_memory_bin
    pk_cherry_bin="$(resolve_bin pk-cherry)";  [ -n "$pk_cherry_bin" ] || pk_cherry_bin="$BIN_FALLBACK_DIR/pk-cherry"
    forge_bin="$(resolve_bin forge)";          [ -n "$forge_bin" ]     || forge_bin="$BIN_FALLBACK_DIR/forge"
    docker_bin="$(resolve_bin docker)";        [ -n "$docker_bin" ]    || docker_bin="/usr/local/bin/docker"
    surreal_bin="$(resolve_bin surreal)";      [ -n "$surreal_bin" ]   || surreal_bin="$SURREAL_FALLBACK"
    surreal_memory_bin="$(resolve_bin surreal-memory-server)"
    [ -n "$surreal_memory_bin" ] || surreal_memory_bin="$REPO_ROOT/tools/surreal-memory-server/target/release/surreal-memory-server"
    [ -f "$surreal_memory_bin" ] || surreal_memory_bin="$BIN_FALLBACK_DIR/surreal-memory-server"

    PROMETHEUS_USER="$PROMETHEUS_USER" PROMETHEUS_HOME="$PROMETHEUS_HOME" \
    PROMETHEUS_ROOT="$REPO_ROOT" PROMETHEUS_LOG_DIR="$LOG_DIR" PROMETHEUS_PATH="$PROMETHEUS_PATH" \
    PK_CHERRY_BIN="$pk_cherry_bin" FORGE_BIN="$forge_bin" DOCKER_BIN="$docker_bin" \
    SURREAL_BIN="$surreal_bin" SURREAL_MEMORY_BIN="$surreal_memory_bin" \
    python3 - "$src" "$output" <<'PY'
import os, pathlib, sys
src, dst = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
text = src.read_text()
for k, env in {
    "__PROMETHEUS_USER__":    "PROMETHEUS_USER",
    "__PROMETHEUS_HOME__":    "PROMETHEUS_HOME",
    "__PROMETHEUS_ROOT__":    "PROMETHEUS_ROOT",
    "__PROMETHEUS_LOG_DIR__": "PROMETHEUS_LOG_DIR",
    "__PROMETHEUS_PATH__":    "PROMETHEUS_PATH",
    "__PK_CHERRY_BIN__":      "PK_CHERRY_BIN",
    "__FORGE_BIN__":          "FORGE_BIN",
    "__DOCKER_BIN__":         "DOCKER_BIN",
    "__SURREAL_BIN__":        "SURREAL_BIN",
    "__SURREAL_MEMORY_BIN__": "SURREAL_MEMORY_BIN",
}.items():
    text = text.replace(k, os.environ[env])
dst.write_text(text)
PY
}

# ════════════════════════════════════════════════════════════════════════════
# macOS (launchd)
# ════════════════════════════════════════════════════════════════════════════
macos_install() {
    $DRY_RUN || mkdir -p "$LAUNCH_AGENTS_DIR" "$LOG_DIR" "$KNOWLEDGE_DIR"
    local all=("${DAEMON_LABELS[@]}" "$NUDGE_LABEL")
    for label in "${all[@]}"; do
        local src="$REPO_ROOT/shared/launchagents/$label.plist"
        local out="$LAUNCH_AGENTS_DIR/$label.plist"
        echo "→ rendering $label"
        if ! $DRY_RUN; then
            render_template "$src" "$out"
            plutil -lint "$out" >/dev/null || echo "  WARN: plist lint failed for $out" >&2
        fi
        if [ "$label" = "$NUDGE_LABEL" ]; then
            echo "  ↳ nudge: registered (fires every 4h, not at load)"
            $DRY_RUN || { launchctl bootout "$GUI_DOMAIN/$label" >/dev/null 2>&1 || true; launchctl bootstrap "$GUI_DOMAIN" "$out" 2>/dev/null || true; }
            continue
        fi
        # Reuse if already serving on its port (any provenance).
        if check_running_service "$label" "${DAEMON_PORT[$label]}" "${DAEMON_PATH[$label]}"; then
            echo "  ↳ reusing running instance — skipping bootstrap"
            continue
        fi
        echo "  ↳ bootstrapping $label"
        if ! $DRY_RUN; then
            launchctl bootout "$GUI_DOMAIN/$label" >/dev/null 2>&1 || true
            launchctl bootstrap "$GUI_DOMAIN" "$out"
            launchctl enable "$GUI_DOMAIN/$label"
            launchctl kickstart -k "$GUI_DOMAIN/$label"
        fi
        echo "  ✓ $label loaded"
    done
}
macos_unload() {
    for label in "${DAEMON_LABELS[@]}" "$NUDGE_LABEL"; do
        run launchctl bootout "$GUI_DOMAIN/$label" >/dev/null 2>&1 || true
        echo "unloaded $label"
    done
}

# ════════════════════════════════════════════════════════════════════════════
# Linux (systemd --user)
# ════════════════════════════════════════════════════════════════════════════
linux_install() {
    command -v systemctl >/dev/null 2>&1 || { echo "systemctl not found — systemd required on Linux." >&2; exit 1; }
    $DRY_RUN || mkdir -p "$SYSTEMD_USER_DIR" "$LOG_DIR" "$KNOWLEDGE_DIR"

    # Persist user services across logout / on a headless box.
    if ! loginctl show-user "$PROMETHEUS_USER" -p Linger 2>/dev/null | grep -q 'Linger=yes'; then
        run loginctl enable-linger "$PROMETHEUS_USER" || echo "  WARN: could not enable-linger (services may stop on logout)" >&2
    fi

    # Render every unit (4 daemons + nudge service + nudge timer).
    for f in "${DAEMON_LABELS[@]/%/.service}" "$NUDGE_LABEL.service" "$NUDGE_LABEL.timer"; do
        local src="$REPO_ROOT/shared/systemd/$f"
        local out="$SYSTEMD_USER_DIR/$f"
        echo "→ rendering $f"
        $DRY_RUN || render_template "$src" "$out"
    done
    run systemctl --user daemon-reload

    # Daemons in dependency order, reusing anything already on its port.
    for label in "${DAEMON_LABELS[@]}"; do
        if check_running_service "$label" "${DAEMON_PORT[$label]}" "${DAEMON_PATH[$label]}"; then
            echo "  ↳ reusing running instance — skipping enable/start"
            continue
        fi
        echo "  ↳ enabling + starting $label.service"
        run systemctl --user enable --now "$label.service"
        echo "  ✓ $label started"
    done

    # Nudge: enable the timer (drives the oneshot every 4h), not the service.
    echo "  ↳ enabling nudge timer (fires every 4h)"
    run systemctl --user enable --now "$NUDGE_LABEL.timer"

    echo ""
    echo "All daemons installed. Verify with: bash scripts/check-mcp-health.sh"
}
linux_unload() {
    for label in "${DAEMON_LABELS[@]}"; do
        run systemctl --user disable --now "$label.service" 2>/dev/null || true
        echo "stopped $label"
    done
    run systemctl --user disable --now "$NUDGE_LABEL.timer" 2>/dev/null || true
    echo "stopped $NUDGE_LABEL.timer"
}

case "$OS/$ACTION" in
    macos/install) macos_install ;;
    macos/unload)  macos_unload ;;
    linux/install) linux_install ;;
    linux/unload)  linux_unload ;;
esac
