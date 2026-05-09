# Plan — prometheus-skill-pack All Phases (2–6)

**Project:** prometheus-skill-pack
**Date:** 2026-05-09
**Planner:** Claude Sonnet 4.6 (claude-sonnet-4-6)
**Phase:** skill-pack-all-phases-2026-05-09 (covering Phases 2–6)
**OpenSpec available:** YES
**Assessment input:** `.kbd-orchestrator/phases/assess/all-phases-future-work-assessment.md`
**Change backend:** OpenSpec

**Phase 1 is COMPLETE.** This plan covers Phases 2–6 of the future-work pack (45 remaining tasks from `docs/future-work/STATUS.md`). Each phase is treated as a discrete KBD execution unit with its own assess → execute → reflect sub-cycle.

---

## Progress Signaling Convention (MANDATORY for all executing agents)

Every agent executing any change in this plan MUST emit before starting each phase:

```
Starting phase <N> out of 6: <phase-name>
```

And after completing the phase:

```
Completed phase <N> out of 6: <phase-name>
```

And before each individual change:

```
Starting change <N> of <total>: <change-id>
```

And after:

```
Completed change <N> of <total>: <change-id>
```

Counts must be read from `progress.json` — never guessed.

---

## Phase 2 — Boundary Conditions

**Goal:** Lock down the operational boundaries that downstream phases depend on. Small deliverables; no long-running tasks. Eliminate the testid drift risk before more BDD work lands; establish KB scoping before memory architecture work begins.

**Estimated elapsed:** 4–6 days with 4 parallel sessions.

### Change List (7 changes)

---

**change-001-sp008-per-project-kb-scoping**
- Scope: Rust (prometheus-cli or pk tool), shell config
- Depends on: NONE
- Recommended agent: Claude Code (rust-codegraph role)
- Est. complexity: M
- Complexity score: Medium
- Model class: medium
- Customer value: HIGH (P0 — confidentiality fix; also unblocks SP-019, XC-005)
- Details: Implement per-project KB scoping in the `pk` (Karpathy) tool so that knowledge-base entries are namespaced to the active project. Today the KB is global; entries from one project leak into another. Deliverable: `pk kb --project <name>` flag or automatic project detection from `CLAUDE.md`/`.kbd-orchestrator/project.json`.

---

**change-002-bdd005-testid-drift-detection**
- Scope: shared/scripts/, CI
- Depends on: NONE
- Recommended agent: Claude Code (hooks-engineer role)
- Est. complexity: M
- Complexity score: Medium
- Model class: medium
- Customer value: HIGH (P0 — makes BDD-006 immutable-tests rule operationally enforceable)
- Details: Create a `shared/scripts/detect-testid-drift.sh` script (or TypeScript equivalent) that scans step definition files for `data-testid` selectors and verifies each one appears in the current rendered DOM snapshot. Wire into CI as a `testid-drift` job. Deliverable: passing CI job that fails when a testid in steps no longer exists in the UI.

---

**change-003-bdd007-candidate-drafts-directory**
- Scope: ssr-frontend repo (`tests/features/drafts/`)
- Depends on: NONE
- Recommended agent: Claude Code (bdd-engineer role)
- Est. complexity: S
- Complexity score: Low
- Model class: small
- Customer value: MEDIUM (P1 — required outlet for agents contributing test coverage without touching existing steps; unblocks BDD-015)
- Details: Create `tests/features/drafts/` directory in ssr-frontend with a `README.md` explaining the draft lifecycle (draft → reviewed → promoted to `tests/features/`). Add a `drafts/.gitkeep`. Document in `tests/README.md` that agents MUST use this directory for new scenario proposals. **Working directory: ssr-frontend repo.**

---

**change-004-sp016-skill-description-collision**
- Scope: scripts/, skill catalog
- Depends on: NONE
- Recommended agent: Claude Code (skill-pack-maintainer role)
- Est. complexity: M
- Complexity score: Medium
- Model class: medium
- Customer value: MEDIUM (P1 — catches latent collisions in 64-skill catalog before they cause agent routing failures)
- Details: Create `scripts/detect-skill-collisions.js` that reads all `SKILL.md` frontmatter `description` fields and computes pairwise cosine similarity (or edit distance as a lightweight proxy). Report pairs with similarity > 0.8. Wire into `npm run validate:strict` as a warning (not a hard failure initially). Deliverable: script + CI warning + report of any existing collisions found.

---

