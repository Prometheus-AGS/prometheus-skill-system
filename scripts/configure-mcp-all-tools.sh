#!/usr/bin/env bash
# configure-mcp-all-tools.sh — idempotent MCP config writer for all supported AI tools.
#
# Reads service definitions from scripts/mcp-port-table.json and merges them
# into each installed tool's native config file without overwriting existing entries.
#
# Supported tools:
#   claude-code   ~/.claude/mcp.json              (JSON, mcpServers)
#   opencode      ~/.opencode/opencode.json        (JSON, mcp)
#   codex         ~/.codex/config.toml             (TOML, [mcp_servers.*])
#   kimi-code     ~/.kimi-code/config.toml         (TOML, [mcp_servers.*])
#   minimax       ~/.minimax/mcp/mcp.json          (JSON, mcpServers)
#   cursor        ~/.cursor/mcp.json               (JSON, mcpServers)
#   windsurf      ~/.codeium/windsurf/mcp/mcp.json (JSON, mcpServers) [if installed]
#
# Usage:
#   bash scripts/configure-mcp-all-tools.sh [--dry-run] [--tool <name>]

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PORT_TABLE="$REPO_ROOT/scripts/mcp-port-table.json"
DRY_RUN=false
ONLY_TOOL=""
if [ -z "${TAVILY_API_KEY:-}" ]; then
  echo "Error: TAVILY_API_KEY environment variable is required." >&2
  echo "  Set it before running: TAVILY_API_KEY=<your-key> bash scripts/configure-mcp-all-tools.sh" >&2
  echo "  Obtain a key at: https://tavily.com" >&2
  exit 1
fi
FIRECRAWL_API_URL="${FIRECRAWL_API_URL:-https://api.firecrawl.dev}"
FIRECRAWL_API_KEY="${FIRECRAWL_API_KEY:-}"

while [ "$#" -gt 0 ]; do
    case "$1" in
        --dry-run)   DRY_RUN=true; shift ;;
        --tool)      ONLY_TOOL="${2:?missing tool name}"; shift 2 ;;
        --help|-h)   grep '^#' "$0" | sed 's/^# \?//'; exit 0 ;;
        *)           echo "Unknown argument: $1" >&2; exit 2 ;;
    esac
done

[ -f "$PORT_TABLE" ] || { echo "mcp-port-table.json not found at $PORT_TABLE" >&2; exit 1; }
command -v node >/dev/null 2>&1 || { echo "node is required for JSON manipulation" >&2; exit 1; }

RESULTS=()
SKIPPED=()

# ---------------------------------------------------------------------------
# JSON tools (claude-code, opencode, minimax, cursor, windsurf)
# ---------------------------------------------------------------------------

