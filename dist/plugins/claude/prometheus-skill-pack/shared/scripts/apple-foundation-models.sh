#!/bin/bash
# apple-foundation-models.sh — bring up Apple's on-device Foundation Model
# (Apple Intelligence, ~3B parameters) as a supervised local OpenAI-compatible
# endpoint, and register it with liter-llm and the KBD `local` role.
#
#   --detect     JSON eligibility report; changes nothing
#   --install    on an eligible Mac: brew install apfel (if absent), install and
#                start the ai.prometheus.apple-fm LaunchAgent, register the
#                liter-llm provider and the KBD `local` role (only if absent)
#   --check      JSON health of the running service
#   --uninstall  stop and remove the LaunchAgent (the apfel formula is kept)
#   --quiet      print nothing when the machine is ineligible or disabled
#
# This is an optional capability (integration-contract rule 2): absence is the
# normal case, and every failure path warns and exits 0 so an installer can never
# be aborted by it. Apple Intelligence itself is a system setting; this script
# reports how to enable it and never changes it.
#
# Environment: PROMETHEUS_APPLE_FM=0 disables it; PROMETHEUS_APPLE_FM_PORT
# (default 11437 — Ollama owns apfel's default 11434);
# PROMETHEUS_APPLE_FM_FORCE_INELIGIBLE=1 simulates an ineligible machine.
# Bash 3.2 compatible (no mapfile, no associative arrays).
set -u

MODE=""; QUIET=0
for arg in "$@"; do
  case "$arg" in
    --detect|--install|--check|--uninstall) MODE="${arg#--}" ;;
    --quiet) QUIET=1 ;;
    -h|--help) sed -n '2,24p' "$0"; exit 0 ;;
    *) echo "apple-foundation-models: unknown argument $arg" >&2; exit 2 ;;
  esac
done
[ -n "$MODE" ] || { echo "usage: apple-foundation-models.sh --detect|--install|--check|--uninstall [--quiet]" >&2; exit 2; }

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PORT="${PROMETHEUS_APPLE_FM_PORT:-11437}"
LABEL="ai.prometheus.apple-fm"
AGENTS_DIR="$HOME/Library/LaunchAgents"
PLIST="$AGENTS_DIR/$LABEL.plist"
LOG_DIR="$HOME/.prometheus/logs"
MODEL_ID="apple-foundationmodel"

say()  { [ "$QUIET" -eq 1 ] || echo "$*"; }
warn() { echo "apple-foundation-models: $*" >&2; }
json_str() { printf '"%s"' "$(printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g')"; }

find_apfel() {
  local c
  c="$(command -v apfel 2>/dev/null || true)"
  [ -n "$c" ] && { echo "$c"; return 0; }
  for c in /opt/homebrew/bin/apfel /usr/local/bin/apfel; do [ -x "$c" ] && { echo "$c"; return 0; }; done
  return 1
}

# Sets ELIGIBLE (1/0), REASON, MACOS, ARCH, APFEL, MODEL_STATUS.
detect() {
  ELIGIBLE=0; REASON=""; MODEL_STATUS="unknown"; APFEL=""
  MACOS="$(sw_vers -productVersion 2>/dev/null || echo none)"; ARCH="$(uname -m)"
  if [ "${PROMETHEUS_APPLE_FM:-1}" = "0" ]; then REASON="disabled (PROMETHEUS_APPLE_FM=0)"; return; fi
  if [ "${PROMETHEUS_APPLE_FM_FORCE_INELIGIBLE:-0}" = "1" ]; then REASON="forced ineligible (PROMETHEUS_APPLE_FM_FORCE_INELIGIBLE=1)"; return; fi
  [ "$(uname -s)" = "Darwin" ] || { REASON="not macOS"; return; }
  [ "$ARCH" = "arm64" ] || { REASON="requires Apple Silicon (arch $ARCH)"; return; }
  [ "${MACOS%%.*}" -ge 26 ] 2>/dev/null || { REASON="requires macOS 26 or newer (found $MACOS)"; return; }
  APFEL="$(find_apfel || true)"
  if [ -n "$APFEL" ]; then
    "$APFEL" --model-info >/dev/null 2>&1; case $? in
      0) MODEL_STATUS="available" ;;
      5) MODEL_STATUS="unavailable"; REASON="Apple Intelligence is off or its model is not ready: System Settings > Apple Intelligence & Siri" ; return ;;
      *) MODEL_STATUS="error" ;;
    esac
  fi
  ELIGIBLE=1; REASON="eligible"
}

detect_json() {
  printf '{"eligible":%s,"reason":%s,"macos":%s,"arch":%s,"apfel":%s,"model":%s,"port":%s}\n' \
    "$([ "$ELIGIBLE" -eq 1 ] && echo true || echo false)" "$(json_str "$REASON")" "$(json_str "$MACOS")" \
    "$(json_str "$ARCH")" "$(json_str "${APFEL:-}")" "$(json_str "$MODEL_STATUS")" "$PORT"
}