**change-005-sp001-claude-md-unification**
- Scope: CLAUDE.md (skill-pack root), documentation
- Depends on: NONE
- Recommended agent: Claude Code (skill-pack-maintainer role)
- Est. complexity: S
- Complexity score: Low
- Model class: small
- Customer value: MEDIUM (P1 — makes XC-001 bug-fix ledger and SP-017 slash-command merge cleaner)
- Details: Audit the two existing CLAUDE.md files (skill-pack root and any project-level CLAUDE.md). Extract invariants that belong in the single canonical skill-pack root CLAUDE.md. Remove redundancy. Ensure the root CLAUDE.md is the authoritative source and project-level files reference it (or complement it without duplicating it). Document the two-file relationship clearly.

---

**change-006-sp014-subagent-fallback-verification**
- Scope: shared/scripts/, hooks/hooks.json, tests
- Depends on: NONE (SP-006 already done)
- Recommended agent: Claude Code (hooks-engineer role)
- Est. complexity: S
- Complexity score: Low
- Model class: small
- Customer value: LOW (P2 — filler; locks in SP-013 matcher behavior with a test)
- Details: Write a shell test (`shared/scripts/tests/test-fallback-subagent-stop.sh`) that sends a synthetic SubagentStop hook event to the fallback matcher and verifies: (1) the fallback script fires, (2) a JSONL entry is written to `~/.prometheus/hooks.log`. Deliverable: runnable test + confirmation that the fallback path is covered.

---

**change-007-sp007-trace-capture-verification**
- Scope: shared/scripts/
- Depends on: NONE (SP-006 already done)
- Recommended agent: Claude Code (hooks-engineer role)
- Est. complexity: M
- Complexity score: Medium
- Model class: medium
- Customer value: MEDIUM (P1 — unblocks SP-019 LibrarianEvent persistence)
- Details: Create `shared/scripts/verify-trace-capture.sh` that: (1) confirms the trace file path exists and is non-empty after a session, (2) validates the trace JSONL structure against a schema, (3) emits a structured result suitable for SP-019 to consume. This is the verification layer SP-019 references before persisting events.

---

### Phase 2 Execution Rounds

```
Round 1 (parallel — no dependencies):
  change-001-sp008-per-project-kb-scoping       (rust-codegraph session)
  change-002-bdd005-testid-drift-detection      (hooks-engineer session)
  change-003-bdd007-candidate-drafts-directory  (bdd-engineer session)
  change-004-sp016-skill-description-collision  (skill-pack-maintainer session)
  change-005-sp001-claude-md-unification        (skill-pack-maintainer session — serialize with change-004)
  change-006-sp014-subagent-fallback-verification (hooks-engineer — serialize after change-002 if contention)
  change-007-sp007-trace-capture-verification   (hooks-engineer — same)

Round 2: None (all Phase 2 changes are independent)
```

### Phase 2 OpenSpec Commands
```
/opsx:new change-001-sp008-per-project-kb-scoping
/opsx:new change-002-bdd005-testid-drift-detection
/opsx:new change-003-bdd007-candidate-drafts-directory
/opsx:new change-004-sp016-skill-description-collision
/opsx:new change-005-sp001-claude-md-unification
/opsx:new change-006-sp014-subagent-fallback-verification
/opsx:new change-007-sp007-trace-capture-verification
```

---

## Phase 3 — Foundational Architecture

**Goal:** Lay the architectural foundations that unlock Phases 4 and 5. BDD-008 is the long pole (1–2 weeks). SP-007 and BDD-008 run in parallel. SP-019 starts after both SP-007 and SP-008 are done.

**Estimated elapsed:** 2–3 weeks.

### Change List (3 changes)

---

**change-008-bdd008-pk-codegraph-extraction**
- Scope: Rust (new `pk-codegraph` crate or subcommand), ssr-frontend integration
- Depends on: NONE
- Recommended agent: Claude Code (rust-codegraph role — dedicated long-running session)
- Est. complexity: L
- Complexity score: High
- Model class: frontier
- Customer value: HIGH (P0 — unblocks 4 tasks transitively: BDD-009, BDD-010, BDD-013 and indirectly BDD-014)
- Details: Implement `pk codegraph extract` — a static AST analysis pass that walks the ssr-frontend TypeScript source and produces a JSON graph mapping Cucumber scenario IDs to the source files they exercise (via `data-testid` attributes, component names, and import chains). Output: `tests/reports/codegraph.json`. This is a 1–2 week effort; assign a dedicated session owner. Do not attempt to combine with other changes.

---

