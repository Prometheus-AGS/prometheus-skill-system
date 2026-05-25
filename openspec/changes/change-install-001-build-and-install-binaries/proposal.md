# change-install-001-build-and-install-binaries

**Phase**: machine-installation-2026-05-25  
**Status**: PENDING  
**Gaps closed**: G-BIN-1, G-BIN-2, G-SVC-3

## Summary

Build all missing project binaries and install every project binary to `~/.local/bin/`.

## Files Modified

- Shell: `cp` commands only — no source files changed
- Builds: `tools/prometheus-knowledge/`, `tools/liter-llm/`, `tools/forge-rs/`

## Acceptance Criteria

- `which pk-mcp` → `~/.local/bin/pk-mcp`
- `which liter-llm` → `~/.local/bin/liter-llm`
- `which forge-mcp` → `~/.local/bin/forge-mcp`
- `which prometheus` → `~/.local/bin/prometheus`
- `which forge` → `~/.local/bin/forge`
