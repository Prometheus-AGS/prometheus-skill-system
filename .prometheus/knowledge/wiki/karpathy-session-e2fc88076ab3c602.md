---
type: SessionRecord
id: karpathy-session-e2fc88076ab3c602
title: Karpathy session e2fc88076ab3
tags:
- karpathy
- session-learning
sources:
- resource: session:cfc5bf77-8f81-45e6-975a-0283553c3962
generated:
  by: pk/1.11.0
  at: 2026-10-09T14:01:56.495067+00:00
created_at: 2026-10-09T14:01:56.495067+00:00
updated_at: 2026-10-09T14:01:56.495067+00:00
revision: 0
---

## Delta

None of the four remaining warnings is a daemon fault. Two have fixable causes, one is blocked on your decision, and one is just time. I changed nothing in this pass.

**1. Marketplace source "uncommitted or untracked changes".**
- **Cause:** the plugin source is this checkout, and the `.gitignore` deliberately tracks `.prometheus/knowledge/wiki`. The wiki hook writes there every session, so the checkout is always dirty. `scripts/lib/plugin-source-topology.js` counts any `git status --porcelain` output as dirty.
- **What's dirty:** the wiki files, `crates/prometheus-exec/.prometheus/` (wiki output from a session run in that crate), a changed-content flag on the surreal-memory submodule, and `history.txt`. That last one is a 17-byte SurrealDB shell history file (`INFO FOR DB;`) from a `surreal sql` session. It is junk and safe to delete.
- **Fix:** make the topology check ignore `.prometheus/knowledge/wiki`, which is intentional churn. Then delete `history.txt` and ignore `crates/*/.prometheus/`.

**2. KBD runtime authority 404.** This corrects my earlier explanation, which blamed a stale daemon.
- **What I found:** the running daemon is healthy. All three registered projects return the same 404, including `/status` with "kbd runtime is not initialized". `prometheus kbd status` for this repo says `mode: legacy` and `runtimeInitialized: false`. `migrate --check` says `journalMigrationRequired: true`, with only 5 of 75 progress files migrated and 2 phases `legacy-read-only`.
- **What it means:** the warning reflects an unfinished KBD migration, not a bug.
- **Next step:** running the migration mutates your KBD state, so it needs your explicit go-ahead.

**3. Rollout gate.** Shadow mode needs 7 days, 100 real mutations and 10,000 synthetic ones, and you have 1 day and 2 mutations. Nothing to do but time.

**4. Discovery budgets.**
- **Cause:** all four harnesses are unmeasured, and the tool deliberately won't invent a ceiling. Real harness traces are needed.
- **Stale numbers:** the doctor's green condition and the spec assume 145 skills and 44,882 characters. The inventory is now 192 skills and 58,899 characters, so even with traces that check needs updating.

**A real problem that isn't a doctor warning:** the live Codex catalog measures **DEGRADED**. About 350 entries across the skill roots give descriptions too short for the model to tell skills apart, so Codex auto-triggering is impaired. `config/codex-catalog.txt` curates only 40 entries, but the generation installer doesn't seem to apply it (I haven't confirmed that). This is probably worth fixing first.

**Decision for you:** which of these should I fix? I'd start with the Codex catalog, then the topology-check change. The KBD migration is your call.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: cfc5bf77-8f81-45e6-975a-0283553c3962
- Captured: 2026-10-09T14:01:54.889208Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-system

## Changed Paths

- tools/surreal-memory-server
- crates/prometheus-exec/.prometheus/
- history.txt
