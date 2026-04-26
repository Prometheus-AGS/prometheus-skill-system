#!/usr/bin/env bash
# configure-mcp.sh — manage liter-llm config and harness MCP registration.
#
# Subcommands:
#   write-toml <small-alias> <medium-alias> <frontier-alias>
#       Write/merge ~/.config/liter-llm/config.toml with the given aliases.
#       Each alias is a "provider/model" identifier, e.g. "anthropic/claude-sonnet-4-6".
#
#   register [--harness <name>]
#       Auto-detect the active harness (or use --harness override) and add a
#       liter-llm MCP server entry to its config. Prints the path written.
#       Supported: claude-code, opencode, cursor, codex.
#
#   status
#       Report current liter-llm config + which harnesses have it registered.
#
# All writes are merge-style — never overwrite user customizations under
# [providers.*] in liter-llm config.

set -euo pipefail

CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/liter-llm"
CONFIG_FILE="$CONFIG_DIR/config.toml"

cmd_write_toml() {
  local small="${1:?small alias required}"
  local medium="${2:?medium alias required}"
  local frontier="${3:?frontier alias required}"

  mkdir -p "$CONFIG_DIR"

  if [[ ! -f "$CONFIG_FILE" ]]; then
    cat > "$CONFIG_FILE" <<EOF
# liter-llm config — managed by liter-llm-bridge skill.
# Edit [providers.*] freely; [aliases] is overwritten on each \`configure\` run.

[aliases]
small    = "$small"
medium   = "$medium"
frontier = "$frontier"
EOF
    echo "Wrote new config: $CONFIG_FILE"
  else
    # Merge: replace [aliases] block, preserve everything else
    local tmp
    tmp="$(mktemp)"
    awk -v s="$small" -v m="$medium" -v f="$frontier" '
      BEGIN { in_aliases = 0; printed = 0 }
      /^\[aliases\]/ {
        in_aliases = 1
        if (!printed) {
          print "[aliases]"
          print "small    = \"" s "\""
          print "medium   = \"" m "\""
          print "frontier = \"" f "\""
          printed = 1
        }
        next
      }
      /^\[/ && in_aliases { in_aliases = 0 }
      !in_aliases { print }
    ' "$CONFIG_FILE" > "$tmp"

    if ! grep -q "^\[aliases\]" "$tmp"; then
      printf '\n[aliases]\nsmall    = "%s"\nmedium   = "%s"\nfrontier = "%s"\n' \
        "$small" "$medium" "$frontier" >> "$tmp"
    fi
    mv "$tmp" "$CONFIG_FILE"
    echo "Updated aliases in: $CONFIG_FILE"
  fi
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
    cat > "$mcp_file" <<'EOF'
{
  "mcpServers": {
    "liter-llm": {
      "command": "liter-llm",
      "args": ["mcp", "--transport", "stdio"]
    }
  }
}
EOF
    echo "$mcp_file"
  else
    # Use python for safe JSON merge
    python3 - "$mcp_file" <<'PYEOF'
import json, sys
path = sys.argv[1]
with open(path) as f:
    cfg = json.load(f)
cfg.setdefault("mcpServers", {})
cfg["mcpServers"]["liter-llm"] = {
    "command": "liter-llm",
    "args": ["mcp", "--transport", "stdio"],
}
with open(path, "w") as f:
    json.dump(cfg, f, indent=2)
PYEOF
    echo "$mcp_file"
  fi
}

register_opencode() {
  local f="$HOME/.config/opencode/config.json"
  mkdir -p "$(dirname "$f")"
  [[ -f "$f" ]] || echo '{}' > "$f"
  python3 - "$f" <<'PYEOF'
import json, sys
path = sys.argv[1]
with open(path) as f:
    cfg = json.load(f)
cfg.setdefault("mcp_servers", {})
cfg["mcp_servers"]["liter-llm"] = {
    "command": "liter-llm",
    "args": ["mcp", "--transport", "stdio"],
}
with open(path, "w") as f:
    json.dump(cfg, f, indent=2)
PYEOF
  echo "$f"
}

register_cursor() {
  local f="$HOME/.cursor/mcp.json"
  mkdir -p "$(dirname "$f")"
  [[ -f "$f" ]] || echo '{"mcpServers":{}}' > "$f"
  python3 - "$f" <<'PYEOF'
import json, sys
path = sys.argv[1]
with open(path) as f:
    cfg = json.load(f)
cfg.setdefault("mcpServers", {})
cfg["mcpServers"]["liter-llm"] = {
    "command": "liter-llm",
    "args": ["mcp", "--transport", "stdio"],
}
with open(path, "w") as f:
    json.dump(cfg, f, indent=2)
PYEOF
  echo "$f"
}

register_codex() {
  local f="$HOME/.config/codex/config.toml"
  mkdir -p "$(dirname "$f")"
  if ! grep -q "^\[mcp.servers.liter-llm\]" "$f" 2>/dev/null; then
    cat >> "$f" <<'EOF'

[mcp.servers.liter-llm]
command = "liter-llm"
args = ["mcp", "--transport", "stdio"]
EOF
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
    echo "--- aliases ---"
    awk '/^\[aliases\]/,/^\[/' "$CONFIG_FILE" | grep -v "^\[" || true
  else
    echo "(not configured)"
  fi
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
