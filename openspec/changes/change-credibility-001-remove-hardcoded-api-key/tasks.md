# Tasks: change-credibility-001-remove-hardcoded-api-key

- [ ] Confirm with user that Tavily key has been rotated at tavily.com before proceeding
- [ ] Replace hardcoded key default in `scripts/configure-mcp-all-tools.sh:25` with error-on-missing guard
- [ ] Add `gitleaks/gitleaks-action@v2` secret-scan job to `.github/workflows/validate.yml`
- [ ] Run `grep -r "tvly-" scripts/` to verify no remaining hardcoded keys
- [ ] Test script exits with non-zero code when TAVILY_API_KEY is unset
