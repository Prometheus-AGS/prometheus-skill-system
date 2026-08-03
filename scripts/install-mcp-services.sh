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
#                             → pk-cherry(:8942) → forge-mcp(:8943) → surface-bridge(:7890)
#                             → sovereign-sync(:7892);
#                             plus a nudge timer.
# The bundled SurrealDB binds :28000 and never touches an external instance on :8000.
#
# Usage:
#   bash scripts/install-mcp-services.sh [--unload] [--restart] [--learning-recovery]
#       [--user <username>] [--dry-run] [--render-only <directory>]
#       [--exclude <service> ...]
#
# Flags:
#   --unload      Stop/boot out all managed services (does not delete unit files)
#   --restart     Reload managed definitions and restart services even when healthy
#   --learning-recovery
#                 Install only pk-cherry, the learning worker, and hook rotation.
#                 This mode never initializes, renders, stops, or starts sovereign-sync.
#   --user <u>    Target a different user (requires matching uid / privileges)
#   --render-only <directory>
#                 Render non-excluded managed service definitions and exit
#   --dry-run     Print actions without executing them
#   --help        Show this message

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ACTION="install"
PROMETHEUS_USER="${PROMETHEUS_USER:-$(id -un)}"
DRY_RUN=false
FORCE_RESTART=false
RENDER_ONLY_DIR=""
LEARNING_RECOVERY=false
EXCLUDED_SERVICES=""

while [ "$#" -gt 0 ]; do
    case "$1" in
        --unload)   ACTION="unload"; shift ;;
        --restart)  FORCE_RESTART=true; shift ;;
        --dry-run)  DRY_RUN=true; shift ;;
        --learning-recovery) LEARNING_RECOVERY=true; shift ;;
        --exclude)
            [ "$#" -ge 2 ] || { echo "Missing value for --exclude" >&2; exit 2; }
            EXCLUDED_SERVICES="$EXCLUDED_SERVICES${2#service:}
"
            shift 2
            ;;
        --user)     PROMETHEUS_USER="${2:?missing value for --user}"; shift 2 ;;
        --render-only)
            RENDER_ONLY_DIR="${2:?missing value for --render-only}"
            shift 2
            ;;
        --help|-h)  grep '^#' "$0" | sed 's/^# \?//'; exit 0 ;;
        *)          echo "Unknown argument: $1" >&2; exit 2 ;;
    esac
done

service_is_excluded() {
    local name="${1#ai.prometheus.}"
    printf '%s' "$EXCLUDED_SERVICES" | grep -qx "$name"
}

# Shared provenance-agnostic reachability helpers (probe_port, check_running_service).
# shellcheck source-path=SCRIPTDIR
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

ensure_sovereign_config() {
    local config_path="$PROMETHEUS_HOME/.config/sovereign-sync/config.toml"
    local device_key_path="$PROMETHEUS_HOME/.config/sovereign-sync/device-key.json"
    if $DRY_RUN; then
        echo "[dry-run] ensure sovereign-sync operator namespace in $config_path"
        return
    fi
    python3 - "$config_path" <<'PY'
import os
import pathlib
import re
import secrets
import sys

path = pathlib.Path(sys.argv[1])
path.parent.mkdir(parents=True, exist_ok=True)
text = path.read_text() if path.exists() else ""
operator = secrets.token_hex(32)
assignment = f'operator_id = "{operator}"'

match = re.search(r'(?m)^operator_id\s*=\s*"([^"]*)"\s*$', text)
if match and match.group(1).strip():
    os.chmod(path, 0o600)
    raise SystemExit(0)
if match:
    text = text[:match.start()] + assignment + text[match.end():]
elif re.search(r'(?m)^\[node\]\s*$', text):
    text = re.sub(r'(?m)^(\[node\]\s*)$', rf'\1\n{assignment}', text, count=1)
else:
    prefix = f"[node]\n{assignment}\n"
    text = prefix + ("\n" + text if text else "")

temporary = path.with_name(path.name + ".tmp")
temporary.write_text(text.rstrip() + "\n")
os.chmod(temporary, 0o600)
os.replace(temporary, path)
PY
    local sovereign_sync_bin
    sovereign_sync_bin="$(resolve_bin sovereign-sync)"
    [ -n "$sovereign_sync_bin" ] || sovereign_sync_bin="$BIN_FALLBACK_DIR/sovereign-sync"
    "$sovereign_sync_bin" --mode init --config "$config_path" >/dev/null
    chmod 600 "$device_key_path"
    echo "  ✓ sovereign-sync operator namespace configured"
    echo "  ✓ sovereign-sync headless device key configured"
}

