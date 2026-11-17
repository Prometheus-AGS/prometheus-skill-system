#!/usr/bin/env bash
# configure-mcp.sh — manage liter-llm config and harness MCP registration.
#
# Subcommands:
#   write-toml    REMOVED — see configure-models.sh (this now exits 2 with guidance)
#
#   register [--harness <name>]
#       Auto-detect the active harness (or use --harness override) and add a
#       liter-llm MCP server entry to its config. Prints the path written.
#       Supported: claude-code, opencode, cursor, codex.
#       The entry runs the pack's launcher, ~/.local/bin/liter-llm-mcp, never
#       the binary directly: the launcher sources ~/.prometheus/kbd/secrets.env
#       first, so the server starts in a harness launched without the keys in
#       its environment. No secret is written to any harness config.
#
#   status
#       Report current liter-llm config + which harnesses have it registered.
#
# All writes are merge-style — never overwrite user customizations under
# [providers.*] in liter-llm config.

set -euo pipefail

CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/liter-llm"
# liter-llm's REAL config. `config.toml` (flat [aliases]) is retired — it is not a
# schema liter-llm can load.
CONFIG_FILE="${LITER_LLM_CONFIG:-$CONFIG_DIR/liter-llm-proxy.toml}"
LEGACY_CONFIG="$CONFIG_DIR/config.toml"
DEFAULT_CONFIG_FILE="$HOME/.config/liter-llm/liter-llm-proxy.toml"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LAUNCHER="$HOME/.local/bin/liter-llm-mcp"
LAUNCHER_NAME="shared/scripts/liter-llm-mcp-launch.sh"

# ensure_launcher — make sure the MCP launcher is installed, copying it from the
# first place this skill can find it: the loading plugin, the active pack
# generation, or the checkout this script runs from.
ensure_launcher() {
  [[ -x "$LAUNCHER" ]] && return 0
  local source=""
  local candidate
  for candidate in \
      "${CLAUDE_PLUGIN_ROOT:+$CLAUDE_PLUGIN_ROOT/$LAUNCHER_NAME}" \
      "${PROMETHEUS_PLUGIN_ROOT:-$HOME/.prometheus/plugins/prometheus-skill-pack}/current/$LAUNCHER_NAME" \
      "$SCRIPT_DIR/../../../../$LAUNCHER_NAME"; do
    if [[ -n "$candidate" && -f "$candidate" ]]; then
      source="$candidate"
      break
    fi
  done
  if [[ -z "$source" ]]; then
    echo "ERROR: the liter-llm MCP launcher is not installed at $LAUNCHER and no copy" >&2
    echo "       of $LAUNCHER_NAME was found to install it from." >&2
    echo "       Run scripts/install-binaries.sh from the Prometheus skill pack checkout." >&2
    return 5
  fi
  mkdir -p "$(dirname "$LAUNCHER")"
  cp "$source" "$LAUNCHER.tmp.$$" && chmod 755 "$LAUNCHER.tmp.$$" && mv "$LAUNCHER.tmp.$$" "$LAUNCHER"
}

# A non-default config path cannot be an argument any more (the launcher takes
# none), so it travels as LITER_LLM_CONFIG in the entry's env. A path is not a
# secret. Empty when the default config is in use.
custom_config() {
  [[ "$CONFIG_FILE" == "$DEFAULT_CONFIG_FILE" ]] || printf '%s' "$CONFIG_FILE"
}

cmd_write_toml() {
  # REMOVED. This used to write ~/.config/liter-llm/config.toml with a flat
  # [aliases] table of small/medium/frontier.
  #
  # That file shape is NOT a schema liter-llm can load: its real config is
  # liter-llm-proxy.toml, where [[aliases]] is an ARRAY OF TABLES keyed by
  # `pattern`. A flat table parsed to nothing, so every caller silently fell back
  # to sending the class name ("frontier") as a model id. Generating that file was
  # the root cause of the adversarial judge never reaching a second model.
  #
  # Fail loudly rather than keep producing a config that cannot work.
  cat >&2 <<'MSG'
configure-mcp.sh write-toml has been REMOVED.

It wrote ~/.config/liter-llm/config.toml with a flat [aliases] table — a shape
liter-llm cannot load, which silently broke model resolution.

Use instead:
  bash scripts/configure-models.sh repair              # create/repair valid config
  bash scripts/configure-models.sh add-provider <name> # local-proxy|kimi|minimax|qwen|glm|glm-coding
  bash scripts/configure-models.sh verify              # live completion per role
  bash scripts/configure-models.sh migrate             # retire the old config.toml

Roles live in ~/.prometheus/kbd/models.toml; model definitions in
~/.config/liter-llm/liter-llm-proxy.toml. See
skills/process/adversarial-review/references/model-configuration.md
MSG
  return 2
}

detect_harness() {
  if [[ -n "${CLAUDE_CODE_SESSION_ID:-}${CLAUDECODE:-}" ]]; then echo "claude-code"; return; fi
  if [[ -n "${OPENCODE_SESSION:-}" ]]; then echo "opencode"; return; fi
  if [[ -n "${CURSOR_SESSION:-}${CURSOR_AGENT:-}" ]]; then echo "cursor"; return; fi
  if [[ -n "${CODEX_SESSION:-}" ]]; then echo "codex"; return; fi
  echo "unknown"
}