**change-009-sp019-librarian-event-persistence**
- Scope: Rust (prometheus-knowledge or prometheus-cli crate), surreal-memory integration
- Depends on: change-001-sp008-per-project-kb-scoping AND change-007-sp007-trace-capture-verification (both Phase 2)
- Recommended agent: Claude Code (rust-codegraph role)
- Est. complexity: L
- Complexity score: High
- Model class: frontier
- Customer value: HIGH (P0 — unblocks SP-020; architectural milestone for episodic memory)
- Details: Implement first-class persistence for `LibrarianEvent` structs into the surreal-memory graph. Each event gets an entity node with relations to wiki entries, session ID, and project scope. Deliverable: `pk librarian persist` command that writes events to surreal-memory using the scoped `user_id` established in SP-008. Requires SP-007's trace verification to confirm event source integrity.

---

### Phase 3 Execution Rounds

```
Round 1 (parallel):
  change-008-bdd008-pk-codegraph-extraction    (long-running; dedicated session)
  [change-007-sp007 from Phase 2 may still be completing — coordinate]

Round 2 (after Phase 2 change-001 + change-007 complete):
  change-009-sp019-librarian-event-persistence
```

### Phase 3 OpenSpec Commands
```
/opsx:new change-008-bdd008-pk-codegraph-extraction
/opsx:new change-009-sp019-librarian-event-persistence
```

---

## Phase 4 — Selective Execution Payoff

**Goal:** Realize the value of the codegraph and memory foundations. BDD selective execution (~95% skip rate on no-change PRs) becomes operational. Memory dual-store decoupled.

**Estimated elapsed:** 2 weeks from Phase 3 completion.

**Hard dependency:** Phase 4 cannot start until `change-008-bdd008` is complete.

### Change List (5 changes)

---

**change-010-bdd009-runtime-coverage-ingestion**
- Scope: Rust/TypeScript, ssr-frontend test runner
- Depends on: change-008-bdd008-pk-codegraph-extraction
- Recommended agent: Claude Code (rust-codegraph role)
- Est. complexity: L
- Complexity score: High
- Model class: frontier
- Customer value: HIGH (P1 — augments static codegraph with runtime trace for precise mapping)
- Details: Instrument `run-video-proof.ts` to emit a runtime coverage file (`tests/reports/runtime-coverage.json`) that records which source files were actually loaded/exercised during each scenario run. Merge with the static codegraph from BDD-008 to produce a precise scenario→file map.

---

**change-011-bdd010-impact-set-hash-runner**
- Scope: TypeScript (test runner scripts), CI
- Depends on: change-008-bdd008 AND change-010-bdd009
- Recommended agent: Claude Code (bdd-engineer role)
- Est. complexity: M
- Complexity score: Medium
- Model class: medium
- Customer value: HIGH (P0 — the selective-execution payoff; ~95% scenario skip on no-change PRs)
- Details: Implement `scripts/run-impact-set.ts` that: (1) computes a git diff hash of changed files, (2) intersects with the scenario→file map to find the minimal affected scenario set, (3) passes only that set to the Cucumber runner. Wire into CI as the default per-PR job.

---

**change-012-bdd011-environment-hash-augmentation**
- Scope: TypeScript (hash computation)
- Depends on: change-011-bdd010-impact-set-hash-runner
- Recommended agent: Claude Code (bdd-engineer role)
- Est. complexity: S
- Complexity score: Low
- Model class: small
- Customer value: MEDIUM (P1 — correctness backstop for BDD-010; forces full run when env changes)
- Details: Extend the impact-set hash in BDD-010 to include: env variables referenced in feature files, `.env` file hashes, and `package.json` lock file hash. If the env hash changes, force a full run regardless of file diff.

---

**change-013-bdd012-two-phase-test-gates**
- Scope: CI (GitHub Actions), test runner
- Depends on: change-011-bdd010 AND change-012-bdd011
- Recommended agent: Claude Code (bdd-engineer role)
- Est. complexity: M
- Complexity score: Medium
- Model class: medium
- Customer value: HIGH (P1 — per-PR fast gate + nightly thorough gate)
- Details: Create two CI workflow jobs: (1) `bdd-fast` — runs impact-set scenarios only, required for PR merge, target < 10 min; (2) `bdd-thorough` — runs full suite, scheduled nightly, blocks release branches. Reuse existing `run-video-proof.ts` infrastructure.

---

**change-014-sp020-memory-dual-store-separation**
- Scope: Rust (surreal-memory integration layer)
- Depends on: change-009-sp019-librarian-event-persistence
- Recommended agent: Claude Code (rust-codegraph role)
- Est. complexity: L
- Complexity score: High
- Model class: frontier
- Customer value: MEDIUM (P1 — architectural; decouples knowledge graph from episodic memory)
- Details: Refactor the surreal-memory client in prometheus-knowledge to route: (1) factual/structural data (entities, relations) → knowledge graph store; (2) episodic/session data (events, observations) → episodic store. Two separate `user_id` namespaces or separate SurrealDB tables. Prevents episodic noise from polluting graph queries.

