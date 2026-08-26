#!/usr/bin/env bash
# Manage Prometheus MCP services as macOS user LaunchAgents.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ACTION="${1:-}"
shift || true

# Identity defaults come from config/defaults.env so the three installer scripts
# cannot drift apart again. This previously hard-coded a personal username as the
# default and then enforced the match below, which meant the script refused to
# run for any other operator.
# shellcheck source=../config/defaults.env
. "$REPO_ROOT/config/defaults.env"
ALLOW_USER_OVERRIDE=false
EXCLUDED_SERVICES=""

while [ "$#" -gt 0 ]; do
    case "$1" in
        --user)
            PROMETHEUS_USER="${2:-}"
            [ -n "$PROMETHEUS_USER" ] || { echo "Missing value for --user" >&2; exit 2; }
            ALLOW_USER_OVERRIDE=true
            shift 2
            ;;
        --exclude)
            [ "$#" -ge 2 ] || { echo "Missing value for --exclude" >&2; exit 2; }
            EXCLUDED_SERVICES="$EXCLUDED_SERVICES${2#service:}
"
            shift 2
            ;;
        --help|-h)
            ACTION="help"
            shift
            ;;
        *)
            echo "Unknown argument: $1" >&2
            exit 2
            ;;
    esac
done

service_is_excluded() {
    local name="${1#ai.prometheus.}"
    printf '%s' "$EXCLUDED_SERVICES" | grep -qx "$name"
}

LABELS=("ai.prometheus.surrealdb-native" "ai.prometheus.surreal-memory-native" "ai.prometheus.pk-cherry" "ai.prometheus.forge-mcp" "ai.prometheus.prometheus-nudge")
TEMPLATES=("ai.prometheus.surrealdb-native.plist" "ai.prometheus.surreal-memory-native.plist" "ai.prometheus.pk-cherry.plist" "ai.prometheus.forge-mcp.plist" "ai.prometheus.prometheus-nudge.plist")
# ai.prometheus.exec is intentionally absent from LABELS/TEMPLATES: it is a
# socket daemon rendered by scripts/install-prometheus-exec-service.sh, which
# owns its identity/version/hash contract. It IS reported here so `doctor` and
# `status` never show a silently-missing execution engine.
DOCTOR_LABELS=("${LABELS[@]}" "ai.prometheus.learning-worker" "ai.prometheus.hooks-logrotate" "ai.prometheus.exec")

usage() {
    cat <<'EOF'
Usage: scripts/prometheus-services.sh <command> [--user <account>] [--exclude service]

Commands:
  install   Render LaunchAgent plists into ~/Library/LaunchAgents
  load      Bootstrap, enable, and kickstart the LaunchAgents
  unload    Boot out the LaunchAgents
  status    Show launchctl state and MCP HTTP probes
  doctor    Report OS, user, binaries, Docker, plist, launchctl, and MCP readiness
  logs      Tail recent service logs

Managed LaunchAgents:
  ai.prometheus.surrealdb-native       SurrealDB 3.2.4 native binary on 127.0.0.1:28000
  ai.prometheus.surreal-memory-native  Native surreal-memory-server -> SurrealDB 3.2.4 (port 23001)
  ai.prometheus.pk-cherry              pk-cherry HTTP MCP for Karpathy KB (port 8942)
  ai.prometheus.forge-mcp              Forge code-enrichment MCP (port 8943)
  ai.prometheus.prometheus-nudge       Periodic self-learning nudge (every 4h, cron-style)

Stdio-only services (managed by AI client, not launchd):
  sycophancy-correction, liter-llm, sequential-thinking, tavily

For a full health table: bash scripts/check-mcp-health.sh
For idempotent full install: bash scripts/install-mcp-services.sh
EOF
}

require_macos() {
    if [ "$(uname -s)" != "Darwin" ]; then
        echo "Prometheus LaunchAgents are only supported on macOS (Darwin)." >&2
        exit 1
    fi
}

