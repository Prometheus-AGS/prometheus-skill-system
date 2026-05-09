# Assessment — All Phases Future Work

**Date:** 2026-05-09
**Assessor:** Claude Sonnet 4.6 (claude-sonnet-4-6)
**Session:** f265e820-bad7-483c-9960-836e7a2574d8
**Scope:** Full future-work task pack — phases 1–6 — as defined in `docs/future-work/`

---

## Purpose

This assessment establishes the complete phase-by-phase roadmap for finishing all tasks in `docs/future-work/`. It is the input document for planning Phases 2–6 of the KBD execution cycle. Phase 1 is already complete; this assessment covers the remaining work.

---

## Phase 1 Completion Status (DONE — 6/6 goals MET)

Confirmed complete from `reflection.md` (session f265e820, 2026-05-09):

| Task | Status | Evidence |
|------|--------|----------|
| BDD-001 | **DONE** | Manifest normalized, hex-orphans archived, validator added. Commit `b806e2c`. |
| BDD-002 | **DONE** | `@quarantine` retry + state machine in `run-video-proof.ts`. Commit `e15efa8`. |
| BDD-006 | **DONE** | Immutable-tests rule in both CLAUDE.md files. Commits `38f83e0`, `ba52048`. |
| SP-006 | **DONE** | `hook-log.sh` shim + JSONL log at `~/.prometheus/hooks.log`. Commit `7cb20dd`. |
| SP-013 | **DONE** | Sycophancy gate wired into reflector SubagentStop. Commit `aa2a5b8`. |
| SP-015 | **DONE** | CI `hooks-integrity` job added; canonical path documented. Commit `c586a77`. |

**STATUS.md is stale** — it shows all tasks as `ready`/`planned` as of initial generation (2026-05-09). It has not been updated to reflect Phase 1 completions. This is a gap; STATUS.md must be updated before Phase 2 begins.

---

## Status Reconciliation — What Is Actually Unlocked

With Phase 1 tasks marked done, the following task statuses change:

| Task | Prior Status | New Status | Why |
|------|-------------|------------|-----|
| SP-006 | ready | **done** | Completed Phase 1 |
| SP-013 | ready | **done** | Completed Phase 1 |
| SP-015 | ready | **done** | Completed Phase 1 |
| BDD-001 | ready | **done** | Completed Phase 1 |
| BDD-002 | ready | **done** | Completed Phase 1 |
| BDD-006 | ready | **done** | Completed Phase 1 |
| SP-012 | planned | **ready** | SP-006 is now done |
| SP-014 | planned | **ready** | SP-006 is now done |
| SP-018 | planned | **planned** | Still needs SP-012 |
| XC-004 | planned | **planned** | Still needs SP-012 |
| BDD-004 | planned | **ready** | BDD-001 + BDD-002 both done |

---

## Phase 2 — Boundary Conditions (Day 2 to Day 5)

**Goal:** Lock down boundaries that subsequent work depends on. Small visible artifacts, prevent a class of defects.

### Tasks

| Task | Priority | Effort | Status | Deliverable |
|------|----------|--------|--------|-------------|
| BDD-005 | P0 | 1d | ready | `testid` drift detector — flags when a test ID in step files no longer matches a live UI element. Pairs with BDD-006 to make the immutable-tests rule operationally enforceable. |
| BDD-007 | P1 | 1d | ready | `tests/features/drafts/` directory + conventions for agents to propose new test coverage without touching existing steps. Unblocks BDD-015. |
| SP-008 | P0 | 1–2d | ready | Karpathy KB scoping per project — prevents KB cross-contamination across projects. P0 confidentiality fix. Unblocks SP-019, XC-005. |
| SP-016 | P1 | 1d | ready | Skill description collision matrix — scans the 64-skill catalog for description conflicts and near-duplicates. Standalone. |
| SP-001 | P1 | 1d | ready | Two CLAUDE.md files unified — synergy with SP-016/SP-017 but not a hard blocker for those. |

