---
id: change-credibility-001-remove-hardcoded-api-key
title: Remove hardcoded Tavily API key + add gitleaks CI scanner
phase: phase-credibility-closure
priority: P0
effort: S
wave: 1
agent: claude
status: done
gap_id: P0-A
verdict: BUILD+ADOPT
library: gitleaks v8
scope:
  - scripts/configure-mcp-all-tools.sh
  - .github/workflows/validate.yml
  - .gitignore
---

# change-credibility-001 — Remove hardcoded Tavily API key + add gitleaks CI scanner

## Context

`scripts/configure-mcp-all-tools.sh:25` contains a hardcoded Tavily API key as a default fallback:
```bash
TAVILY_API_KEY="${TAVILY_API_KEY:-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}"
```

This was committed with PR #13 and exists in git history. This is a P0 security finding confirmed by the 2026-06-29 independent credibility assessment.

**PREREQUISITE:** User must rotate the key at tavily.com BEFORE pushing this change. The code change removes the default; the key must already be inert at push time.

## Scope

1. Replace hardcoded key default with empty string + error message in `configure-mcp-all-tools.sh`
2. Add `gitleaks` secret scanning step to CI (`validate.yml`)
3. Add `gitleaks.toml` allowlist config if needed for any false positives

## Implementation Notes

In `configure-mcp-all-tools.sh:25`, replace:
```bash
TAVILY_API_KEY="${TAVILY_API_KEY:-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}"
```
With:
```bash
if [ -z "${TAVILY_API_KEY:-}" ]; then
  echo "Error: TAVILY_API_KEY environment variable is required" >&2
  exit 1
fi
```

In `validate.yml`, add a `secret-scan` job using `gitleaks/gitleaks-action@v2` (MIT).

## Verification

- `grep -r "tvly-" scripts/` → returns no matches
- CI secret-scan job exits 0 on a clean run
- Script errors cleanly when TAVILY_API_KEY is unset
