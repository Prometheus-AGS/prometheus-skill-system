---
id: SP-009
title: pk lint --fix scheduled job
status: ready
priority: P2
estimated_effort: 0.5d
agent_role: hooks-engineer
depends_on: []
unblocks: []
related: [SP-006]
created_from_conversation_turn: 3-4
---

# SP-009 — pk lint --fix scheduled job

## Problem

`pk lint` exists in the prometheus-knowledge CLI and detects DUPLICATE entries (and likely other quality issues). `pk-lint-cron.sh` exists in `shared/scripts/` but is unwired — no Stop hook entry runs it, no system cron invokes it. The detection capability is dormant.

## Evidence

1. `cargo run --bin pk -- lint --help` shows the lint command exists.
2. `ls shared/scripts/pk-lint-cron.sh` shows the cron script exists.
3. `grep -r 'pk-lint-cron' .claude-plugin/ hooks/` returns nothing — not registered.

## Why it matters

Knowledge-base quality decays without periodic linting. Duplicate entries proliferate as the librarian ingests overlapping content. Stale entries linger past their useful life. Running lint reactively (only when the user manually fires it) means it never runs.

## Proposed fix

Register `pk-lint-cron.sh` in the Stop chain to run *at most once per week per project*. The script itself reads a timestamp file (`.prometheus/last-lint.txt` in project root) and exits early if the last run was within 7 days.

When it runs, it executes `pk lint --fix --auto-confirm-low-risk` which:
- Auto-removes exact-duplicate entries.
- Flags near-duplicates for human review (writes `.prometheus/lint-pending.md` with the candidates).
- Logs to `~/.prometheus/hooks.log` per SP-006.

The user reviews `lint-pending.md` at their leisure; auto-fix never deletes anything that requires judgment.

## Trade-offs and risks

- **Risk: auto-fix is too aggressive and removes useful duplicate entries.** Mitigation: "low-risk" means string-identical content with same metadata; anything with different timestamps, source-projects, or content variations is marked pending, not removed.
- **Cost: weekly run on a busy KB takes 30-60s.** Stop-chain timing isn't a primary concern (Stop runs after the user is already done); this is acceptable.

## Acceptance criteria

- [ ] `pk-lint-cron.sh` is registered in the Stop chain.
- [ ] It runs at most once per 7 days per project (gated by `.prometheus/last-lint.txt`).
- [ ] After a successful run, `.prometheus/last-lint.txt` is updated.
- [ ] Auto-fix removes exact duplicates only.
- [ ] Near-duplicates are written to `.prometheus/lint-pending.md` with enough context for the user to make a decision.
- [ ] Run results are logged via SP-006's hook log shim.

## Implementation steps

1. Add `pk-lint-cron.sh` to `.claude-plugin/hooks/hooks.json` and `hooks/hooks.json` (per SP-015 these should be symlinked).
2. Add the timestamp gate at the top of the script.
3. Implement the `--auto-confirm-low-risk` mode in `pk lint` if not already present.
4. Implement the `lint-pending.md` writer.
5. Test by setting timestamp to 8 days ago and verifying run, then 1 day ago and verifying skip.

## Dependencies

None for the basic version. Recommended to land after SP-006 so the hook-log shim is available.

## Open questions

- Should weekly cadence be configurable? Yes via `PK_LINT_INTERVAL_DAYS` env var, default 7.
- Should `lint-pending.md` get auto-promoted to a GitHub issue or surfaced via a slash command? Out of scope; track separately.