register_claude_code() {
  local mcp_file="$HOME/.claude/mcp_servers.json"
  mkdir -p "$(dirname "$mcp_file")"
  if [[ ! -f "$mcp_file" ]]; then
    echo '{"mcpServers":{}}' > "$mcp_file"
  fi
  write_json_entry "$mcp_file" mcpServers
}

# write_json_entry <file> <key> — set <key>."liter-llm" to the launcher entry.
write_json_entry() {
  LITER_LAUNCHER="$LAUNCHER" LITER_CUSTOM_CFG="$(custom_config)" python3 - "$1" "$2" <<'PYEOF'
import json, os, sys
path, key = sys.argv[1], sys.argv[2]
with open(path) as f:
    cfg = json.load(f)
entry = {"command": os.environ["LITER_LAUNCHER"]}
if os.environ.get("LITER_CUSTOM_CFG"):
    entry["env"] = {"LITER_LLM_CONFIG": os.environ["LITER_CUSTOM_CFG"]}
cfg.setdefault(key, {})
cfg[key]["liter-llm"] = entry
with open(path, "w") as f:
    json.dump(cfg, f, indent=2)
PYEOF
  echo "$1"
}

register_opencode() {
  local f="$HOME/.config/opencode/config.json"
  mkdir -p "$(dirname "$f")"
  [[ -f "$f" ]] || echo '{}' > "$f"
  write_json_entry "$f" mcp_servers
}

register_cursor() {
  local f="$HOME/.cursor/mcp.json"
  mkdir -p "$(dirname "$f")"
  [[ -f "$f" ]] || echo '{"mcpServers":{}}' > "$f"
  write_json_entry "$f" mcpServers
}

register_codex() {
  local f="$HOME/.config/codex/config.toml"
  mkdir -p "$(dirname "$f")"
  if ! grep -q "^\[mcp.servers.liter-llm\]" "$f" 2>/dev/null; then
    local custom
    custom="$(custom_config)"
    {
      printf '\n[mcp.servers.liter-llm]\ncommand = "%s"\n' "$LAUNCHER"
      [[ -z "$custom" ]] || printf 'env = { LITER_LLM_CONFIG = "%s" }\n' "$custom"
    } >> "$f"
  fi
  echo "$f"
}

cmd_register() {
  local harness=""
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --harness) harness="$2"; shift 2 ;;
      *) echo "Unknown arg: $1" >&2; exit 2 ;;
    esac
  done

  [[ -z "$harness" ]] && harness="$(detect_harness)"

  case "$harness" in
    claude-code|opencode|cursor|codex) ensure_launcher || exit $? ;;
  esac

  case "$harness" in
    claude-code) register_claude_code ;;
    opencode)    register_opencode ;;
    cursor)      register_cursor ;;
    codex)       register_codex ;;
    unknown)
      echo "ERROR: could not detect harness. Re-run with --harness <name>" >&2
      echo "       Supported: claude-code, opencode, cursor, codex" >&2
      exit 3
      ;;
    *)
      echo "ERROR: unknown harness: $harness" >&2
      exit 4
      ;;
  esac
}

cmd_status() {
  echo "liter-llm config: $CONFIG_FILE"
  if [[ -f "$CONFIG_FILE" ]]; then
    echo "--- [[models]] names ---"
    grep -E '^[[:space:]]*name[[:space:]]*=' "$CONFIG_FILE" | sed -E 's/.*"([^"]*)".*/  \1/' || true
    grep -qE '^[[:space:]]*master_key[[:space:]]*=' "$CONFIG_FILE" \
      || echo "  WARNING: no [general] master_key — every /v1/* request will 401"
  else
    echo "(not configured — run: bash scripts/configure-models.sh repair)"
  fi
  if [[ -f "$LEGACY_CONFIG" ]]; then
    echo
    echo "NOTE: retired $LEGACY_CONFIG still present."
    echo "      Retire it with: bash scripts/configure-models.sh migrate"
  fi
  echo
  if [[ -x "$LAUNCHER" ]]; then
    # --check runs the launcher's own start-up checks without starting the server.
    if "$LAUNCHER" --check 2>&1 | sed 's/^/  /'; then :; fi
  else
    echo "  WARNING: MCP launcher not installed at $LAUNCHER — run 'register' or scripts/install-binaries.sh"
  fi
  echo
  echo "Roles: ${PROMETHEUS_KBD_MODELS_CONFIG:-$HOME/.prometheus/kbd/models.toml}"
  echo
  echo "Harness registrations:"
  for h in "$HOME/.claude/mcp_servers.json" \
           "$HOME/.config/opencode/config.json" \
           "$HOME/.cursor/mcp.json" \
           "$HOME/.config/codex/config.toml"; do
    if [[ -f "$h" ]] && grep -q "liter-llm" "$h" 2>/dev/null; then
      echo "  ✓ $h"
    fi
  done
}

main() {
  local sub="${1:-}"
  shift || true
  case "$sub" in
    write-toml) cmd_write_toml "$@" ;;
    register)   cmd_register "$@" ;;
    status)     cmd_status ;;
    "")
      echo "Usage: $0 {write-toml|register|status} [args...]" >&2
      exit 2
      ;;
    *)
      echo "Unknown subcommand: $sub" >&2
      exit 2
      ;;
  esac
}

main "$@"
