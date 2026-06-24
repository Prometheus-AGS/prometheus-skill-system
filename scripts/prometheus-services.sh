#!/usr/bin/env bash
# Manage Prometheus MCP services as macOS user LaunchAgents.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ACTION="${1:-}"
shift || true

PROMETHEUS_USER="${PROMETHEUS_USER:-gqadonis}"
ALLOW_USER_OVERRIDE=false

while [ "$#" -gt 0 ]; do
    case "$1" in
        --user)
            PROMETHEUS_USER="${2:-}"
            [ -n "$PROMETHEUS_USER" ] || { echo "Missing value for --user" >&2; exit 2; }
            ALLOW_USER_OVERRIDE=true
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

LABELS=("ai.prometheus.surreal-memory-docker" "ai.prometheus.pk-cherry" "ai.prometheus.forge-mcp" "ai.prometheus.prometheus-nudge")
TEMPLATES=("ai.prometheus.surreal-memory-docker.plist" "ai.prometheus.pk-cherry.plist" "ai.prometheus.forge-mcp.plist" "ai.prometheus.prometheus-nudge.plist")

usage() {
    cat <<'EOF'
Usage: scripts/prometheus-services.sh <command> [--user gqadonis]

Commands:
  install   Render LaunchAgent plists into ~/Library/LaunchAgents
  load      Bootstrap, enable, and kickstart the LaunchAgents
  unload    Boot out the LaunchAgents
  status    Show launchctl state and MCP HTTP probes
  doctor    Report OS, user, binaries, Docker, plist, launchctl, and MCP readiness
  logs      Tail recent service logs

Managed LaunchAgents:
  ai.prometheus.surreal-memory-docker  Docker keepalive for surreal-memory-server (port 23001)
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
    PROMETHEUS_PATH="/usr/local/bin:/opt/homebrew/bin:$PROMETHEUS_HOME/.cargo/bin:$PROMETHEUS_HOME/.local/bin:/usr/bin:/bin:/usr/sbin:/sbin"
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
    local pk_cherry_bin forge_bin docker_bin

    pk_cherry_bin="$(resolve_bin pk-cherry)"
    forge_bin="$(resolve_bin forge)"
    docker_bin="$(resolve_bin docker)"

    [ -n "$pk_cherry_bin" ] || pk_cherry_bin="/usr/local/bin/pk-cherry"
    [ -n "$forge_bin" ] || forge_bin="/usr/local/bin/forge"
    [ -n "$docker_bin" ] || docker_bin="/usr/local/bin/docker"

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
        launchctl bootout "$GUI_DOMAIN/$label" >/dev/null 2>&1 || true
        echo "Unloaded $label"
    done
}

probe() {
    local label="$1"
    local url="$2"
    local code
    code=$(curl -sI -o /dev/null -w '%{http_code}' --connect-timeout 1 --max-time 2 "$url" 2>/dev/null || true)
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
        print_launchctl_summary "$label"
    done
    echo ""
    probe "surreal-memory"       "http://localhost:23001/health"
    probe "prometheus-knowledge" "http://localhost:8942/mcp"
    probe "forge-rs"             "http://localhost:8943/mcp"
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

    for bin in pk-cherry forge docker curl plutil launchctl; do
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

    for label in "${LABELS[@]}"; do
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