---

### Phase 4 Execution Rounds

```
Round 1 (after Phase 3 change-008):
  change-010-bdd009-runtime-coverage-ingestion  (rust-codegraph)
  change-014-sp020-memory-dual-store-separation (rust-codegraph — parallel, different crate)

Round 2 (after change-010):
  change-011-bdd010-impact-set-hash-runner      (bdd-engineer)

Round 3 (after change-011):
  change-012-bdd011-environment-hash-augmentation (bdd-engineer)

Round 4 (after change-011 + change-012):
  change-013-bdd012-two-phase-test-gates        (bdd-engineer)
```

### Phase 4 OpenSpec Commands
```
/opsx:new change-010-bdd009-runtime-coverage-ingestion
/opsx:new change-011-bdd010-impact-set-hash-runner
/opsx:new change-012-bdd011-environment-hash-augmentation
/opsx:new change-013-bdd012-two-phase-test-gates
/opsx:new change-014-sp020-memory-dual-store-separation
```

---

## Phase 5 — Loop Closure

**Goal:** Close the user-feedback ↔ tests ↔ docs loop. Three streams (BDD story-feature contract, feedback aggregation, pk-focus quality) run largely in parallel.

**Estimated elapsed:** 3–4 weeks from Phase 4 completion.

**Hard dependencies:** BDD-013 needs BDD-008 (Phase 3). BDD-015 needs BDD-007 (Phase 2). SP-004 needs SP-002.

### Change List (6 changes)

---

**change-015-bdd013-story-feature-contract**
- Scope: ssr-frontend feature files, OpenSpec change-id tagging convention, docs
- Depends on: change-008-bdd008-pk-codegraph-extraction
- Recommended agent: Claude Code (docs-writer role)
- Est. complexity: M
- Complexity score: Medium
- Model class: medium
- Customer value: MEDIUM (P1 — single-direction traceability: story → change-id → feature file → test result)
- Details: Define and document a convention for tagging Cucumber feature files with `@change-id:<openspec-change-id>` tags. Update the codegraph extractor (BDD-008) to index these tags. Write a validator that checks every feature file in `tests/features/` has at least one `@change-id` tag. Deliver a `STORY-FEATURE-CONTRACT.md` reference document.

---

**change-016-bdd014-feedback-aggregation-docs**
- Scope: ssr-frontend docs site (if applicable), feedback tooling
- Depends on: change-015-bdd013-story-feature-contract
- Recommended agent: Claude Code (bdd-engineer role)
- Est. complexity: L
- Complexity score: High
- Model class: frontier
- Customer value: MEDIUM (P1 — user signals visible alongside documentation)
- Details: Implement a feedback aggregation pipeline that collects user signals (thumbs up/down, inline comments, or equivalent) from the docs site and stores them as structured `FeedbackRecord` objects. Wire to a JSON file or surreal-memory endpoint. Deliverable: feedback collection UI element + backend storage. Feeds BDD-015.

---

**change-017-bdd015-feedback-to-draft-scenario**
- Scope: shared/scripts/, ssr-frontend `tests/features/drafts/`
- Depends on: change-003-bdd007-candidate-drafts-directory (Phase 2) AND change-016-bdd014-feedback-aggregation-docs
- Recommended agent: Claude Code (bdd-engineer role)
- Est. complexity: M
- Complexity score: Medium
- Model class: medium
- Customer value: MEDIUM (P1 — triage path from user feedback to draft test coverage)
- Details: Create `shared/scripts/feedback-to-draft.sh` (or TypeScript) that reads `FeedbackRecord` objects, applies a threshold filter (e.g., ≥3 thumbs-down on the same feature), and emits a skeleton Cucumber `.feature` file into `tests/features/drafts/`. The draft names the scenario after the feedback subject and includes a `@feedback-sourced` tag.

---

**change-018-sp002-pk-focus-extraction-quality**
- Scope: Rust (pk-focus command)
- Depends on: NONE
- Recommended agent: Claude Code (hooks-engineer role)
- Est. complexity: M
- Complexity score: Medium
- Model class: medium
- Customer value: MEDIUM (P1 — pk-focus loses signal on long prompts; unblocks SP-003, SP-004)
- Details: Improve the `pk focus` keyword extraction to handle long multi-turn prompts. Replace the current single-pass extraction with a sliding-window approach that aggregates keyword scores across prompt chunks. Deliverable: measurable improvement in keyword recall on prompts > 2000 tokens. Unblocks SP-003 and SP-004.

