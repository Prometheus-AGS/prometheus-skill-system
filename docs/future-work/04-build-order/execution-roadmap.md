# Execution Roadmap

The recommended sequence for picking up tasks in this pack, with rationale. Six phases, each producing visible value before the next begins.

## Operating principle

Sequence by *unblocking power* and *blast radius*. A task that unblocks four downstream tasks is preferred over one that unblocks none, all else equal. A task whose blast radius is narrow (single file or small subset) is preferred over a sweeping refactor with the same priority, because narrow tasks integrate without cross-stream conflict.

P0 tasks come first within each phase. P1 tasks fill the parallelism slots. P2 tasks are interleaved opportunistically.

## Phase 1 — Quick wins (Day 0 to Day 2)

Goal: noise reduction and immediate-value cleanups. Land these first so the team feels momentum and the developer experience improves immediately.

| Task | Why first |
|------|-----------|
| BDD-001 | Manifest dual-key cleanup. 0.5 day. Immediately reduces validation noise and storage waste in every video run that follows. |
| BDD-002 | Flake quarantine. 1 day. Eliminates the failFast-on-flake productivity tax developers have been working around with `@no-guide-video` escape tags. |
| BDD-006 | Immutable-tests CLAUDE.md rule. 0.5 day. **Critical to land early** so subsequent BDD work doesn't accidentally try to satisfy the original "auto-update tests" framing. |
| SP-013 | Sycophancy correction in reflector. 1-2 days. **Highest-leverage skill-pack fix in the entire pack.** Lands fast; structural impact is large; no dependencies. |
| SP-015 | hooks.json symlink fix. 0.5 day. Tiny task that prevents drift in every subsequent hook-modification task. Land before SP-006/009/011/012 modify hooks. |
| SP-006 | Stop hook observability. 1 day. Required by many downstream tasks. Without it, debugging anything in the hook layer is archaeology. |

End of Phase 1: the team has a working pipeline with reduced noise, the highest-leverage fix is in, the rule that scopes test mutability is documented, and observability is in place to debug subsequent tasks.

## Phase 2 — Boundary conditions (Day 2 to Day 5)

Goal: lock down the boundaries that subsequent work depends on. These tasks each produce a small visible artifact and prevent a class of defects.

| Task | Why now |
|------|---------|
| BDD-005 | testid drift detection. 1 day. Pairs with BDD-006 to make the immutable-tests rule operationally sustainable. |
| BDD-007 | Candidate test drafts dir. 1 day. Provides the outlet that BDD-006 requires (agents must have a place to contribute coverage without editing existing tests). |
| SP-008 | Per-project KB scoping. 1-2 days. **P0 confidentiality fix.** Land before SP-019 / SP-020 / XC-005 which all depend on it. |
| SP-016 | Skill description collision detection. 1 day. Catches existing latent collisions in 64-skill catalog. Standalone. |
| SP-001 | CLAUDE.md unification. 1 day. Documentation-quality work that makes XC-001 (bug-fix ledger) and SP-017 (slash command merge) cleaner to land later. |

End of Phase 2: drift-detection backstops are in place, KB confidentiality is repaired, the canonical CLAUDE.md and skill-discovery surface are clean.

## Phase 3 — Foundational architecture (Week 1 to Week 3)

Goal: the big foundation pieces. These take longer but unblock the biggest downstream payoffs.

| Task | Why now | Effort |
|------|---------|--------|
| BDD-008 | pk-codegraph extraction. **The foundation for selective execution and impact analysis.** Unblocks BDD-009, BDD-010, BDD-013. | 1-2 weeks |
| SP-019 | LibrarianEvent first-class persistence. **Foundation for episodic memory.** Unblocks SP-020. | 1 week |
| SP-007 | Trace capture verification. Verifies/implements the trace layer SP-019 references. | 2 days |

These three can run in parallel — different agent roles, different file scopes. Phase 3 is roughly 2-3 weeks elapsed time with sufficient parallelism.

End of Phase 3: the codegraph exists, events persist with relations to wiki entries, traces exist in a known location. The architectural backbone is in place.

## Phase 4 — Selective execution payoff (Week 3 to Week 4)

Goal: the value of Phase 3's foundations becomes visible.

| Task | Why now |
|------|---------|
| BDD-009 | pk-codegraph runtime coverage. Adds the runtime-trace ingestion that turns approximate-static into precise scenario-to-file mapping. |
| BDD-010 | Impact-set hash test runner. The selective execution payoff. ~95% scenario skip rates on no-change PRs. |
| BDD-011 | Environment hash augmentation. Correctness backstop for BDD-010. |
| BDD-012 | Two-phase gates. Per-PR fast / nightly thorough. |
| SP-020 | Memory dual-store separation. KG vs episodic decoupling. |