user_home() {
    local user="$1"
    local home
    home=$(dscl . -read "/Users/$user" NFSHomeDirectory 2>/dev/null | awk '{print $2}')
    if [ -z "$home" ]; then
        home=$(eval "printf '%s' ~$user")
    fi
    printf '%s' "$home"
}

init_user() {
    PROMETHEUS_HOME="$(user_home "$PROMETHEUS_USER")"
    if [ -z "$PROMETHEUS_HOME" ] || [ "$PROMETHEUS_HOME" = "~$PROMETHEUS_USER" ]; then
        echo "Could not resolve home directory for user $PROMETHEUS_USER." >&2
        exit 1
    fi

    PROMETHEUS_UID="$(id -u "$PROMETHEUS_USER" 2>/dev/null || true)"
    if [ -z "$PROMETHEUS_UID" ]; then
        echo "Could not resolve uid for user $PROMETHEUS_USER." >&2
        exit 1
    fi

    local current_user
    current_user="$(id -un)"
    if [ "$current_user" != "$PROMETHEUS_USER" ] && [ "$ALLOW_USER_OVERRIDE" != true ]; then
        echo "Run this as $PROMETHEUS_USER so the LaunchAgents attach to that GUI session." >&2
        echo "Current user: $current_user. To inspect another user explicitly, pass --user $PROMETHEUS_USER." >&2
        exit 1
    fi

    LAUNCH_AGENTS_DIR="$PROMETHEUS_HOME/Library/LaunchAgents"
    LOG_DIR="$PROMETHEUS_HOME/.prometheus/logs"
    KNOWLEDGE_DIR="$PROMETHEUS_HOME/.prometheus/knowledge"
    GUI_DOMAIN="gui/$PROMETHEUS_UID"
    PROMETHEUS_PATH="/usr/local/bin:/usr/local/sbin:/opt/homebrew/bin:/opt/homebrew/sbin:$PROMETHEUS_HOME/.cargo/bin:$PROMETHEUS_HOME/.local/bin:/usr/bin:/bin:/usr/sbin:/sbin"
}

resolve_bin() {
    local name="$1"
    PATH="$PROMETHEUS_PATH" command -v "$name" 2>/dev/null || true
}

ensure_dirs() {
    mkdir -p "$LAUNCH_AGENTS_DIR" "$LOG_DIR" "$KNOWLEDGE_DIR"
}

render_template() {
    local template="$1"
    local output="$2"
    local pk_cherry_bin forge_bin docker_bin surreal_bin surreal_memory_bin surreal_mlx_executor

    pk_cherry_bin="$(resolve_bin pk-cherry)"
    forge_bin="$(resolve_bin forge)"
    docker_bin="$(resolve_bin docker)"
    surreal_bin="$(resolve_bin surreal)"
    surreal_memory_bin="$(resolve_bin surreal-memory-server)"
    surreal_mlx_executor="$(resolve_bin surreal-memory-mlx-executor)"

    [ -n "$pk_cherry_bin" ] || pk_cherry_bin="/usr/local/bin/pk-cherry"
    [ -n "$forge_bin" ] || forge_bin="/usr/local/bin/forge"
    [ -n "$docker_bin" ] || docker_bin="/usr/local/bin/docker"
    [ -n "$surreal_bin" ] || surreal_bin="/usr/local/bin/surreal"
    [ -n "$surreal_memory_bin" ] || surreal_memory_bin="/usr/local/bin/surreal-memory-server"
    [ -n "$surreal_mlx_executor" ] || surreal_mlx_executor="/usr/local/bin/surreal-memory-mlx-executor"
    local local_embedding_backend="${PROMETHEUS_LOCAL_EMBEDDING_BACKEND:-mlx}"
    local local_embedding_device="${PROMETHEUS_LOCAL_EMBEDDING_DEVICE:-auto}"
    case "$local_embedding_backend" in candle|mlx) ;; *) echo "PROMETHEUS_LOCAL_EMBEDDING_BACKEND must be candle or mlx" >&2; return 1 ;; esac
    case "$local_embedding_device" in auto|cpu) ;; *) echo "PROMETHEUS_LOCAL_EMBEDDING_DEVICE must be auto or cpu" >&2; return 1 ;; esac

    python3 - "$REPO_ROOT/shared/launchagents/$template" "$output" <<PY