# merge_json_mcp <config_file> <mcp_key> <tool_label>
# Merges HTTP+stdio entries into a JSON config file's mcpServers / mcp block.
merge_json_mcp() {
    local config_file="$1"
    local mcp_key="$2"    # "mcpServers" or "mcp"
    local tool_label="$3"

    if [ ! -f "$config_file" ]; then
        SKIPPED+=("$tool_label (config not found: $config_file)")
        return 0
    fi

    if $DRY_RUN; then
        echo "  [dry-run] would merge into $config_file ($mcp_key)"
        RESULTS+=("$tool_label — dry-run")
        return 0
    fi

    local backup="${config_file}.bak.$(date -u '+%Y%m%d%H%M%S')"
    cp "$config_file" "$backup"

    local node_out
    node_out="$(node - "$config_file" "$mcp_key" "$PORT_TABLE" "$TAVILY_API_KEY" "$FIRECRAWL_API_KEY" <<'JS'
const fs = require('fs');
const [configPath, mcpKey, tablePath, tavilyKey, firecrawlKey] = process.argv.slice(2);

let config = {};
try { config = JSON.parse(fs.readFileSync(configPath, 'utf8')); } catch(e) {}
if (!config[mcpKey]) config[mcpKey] = {};

const table = JSON.parse(fs.readFileSync(tablePath, 'utf8')).services;

// OpenCode uses its own MCP schema (key "mcp"): { type: "remote"|"local", enabled, ... }
// rather than the Claude-style sse/http/stdio used by every other JSON tool.
const isOpencode = mcpKey === 'mcp';
const resolveEnv = (env) => {
    const out = {};
    for (const [k, v] of Object.entries(env)) {
        out[k] = v.replace('${TAVILY_API_KEY}', tavilyKey).replace('${FIRECRAWL_API_KEY}', firecrawlKey);
    }
    return out;
};

const added = [];
for (const [name, svc] of Object.entries(table)) {
    if (config[mcpKey][name]) continue;

    if (svc.type === 'sse' || svc.type === 'http') {
        config[mcpKey][name] = isOpencode
            ? { type: 'remote', url: svc.url, enabled: true }
            : { type: svc.type, url: svc.url }; // sse = SSE, http = Streamable HTTP
    } else if (isOpencode) {
        const entry = { type: 'local', command: [svc.command, ...(svc.args || [])], enabled: true };
        if (svc.env) entry.environment = resolveEnv(svc.env);
        config[mcpKey][name] = entry;
    } else {
        const entry = { type: 'stdio', command: svc.command };
        if (svc.args && svc.args.length > 0) entry.args = svc.args;
        if (svc.env) entry.env = resolveEnv(svc.env);
        config[mcpKey][name] = entry;
    }
    added.push(name);
}

fs.writeFileSync(configPath, JSON.stringify(config, null, 2) + '\n', 'utf8');
process.stdout.write(added.length > 0 ? 'added: ' + added.join(', ') : 'already complete');
JS
    )"
    echo "  $node_out"
    RESULTS+=("$tool_label — $node_out")
}

# ---------------------------------------------------------------------------
# TOML tools (codex, kimi-code) — use bash string append (idempotent grep)
# ---------------------------------------------------------------------------

# merge_toml_mcp <config_file> <tool_label>
# Appends missing [mcp_servers.*] stanzas. Safe: checks before appending.
merge_toml_mcp() {
    local config_file="$1"
    local tool_label="$2"

    if [ ! -f "$config_file" ]; then
        SKIPPED+=("$tool_label (config not found: $config_file)")
        return 0
    fi

    if $DRY_RUN; then
        echo "  [dry-run] would merge into $config_file"
        RESULTS+=("$tool_label — dry-run")
        return 0
    fi

    local backup="${config_file}.bak.$(date -u '+%Y%m%d%H%M%S')"
    cp "$config_file" "$backup"

    local added=()

    # surreal-memory (SSE)
    if ! grep -q "\[mcp_servers.surreal-memory\]" "$config_file" 2>/dev/null; then
        cat >> "$config_file" <<'TOML'

[mcp_servers.surreal-memory]
type = "sse"
url = "http://localhost:23001/mcp/sse"
TOML
        added+=("surreal-memory")
    fi

    # prometheus-knowledge (Streamable HTTP)
    if ! grep -q "\[mcp_servers.prometheus-knowledge\]" "$config_file" 2>/dev/null; then
        cat >> "$config_file" <<'TOML'

[mcp_servers.prometheus-knowledge]
type = "http"
url = "http://localhost:8942/mcp"
TOML
        added+=("prometheus-knowledge")
    fi

    # forge-rs (Streamable HTTP)
    if ! grep -q "\[mcp_servers.forge-rs\]" "$config_file" 2>/dev/null; then
        cat >> "$config_file" <<'TOML'

[mcp_servers.forge-rs]
type = "http"
url = "http://localhost:8943/mcp"
TOML
        added+=("forge-rs")
    fi

    # sycophancy-correction (stdio)
    if ! grep -q "\[mcp_servers.sycophancy-correction\]" "$config_file" 2>/dev/null; then
        cat >> "$config_file" <<'TOML'

[mcp_servers.sycophancy-correction]
command = "/usr/local/bin/sycophancy-correction"
TOML
        added+=("sycophancy-correction")
    fi

    # liter-llm (stdio)
    if ! grep -q "\[mcp_servers.liter-llm\]" "$config_file" 2>/dev/null; then
        cat >> "$config_file" <<TOML

[mcp_servers.liter-llm]
command = "liter-llm"
args = ["mcp", "--transport", "stdio"]
TOML
        added+=("liter-llm")
    fi

    # sequential-thinking (stdio)
    if ! grep -q "\[mcp_servers.sequential-thinking\]" "$config_file" 2>/dev/null; then
        cat >> "$config_file" <<'TOML'

[mcp_servers.sequential-thinking]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-sequential-thinking"]
TOML
        added+=("sequential-thinking")
    fi

    # tavily (stdio)
    if ! grep -q "\[mcp_servers.tavily\]" "$config_file" 2>/dev/null; then
        cat >> "$config_file" <<TOML

[mcp_servers.tavily]
command = "npx"
args = ["-y", "tavily-mcp"]

[mcp_servers.tavily.env]
TAVILY_API_KEY = "$TAVILY_API_KEY"
TOML
        added+=("tavily")
    fi

    # firecrawl (stdio)
    if ! grep -q "\[mcp_servers.firecrawl\]" "$config_file" 2>/dev/null; then
        cat >> "$config_file" <<TOML

[mcp_servers.firecrawl]
command = "npx"
args = ["-y", "firecrawl-mcp"]

[mcp_servers.firecrawl.env]
FIRECRAWL_API_URL = "https://api.firecrawl.dev"
FIRECRAWL_API_KEY = "$FIRECRAWL_API_KEY"
TOML
        added+=("firecrawl")
    fi

    if [ "${#added[@]}" -gt 0 ]; then
        RESULTS+=("$tool_label — added: ${added[*]}")
    else
        RESULTS+=("$tool_label — already complete")
    fi
}