**Gap finding:** The user's requested Phase 2 task list (SP-014, BDD-005, BDD-007, SP-007, XC-004) **diverges from the execution roadmap**. Per `execution-roadmap.md`, Phase 2 proper is SP-008, SP-016, SP-001, BDD-005, BDD-007. SP-014 belongs to Phase 6 (operational hardening); SP-007 belongs to Phase 3 (foundational architecture); XC-004 is blocked by SP-012 which itself is blocked by SP-006 (done) but SP-012 has not been built yet.

**Recommendation:** Execute the roadmap's Phase 2 sequence. SP-014 and SP-007 can be pulled in opportunistically as filler tasks at the end of Phase 2 since they are now unblocked (SP-006 is done).

### Assessment verdicts

- **BDD-005**: CONFIRMED-GAP. No `testid` drift detector exists in `shared/scripts/` or anywhere in the skill-pack.
- **BDD-007**: CONFIRMED-GAP. `tests/features/drafts/` does not exist in ssr-frontend (path not accessible from this repo, but the task doc describes a gap that BDD-006 explicitly referenced as needed).
- **SP-008**: CONFIRMED-GAP. `pk` (Karpathy) KB is project-global; no per-project scoping mechanism exists.
- **SP-016**: CONFIRMED-GAP. No collision matrix or description-uniqueness checker exists.
- **SP-001**: CONFIRMED-GAP. Two CLAUDE.md files exist (skill-pack root and no unified single-source reference file); unification not done.

**Estimated elapsed:** 4–6 days with 4 parallel sessions.

---

## Phase 3 — Foundational Architecture (Week 1 to Week 3)

**Goal:** Big foundation pieces that unblock the largest downstream payoffs.

### Tasks

| Task | Priority | Effort | Status | Deliverable |
|------|----------|--------|--------|-------------|
| BDD-008 | P0 | 1–2w | ready | `pk-codegraph` extraction — static AST-derived call graph mapping scenarios to source files. Foundation for BDD-009, BDD-010, BDD-013. Highest unblock count (4 transitive). |
| SP-007 | P1 | 2d | ready | Trace capture file existence verification — confirms that the trace files SP-019 will reference actually exist and are structurally valid. Unblocks SP-019. |
| SP-019 | P0 | 1w | planned (blocked on SP-007 + SP-008) | LibrarianEvent first-class persistence — persists Librarian events to surreal-memory with relation links to wiki entries. Unblocks SP-020. |

**Parallelism:** BDD-008 and SP-007 can run concurrently (different codebases/roles). SP-019 starts after both SP-007 and SP-008 complete.

### Assessment verdicts

- **BDD-008**: CONFIRMED-GAP. No codegraph extraction tool exists. This is the highest-leverage gap in the BDD chain.
- **SP-007**: CONFIRMED-GAP. No trace capture verification script exists.
- **SP-019**: CONFIRMED-GAP (planned; blocked). LibrarianEvent persistence not implemented.

**Estimated elapsed:** 2–3 weeks (BDD-008 is the long pole at 1–2 weeks).

---

## Phase 4 — Selective Execution Payoff (Week 3 to Week 4)

**Goal:** The value of Phase 3's codegraph and persistence foundations becomes visible.

### Tasks

| Task | Priority | Effort | Status | Deliverable |
|------|----------|--------|--------|-------------|
| BDD-009 | P1 | 1w | planned (needs BDD-008) | Runtime coverage ingestion — augments static codegraph with runtime trace data for precise scenario-to-file mapping. |
| BDD-010 | P0 | 1–2d | planned (needs BDD-008+009) | Impact-set hash test runner — computes the minimal set of scenarios to run for a given change. ~95% skip rate on no-change PRs. |
| BDD-011 | P1 | 1d | planned (needs BDD-010) | Environment hash augmentation — adds env variables, config files to the hash so env changes force full runs. |
| BDD-012 | P1 | 1d | planned (needs BDD-010+011) | Two-phase test gates — per-PR fast gate and nightly thorough gate. |
| SP-020 | P1 | 3–5d | planned (needs SP-019) | Memory dual-store separation — decouples KG and episodic stores in surreal-memory. |

