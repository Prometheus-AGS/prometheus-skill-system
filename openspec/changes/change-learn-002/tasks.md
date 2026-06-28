# Tasks — change-learn-002

- [ ] Document surface tier signals per harness (Claude Code, OpenCode, Codex, Kimi, Zed) — env vars, file markers, API availability
- [ ] Write `shared/scripts/detect-surface-tier.sh` that probes signals and exits with `SURFACE_TIER=0|1|2`
- [ ] Test probe script on each supported harness and record actual outputs
- [ ] Document `SURFACE_TIER` env var convention and expected consumer behavior at each tier
