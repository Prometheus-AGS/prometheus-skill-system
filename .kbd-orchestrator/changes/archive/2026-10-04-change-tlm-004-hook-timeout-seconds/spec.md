# change-tlm-004-hook-timeout-seconds

**Title:** Correct hook timeout units: both harnesses read seconds (reverses the PR #121 misdiagnosis)
**Repository:** `prometheus-skill-pack`
**Phase:** team-aware-learning-memory
**Depends on:** none
**Backend:** native-kbd
**Base branch:** a worktree branch off `origin/main` at or after `20d97f2` (contains PRs #111, #121–#123), at `/Users/gqadonis/Projects/prometheus/worktrees/tlm-004`. Never this checkout's `codex/delivery-cadence-recovery` branch, which lacks #121. Delivered through a pull request.

## Why

Claude Code and Codex both read hook `timeout` in **seconds**. Sources: code.claude.com/docs/en/hooks.md ("Seconds before canceling", default 600); learn.chatgpt.com/docs/hooks.md ("`timeout` is in seconds", default 600); and the codex-cli 0.158 binary schema.

`shared/harnesses/hook-contract.json` uses 1000–35000 everywhere, so hooks run with about 17-minute to 10-hour timeouts.

PR #121 added `CODEX_MIN_HOOK_TIMEOUT_MS = 5000` and documented Codex as using milliseconds. That came from a misread probe: the failing hook was the one whose stdout began with JSON. Its failure persisted after the floor was added and cleared only when the JSON was removed. The floor and the documentation are wrong (assessment section I).

## What Changes

- Convert every `timeout` in `shared/harnesses/hook-contract.json` from milliseconds to seconds as `max(10, ceil(value / 1000))`. The 10-second floor covers the about 1 s `hook-entry.mjs` startup (observed 2026-10-03), so kbd-control, precompact-kbd-control and the blocking taskcompleted-kbd-receipt gate are not cut to 1 s. The floor applies **only to this one-time conversion**. The standing generator rule is 1 ≤ timeout ≤ 600 seconds, so later hooks such as the ≤5 s delivery hooks in change-tlm-002 stay valid. Kimi/OpenCode-specific budgets (`controlTimeoutMs`) stay as they are, because they are a different field with different semantics.
- Remove `CODEX_MIN_HOOK_TIMEOUT_MS` from `scripts/lib/hook-config.js` and its use in `scripts/generate-harness-adapters.js`. Codex receives the same seconds as Claude.
- In the generator, fail when a contract `timeout` is below 1 or exceeds 600 seconds (the harness default), so a millisecond-sized value cannot creep back in.
- Correct the Codex hooks paragraph in `CLAUDE.md` and the Hooks section of `docs/codex-plugin.md` (constraint C-03).
- Regenerate with `node scripts/generate-harness-adapters.js` then `node scripts/generate-skill-system-distribution.js`: `hooks/hooks.json`, `hooks/codex-hooks.json`, `shared/harnesses/generated/*`, `shared/scripts/generated/hook-dispatch-v1.sh` and `dist/plugins/*` (C-01). Generation must be idempotent (C-04).

## Scope

- `shared/harnesses/hook-contract.json`
- `scripts/lib/hook-config.js`
- `scripts/generate-harness-adapters.js`
- `scripts/tests/hook-dispatch.test.mjs`
- `CLAUDE.md`
- `docs/codex-plugin.md`
- `hooks/hooks.json`, `hooks/codex-hooks.json`
- `shared/harnesses/generated/*`, `shared/scripts/generated/hook-dispatch-v1.sh`
- `dist/plugins/claude/prometheus-skill-pack/**`, `dist/plugins/codex/prometheus-skill-pack/**`

## Bundle identity

Changing the contract changes the hook `bundleId`. Following the existing practice of PRs #121 and #123, this PR regenerates the bundle-bearing files. Signed plugin-generation publication and the 14-target receipt check happen at the next release or install, not in this PR.

## Out of scope (carried to the revised plan)

- prometheus-skills-mini `hooks/hooks.json` has the same defect. It is fixed in the mini PR of the revised plan.
- SubagentStop bare-name matcher namespacing is part of the identity work (revised plan, memory-loop PR).

## Capabilities

- `hook-contract` (modified): `timeout` is seconds for all command-hook harnesses.

## MODIFIED Requirements

### Requirement: hook timeouts are expressed in seconds
Every command hook in the generated Claude and Codex hook files SHALL carry `timeout` in seconds, no greater than 600, and identical for both harnesses.

#### Scenario: a millisecond value is rejected
- WHEN the contract declares `"timeout": 5000`
- THEN the generator fails with a message naming the hook and the 600-second ceiling