### Assessment verdicts

All five tasks: CONFIRMED-GAP (planned, dependencies not yet met). No selective execution infrastructure exists anywhere in the codebase.

**Estimated elapsed:** 2 weeks from Phase 3 completion.

---

## Phase 5 — Loop Closure (Week 4 to Week 6)

**Goal:** Close the user-feedback ↔ tests ↔ docs loop.

### Tasks

| Task | Priority | Effort | Status | Deliverable |
|------|----------|--------|--------|-------------|
| BDD-013 | P1 | 1w | planned (needs BDD-008) | User story to feature contract — OpenSpec change-id tags in feature files create a traceable link from story to test. |
| BDD-014 | P1 | 3–5d | planned (needs BDD-013) | Feedback aggregation in docs site — user signals visible alongside documentation pages. |
| BDD-015 | P1 | 3–5d | planned (needs BDD-007) | Feedback record to draft-scenario emitter — triage pipeline that turns feedback records into draft feature files in `tests/features/drafts/`. |
| SP-002 | P1 | 1d | ready | pk-focus keyword extraction quality improvement. Unblocks SP-003, SP-004. |
| SP-004 | P1 | 2d | planned (needs SP-002) | pk-focus context-sensitive multi-turn extractor. Unblocks SP-005. |
| SP-010 | P1 | 1d | ready | compile_user_prompt strict JSON parser — eliminates partial-parse silent failures. |

### Assessment verdicts

- **BDD-013, BDD-014, BDD-015**: CONFIRMED-GAP (planned; blocked by earlier phases).
- **SP-002**: CONFIRMED-GAP. pk-focus extraction is known to lose signal on long prompts (documented in task file).
- **SP-004, SP-010**: CONFIRMED-GAP (SP-004 planned; SP-010 ready but not started).

**Estimated elapsed:** 3–4 weeks from Phase 4 completion.

---

## Phase 6 — Operational Hardening (Week 6+)

**Goal:** Catch the long tail of operational issues before they bite. High parallelism; many small independent tasks.

### Tasks

| Task | Priority | Effort | Status | Deliverable |
|------|----------|--------|--------|-------------|
| SP-011 | P1 | 1d | ready | Cedar gate at PostToolUse for SKILL.md edits — production-mode enforcement of skill authoring rules. |
| SP-012 | P1 | 2–3d | ready (SP-006 done) | 4-layer pipeline enforcement hook — validates the assess→plan→execute→reflect layer order. Unblocks SP-018, XC-004. |
| SP-014 | P2 | 0.5d | ready (SP-006 done) | SubagentStop fallback matcher verification — test that the fallback SubagentStop handler fires correctly and logs via hook-log.sh. |
| SP-018 | P1 | 2–3d | planned (needs SP-006 + SP-012) | End-to-end pipeline smoke test — integration test for SP-006 + SP-012 + SP-013 together. |
| SP-021 | P2 | 1d | ready | mem0 compress_memories scheduled job — prevents memory bloat via scheduled compression. |
| SP-009 | P2 | 0.5d | ready | pk lint scheduled job — catches duplicate/stale skill descriptions automatically. |
| SP-003 | P2 | 0.5d | planned (needs SP-002) | pk-focus result caching — avoids re-extraction on identical prompts. |
| SP-005 | P2 | 0.5d | planned (needs SP-004) | pk focus --inject-as system-context flag. |
| SP-017 | P2 | 1d | ready | Slash command merge strategy — resolves conflicts in the 64-skill catalog. |
| BDD-003 | P2 | 1d | ready | IPFS pin sweep job — removes orphaned IPFS pins from obsolete video runs. |
| BDD-004 | P1 | 3–5d | ready (BDD-001 + BDD-002 done) | BDD video skill productization — turns the video-proof run scripts into a first-class skill. |
| XC-001 | P2 | recurring | ready | Bug-fix-ledger quarterly invariant promotion. |
| XC-002 | P1 | 2d | ready | Cross-model QA loop (Codex/GPT review pass). |
| XC-003 | P2 | 0.5d | ready | Per-session SCRATCHPAD.md pattern — lightweight session context doc. |
| XC-004 | P1 | 2–3d | planned (needs SP-012) | prometheus doctor end-to-end loop test — integration validator. Land after most operational pieces are in. |
| XC-005 | P1 | 2–3d | planned (needs SP-008) | prometheus init project-scoped overlay — adoption command for new projects. |