# ── Daemons in dependency order: label | probe-port | probe-path ─────────────
declare -a DAEMON_LABELS=(
    "ai.prometheus.surrealdb-native"
    "ai.prometheus.surreal-memory-native"
    "ai.prometheus.pk-cherry"
    "ai.prometheus.forge-mcp"
    "ai.prometheus.surface-bridge"
    "ai.prometheus.sovereign-sync"
)
declare -A DAEMON_PORT=(
    [ai.prometheus.surrealdb-native]=28000
    [ai.prometheus.surreal-memory-native]=23001
    [ai.prometheus.pk-cherry]=8942
    [ai.prometheus.forge-mcp]=8943
    [ai.prometheus.surface-bridge]=7890
    [ai.prometheus.sovereign-sync]=7892
)
declare -A DAEMON_PATH=(
    [ai.prometheus.surrealdb-native]=/health
    [ai.prometheus.surreal-memory-native]=/health
    [ai.prometheus.pk-cherry]=/mcp
    [ai.prometheus.forge-mcp]=/mcp
    [ai.prometheus.surface-bridge]=/health
    [ai.prometheus.sovereign-sync]=/health
)
NUDGE_LABEL="ai.prometheus.prometheus-nudge"
LEARNING_LABEL="ai.prometheus.learning-worker"
ROTATION_LABEL="ai.prometheus.hooks-logrotate"

# ── Shared template rendering (identical __PLACEHOLDER__ map for both OSes) ───
render_template() {
    local src="$1" output="$2"
    [ -f "$src" ] || { echo "Template not found: $src" >&2; return 1; }

    local pk_cherry_bin forge_bin docker_bin surreal_bin surreal_memory_bin surface_bridge_bin sovereign_sync_bin learning_worker_bin logrotate_bin flock_bin
    pk_cherry_bin="$(resolve_bin pk-cherry)";  [ -n "$pk_cherry_bin" ] || pk_cherry_bin="$BIN_FALLBACK_DIR/pk-cherry"
    forge_bin="$(resolve_bin forge)";          [ -n "$forge_bin" ]     || forge_bin="$BIN_FALLBACK_DIR/forge"
    docker_bin="$(resolve_bin docker)";        [ -n "$docker_bin" ]    || docker_bin="/usr/local/bin/docker"
    surreal_bin="$(resolve_bin surreal)";      [ -n "$surreal_bin" ]   || surreal_bin="$SURREAL_FALLBACK"
    surreal_memory_bin="$(resolve_bin surreal-memory-server)"
    [ -n "$surreal_memory_bin" ] || surreal_memory_bin="$REPO_ROOT/tools/surreal-memory-server/target/release/surreal-memory-server"
    [ -f "$surreal_memory_bin" ] || surreal_memory_bin="$BIN_FALLBACK_DIR/surreal-memory-server"
    surface_bridge_bin="$(resolve_bin surface-bridge)"; [ -n "$surface_bridge_bin" ] || surface_bridge_bin="$BIN_FALLBACK_DIR/surface-bridge"
    sovereign_sync_bin="$(resolve_bin sovereign-sync)"; [ -n "$sovereign_sync_bin" ] || sovereign_sync_bin="$BIN_FALLBACK_DIR/sovereign-sync"
    learning_worker_bin="$(resolve_bin prometheus-learning-worker)"; [ -n "$learning_worker_bin" ] || learning_worker_bin="$BIN_FALLBACK_DIR/prometheus-learning-worker"
    logrotate_bin="$(resolve_bin logrotate)"; [ -n "$logrotate_bin" ] || logrotate_bin="/opt/homebrew/opt/logrotate/sbin/logrotate"
    flock_bin="$(resolve_bin flock)"; [ -n "$flock_bin" ] || flock_bin="/usr/bin/flock"

    local device_key_file="$PROMETHEUS_HOME/.config/sovereign-sync/device-key.json"
    PROMETHEUS_DEVICE_KEY_FILE="$device_key_file" \
    PROMETHEUS_USER="$PROMETHEUS_USER" PROMETHEUS_HOME="$PROMETHEUS_HOME" \
    PROMETHEUS_ROOT="$REPO_ROOT" PROMETHEUS_LOG_DIR="$LOG_DIR" PROMETHEUS_PATH="$PROMETHEUS_PATH" \
    PK_CHERRY_BIN="$pk_cherry_bin" FORGE_BIN="$forge_bin" DOCKER_BIN="$docker_bin" \
    SURREAL_BIN="$surreal_bin" SURREAL_MEMORY_BIN="$surreal_memory_bin" SURFACE_BRIDGE_BIN="$surface_bridge_bin" \
    SOVEREIGN_SYNC_BIN="$sovereign_sync_bin" LEARNING_WORKER_BIN="$learning_worker_bin" \
    LOGROTATE_BIN="$logrotate_bin" FLOCK_BIN="$flock_bin" \
    python3 - "$src" "$output" <<'PY'
import os, pathlib, sys
from xml.sax.saxutils import escape as xml_escape

src, dst = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
text = src.read_text()

def systemd_escape(value):
    """Escape a value that will be inserted inside systemd double quotes."""
    escaped = []
    for char in value:
        if char == "\\":
            escaped.append("\\\\")
        elif char == '"':
            escaped.append('\\"')
        elif char == "%":
            escaped.append("%%")
        elif ord(char) < 0x20 or ord(char) == 0x7f:
            escaped.append(f"\\x{ord(char):02x}")
        else:
            escaped.append(char)
    return "".join(escaped)

escape_value = xml_escape if src.suffix == ".plist" else systemd_escape
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
    "__SURFACE_BRIDGE_BIN__": "SURFACE_BRIDGE_BIN",
    "__SOVEREIGN_SYNC_BIN__": "SOVEREIGN_SYNC_BIN",
    "__LEARNING_WORKER_BIN__": "LEARNING_WORKER_BIN",
    "__LOGROTATE_BIN__": "LOGROTATE_BIN",
    "__FLOCK_BIN__": "FLOCK_BIN",
    "__PROMETHEUS_DEVICE_KEY_FILE__": "PROMETHEUS_DEVICE_KEY_FILE",
}.items():
    text = text.replace(k, escape_value(os.environ[env]))
