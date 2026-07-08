# Handoff: plan → execute

_Written: 2026-07-08 by kbd-plan_

## Summary

8 OpenSpec changes in 4 waves. Start with `change-prb-001-scaffold-crate`; each subsequent
change builds on the prior. Wave 3 changes (005, 006) can only start after wave 2 is done.
All tests must pass before tagging v1.6.0.

## First command

```
/kbd-apply change-prb-001-scaffold-crate
```

## Critical implementation notes

1. **Port 7891** — hard-coded default; never conflict with surface-bridge (7890) or sovereign-sync (7892)
2. **SIGTERM via `nix` crate** — gate with `#[cfg(unix)]` for portability
3. **Static assets** — use `include_bytes!()` so binary is self-contained
4. **rmcp 1.8 pattern** — follow `substrate/sovereign-sync/src/mcp_server/` exactly
5. **`cargo build --release` after each change** — never leave the build broken
