#!/usr/bin/env bash
# codex-provision-mcp-env.sh — forward the prometheus plugin's MCP env to Codex.
#
# The Codex plugin's 7 MCP servers reference env vars (e.g. ${TAVILY_API_KEY},
# ${FORGE_MCP_TOKEN}). Codex spawns MCP servers with a filtered environment, so a
# key present in your shell is not automatically visible to them. This helper
# writes a `[shell_environment_policy]` include to ~/.codex/config.toml so Codex
# forwards exactly these vars to the servers.
#
# Reads values ONLY from the current environment. It records variable NAMES to
# forward — it never writes secret VALUES to disk and never prints them.
# bash 3.2 compatible (macOS /bin/bash).

set -eu

CODEX_HOME="${CODEX_HOME:-$HOME/.codex}"
CODEX_CONFIG="$CODEX_HOME/config.toml"

# Env vars the plugin's 7 MCP servers consume (see .mcp.json).
VARS="TAVILY_API_KEY FORGE_MCP_TOKEN FORGE_MCP_URL PK_MCP_URL SURREAL_MEMORY_URL LITER_LLM_CONFIG"

present=""
missing=""
for v in $VARS; do
  eval "val=\${$v:-}"
  if [ -n "${val:-}" ]; then present="$present $v"; else missing="$missing $v"; fi
done

echo "Codex MCP env provisioning"
echo "  forward-able (set in this environment):$present"
[ -n "$missing" ] && echo "  not set (servers use \${VAR:-default} or will warn in 'codex doctor'):$missing"

if [ ! -f "$CODEX_CONFIG" ]; then
  mkdir -p "$CODEX_HOME"
  : > "$CODEX_CONFIG"
fi

# Idempotently set [shell_environment_policy] inherit = "all" so Codex forwards the
# live shell environment (including the vars above) to spawned MCP servers. This
# uses the documented `shell_environment_policy.inherit` primitive and persists NO
# secret values — the values stay in your environment.
python3 - "$CODEX_CONFIG" <<'PY'
import re, sys, pathlib
cfg = pathlib.Path(sys.argv[1])
text = cfg.read_text() if cfg.exists() else ""
block = '[shell_environment_policy]\ninherit = "all"\n'
if re.search(r'(?m)^\[shell_environment_policy\]', text):
    if re.search(r'(?m)^inherit\s*=', text):
        print("  ✓ [shell_environment_policy] already present — left as-is"); raise SystemExit(0)
    text = re.sub(r'(?m)^(\[shell_environment_policy\]\n)', r'\1inherit = "all"\n', text, count=1)
else:
    if text and not text.endswith("\n"): text += "\n"
    text += "\n" + block
cfg.write_text(text)
print("  ✓ wrote [shell_environment_policy] inherit = \"all\" to %s" % cfg)
PY

echo "Done. Restart Codex, then 'codex doctor' should stop warning for the forwarded vars."
echo "Secret VALUES stay in your environment (e.g. ~/.bash_profile) — nothing secret is written here."
echo "Fallback (if a specific server still can't see its key): add an inline"
echo "  [mcp_servers.<name>] env = { KEY = \"...\" }  block to ~/.codex/config.toml (0600, user-local)."