dst.write_text(text)
PY
}

render_logrotate_config() {
    local output="$1"
    python3 - "$REPO_ROOT/shared/config/logrotate.d/prometheus-hooks" "$output" "$PROMETHEUS_HOME/.prometheus/hooks.log" <<'PY'
import pathlib, sys
src, dst, hook_log = map(pathlib.Path, sys.argv[1:])
text = src.read_text().replace("__PROMETHEUS_HOOK_LOG__", str(hook_log))
dst.parent.mkdir(parents=True, exist_ok=True)
dst.write_text(text)
PY
    chmod 600 "$output"
}

if [ -n "$RENDER_ONLY_DIR" ]; then
    mkdir -p "$RENDER_ONLY_DIR"
    if ! service_is_excluded sovereign-sync; then
        render_template \
            "$REPO_ROOT/shared/launchagents/ai.prometheus.sovereign-sync.plist" \
            "$RENDER_ONLY_DIR/ai.prometheus.sovereign-sync.plist"
        render_template \
            "$REPO_ROOT/shared/systemd/ai.prometheus.sovereign-sync.service" \
            "$RENDER_ONLY_DIR/ai.prometheus.sovereign-sync.service"
    fi
    render_template "$REPO_ROOT/shared/launchagents/$LEARNING_LABEL.plist" "$RENDER_ONLY_DIR/$LEARNING_LABEL.plist"
    render_template "$REPO_ROOT/shared/launchagents/$ROTATION_LABEL.plist" "$RENDER_ONLY_DIR/$ROTATION_LABEL.plist"
    for f in "$LEARNING_LABEL.service" "$LEARNING_LABEL.path" "$LEARNING_LABEL.timer" "$ROTATION_LABEL.service" "$ROTATION_LABEL.timer"; do
        render_template "$REPO_ROOT/shared/systemd/$f" "$RENDER_ONLY_DIR/$f"
    done
    render_logrotate_config "$RENDER_ONLY_DIR/prometheus-hooks.conf"
    echo "Rendered non-excluded service definitions in $RENDER_ONLY_DIR"
    exit 0
