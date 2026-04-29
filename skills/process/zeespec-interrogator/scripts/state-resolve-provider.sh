#!/usr/bin/env bash
# state-resolve-provider.sh — Resolve the active state provider for ZeeSpec
# Output: JSON provider configuration to stdout
# Exit 0 = resolved

set -euo pipefail

resolve_provider() {
  if [ -n "${ZEESPEC_PROVIDER_CONFIG:-}" ] && [ -f "$ZEESPEC_PROVIDER_CONFIG" ]; then
    cat "$ZEESPEC_PROVIDER_CONFIG"
    return 0
  fi

  if [ -f ".zeespec-provider.json" ]; then
    cat ".zeespec-provider.json"
    return 0
  fi

  local global_config="${HOME}/.zeespec/provider.json"
  if [ -f "$global_config" ]; then
    cat "$global_config"
    return 0
  fi

  if [ -f ".mcp.json" ]; then
    local has_state_server
    has_state_server=$(python3 -c "
import json, sys
try:
    cfg = json.load(open('.mcp.json'))
    servers = cfg.get('mcpServers', {})
    for name in servers:
        if 'state' in name.lower() or 'zeespec' in name.lower():
            print(json.dumps({'provider_type': 'mcp_tool', 'config': {'server_name': name}}))
            sys.exit(0)
except: pass
sys.exit(1)
" 2>/dev/null) && {
      echo "$has_state_server"
      return 0
    }
  fi

  if [ -f ".mcp.json" ]; then
    local has_memory
    has_memory=$(python3 -c "
import json, sys
try:
    cfg = json.load(open('.mcp.json'))
    servers = cfg.get('mcpServers', {})
    for name in servers:
        if 'memory' in name.lower():
            print(json.dumps({'provider_type': 'agent_memory', 'config': {'memory_server': name, 'entity_prefix': 'zeespec'}}))
            sys.exit(0)
except: pass
sys.exit(1)
" 2>/dev/null) && {
      echo "$has_memory"
      return 0
    }
  fi

  echo '{"provider_type": "filesystem", "config": {"state_directory": ".zeespec", "scope": "project"}}'
  return 0
}

resolve_provider