---

**change-019-sp004-pk-focus-context-sensitive**
- Scope: Rust (pk-focus)
- Depends on: change-018-sp002-pk-focus-extraction-quality
- Recommended agent: Claude Code (hooks-engineer role)
- Est. complexity: M
- Complexity score: Medium
- Model class: medium
- Customer value: MEDIUM (P1 — multi-turn context-aware extraction; unblocks SP-005)
- Details: Extend `pk focus` to track context across turns in a multi-turn session. Keywords from earlier turns decay but don't disappear; new turns can reinforce or override prior focus. Deliverable: `pk focus --context-window <n>` flag that reads prior turn summaries.

---

**change-020-sp010-strict-json-parser**
- Scope: Rust (compile_user_prompt function)
- Depends on: NONE
- Recommended agent: Claude Code (rust-codegraph role)
- Est. complexity: M
- Complexity score: Medium
- Model class: medium
- Customer value: MEDIUM (P1 — eliminates partial-parse silent failures in prompt compilation)
- Details: Replace the current lenient JSON parser in `compile_user_prompt` with a strict parser that returns a typed `Result<Prompt, ParseError>` and surfaces parse failures via `hook_log_error`. No silent fallback to empty prompt. Deliverable: strict parser + unit tests covering malformed JSON edge cases.

---

### Phase 5 Execution Rounds

```
Round 1 (parallel — no cross-Phase-5 dependencies):
  change-015-bdd013-story-feature-contract      (docs-writer)
  change-018-sp002-pk-focus-extraction-quality  (hooks-engineer)
  change-020-sp010-strict-json-parser           (rust-codegraph)

Round 2 (after change-015):
  change-016-bdd014-feedback-aggregation-docs   (bdd-engineer)

Round 3 (after change-016 and change-003 from Phase 2):
  change-017-bdd015-feedback-to-draft-scenario  (bdd-engineer)

Round 4 (after change-018):
  change-019-sp004-pk-focus-context-sensitive   (hooks-engineer)
```

### Phase 5 OpenSpec Commands
```
/opsx:new change-015-bdd013-story-feature-contract
/opsx:new change-016-bdd014-feedback-aggregation-docs
/opsx:new change-017-bdd015-feedback-to-draft-scenario
/opsx:new change-018-sp002-pk-focus-extraction-quality
/opsx:new change-019-sp004-pk-focus-context-sensitive
/opsx:new change-020-sp010-strict-json-parser
```

---

## Phase 6 — Operational Hardening

**Goal:** Catch the long tail of operational issues. Highly parallel — many small independent tasks. The critical-path item is SP-012 (pipeline enforcement hook), which unblocks SP-018 and XC-004.

**Estimated elapsed:** 2–3 weeks from Phase 5 completion. Many tasks can start immediately (standalone).

### Change List (16 changes)

---

**change-021-sp012-pipeline-enforcement-hook**
- Scope: shared/scripts/, hooks/hooks.json
- Depends on: NONE (SP-006 done in Phase 1)
- Recommended agent: Claude Code (hooks-engineer role)
- Est. complexity: L
- Complexity score: High
- Model class: frontier
- Customer value: HIGH (P1 — critical path; unblocks SP-018 and XC-004)
- Details: Create a PreToolUse or Stop hook that validates the assess→plan→execute→reflect layer order in the KBD cycle. Specifically: detects if an agent attempts to execute changes without a completed `plan.md`, or attempts to reflect without a `progress.json` showing all changes DONE. Emits a blocking error (exit 2) with guidance. Wire into `hooks/hooks.json` as a new `pipeline-enforce` matcher.

---

**change-022-sp011-cedar-skill-edit-gate**
- Scope: shared/scripts/, hooks/hooks.json
- Depends on: NONE
- Recommended agent: Claude Code (hooks-engineer role)
- Est. complexity: M
- Complexity score: Medium
- Model class: medium
- Customer value: MEDIUM (P1 — production-mode enforcement for skill authoring rules)
- Details: Add a PreToolUse hook that fires when a Write or Edit tool targets a `SKILL.md` file. The hook checks the proposed change against the skill authoring rules in `AGENTS.md` (name pattern, required frontmatter fields, no backslashes). Rejects (exit 2) changes that violate the rules. Uses `hook_log_error` from the Phase 1 shim for all failures.

---