fi

# ════════════════════════════════════════════════════════════════════════════
# macOS (launchd)
# ════════════════════════════════════════════════════════════════════════════
reload_launch_agent() {
    local label="$1" plist="$2"
    launchctl bootout "$GUI_DOMAIN/$label" >/dev/null 2>&1 || true
    for _ in 1 2 3 4 5 6 7 8 9 10; do
        if ! launchctl print "$GUI_DOMAIN/$label" >/dev/null 2>&1; then
            break
        fi
        sleep 0.1
    done
    if ! launchctl bootstrap "$GUI_DOMAIN" "$plist"; then
        # launchd may need a short interval after bootout before the label can
        # be registered again, even after it disappears from `print`.
        sleep 1
        launchctl bootstrap "$GUI_DOMAIN" "$plist"
    fi
    launchctl enable "$GUI_DOMAIN/$label"
    launchctl kickstart -k "$GUI_DOMAIN/$label"
}

reload_scheduled_launch_agent() {
    local label="$1" plist="$2"
    launchctl bootout "$GUI_DOMAIN/$label" >/dev/null 2>&1 || true
    launchctl bootstrap "$GUI_DOMAIN" "$plist"
    launchctl enable "$GUI_DOMAIN/$label"
}

macos_install() {
    $DRY_RUN || mkdir -p "$LAUNCH_AGENTS_DIR" "$LOG_DIR" "$KNOWLEDGE_DIR" \
        "$PROMETHEUS_HOME/.prometheus/logrotate" \
        "$PROMETHEUS_HOME/.prometheus/learning-queue/pending" \
        "$PROMETHEUS_HOME/.prometheus/learning-queue/processing" \
        "$PROMETHEUS_HOME/.prometheus/learning-queue/completed" \
        "$PROMETHEUS_HOME/.prometheus/learning-queue/retry" \
        "$PROMETHEUS_HOME/.prometheus/learning-queue/dead-letter" \
        "$PROMETHEUS_HOME/.prometheus/learning-queue/memory/pending" \
        "$PROMETHEUS_HOME/.prometheus/learning-queue/memory/retry"
    if ! $DRY_RUN; then
        render_logrotate_config "$PROMETHEUS_HOME/.prometheus/logrotate/prometheus-hooks.conf"
        chmod 700 "$PROMETHEUS_HOME/.prometheus" "$PROMETHEUS_HOME/.prometheus/logrotate" "$PROMETHEUS_HOME/.prometheus/learning-queue"
    fi
    if $LEARNING_RECOVERY; then
        local recovery_labels=("ai.prometheus.pk-cherry" "$LEARNING_LABEL" "$ROTATION_LABEL")
        local recovery_label recovery_src recovery_out
        for recovery_label in "${recovery_labels[@]}"; do
            recovery_src="$REPO_ROOT/shared/launchagents/$recovery_label.plist"
            recovery_out="$LAUNCH_AGENTS_DIR/$recovery_label.plist"
            echo "→ rendering $recovery_label"
            if ! $DRY_RUN; then
                render_template "$recovery_src" "$recovery_out"
                plutil -lint "$recovery_out" >/dev/null
            fi
            case "$recovery_label" in
                "$ROTATION_LABEL")
                    echo "  ↳ hook rotation: registered daily at 03:15"
                    $DRY_RUN || reload_scheduled_launch_agent "$recovery_label" "$recovery_out"
                    ;;
                "$LEARNING_LABEL")
                    echo "  ↳ learning worker: registered for queue changes and five-minute retries"
                    $DRY_RUN || reload_launch_agent "$recovery_label" "$recovery_out"
                    ;;
                *)
                    echo "  ↳ bootstrapping $recovery_label"
                    $DRY_RUN || reload_launch_agent "$recovery_label" "$recovery_out"
                    ;;
            esac
        done
        return
    fi

    # Older installers registered these daemons under com.prometheusags.*.
    # Remove them before probing ports; otherwise a healthy legacy process can
    # permanently prevent its canonical ai.prometheus.* replacement starting.
    local legacy_label legacy_plist archived_plist
    for legacy_label in com.prometheusags.surface-bridge com.prometheusags.sovereign-sync; do
        service_is_excluded "${legacy_label#com.prometheusags.}" && continue
        legacy_plist="$LAUNCH_AGENTS_DIR/$legacy_label.plist"
        if $DRY_RUN; then
            echo "[dry-run] migrate legacy service $legacy_label"
            continue
        fi
        launchctl bootout "$GUI_DOMAIN/$legacy_label" >/dev/null 2>&1 || true
        if [ -f "$legacy_plist" ]; then
            archived_plist="$legacy_plist.deprecated.$(date -u +%Y%m%dT%H%M%SZ)"
            mv "$legacy_plist" "$archived_plist"
            echo "→ archived legacy service: $archived_plist"
        fi
    done
    local all=("${DAEMON_LABELS[@]}" "$NUDGE_LABEL" "$LEARNING_LABEL" "$ROTATION_LABEL")
    for label in "${all[@]}"; do
        service_is_excluded "$label" && continue
        local src="$REPO_ROOT/shared/launchagents/$label.plist"
        local out="$LAUNCH_AGENTS_DIR/$label.plist"
        echo "→ rendering $label"
        if ! $DRY_RUN; then
            render_template "$src" "$out"
            plutil -lint "$out" >/dev/null || echo "  WARN: plist lint failed for $out" >&2
        fi
        if [ "$label" = "$NUDGE_LABEL" ]; then
            echo "  ↳ nudge: registered (fires every 4h, not at load)"
            $DRY_RUN || reload_scheduled_launch_agent "$label" "$out"
            continue
        fi
        if [ "$label" = "$ROTATION_LABEL" ]; then
            echo "  ↳ hook rotation: registered daily at 03:15"
            $DRY_RUN || reload_scheduled_launch_agent "$label" "$out"
            continue
        fi
        if [ "$label" = "$LEARNING_LABEL" ]; then
            echo "  ↳ learning worker: registered for queue changes and five-minute retries"
            $DRY_RUN || reload_launch_agent "$label" "$out"
            continue
        fi
        # Reuse if already serving on its port (any provenance), unless the
        # caller explicitly requested a definition reload after a rebuild.
        if ! $FORCE_RESTART && check_running_service "$label" "${DAEMON_PORT[$label]}" "${DAEMON_PATH[$label]}"; then
            echo "  ↳ reusing running instance — skipping bootstrap"
            continue
        fi
        echo "  ↳ bootstrapping $label"
        if ! $DRY_RUN; then
            reload_launch_agent "$label" "$out"
        fi
        echo "  ✓ $label loaded"
    done
}
macos_unload() {
    for label in "${DAEMON_LABELS[@]}" "$NUDGE_LABEL" "$LEARNING_LABEL" "$ROTATION_LABEL"; do
        service_is_excluded "$label" && continue
        run launchctl bootout "$GUI_DOMAIN/$label" >/dev/null 2>&1 || true
        echo "unloaded $label"
    done
}