render_plist() {
  mkdir -p "$AGENTS_DIR" "$LOG_DIR"
  local path_value="/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin"
  sed -e "s|__PROMETHEUS_ROOT__|$ROOT|g" -e "s|__PROMETHEUS_HOME__|$HOME|g" \
      -e "s|__PROMETHEUS_USER__|${USER:-$(id -un)}|g" -e "s|__PROMETHEUS_PATH__|$path_value|g" \
      -e "s|__PROMETHEUS_LOG_DIR__|$LOG_DIR|g" -e "s|__APPLE_FM_PORT__|$PORT|g" \
      "$ROOT/shared/launchagents/$LABEL.plist" > "$PLIST.tmp" && mv "$PLIST.tmp" "$PLIST"
}

health() { curl -fsS -m 3 "http://127.0.0.1:$PORT/health" >/dev/null 2>&1; }

start_service() {
  local domain="gui/$(id -u)"
  launchctl bootout "$domain/$LABEL" >/dev/null 2>&1 || true
  launchctl bootstrap "$domain" "$PLIST" >/dev/null 2>&1 || launchctl kickstart -k "$domain/$LABEL" >/dev/null 2>&1 || true
  local i=0
  while [ "$i" -lt 30 ]; do health && return 0; sleep 1; i=$((i + 1)); done
  return 1
}

register() {
  local bridge="$ROOT/skills/process/liter-llm-bridge/scripts/configure-models.sh"
  if [ -f "$bridge" ] && [ -f "${LITER_LLM_CONFIG:-$HOME/.config/liter-llm/liter-llm-proxy.toml}" ]; then
    PROMETHEUS_APPLE_FM_PORT="$PORT" bash "$bridge" add-provider apple-fm >/dev/null 2>&1 \
      || warn "liter-llm registration failed; run: bash $bridge add-provider apple-fm"
    launchctl kickstart -k "gui/$(id -u)/ai.prometheus.liter-llm-api" >/dev/null 2>&1 || true
  else
    say "  liter-llm config absent; skipped provider registration (run /liter-llm-bridge configure, then add-provider apple-fm)"
  fi
  local models="$HOME/.prometheus/kbd/models.toml"
  if [ -f "$models" ] && ! grep -Eq '^[[:space:]]*local[[:space:]]*=' "$models"; then
    if grep -q '^\[roles\]' "$models"; then
      awk -v id="$MODEL_ID" '{print} /^\[roles\]/ && !done {print "# Apple on-device Foundation Model (4,096-token window). Small private/offline work only; never judge or critic."; print "local = \"" id "\""; done=1}' "$models" > "$models.tmp" && mv "$models.tmp" "$models"
    fi
  fi
}

case "$MODE" in
  detect) detect; detect_json; exit 0 ;;
  check)
    detect
    up=false; listed=false
    health && up=true
    curl -fsS -m 3 "http://127.0.0.1:$PORT/v1/models" 2>/dev/null | grep -q "$MODEL_ID" && listed=true
    printf '{"eligible":%s,"reason":%s,"serviceUp":%s,"modelListed":%s,"port":%s,"plist":%s}\n' \
      "$([ "$ELIGIBLE" -eq 1 ] && echo true || echo false)" "$(json_str "$REASON")" "$up" "$listed" "$PORT" \
      "$([ -f "$PLIST" ] && json_str "$PLIST" || echo null)"
    exit 0 ;;
  uninstall)
    launchctl bootout "gui/$(id -u)/$LABEL" >/dev/null 2>&1 || true
    rm -f "$PLIST"
    say "Apple Foundation Models service removed (apfel formula kept)."
    exit 0 ;;
  install)
    detect
    if [ "$ELIGIBLE" -ne 1 ]; then
      say "Apple Foundation Models: skipped — $REASON"   # silent under --quiet (optional capability)
      exit 0
    fi
    if [ -z "$APFEL" ]; then
      if command -v brew >/dev/null 2>&1; then
        say "Apple Foundation Models: installing apfel (Homebrew core)…"
        brew install apfel >/dev/null 2>&1 || { warn "brew install apfel failed; install it manually: brew install apfel"; exit 0; }
        APFEL="$(find_apfel || true)"
      else
        warn "Homebrew not found; install apfel (https://apfel.franzai.com) to enable Apple Foundation Models"; exit 0
      fi
      [ -n "$APFEL" ] || { warn "apfel not found after install"; exit 0; }
      detect
      [ "$ELIGIBLE" -eq 1 ] || { warn "$REASON"; exit 0; }
    fi
    render_plist || { warn "could not write $PLIST"; exit 0; }
    if ! start_service; then warn "service did not become healthy on :$PORT; see $LOG_DIR/apple-fm.stderr.log"; exit 0; fi
    register
    say "Apple Foundation Models: serving $MODEL_ID on http://127.0.0.1:$PORT/v1 ($("$APFEL" --version 2>/dev/null | head -1))"
    exit 0 ;;
esac
