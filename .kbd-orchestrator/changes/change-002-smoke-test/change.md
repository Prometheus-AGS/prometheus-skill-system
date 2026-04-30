---
id: change-002-smoke-test
title: "End-to-end WASM smoke test + stage 6 update"
phase: phase-librefang-wasm-onramp
gaps: [G2, G3]
priority: P0
effort: XS
agent: native-tool
evolver_item_id: null
status: proposed
depends_on: [change-001-forge-package-librefang]
---

# change-002 — End-to-end smoke test + stage 6 update

## Context

Once `forge package-librefang` lands (change-001), we need:
1. An automated verification path that exercises the full pipeline
2. The `/start-business-build` skill updated to use the new subcommand (removing the manual `zip` fallback as the primary path)

`scripts/smoke-test.sh` already exists and runs `npm run doctor` checks. This change adds a `test_forge_package_librefang()` section to it.

Closing §9 criteria 4–7 requires the smoke test to pass on the development machine (librefang install step may SKIP if librefang not present — that is acceptable per the assessment verdict).

## Files to Change

| File | Action |
|------|--------|
| `scripts/smoke-test.sh` | Add `test_forge_package_librefang()` + call it |
| `skills/process/native-agent/SKILL.md` | Update stage 6: `forge package-librefang` as primary, manual zip as legacy fallback |

## Tasks

- [ ] Add `test_forge_package_librefang()` function to `scripts/smoke-test.sh` (see plan.md for full code)
- [ ] Call `test_forge_package_librefang` before the final summary block
- [ ] Update `skills/process/native-agent/SKILL.md` stage 6 section to use `forge package-librefang`
- [ ] `bash -n scripts/smoke-test.sh` → syntax OK
- [ ] `bash scripts/smoke-test.sh` → forge section PASS (or SKIP if forge not in PATH), no FAIL
- [ ] `npm run validate` → 0 errors

## Acceptance Criteria

- [ ] `bash scripts/smoke-test.sh` produces PASS or SKIP for the forge-package-librefang section (never FAIL due to design)
- [ ] Zip content check (skill.toml + .wasm) produces PASS when forge binary present and WASM pre-built
- [ ] stage 6 in `/start-business-build` shows `forge package-librefang` as the primary command
- [ ] `npm run validate` stays green
