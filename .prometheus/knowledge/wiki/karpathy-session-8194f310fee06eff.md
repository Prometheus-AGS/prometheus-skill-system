---
type: SessionRecord
id: karpathy-session-8194f310fee06eff
title: Karpathy session 8194f310fee0
tags:
- karpathy
- session-learning
sources:
- resource: session:733a99fd-b357-4127-87d1-0c794fcde506
generated:
  by: pk/1.9.0
  at: 2026-10-04T13:14:23.660089+00:00
created_at: 2026-10-04T13:14:23.660089+00:00
updated_at: 2026-10-04T13:14:23.660089+00:00
revision: 0
---

## Delta

I split the work across subagents and three of them finished their changes. 10 of 25 changes are now done (32 of 82 tasks). Four PRs are waiting for you to merge, and two more pieces of work are still running.

**PRs ready for you to merge:**

| PR | Change | What it does | Who built it |
|---|---|---|---|
| [prometheus-skill-system#130](https://github.com/Prometheus-AGS/prometheus-skill-system/pull/130) | A5a | Moves the skill-pack pins to pk and surreal-memory `v1.10.0`. | me |
| [prometheus-skill-system#129](https://github.com/Prometheus-AGS/prometheus-skill-system/pull/129) | B3b | The agent-team memory writer adopts the shared lesson label, picks the right owner key per visibility level, and refuses to write without a project id. | agent |
| [prometheus-knowledge-rs#36](https://github.com/Prometheus-AGS/prometheus-knowledge-rs/pull/36) | C1a | pk spots lessons that recur across projects and adds `pk candidates list/accept/reject`, with an OpenSpec record. | agent |

All three gates pass.

**Built but no PR yet, held until what they depend on merges:**
- **B3 (lesson write path):** branch `feat/learning-write`. Every check passes except delivery to a real server; the B3 note further down has the details.
- **D1a (mini file-based delivery):** branch `feat/team-aware-file-tier`, 20/20 tests, no new mini failures. It waits on B7 and is still marked open in the status below. Before its PR I'll switch it to the shared label format: it currently writes its own variant.

**Still running:**
- An agent is building B4 (recall plus the KBD memory loop), on top of the B3 branch.
- An agent is fixing two regressions my earlier pk PRs left on `main`. A3 pushed `run_context` over Clippy's argument limit, and A2 left the worker's unit tests unable to compile. That agent will open its own PR.

**Things I found and corrected:**
- **Three different label formats.** The writers had each picked their own label format. They now share one, `<!-- prometheus-envelope {json} -->`. I confirmed the Python and Node code compute identical content hashes, including for accented characters.
- **B3 delivery test.** It's blocked because my surreal-memory build lacked the `local-embeddings` feature the real service uses. It's rebuilding with that feature now, and the test re-runs automatically after.
- **A5a gate amended.** The installer can't run from a worktree, because the certified `prometheus-exec` binary hash depends on where it's built. The gate instead proves the pins produce 1.10.0 binaries. Installing on your machine happens after merge, from the main checkout.

**Flagged as separate tasks, each offered to you as a chip:**
- Make the `prometheus-exec` install work from worktrees.
- Mini's hook entry ignores stdin when it arrives as a socket. If that's how Claude Code delivers it, every mini hook currently runs without its payload.

**Waiting on you:**
1. Merge #130, #129 and #36.
2. Approve filing the mini pin request (A5b task 2) as an issue in prometheus-skills-mini. Its text is ready.

---

## KBD status: team-aware-learning-memory-impl

**Progress:** 10/25 changes · 32/82 tasks (39%)

**Done:** A1, A2, A3, A4, A5a, B1, B2, B3b, C1a, D2

### Remaining

**A5b** (1/3, in progress): Operator pin request: mini gitlinks and `versions.toml` to v1.10.0 (operator-authored)
- ✅ 1. Write the operator pin request with exact commits and `versions.toml` diff
- ☐ 2. File the request as an operator-labelled issue in prometheus-skills-mini (after user confirmation)
- ☐ 3. Operator lands the mini pin commit (owner: operator; agent re-checks)

**B3** (0/4, ready, built on a branch with every check passing except live delivery, which is re-running): learning_write, one attributed write path to surreal-memory, pk and the Karpathy log
- ☐ 1. Write `learning_write.py` (envelope build, validation, surreal-memory encoding, audience copies, dedupe, pk ingest, Karpathy append, outbox fallback)
- ☐ 2. Delegate memory-bridge `mem_add_memory` to learning_write
- ☐ 3. Add t

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 733a99fd-b357-4127-87d1-0c794fcde506
- Captured: 2026-10-04T13:12:31.574045Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