End of Phase 4: per-PR test runs are 4-10x faster while preserving release confidence; memory architecture cleanly separates the two stores.

## Phase 5 — Loop closure (Week 4 to Week 6)

Goal: close the user-feedback ↔ tests ↔ docs loop with the right structural choices.

| Task | Why now |
|------|---------|
| BDD-013 | Story-feature contract via OpenSpec change-id tagging. Single direction. |
| BDD-014 | Feedback aggregation in docs site. User signals visible alongside docs. |
| BDD-015 | Feedback record to draft-scenario emitter. Triage → coverage path. |
| SP-002 | pk-focus keyword extraction quality. |
| SP-004 | pk-focus context-sensitive multi-turn extractor. |
| SP-010 | compile_user_prompt strict JSON parser. |

End of Phase 5: the original "use cases ↔ tests ↔ docs" ask is fulfilled in the right shape, with feedback flowing into both docs and (as drafts) test coverage. Librarian retrieval quality is meaningfully better.

## Phase 6 — Operational hardening (Week 6+)

Goal: catch the long tail of operational issues before they bite.

| Task | Why now |
|------|---------|
| SP-011 | Cedar gate at PostToolUse for SKILL.md edits. Production-mode enforcement. |
| SP-012 | 4-layer pipeline enforcement. Hardens the doc-only contract. |
| SP-013 already landed | (Already in Phase 1) |
| SP-014 | SubagentStop fallback verification. Locks in SP-013's matcher behavior with a test. |
| SP-018 | End-to-end pipeline smoke test. Integration test for SP-006 + SP-012 + SP-013. |
| SP-021 | mem0 compress on schedule. Operational hygiene. |
| SP-009 | pk lint scheduled. Same. |
| XC-001 | Bug-fix-ledger quarterly review. Process change. Recurring. |
| XC-002 | Cross-model QA loop. Selective; high-stakes only. |
| XC-003 | Per-session SCRATCHPAD pattern. Documentation + light tooling. |
| XC-004 | prometheus doctor end-to-end loop test. **Integration validator.** Land after most of the other operational pieces are in. |
| XC-005 | prometheus init overlay. Adoption command. |
| BDD-004 | BDD video skill productization. |
| SP-003 | pk-focus result caching upgrade. |
| SP-005 | pk focus --inject-as flag. |
| SP-017 | Slash command merge strategy. |

End of Phase 6: the loop is operational, scheduled jobs are running, doctor reports green, new project adoption is one command.

## Slot ordering inside each phase

Within a phase, prefer:

1. **P0 first.** No exceptions.
2. **Tasks with the highest unblock count next** — finishing them frees more downstream work.
3. **Standalone tasks last** — they don't compete for review attention with critical-path work.

If a phase's P0 tasks all complete and you have remaining capacity, opportunistically pull a low-effort P2 task from a later phase (e.g. SP-015 in Phase 1) rather than rush a higher-effort task that won't fit in the remaining time.

## Honest framing

This roadmap assumes:

- **Sufficient agent parallelism.** The recommended slot allocations in `parallel-work-streams.md` need to actually run. If you have only one agent, the timeline triples.
- **No surprise refactoring.** A task that uncovers structural issues mid-implementation can blow its estimate. The estimates here are 50th-percentile, not p99.
- **Discipline on scope.** Each task is bounded. Combining "while I'm in here, let me also fix..." patterns will erode the timeline. The `Trade-offs and risks` section in each task is the structural defense.
- **Phase boundaries are soft.** It is fine to start a Phase 3 task while finishing Phase 2 if dependencies are met. The phases are organizing principles, not gates.

## What "done" looks like for the whole pack

- All 55 task files referenced by the pack are `done` in STATUS.md.
- `prometheus doctor --json` returns all-green.
- `pk lint` finds no duplicates.
- BDD per-PR gate completes in <30 minutes typical; release-gate has run within last 7 days.
- Bug-fix-ledger has been reviewed at least once.
- Skill-pack canonical CLAUDE.md has been reduced to invariants only; project-level CLAUDE.md files reference it.
- New project adopts the stack via `prometheus init` + verifies via `prometheus doctor`.

When that state is achieved, the `docs/future-work/` directory can be archived (committed in its final state) and a fresh review cycle begins.