# ---------------------------------------------------------------------------
# Configure each tool
# ---------------------------------------------------------------------------

should_run() {
    [ -z "$ONLY_TOOL" ] || [ "$ONLY_TOOL" = "$1" ]
}

echo "Configuring MCP servers across all AI tools..."
echo "Source: $PORT_TABLE"
echo ""

# Claude Code — user-level mcp.json
should_run "claude-code" && {
    echo "→ claude-code (~/.claude/mcp.json)"
    merge_json_mcp "$HOME/.claude/mcp.json" "mcpServers" "claude-code"
}

# OpenCode — opencode.json uses "mcp" key
should_run "opencode" && {
    echo "→ opencode (~/.opencode/opencode.json)"
    merge_json_mcp "$HOME/.opencode/opencode.json" "mcp" "opencode"
}

# Codex — TOML
should_run "codex" && {
    echo "→ codex (~/.codex/config.toml)"
    merge_toml_mcp "$HOME/.codex/config.toml" "codex"
}

# Kimi Code — TOML
should_run "kimi-code" && {
    echo "→ kimi-code (~/.kimi-code/config.toml)"
    merge_toml_mcp "$HOME/.kimi-code/config.toml" "kimi-code"
}

# MiniMax — JSON
should_run "minimax" && {
    echo "→ minimax (~/.minimax/mcp/mcp.json)"
    merge_json_mcp "$HOME/.minimax/mcp/mcp.json" "mcpServers" "minimax"
}

# Cursor — JSON
should_run "cursor" && {
    echo "→ cursor (~/.cursor/mcp.json)"
    merge_json_mcp "$HOME/.cursor/mcp.json" "mcpServers" "cursor"
}

# Windsurf — JSON (optional, only if installed)
should_run "windsurf" && {
    echo "→ windsurf (~/.codeium/windsurf/mcp/mcp.json)"
    merge_json_mcp "$HOME/.codeium/windsurf/mcp/mcp.json" "mcpServers" "windsurf"
}

# ---------------------------------------------------------------------------
# Report
# ---------------------------------------------------------------------------
echo ""
echo "Results:"
for r in "${RESULTS[@]}"; do
    echo "  ✓ $r"
done
if [ "${#SKIPPED[@]}" -gt 0 ]; then
    echo ""
    echo "Skipped (tool not installed):"
    for s in "${SKIPPED[@]}"; do
        echo "  ○ $s"
    done
fi
echo ""
echo "Run 'bash scripts/check-mcp-health.sh' to verify HTTP services."
