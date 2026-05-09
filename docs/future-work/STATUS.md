# Status Tracker

This file is the live status board for every task in this pack. It substitutes for what would have been a `surreal-memory` entity graph. The schema mirrors the Surreal schema in `00-meta/memory-schema.surql` so it can be hydrated into Surreal once that MCP is online (see `00-meta/memory-bootstrap.md`).

## How to update

When a Claude Code agent starts work on a task:

1. Find the task block below.
2. Set `status: in-progress`.
3. Set `assigned_to: <session-id-or-name>`.
4. Set `started_at: <ISO-8601 timestamp>`.

When done:

1. Set `status: done`.
2. Set `completed_at: <ISO-8601 timestamp>`.
3. Add a `notes:` line if anything diverged from the task doc.

If you decide a task should be **abandoned** (e.g. you discover it's redundant with another), set `status: abandoned` and add `notes: <reason>`. Do NOT delete the entry.

## Status values

- `planned` — written, not yet ready for an agent (waiting on dependency)
- `ready` — all `depends_on` are `done`, can be picked up
- `in-progress` — actively being worked on
- `done` — acceptance criteria met, verified
- `blocked` — was in-progress, hit an unanticipated blocker (note explains)
- `abandoned` — superseded or determined unnecessary

## Tasks

```yaml
project: prometheus-skill-pack-future-work
generated_at: 2026-05-09
last_updated: 2026-05-09

# ── 01 SKILL-PACK FIXES ────────────────────────────────────────────

- id: SP-001
  title: Two CLAUDE.md files unification
  category: skill-pack-fixes
  priority: P1
  effort: 1d
  agent_role: skill-pack-maintainer
  status: done
  depends_on: []
  unblocks: []
  completed_at: "2026-05-09"
  notes: "Documentation hierarchy table added to skill-pack CLAUDE.md (canonical designation). prometheus-knowledge CLAUDE.md gets header pointing to canonical. Both files cover only their own concerns — no duplicate rules found. Commits: 202ad73 (skill-pack), 2594e6f (knowledge)."

- id: SP-002
  title: pk-focus keyword extraction quality
  category: skill-pack-fixes
  priority: P1
  effort: 1d
  agent_role: hooks-engineer
  status: ready
  depends_on: []
  unblocks: [SP-003, SP-004]

- id: SP-003
  title: pk-focus result caching
  category: skill-pack-fixes
  priority: P2
  effort: 0.5d
  agent_role: hooks-engineer
  status: planned
  depends_on: [SP-002]
  unblocks: []

- id: SP-004
  title: pk-focus context-sensitive multi-turn extractor
  category: skill-pack-fixes
  priority: P1
  effort: 2d
  agent_role: hooks-engineer
  status: planned
  depends_on: [SP-002]
  unblocks: [SP-005]

- id: SP-005
  title: pk focus --inject-as system-context flag
  category: skill-pack-fixes
  priority: P2
  effort: 0.5d
  agent_role: rust-codegraph
  status: planned
  depends_on: [SP-004]
  unblocks: []

- id: SP-006
  title: Stop hook observability log
  category: skill-pack-fixes
  priority: P0
  effort: 1d
  agent_role: hooks-engineer
  status: done
  depends_on: []
  unblocks: [SP-014, SP-018]
  completed_at: "2026-05-09"
  notes: "hook-log.sh shim + JSONL log at ~/.prometheus/hooks.log. Wired into all 5 Stop-chain scripts. Commit 7cb20dd."

- id: SP-007
  title: Trace capture file existence verification
  category: skill-pack-fixes
  priority: P1
  effort: 2d
  agent_role: hooks-engineer
  status: done
  depends_on: []
  unblocks: [SP-019]
  completed_at: "2026-05-09"
  notes: "Traces confirmed written by prometheus-learn Rust crate at .prometheus/traces/<skill-name>/<timestamp>.json. Added PROMETHEUS_TRACE_DIR env var override to TraceStore::default_for_project(). Added shared/scripts/verify-trace-state.sh for state inspection. Commit: abd79c0."

- id: SP-008
  title: Karpathy KB per-project scoping
  category: skill-pack-fixes
  priority: P0
  effort: 1-2d
  agent_role: rust-codegraph
  status: done
  depends_on: []
  unblocks: [SP-019]
  completed_at: "2026-05-09"
  notes: "KbScope enum + --scope/--yes flags on ingest + migrate-to-per-project subcommand. project-root detection walks .git/.kbd-orchestrator/Cargo.toml/package.json/pyproject.toml. Commit 84aa366."

- id: SP-009
  title: pk lint --fix scheduled job
  category: skill-pack-fixes
  priority: P2
  effort: 0.5d
  agent_role: hooks-engineer
  status: ready
  depends_on: []
  unblocks: []

- id: SP-010
  title: compile_user_prompt strict JSON parser
  category: skill-pack-fixes
  priority: P1
  effort: 1d
  agent_role: rust-codegraph
  status: ready
  depends_on: []
  unblocks: []

- id: SP-011
  title: Cedar gate at PostToolUse for SKILL.md edits
  category: skill-pack-fixes
  priority: P1
  effort: 1d
  agent_role: hooks-engineer
  status: ready
  depends_on: []
  unblocks: []

- id: SP-012
  title: 4-layer pipeline enforcement hook
  category: skill-pack-fixes
  priority: P1
  effort: 2-3d
  agent_role: hooks-engineer
  status: ready
  depends_on: [SP-006]
  unblocks: [SP-018]

- id: SP-013
  title: Sycophancy correction in SubagentStop(reflector) hook
  category: skill-pack-fixes
  priority: P0
  effort: 1-2d
  agent_role: hooks-engineer
  status: done
  depends_on: []
  unblocks: []
  completed_at: "2026-05-09"
  notes: "sycophancy-check-reflection.sh wired as first command in reflector SubagentStop matcher. 2-rejection soft cap. PROMETHEUS_REFLECT_STRICTNESS env var. Commit aa2a5b8."

- id: SP-014
  title: SubagentStop fallback matcher verification
  category: skill-pack-fixes
  priority: P2
  effort: 0.5d
  agent_role: hooks-engineer
  status: done
  depends_on: [SP-006]
  unblocks: []
  completed_at: "2026-05-09"
  notes: "shared/scripts/tests/test-subagent-fallback.sh — 10 assertions, 5/5 consecutive passes. Verifies script existence, syntax, exit 0 for all agent types, hook log entry. Commit a374bd5."

- id: SP-015
  title: hooks.json symlink fix
  category: skill-pack-fixes
  priority: P2
  effort: 0.5d
  agent_role: skill-pack-maintainer
  status: done
  completed_at: "2026-05-09"
  notes: "Direction was already correct (.claude-plugin/hooks -> ../hooks). CI hooks-integrity job added. CLAUDE.md canonical path documented. Commit c586a77."
  depends_on: []
  unblocks: []

- id: SP-016
  title: Skill description collision detection
  category: skill-pack-fixes
  priority: P1
  effort: 1d
  agent_role: skill-pack-maintainer
  status: done
  depends_on: []
  unblocks: []
  completed_at: "2026-05-09"
  notes: "scripts/skill-matrix.js (Jaccard pairwise, --ci mode); skill-collision-allowlist.json (1 known structural collision); CI job in validate.yml. npm run skill-matrix. Commit 6d40af4."

- id: SP-017
  title: Slash command merge strategy
  category: skill-pack-fixes
  priority: P2
  effort: 1d
  agent_role: skill-pack-maintainer
  status: ready
  depends_on: []
  unblocks: []

- id: SP-018
  title: End-to-end pipeline smoke test
  category: skill-pack-fixes
  priority: P1
  effort: 2-3d
  agent_role: hooks-engineer
  status: planned
  depends_on: [SP-006, SP-012]
  unblocks: []

- id: SP-019
  title: LibrarianEvent first-class persistence
  category: skill-pack-fixes
  priority: P0
  effort: 1w
  agent_role: rust-codegraph
  status: done
  depends_on: [SP-007, SP-008]
  unblocks: [SP-020]
  completed_at: "2026-05-09"
  notes: "pk-event-store crate added: EventRecord, EventStore (SurrealDB HTTP + JSONL fallback), JsonlFallback. Schema at pk-event-store/schema/events.surql. Background subscriber in pk-cli persists every LibrarianEvent. pk events list/for-entry subcommands added. Commit: f041b11."

- id: SP-020
  title: Memory dual-store separation
  category: skill-pack-fixes
  priority: P1
  effort: 3-5d
  agent_role: rust-codegraph
  status: done
  depends_on: [SP-019]
  unblocks: []
  completed_at: "2026-05-09T17:30:00Z"
  notes: dual_store.rs + migrate.rs + schema files + MigrateStores CLI subcommand; commit f8dce14 (prometheus-knowledge)

- id: SP-021
  title: mem0 compress_memories scheduled job
  category: skill-pack-fixes
  priority: P2
  effort: 1d
  agent_role: hooks-engineer
  status: ready
  depends_on: []
  unblocks: []

# ── 02 BDD TESTING EVOLUTION ───────────────────────────────────────

- id: BDD-001
  title: Manifest dual-key cleanup migration
  category: bdd-testing
  priority: P0
  effort: 0.5d
  agent_role: bdd-engineer
  status: done
  depends_on: []
  unblocks: []
  completed_at: "2026-05-09"
  notes: "videos-manifest.json normalized 374->345 entries. 29 hex orphans archived. assertNoHexKeysInManifest() added. Commit b806e2c."

- id: BDD-002
  title: Flake quarantine system
  category: bdd-testing
  priority: P0
  effort: 1d
  agent_role: bdd-engineer
  status: done
  depends_on: []
  unblocks: []
  completed_at: "2026-05-09"
  notes: "@quarantine retry + state machine in run-video-proof.ts. quarantine-state.json tracks runs. Promote (5 clean) / escalate (10 retry) thresholds. Commit e15efa8."

- id: BDD-003
  title: IPFS pin sweep job
  category: bdd-testing
  priority: P2
  effort: 1d
  agent_role: bdd-engineer
  status: ready
  depends_on: []
  unblocks: []

- id: BDD-004
  title: BDD video skill productization
  category: bdd-testing
  priority: P1
  effort: 3-5d
  agent_role: skill-pack-maintainer
  status: ready
  depends_on: [BDD-001, BDD-002]
  unblocks: []

- id: BDD-005
  title: testid drift detection
  category: bdd-testing
  priority: P0
  effort: 1d
  agent_role: bdd-engineer
  status: done
  depends_on: []
  unblocks: []
  completed_at: "2026-05-09"
  notes: "validate-testid-coverage.ts + testid-patterns.json + testid-allowlist.json (52 baseline entries) + CI workflow testid-drift.yml. pnpm validate:testids. Commit bf4176d in ssr-frontend."

- id: BDD-006
  title: Immutable-tests CLAUDE.md rule
  category: bdd-testing
  priority: P0
  effort: 0.5d
  agent_role: docs-writer
  status: done
  depends_on: []
  unblocks: []
  completed_at: "2026-05-09"
  notes: "Added to ssr-frontend/CLAUDE.md (commit ba52048) and prometheus-skill-pack/CLAUDE.md (commit 38f83e0). Explicit prose: agents may not edit tests/steps/*.steps.ts."

- id: BDD-007
  title: Candidate test drafts directory
  category: bdd-testing
  priority: P1
  effort: 1d
  agent_role: bdd-engineer
  status: done
  depends_on: []
  unblocks: [BDD-015]
  completed_at: "2026-05-09"
  notes: "tests/features/drafts/ created; cucumber.js excludes drafts from default + ui profiles; test:bdd:drafts profile added; promote-draft.ts script; CLAUDE.md Candidate Test Drafts section. Commit f34fc58 in ssr-frontend."

- id: BDD-008
  title: pk-codegraph extraction
  category: bdd-testing
  priority: P0
  effort: 1-2w
  agent_role: rust-codegraph
  status: done
  depends_on: []
  unblocks: [BDD-009, BDD-010, BDD-013]
  completed_at: "2026-05-09"
  notes: "Static codegraph extraction complete. scripts/codegraph-extract.ts (ssr-frontend) produces tests/reports/codegraph.json (38 features, 351 scenarios, 80 testids, 37 source files). CI workflow added (.github/workflows/codegraph.yml). pk codegraph extract subcommand added to pk-cli. Commits: 1564b18 (ssr-frontend), da2b120 (prometheus-knowledge)."

- id: BDD-009
  title: pk-codegraph runtime coverage
  category: bdd-testing
  priority: P1
  effort: 1w
  agent_role: rust-codegraph
  status: planned
  depends_on: [BDD-008]
  unblocks: [BDD-010]

- id: BDD-010
  title: Impact-set hash test runner
  category: bdd-testing
  priority: P0
  effort: 1-2d
  agent_role: bdd-engineer
  status: planned
  depends_on: [BDD-008, BDD-009]
  unblocks: [BDD-012]

- id: BDD-011
  title: Environment hash augmentation
  category: bdd-testing
  priority: P1
  effort: 1d
  agent_role: bdd-engineer
  status: planned
  depends_on: [BDD-010]
  unblocks: []

- id: BDD-012
  title: Two-phase test gates
  category: bdd-testing
  priority: P1
  effort: 1d
  agent_role: bdd-engineer
  status: planned
  depends_on: [BDD-010, BDD-011]
  unblocks: []

- id: BDD-013
  title: User-story to feature contract
  category: bdd-testing
  priority: P1
  effort: 1w
  agent_role: docs-writer
  status: planned
  depends_on: [BDD-008]
  unblocks: [BDD-014]

- id: BDD-014
  title: Feedback aggregation in docs site
  category: bdd-testing
  priority: P1
  effort: 3-5d
  agent_role: bdd-engineer
  status: planned
  depends_on: [BDD-013]
  unblocks: []

- id: BDD-015
  title: Feedback record to draft-scenario emitter
  category: bdd-testing
  priority: P1
  effort: 3-5d
  agent_role: bdd-engineer
  status: planned
  depends_on: [BDD-007]
  unblocks: []

# ── 03 CROSS-CUTTING ───────────────────────────────────────────────

- id: XC-001
  title: Bug-fix-ledger quarterly invariant promotion
  category: cross-cutting
  priority: P2
  effort: recurring
  agent_role: skill-pack-maintainer
  status: ready
  depends_on: []
  unblocks: []

- id: XC-002
  title: Cross-model QA loop (Codex/GPT review)
  category: cross-cutting
  priority: P1
  effort: 2d
  agent_role: hooks-engineer
  status: ready
  depends_on: []
  unblocks: []

- id: XC-003
  title: Per-session SCRATCHPAD.md pattern
  category: cross-cutting
  priority: P2
  effort: 0.5d
  agent_role: docs-writer
  status: ready
  depends_on: []
  unblocks: []

- id: XC-004
  title: prometheus doctor end-to-end loop test
  category: cross-cutting
  priority: P1
  effort: 2-3d
  agent_role: skill-pack-maintainer
  status: planned
  depends_on: [SP-006, SP-012]
  unblocks: []

- id: XC-005
  title: prometheus init project-scoped overlay
  category: cross-cutting
  priority: P1
  effort: 2-3d
  agent_role: skill-pack-maintainer
  status: planned
  depends_on: [SP-008]
  unblocks: []
```

## Aggregate stats

```yaml
counts_by_status:
  ready: 20
  planned: 15
  in-progress: 0
  done: 6
  blocked: 0
  abandoned: 0

counts_by_priority:
  P0: 9
  P1: 22
  P2: 10

counts_by_agent_role:
  skill-pack-maintainer: 9
  hooks-engineer: 13
  rust-codegraph: 9
  bdd-engineer: 12
  docs-writer: 4
```

## Last update

2026-05-09 — Phase 1 complete. Marked done: SP-006, SP-013, SP-015, BDD-001, BDD-002, BDD-006. Promoted to ready: SP-012, SP-014, BDD-004. Session f265e820.