**change-023-sp018-pipeline-smoke-test**
- Scope: shared/scripts/tests/, CI
- Depends on: change-021-sp012-pipeline-enforcement-hook
- Recommended agent: Claude Code (hooks-engineer role)
- Est. complexity: M
- Complexity score: Medium
- Model class: medium
- Customer value: HIGH (P1 — integration test for SP-006 + SP-012 + SP-013 together)
- Details: Write `shared/scripts/tests/test-pipeline-smoke.sh` that: (1) simulates a full Stop hook chain, (2) verifies JSONL entries appear in `~/.prometheus/hooks.log`, (3) verifies sycophancy gate fires on a known sycophantic artifact, (4) verifies pipeline-enforce hook rejects an out-of-order execution attempt. All three systems tested together.

---

**change-024-xc004-prometheus-doctor-loop-test**
- Scope: prometheus-cli (`prometheus doctor` subcommand or new script)
- Depends on: change-021-sp012-pipeline-enforcement-hook
- Recommended agent: Claude Code (skill-pack-maintainer role)
- Est. complexity: L
- Complexity score: High
- Model class: frontier
- Customer value: HIGH (P1 — user-facing integration validator; "is my prometheus setup healthy?")
- Details: Implement `prometheus doctor` (or `pk doctor`) command that checks: hook shim log is writable, sycophancy binary is present/executable, hooks.json symlink integrity passes, pipeline-enforce hook is registered, KB scoping is configured. Emits a structured health report with PASS/WARN/FAIL per check. `--json` flag for machine-readable output.

---

**change-025-xc005-prometheus-init-overlay**
- Scope: prometheus-cli (`prometheus init` subcommand)
- Depends on: change-001-sp008-per-project-kb-scoping (Phase 2)
- Recommended agent: Claude Code (skill-pack-maintainer role)
- Est. complexity: L
- Complexity score: High
- Model class: frontier
- Customer value: HIGH (P1 — new project adoption in one command)
- Details: Implement `prometheus init` that: reads the current directory's project identity (name, stack), creates the per-project KB scope, copies hook scripts to `~/.claude/hooks/` if not present, generates a starter `CLAUDE.md` with the skill-pack conventions. Deliverable: a new project can be onboarded in < 5 minutes.

---

**change-026-bdd004-video-skill-productization**
- Scope: skills/ (new BDD video skill), scripts/
- Depends on: NONE (BDD-001 + BDD-002 done in Phase 1)
- Recommended agent: Claude Code (skill-pack-maintainer role)
- Est. complexity: L
- Complexity score: High
- Model class: frontier
- Customer value: MEDIUM (P1 — turns ad-hoc video-proof scripts into a first-class skill)
- Details: Package the BDD video proof workflow as a proper `skills/testing/bdd-video-proof/SKILL.md` skill with frontmatter, script references, and references/ documentation. The skill triggers on "run video proof" or "record BDD evidence". Reuses `run-video-proof.ts` without duplication.

---

**change-027-xc002-cross-model-qa-loop**
- Scope: CI workflow, shared/scripts/
- Depends on: NONE
- Recommended agent: Claude Code (hooks-engineer role)
- Est. complexity: M
- Complexity score: Medium
- Model class: medium
- Customer value: MEDIUM (P1 — selective Codex/GPT review pass for high-stakes changes)
- Details: Create a CI workflow `cross-model-qa.yml` (manual trigger) that sends a change's diff to a second model (via API) for an independent review pass. The review is appended to the PR as a comment. Not automatic on every PR — triggered manually for high-stakes changes or at the reviewer's discretion.

---

**change-028-sp014-subagent-fallback-verify** *(if not completed in Phase 2 filler)*
- Scope: shared/scripts/tests/
- Depends on: NONE (SP-006 done)
- Recommended agent: Claude Code (hooks-engineer role)
- Est. complexity: S
- Complexity score: Low
- Model class: small
- Customer value: LOW (P2 — if not already done)
- Details: [See Phase 2 change-006 — same task. Include here only if skipped in Phase 2.]

---

**change-029-sp021-mem0-compress-scheduled**
- Scope: shared/scripts/, cron/launchd config
- Depends on: NONE
- Recommended agent: Claude Code (hooks-engineer role)
- Est. complexity: S
- Complexity score: Low
- Model class: small
- Customer value: LOW (P2 — operational hygiene; prevents memory bloat)
- Details: Add a scheduled job (launchd plist for macOS, cron entry for Linux) that runs `compress_memories` on the `prometheus-skill-pack` user_id weekly. Document setup in `shared/config/README.md`.

---