# ════════════════════════════════════════════════════════════════════════════
# Linux (systemd --user)
# ════════════════════════════════════════════════════════════════════════════
linux_install() {
    command -v systemctl >/dev/null 2>&1 || { echo "systemctl not found — systemd required on Linux." >&2; exit 1; }
    $DRY_RUN || mkdir -p "$SYSTEMD_USER_DIR" "$LOG_DIR" "$KNOWLEDGE_DIR" \
        "$PROMETHEUS_HOME/.prometheus/logrotate" "$PROMETHEUS_HOME/.prometheus/learning-queue/pending" \
        "$PROMETHEUS_HOME/.prometheus/learning-queue/retry" "$PROMETHEUS_HOME/.prometheus/learning-queue/memory/pending" \
        "$PROMETHEUS_HOME/.prometheus/learning-queue/memory/retry"
    $DRY_RUN || render_logrotate_config "$PROMETHEUS_HOME/.prometheus/logrotate/prometheus-hooks.conf"

    if $LEARNING_RECOVERY; then
        local recovery_files=(
            ai.prometheus.pk-cherry.service
            "$LEARNING_LABEL.service" "$LEARNING_LABEL.path" "$LEARNING_LABEL.timer"
            "$ROTATION_LABEL.service" "$ROTATION_LABEL.timer"
        )
        local recovery_file
        for recovery_file in "${recovery_files[@]}"; do
            echo "→ rendering $recovery_file"
            $DRY_RUN || render_template "$REPO_ROOT/shared/systemd/$recovery_file" "$SYSTEMD_USER_DIR/$recovery_file"
        done
        run systemctl --user daemon-reload
        run systemctl --user enable --now ai.prometheus.pk-cherry.service \
            "$LEARNING_LABEL.path" "$LEARNING_LABEL.timer" "$ROTATION_LABEL.timer"
        return
    fi

    # Persist user services across logout / on a headless box.
    if ! loginctl show-user "$PROMETHEUS_USER" -p Linger 2>/dev/null | grep -q 'Linger=yes'; then
        run loginctl enable-linger "$PROMETHEUS_USER" || echo "  WARN: could not enable-linger (services may stop on logout)" >&2
    fi

    # Render every unit (4 daemons + nudge service + nudge timer).
    for f in "${DAEMON_LABELS[@]/%/.service}" "$NUDGE_LABEL.service" "$NUDGE_LABEL.timer"; do
        service_is_excluded "${f%.service}" && continue
        local src="$REPO_ROOT/shared/systemd/$f"
        local out="$SYSTEMD_USER_DIR/$f"
        echo "→ rendering $f"
        $DRY_RUN || render_template "$src" "$out"
    done
    for f in "$LEARNING_LABEL.service" "$LEARNING_LABEL.path" "$LEARNING_LABEL.timer" "$ROTATION_LABEL.service" "$ROTATION_LABEL.timer"; do
        local src="$REPO_ROOT/shared/systemd/$f"
        local out="$SYSTEMD_USER_DIR/$f"
        echo "→ rendering $f"
        $DRY_RUN || render_template "$src" "$out"
    done
    run systemctl --user daemon-reload

    # Daemons in dependency order, reusing anything already on its port unless
    # a rebuilt component requires a definition reload.
    for label in "${DAEMON_LABELS[@]}"; do
        service_is_excluded "$label" && continue
        if ! $FORCE_RESTART && check_running_service "$label" "${DAEMON_PORT[$label]}" "${DAEMON_PATH[$label]}"; then
            echo "  ↳ reusing running instance — skipping enable/start"
            continue
        fi
        if $FORCE_RESTART; then
            echo "  ↳ enabling + restarting $label.service"
            run systemctl --user enable "$label.service"
            run systemctl --user restart "$label.service"
        else
            echo "  ↳ enabling + starting $label.service"
            run systemctl --user enable --now "$label.service"
        fi
        echo "  ✓ $label started"
    done

    # Nudge: enable the timer (drives the oneshot every 4h), not the service.
    echo "  ↳ enabling nudge timer (fires every 4h)"
    run systemctl --user enable --now "$NUDGE_LABEL.timer"
    run systemctl --user enable --now "$LEARNING_LABEL.path" "$LEARNING_LABEL.timer" "$ROTATION_LABEL.timer"

    echo ""
    echo "All daemons installed. Verify with: bash scripts/check-mcp-health.sh"
}
linux_unload() {
    for label in "${DAEMON_LABELS[@]}"; do
        service_is_excluded "$label" && continue
        run systemctl --user disable --now "$label.service" 2>/dev/null || true
        echo "stopped $label"
    done
    run systemctl --user disable --now "$NUDGE_LABEL.timer" 2>/dev/null || true
    echo "stopped $NUDGE_LABEL.timer"
    run systemctl --user disable --now "$LEARNING_LABEL.path" "$LEARNING_LABEL.timer" "$ROTATION_LABEL.timer" 2>/dev/null || true
}

case "$OS/$ACTION" in
    macos/install) $LEARNING_RECOVERY || service_is_excluded sovereign-sync || ensure_sovereign_config; macos_install ;;
    macos/unload)  macos_unload ;;
    linux/install) $LEARNING_RECOVERY || service_is_excluded sovereign-sync || ensure_sovereign_config; linux_install ;;
    linux/unload)  linux_unload ;;
esac