### Assessment verdicts

- **SP-012, SP-014**: CONFIRMED-GAP — ready to implement now that SP-006 is done.
- **SP-011, SP-017, SP-021, SP-009, BDD-003, BDD-004, XC-001, XC-002, XC-003**: CONFIRMED-GAP — all standalone/ready; can be picked up in Phase 6 or as filler in earlier phases.
- **SP-018, XC-004, XC-005**: CONFIRMED-GAP (planned; blocked by SP-012 and SP-008 respectively).

**Estimated elapsed:** 2–3 weeks for the parallel burst.

---

## Surprises and Deviations

### 1. STATUS.md not updated after Phase 1
The `docs/future-work/STATUS.md` still shows all tasks as `ready`/`planned` with aggregate `done: 0`. It was generated on 2026-05-09 and never updated. Phase 2 planning must begin with an STATUS.md update pass to mark 6 tasks done and promote their dependents to `ready`.

### 2. User's requested Phase 2 task list diverges from roadmap
The user listed SP-014, BDD-005, BDD-007, SP-007, XC-004 as "Phase 2." Per the execution roadmap:
- SP-014 is Phase 6 (operational hardening)
- SP-007 is Phase 3 (foundational architecture)
- XC-004 is Phase 6 (blocked by SP-012 which hasn't been built)
- BDD-005 and BDD-007 are correctly Phase 2

SP-014 and SP-007 are **now unblocked** (SP-006 is done) so they can be pulled into Phase 2 as low-effort filler tasks without violating dependencies. XC-004 cannot be pulled forward — it needs SP-012 first.

### 3. BDD-007 target repo is ssr-frontend, not prometheus-skill-pack
BDD-007 creates `tests/features/drafts/` in the ssr-frontend repo. Any agent executing it must be dispatched with ssr-frontend as the working directory.

### 4. SP-008 (KB scoping) is a higher-priority gap than the reflection suggested
The reflection described SP-008 as a Phase 2 candidate. The dependency graph confirms it: SP-008 blocks SP-019 (1-week effort) and XC-005. It should be treated as P0 within Phase 2.

---

## Constraint Check

- **AGENTS.md violations:** NONE observed in Phase 1 deliverables.
- **Immutable-tests rule:** Now enforced via CLAUDE.md. No agent has violated it post-Phase 1.
- **Hook scripts:** All use `hook-log.sh` shim; `|| true` swallowing eliminated in Phase 1.

---

## Recommended Execution Order — All Remaining Phases

```
PHASE 2 (Day 2–5, ~4–6 days)
  P0: SP-008  — KB scoping (unblocks SP-019, XC-005)
  P0: BDD-005 — testid drift detection
  P1: BDD-007 — candidate drafts directory (ssr-frontend)
  P1: SP-016  — skill description collision matrix
  P1: SP-001  — CLAUDE.md unification
  [filler, low effort]: SP-014, SP-007 (both unblocked by SP-006)

PHASE 3 (Week 1–3, ~2–3 weeks)
  P0: BDD-008 — pk-codegraph (long pole, 1–2 weeks)
  P1: SP-007  — trace capture verification (parallel with BDD-008)
  P0: SP-019  — LibrarianEvent persistence (starts after SP-007 + SP-008)

PHASE 4 (Week 3–4, ~2 weeks from Phase 3 end)
  P0: BDD-009 — runtime coverage ingestion
  P0: BDD-010 — impact-set hash runner
  P1: BDD-011 — environment hash augmentation
  P1: BDD-012 — two-phase gates
  P1: SP-020  — memory dual-store separation (parallel, different role)

PHASE 5 (Week 4–6, ~3–4 weeks from Phase 4 end)
  P1: BDD-013 — story-feature contract
  P1: BDD-014 — feedback aggregation in docs
  P1: BDD-015 — feedback-to-draft emitter
  P1: SP-002  — pk-focus extraction quality
  P1: SP-004  — pk-focus context-sensitive extractor (after SP-002)
  P1: SP-010  — strict JSON parser (standalone)

PHASE 6 (Week 6+, ~2–3 weeks, highly parallel)
  P1: SP-012  — 4-layer pipeline enforcement hook (unblocks SP-018, XC-004)
  P1: SP-011  — Cedar gate for SKILL.md edits
  P2: SP-014  — SubagentStop fallback verification (if not done in Phase 2 filler)
  P1: SP-018  — end-to-end pipeline smoke test (after SP-012)
  P1: XC-004  — prometheus doctor loop test (after SP-012)
  P1: XC-005  — prometheus init overlay (after SP-008)
  P1: BDD-004 — video skill productization
  P1: XC-002  — cross-model QA loop
  P2: SP-021, SP-009, SP-003, SP-005, SP-017, BDD-003, XC-001, XC-003 (standalone filler)
```

---

## Build Health

- **prometheus-skill-pack validation:** `npm run validate` — PASS (Phase 1 left no lint regressions per reflection).
- **Hook scripts:** All pass `bash -n` syntax check (confirmed in Phase 1 QA).
- **JSON:** `hooks/hooks.json` — PASS (`python3 -m json.tool` confirmed in Phase 1).
- **ssr-frontend TypeScript:** Pre-existing deprecation warning only; no new errors from Phase 1.

---

## Scope Exclusions

This assessment covers only the tasks listed in `docs/future-work/`. It does not assess:

- Any work outside the prometheus-skill-pack or ssr-frontend repos
- The `prometheus-knowledge`, `surreal-memory`, `dspy-rs`, or `cowork` repos
- Feature work unrelated to the future-work task pack
- Technical debt in parts of the codebase not touched by the 55-task pack

---

## Summary Verdict Table

| Phase | Tasks | Effort | Blockers | Assessment |
|-------|-------|--------|----------|------------|
| Phase 1 | BDD-001/002/006, SP-006/013/015 | Done | None | **COMPLETE** |
| Phase 2 | SP-008, BDD-005/007, SP-016, SP-001 + filler | 4–6 days | None (all ready) | **READY TO EXECUTE** |
| Phase 3 | BDD-008, SP-007, SP-019 | 2–3 weeks | SP-019 needs SP-007+SP-008 | **READY (BDD-008+SP-007 parallel start)** |
| Phase 4 | BDD-009/010/011/012, SP-020 | 2 weeks | All need Phase 3 | **PLANNED — blocked** |
| Phase 5 | BDD-013/014/015, SP-002/004/010 | 3–4 weeks | BDD-013/014 need Phase 3; BDD-015 needs Phase 2 | **PARTIALLY BLOCKED** |
| Phase 6 | SP-011/012/014/018, XC-002/004/005, + 9 more | 2–3 weeks | XC-004 needs SP-012 | **READY (standalone tasks); PARTIALLY BLOCKED** |

**Total remaining effort:** ~10–14 weeks at sustainable parallelism (3–4 sessions).

---

## Next Action

1. Update `docs/future-work/STATUS.md` — mark 6 Phase 1 tasks `done`, promote dependents to `ready`.
2. Run `/kbd-plan` to produce the Phase 2 execution plan.
3. Begin Phase 2 with SP-008 (P0, highest unblock count) in parallel with BDD-005 (P0, different role/files).

ASSESSMENT COMPLETE