**change-030-sp009-pk-lint-scheduled**
- Scope: shared/scripts/, cron/launchd config
- Depends on: NONE
- Recommended agent: Claude Code (hooks-engineer role)
- Est. complexity: S
- Complexity score: Low
- Model class: small
- Customer value: LOW (P2 — catches stale/duplicate skill descriptions automatically)
- Details: Add a scheduled job that runs `pk lint --fix` weekly and commits any auto-fixable changes. Wire to a notification if unfixable issues are found.

---

**change-031-sp003-pk-focus-caching**
- Scope: Rust (pk-focus)
- Depends on: change-018-sp002-pk-focus-extraction-quality (Phase 5)
- Recommended agent: Claude Code (hooks-engineer role)
- Est. complexity: S
- Complexity score: Low
- Model class: small
- Customer value: LOW (P2 — avoids re-extraction on identical prompts)
- Details: Add a content-addressed cache (keyed on prompt SHA256) to `pk focus` so identical prompts don't re-run the extraction pipeline. Cache stored in `~/.prometheus/pk-focus-cache/`.

---

**change-032-sp005-pk-focus-inject-as-flag**
- Scope: Rust (pk-focus)
- Depends on: change-019-sp004-pk-focus-context-sensitive (Phase 5)
- Recommended agent: Claude Code (rust-codegraph role)
- Est. complexity: S
- Complexity score: Low
- Model class: small
- Customer value: LOW (P2)
- Details: Add `--inject-as system-context` flag to `pk focus` that formats the extracted keywords as a system-context block for injection into a Claude Code prompt rather than as a standalone output.

---

**change-033-sp017-slash-command-merge**
- Scope: skills/ (SKILL.md files), plugin.json
- Depends on: change-005-sp001-claude-md-unification (Phase 2) and change-004-sp016-skill-description-collision (Phase 2)
- Recommended agent: Claude Code (skill-pack-maintainer role)
- Est. complexity: M
- Complexity score: Medium
- Model class: medium
- Customer value: MEDIUM (P2 — resolves slash command conflicts in 64-skill catalog)
- Details: After SP-016 identifies collisions and SP-001 unifies CLAUDE.md, execute the merge strategy: for each collision pair, either (a) rename one skill, (b) merge two skills if their scopes overlap significantly, or (c) add a disambiguation prefix. Update all `argument-hint` entries accordingly.

---

**change-034-bdd003-ipfs-pin-sweep**
- Scope: scripts/ (TypeScript), CI
- Depends on: NONE
- Recommended agent: Claude Code (bdd-engineer role)
- Est. complexity: M
- Complexity score: Medium
- Model class: medium
- Customer value: LOW (P2 — removes orphaned IPFS pins; storage hygiene)
- Details: Create `scripts/ipfs-pin-sweep.ts` that reads `docs/videos-manifest.json`, queries the IPFS node for all pinned CIDs, and unpins any CID not referenced in the manifest. Run as a scheduled weekly job. Deliverable: script + dry-run mode + scheduled job config.

---

**change-035-xc001-bug-fix-ledger**
- Scope: docs/ (markdown ledger), process
- Depends on: NONE
- Recommended agent: Manual (recurring process; not a code task)
- Est. complexity: S
- Complexity score: Low
- Model class: small
- Customer value: LOW (P2 — recurring quarterly process)
- Details: Conduct the first quarterly review of the bug-fix ledger. Promote any fix patterns that have been applied ≥3 times to invariants in CLAUDE.md or AGENTS.md. Archive entries older than 6 months that have not recurred.

---

**change-036-xc003-session-scratchpad**
- Scope: docs/, CLAUDE.md
- Depends on: NONE
- Recommended agent: Claude Code (docs-writer role)
- Est. complexity: S
- Complexity score: Low
- Model class: small
- Customer value: LOW (P2 — lightweight session context pattern)
- Details: Document and adopt the `SCRATCHPAD.md` pattern: each Claude Code session creates a `SCRATCHPAD.md` at the repo root (gitignored) for in-session working notes. Document the pattern in CLAUDE.md. Add `SCRATCHPAD.md` to `.gitignore`.

---

### Phase 6 Execution Rounds