import pathlib
import sys

src = pathlib.Path(sys.argv[1])
dst = pathlib.Path(sys.argv[2])
text = src.read_text()
replacements = {
    "__PROMETHEUS_USER__": "$PROMETHEUS_USER",
    "__PROMETHEUS_HOME__": "$PROMETHEUS_HOME",
    "__PROMETHEUS_ROOT__": "$REPO_ROOT",
    "__PROMETHEUS_LOG_DIR__": "$LOG_DIR",
    "__PROMETHEUS_PATH__": "$PROMETHEUS_PATH",
    "__PK_CHERRY_BIN__": "$pk_cherry_bin",
    "__FORGE_BIN__": "$forge_bin",
    "__DOCKER_BIN__": "$docker_bin",
    "__SURREAL_BIN__": "$surreal_bin",
    "__SURREAL_MEMORY_BIN__": "$surreal_memory_bin",
    "__SURREAL_MLX_EXECUTOR__": "$surreal_mlx_executor",
    "__LOCAL_EMBEDDING_BACKEND__": "$local_embedding_backend",
    "__LOCAL_EMBEDDING_DEVICE__": "$local_embedding_device",
}
for key, value in replacements.items():
    text = text.replace(key, value)
dst.write_text(text)
PY
}

plist_path() {
    local label="$1"
    printf '%s/%s.plist' "$LAUNCH_AGENTS_DIR" "$label"
}

install_services() {
    require_macos
    init_user
    ensure_dirs

    for i in "${!LABELS[@]}"; do
        local label="${LABELS[$i]}"
        service_is_excluded "$label" && continue
        local template="${TEMPLATES[$i]}"
        local out
        out="$(plist_path "$label")"
        render_template "$template" "$out"
        plutil -lint "$out"
        echo "Installed $out"
    done

    echo "Logs: $LOG_DIR"
    echo "Knowledge dir: $KNOWLEDGE_DIR"
}

load_services() {
    require_macos
    init_user

    if ! launchctl print "$GUI_DOMAIN" >/dev/null 2>&1; then
        echo "Cannot access launchd GUI domain $GUI_DOMAIN. Is $PROMETHEUS_USER logged in?" >&2
        exit 1
    fi

    for label in "${LABELS[@]}"; do
        service_is_excluded "$label" && continue
        local plist
        plist="$(plist_path "$label")"
        [ -f "$plist" ] || { echo "$plist missing; run install first." >&2; exit 1; }
        launchctl bootout "$GUI_DOMAIN/$label" >/dev/null 2>&1 || true
        launchctl bootstrap "$GUI_DOMAIN" "$plist"
        launchctl enable "$GUI_DOMAIN/$label"
        # Nudge is cron-style (RunAtLoad=false) — don't kickstart it immediately
        if [ "$label" != "ai.prometheus.prometheus-nudge" ]; then
            launchctl kickstart -k "$GUI_DOMAIN/$label"
        fi
        echo "Loaded $label"
    done
}

unload_services() {
    require_macos
    init_user
    for label in "${LABELS[@]}"; do
        service_is_excluded "$label" && continue
        launchctl bootout "$GUI_DOMAIN/$label" >/dev/null 2>&1 || true
        echo "Unloaded $label"
    done
}

probe() {
    local label="$1"
    local url="$2"
    local code
    code=$(curl -s -o /dev/null -w '%{http_code}' --connect-timeout 1 --max-time 2 "$url" 2>/dev/null || true)
    [ -n "$code" ] || code="000"
    printf '%-28s %s %s\n' "$label" "$code" "$url"
}

