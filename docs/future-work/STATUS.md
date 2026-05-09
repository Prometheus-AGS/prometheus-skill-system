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
  status: ready
  depends_on: []
  unblocks: []

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
  status: ready
  depends_on: []
  unblocks: [SP-014, SP-018]

- id: SP-007
  title: Trace capture file existence verification
  category: skill-pack-fixes
  priority: P1
  effort: 2d
  agent_role: hooks-engineer
  status: ready
  depends_on: []
  unblocks: [SP-019]

- id: SP-008
  title: Karpathy KB per-project scoping
  category: skill-pack-fixes
  priority: P0
  effort: 1-2d
  agent_role: rust-codegraph
  status: ready
  depends_on: []
  unblocks: [SP-019]

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
  status: planned
  depends_on: [SP-006]
  unblocks: [SP-018]

- id: SP-013
  title: Sycophancy correction in SubagentStop(reflector) hook
  category: skill-pack-fixes
  priority: P0
  effort: 1-2d
  agent_role: hooks-engineer
  status: ready
  depends_on: []
  unblocks: []
  notes: "Highest-leverage fix in the pack. Wires the existing sycophancy-correction skill into the Reflect phase of PMPO so critic agent never sees generation history."

- id: SP-014
  title: SubagentStop fallback matcher verification
  category: skill-pack-fixes
  priority: P2
  effort: 0.5d
  agent_role: hooks-engineer
  status: planned
  depends_on: [SP-006]
  unblocks: []

- id: SP-015
  title: hooks.json symlink fix
  category: skill-pack-fixes
  priority: P2
  effort: 0.5d
  agent_role: skill-pack-maintainer
  status: ready
  depends_on: []
  unblocks: []

- id: SP-016
  title: Skill description collision detection
  category: skill-pack-fixes
  priority: P1
  effort: 1d
  agent_role: skill-pack-maintainer
  status: ready
  depends_on: []
  unblocks: []

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
  status: planned
  depends_on: [SP-007, SP-008]
  unblocks: [SP-020]

- id: SP-020
  title: Memory dual-store separation
  category: skill-pack-fixes
  priority: P1
  effort: 3-5d
  agent_role: rust-codegraph
  status: planned
  depends_on: [SP-019]
  unblocks: []

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
  status: ready
  depends_on: []
  unblocks: []

- id: BDD-002
  title: Flake quarantine system
  category: bdd-testing
  priority: P0
  effort: 1d
  agent_role: bdd-engineer
  status: ready
  depends_on: []
  unblocks: []

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
  status: planned
  depends_on: [BDD-001, BDD-002]
  unblocks: []

- id: BDD-005
  title: testid drift detection
  category: bdd-testing
  priority: P0
  effort: 1d
  agent_role: bdd-engineer
  status: ready
  depends_on: []
  unblocks: []

- id: BDD-006
  title: Immutable-tests CLAUDE.md rule
  category: bdd-testing
  priority: P0
  effort: 0.5d
  agent_role: docs-writer
  status: ready
  depends_on: []
  unblocks: []
  notes: "This is the reframing of the 'auto-update tests' ask as a category error. Reading the doc before objecting is essential."

- id: BDD-007
  title: Candidate test drafts directory
  category: bdd-testing
  priority: P1
  effort: 1d
  agent_role: bdd-engineer
  status: ready
  depends_on: []
  unblocks: [BDD-015]

- id: BDD-008
  title: pk-codegraph extraction
  category: bdd-testing
  priority: P0
  effort: 1-2w
  agent_role: rust-codegraph
  status: ready
  depends_on: []
  unblocks: [BDD-009, BDD-010, BDD-013]
  notes: "Highest-leverage BDD task. Foundation for impact-set computation and selective test execution."

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
  ready: 23
  planned: 18
  in-progress: 0
  done: 0
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

2026-05-09 — Initial generation. All tasks `ready` or `planned`. No work started.