```
Round 1 (parallel — no Phase-6-internal dependencies):
  change-021-sp012-pipeline-enforcement-hook    (hooks-engineer) ← critical path
  change-022-sp011-cedar-skill-edit-gate        (hooks-engineer)
  change-026-bdd004-video-skill-productization  (skill-pack-maintainer)
  change-027-xc002-cross-model-qa-loop          (hooks-engineer)
  change-029-sp021-mem0-compress-scheduled      (hooks-engineer)
  change-030-sp009-pk-lint-scheduled            (hooks-engineer)
  change-034-bdd003-ipfs-pin-sweep              (bdd-engineer)
  change-035-xc001-bug-fix-ledger               (manual)
  change-036-xc003-session-scratchpad           (docs-writer)

Round 2 (after change-021-sp012):
  change-023-sp018-pipeline-smoke-test          (hooks-engineer)
  change-024-xc004-prometheus-doctor-loop-test  (skill-pack-maintainer)

Round 3 (after Phase 2 change-001-sp008):
  change-025-xc005-prometheus-init-overlay      (skill-pack-maintainer)

Round 4 (after Phase 5 changes):
  change-031-sp003-pk-focus-caching             (hooks-engineer)
  change-032-sp005-pk-focus-inject-as-flag      (rust-codegraph)
  change-033-sp017-slash-command-merge          (skill-pack-maintainer)
```

### Phase 6 OpenSpec Commands
```
/opsx:new change-021-sp012-pipeline-enforcement-hook
/opsx:new change-022-sp011-cedar-skill-edit-gate
/opsx:new change-023-sp018-pipeline-smoke-test
/opsx:new change-024-xc004-prometheus-doctor-loop-test
/opsx:new change-025-xc005-prometheus-init-overlay
/opsx:new change-026-bdd004-video-skill-productization
/opsx:new change-027-xc002-cross-model-qa-loop
/opsx:new change-029-sp021-mem0-compress-scheduled
/opsx:new change-030-sp009-pk-lint-scheduled
/opsx:new change-031-sp003-pk-focus-caching
/opsx:new change-032-sp005-pk-focus-inject-as-flag
/opsx:new change-033-sp017-slash-command-merge
/opsx:new change-034-bdd003-ipfs-pin-sweep
/opsx:new change-035-xc001-bug-fix-ledger
/opsx:new change-036-xc003-session-scratchpad
```

---

## Summary — All Phases

| Phase | Changes | Effort | Hard Dependency |
|-------|---------|--------|-----------------|
| Phase 2 — Boundary Conditions | 7 | 4–6 days | None — all ready now |
| Phase 3 — Foundational Architecture | 2 | 2–3 weeks | Phase 2 (SP-008 + SP-007) for SP-019 |
| Phase 4 — Selective Execution Payoff | 5 | 2 weeks | Phase 3 (BDD-008) |
| Phase 5 — Loop Closure | 6 | 3–4 weeks | Phase 3 (BDD-008) for BDD-013; Phase 2 (BDD-007) for BDD-015 |
| Phase 6 — Operational Hardening | 16 | 2–3 weeks | SP-012 unblocks SP-018 + XC-004; SP-008 unblocks XC-005 |
| **TOTAL** | **36 changes** | **~11–15 weeks** | BDD-008 is the long pole |

**Total remaining tasks from STATUS.md covered:** 36 of 41 pending tasks.
*(5 omitted: SP-006, SP-013, SP-015, BDD-001, BDD-002, BDD-006 — all done in Phase 1.)*

---

## Trade-offs and Explicit Scope Cuts

1. **BDD-008 timeline risk is real.** At 1–2 weeks, it is the single longest task. If it slips, Phase 4 and Phase 5 (BDD-013) slip with it. Mitigation: assign a dedicated session owner; do not combine with other changes.

2. **SP-012 is Phase 6's critical-path item** even though most Phase 6 tasks are standalone. XC-004 (`prometheus doctor`) cannot be fully meaningful without a working pipeline-enforcement hook to validate.

3. **Feedback aggregation (BDD-014) scope is intentionally narrow.** The task doc describes a full docs-site feature; this plan scopes it to structured storage + a minimal UI element. Full docs-site integration is out of scope to keep the change implementable in one session.

4. **XC-001 (bug-fix ledger) is marked Manual** because it is a quarterly process review, not a code task. An agent can assist but a human must make the promotion decisions.

5. **STATUS.md must be updated before Phase 2 begins.** The file currently shows `done: 0`. Six Phase 1 tasks are done. An agent running Phase 2 without updating STATUS.md first risks picking up tasks whose dependencies are already satisfied but not recorded.

---

## Pre-Execution Checklist (before Phase 2 starts)

- [ ] Update `docs/future-work/STATUS.md` — mark BDD-001, BDD-002, BDD-006, SP-006, SP-013, SP-015 as `done`
- [ ] Promote their dependents to `ready` in STATUS.md (SP-012, SP-014, BDD-004)
- [ ] Run `/opsx:new` commands for all 7 Phase 2 changes
- [ ] Confirm ssr-frontend repo path for change-003-bdd007 (bdd-engineer agent needs the correct working directory)

PLAN COMPLETE
