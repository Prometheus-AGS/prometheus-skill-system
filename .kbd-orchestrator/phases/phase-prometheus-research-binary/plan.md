# Plan — phase-prometheus-research-binary

_Generated: 2026-07-08_

## Overview

8 changes in 4 waves. Waves 1-2 can overlap; wave 3 requires wave 1-2 complete;
wave 4 requires wave 3 complete.

Backend: **OpenSpec** (detected `openspec/` directory at repo root).

## Change Order

### Wave 1 — Foundation (no dependencies)

| Change | Goals | Effort | Agent |
|--------|-------|--------|-------|
| `change-prb-001-scaffold-crate` | G-01 | M | general-purpose |
| `change-prb-002-cli-subcommands` | G-02, G-03, G-04 | L | general-purpose |

**Ordering rationale:** Both changes are independent of each other (001 creates the crate
skeleton, 002 fills in the job module). In practice, 002 depends on 001 completing first
since it adds to the scaffolded src tree.

### Wave 2 — Server layers (depends on wave 1)

| Change | Goals | Effort | Agent |
|--------|-------|--------|-------|
| `change-prb-003-mcp-server` | G-05 | L | general-purpose |
| `change-prb-004-http-sse-server` | G-06 (backend) | L | general-purpose |

003 and 004 are independent of each other and can be implemented in parallel.

### Wave 3 — UI + infrastructure (depends on wave 2)

| Change | Goals | Effort | Agent |
|--------|-------|--------|-------|
| `change-prb-005-a2ui-components` | G-06 (UI side) | XL | general-purpose |
| `change-prb-006-launchd-plist` | G-07 | S | general-purpose |

005 depends on 004's component stub route. 006 only requires 003 (MCP binary exists).

### Wave 4 — Verification + release (depends on wave 3)

| Change | Goals | Effort | Agent |
|--------|-------|--------|-------|
| `change-prb-007-tests` | G-01–G-06 (coverage) | M | general-purpose |
| `change-prb-008-tag-v160` | G-08 | S | general-purpose |

008 depends on 007 (all tests pass before tagging).

## First change to apply

```
/kbd-apply change-prb-001-scaffold-crate
```

## OpenSpec change references

- `openspec/changes/change-prb-001-scaffold-crate/`
- `openspec/changes/change-prb-002-cli-subcommands/`
- `openspec/changes/change-prb-003-mcp-server/`
- `openspec/changes/change-prb-004-http-sse-server/`
- `openspec/changes/change-prb-005-a2ui-components/`
- `openspec/changes/change-prb-006-launchd-plist/`
- `openspec/changes/change-prb-007-tests/`
- `openspec/changes/change-prb-008-tag-v160/`

## Key constraints

- Port **7891** only — never 7890 (surface-bridge) or 7892 (sovereign-sync)
- `nix` crate for SIGTERM — macOS/Linux only; guard with `#[cfg(unix)]`
- Static assets vendored via `include_bytes!` — binary is self-contained, no runtime file deps
- `cargo build --release` must pass after every change before marking done
- No `unwrap()` in production code paths — use `?` and `anyhow`
- Follow sovereign-sync's exact rmcp 1.8 pattern for MCP tools