print_launchctl_summary() {
    local label="$1"
    if launchctl print "$GUI_DOMAIN/$label" >/dev/null 2>&1; then
        local state pid exit_status
        state=$(launchctl print "$GUI_DOMAIN/$label" 2>/dev/null | awk -F'= ' '/state =/{print $2; exit}')
        pid=$(launchctl print "$GUI_DOMAIN/$label" 2>/dev/null | awk -F'= ' '/pid =/{print $2; exit}')
        exit_status=$(launchctl print "$GUI_DOMAIN/$label" 2>/dev/null | awk -F'= ' '/last exit code =/{print $2; exit}')
        printf '%-28s loaded state=%s pid=%s last_exit=%s\n' "$label" "${state:-unknown}" "${pid:-n/a}" "${exit_status:-n/a}"
    else
        printf '%-28s not loaded\n' "$label"
    fi
}

status_services() {
    require_macos
    init_user
    for label in "${LABELS[@]}"; do
        service_is_excluded "$label" && continue
        print_launchctl_summary "$label"
    done
    echo ""
    probe "surrealdb"            "http://127.0.0.1:28000/health"
    probe "surreal-memory"       "http://localhost:23001/health"
    probe "prometheus-knowledge" "http://localhost:8942/mcp"
    probe "forge-rs"             "http://localhost:8943/mcp"
    # socket daemon — no HTTP port; a live daemon leaves a socket node
    local exec_sock="${PROMETHEUS_EXEC_SOCKET:-$PROMETHEUS_HOME/.prometheus/run/prometheus-exec.sock}"
    printf '%-28s %s %s\n' "prometheus-exec" \
        "$([ -S "$exec_sock" ] && echo listening || echo down)" "$exec_sock"
    # stdio-only services — no HTTP probe
    printf '%-28s %s\n' "sycophancy-correction" "stdio-only"
    printf '%-28s %s\n' "liter-llm"             "stdio-only"
    printf '%-28s %s\n' "sequential-thinking"   "stdio-only"
    printf '%-28s %s\n' "tavily"                "stdio-only"
}

doctor_services() {
    require_macos
    init_user
    echo "OS: $(uname -s) $(sw_vers -productVersion 2>/dev/null || true)"
    echo "User: $(id -un) (target: $PROMETHEUS_USER, uid: $PROMETHEUS_UID)"
    echo "Home: $PROMETHEUS_HOME"
    echo "Repo: $REPO_ROOT"
    echo "GUI domain: $GUI_DOMAIN"
    echo "PATH: $PROMETHEUS_PATH"
    echo ""

    for bin in surreal prometheus prometheus-exec pk pk-cherry prometheus-learning-worker surreal-memory-server logrotate flock curl plutil launchctl; do
        local found
        found="$(resolve_bin "$bin")"
        printf '%-14s %s\n' "$bin" "${found:-missing}"
    done
    echo ""

    if command -v docker >/dev/null 2>&1; then
        docker ps --format 'docker container: {{.Names}} {{.Ports}}' 2>/dev/null | grep -E 'surreal|23001' || echo "docker surreal-memory: not running"
    else
        echo "docker: missing"
    fi
    echo ""

    for label in "${DOCTOR_LABELS[@]}"; do
        service_is_excluded "$label" && continue
        local plist
        plist="$(plist_path "$label")"
        if [ -f "$plist" ]; then
            plutil -lint "$plist"
        else
            echo "$plist missing"
        fi
        print_launchctl_summary "$label"
    done
    echo ""
    status_services
    probe "surreal-memory-ready" "http://localhost:23001/ready"
    if command -v prometheus >/dev/null 2>&1; then
        prometheus learning status --json
    else
        echo "prometheus learning status: unavailable"
    fi
}

logs_services() {
    require_macos
    init_user
    for file in "$LOG_DIR"/*.log; do
        [ -f "$file" ] || continue
        echo "==> $file <=="
        tail -n 40 "$file"
    done
}

case "$ACTION" in
    install) install_services ;;
    load) load_services ;;
    unload) unload_services ;;
    status) status_services ;;
    doctor) doctor_services ;;
    logs) logs_services ;;
    help|"") usage ;;
    *)
        echo "Unknown command: $ACTION" >&2
        usage >&2
        exit 2
        ;;
esac
