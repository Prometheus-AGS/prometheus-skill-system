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
  notes: "Documentation hierarchy table added to skill-pack CLAUDE.md (canonical designation). prometheus-knowledge CLAUDE.md gets header pointing to canonical. Commits: 202ad73 (skill-pack), 2594e6f (knowledge)."

- id: SP-002
  title: pk-focus keyword extraction quality
  category: skill-pack-fixes
  priority: P1
  effort: 1d
  agent_role: hooks-engineer
  status: done
  depends_on: []
  unblocks: [SP-003, SP-004]
  completed_at: "2026-05-09"
  notes: "Sliding-window extraction in pk-librarian/src/keyword_extract.rs. WINDOW=1000, STEP=600, DECAY=0.85. Dynamic cutoff replaces fixed MIN_SCORE. Commit 206c5f7 (prometheus-knowledge)."

- id: SP-003
  title: pk-focus result caching
  category: skill-pack-fixes
  priority: P2
  effort: 0.5d
  agent_role: hooks-engineer
  status: done
  depends_on: [SP-002]
  unblocks: []
  completed_at: "2026-05-09"
  notes: "SHA256-keyed cache at ~/.prometheus/pk-focus-cache/. --no-cache flag to bypass. sha2 + dirs crates added to workspace. Commit 21651f0 (prometheus-knowledge)."

- id: SP-004
  title: pk-focus context-sensitive multi-turn extractor
  category: skill-pack-fixes
  priority: P1
  effort: 2d
  agent_role: hooks-engineer
  status: done
  depends_on: [SP-002]
  unblocks: [SP-005]
  completed_at: "2026-05-09"
  notes: "extract_query_multi_turn() + pk focus --context-window flag. Per-turn decay. Commit 278b87c (prometheus-knowledge)."

- id: SP-005
  title: pk focus --inject-as system-context flag
  category: skill-pack-fixes
  priority: P2
  effort: 0.5d
  agent_role: rust-codegraph
  status: done
  depends_on: [SP-004]
  unblocks: []
  completed_at: "2026-05-09"
  notes: "--inject-as-system-context flag wraps output in <system-context> tags. Cache-aware. Commit c523649 (prometheus-knowledge)."

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
  notes: "verify-trace-state.sh + PROMETHEUS_TRACE_DIR env var. Commit abd79c0."

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
  notes: "KbScope enum + --scope/--yes flags + migrate-to-per-project subcommand. Commit 84aa366."

- id: SP-009
  title: pk lint --fix scheduled job
  category: skill-pack-fixes
  priority: P2
  effort: 0.5d
  agent_role: hooks-engineer
  status: done
  depends_on: []
  unblocks: []
  completed_at: "2026-05-09"
  notes: "pk-lint.sh + launchd plist (Saturday 03:00) + cron snippet. shared/scripts/scheduled/. Commit dba8af3 (skill-pack)."

- id: SP-010
  title: compile_user_prompt strict JSON parser
  category: skill-pack-fixes
  priority: P1
  effort: 1d
  agent_role: rust-codegraph
  status: done
  depends_on: []
  unblocks: []
  completed_at: "2026-05-09"
  notes: "ParseError::{EmptyResponse,InvalidJson,MissingField} + strip_fences() + parse_json(). Commit e903668 (prometheus-knowledge)."

- id: SP-011
  title: Cedar gate at PostToolUse for SKILL.md edits
  category: skill-pack-fixes
  priority: P1
  effort: 1d
  agent_role: hooks-engineer
  status: done
  depends_on: []
  unblocks: []
  completed_at: "2026-05-09"
  notes: "cedar-skill-gate.sh validates name pattern, required frontmatter fields, no backslashes. PreToolUse Write|Edit hook. Commit 557304a (skill-pack)."

- id: SP-012
  title: 4-layer pipeline enforcement hook
  category: skill-pack-fixes
  priority: P1
  effort: 2-3d
  agent_role: hooks-engineer
  status: done
  depends_on: [SP-006]
  unblocks: [SP-018]
  completed_at: "2026-05-09"
  notes: "pipeline-enforce.sh blocks kbd-execute/kbd-reflect when plan or prior phase missing. 7 smoke tests. Commit cba92ac (skill-pack)."

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
  notes: "sycophancy-check-reflection.sh + 2-rejection soft cap + PROMETHEUS_REFLECT_STRICTNESS env var. Commit aa2a5b8."

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
  notes: "test-subagent-fallback.sh — 10 assertions, 5/5 consecutive passes. Commit a374bd5."

- id: SP-015
  title: hooks.json symlink fix
  category: skill-pack-fixes
  priority: P2
  effort: 0.5d
  agent_role: skill-pack-maintainer
  status: done
  completed_at: "2026-05-09"
  notes: "Direction confirmed correct. CI hooks-integrity job added. Commit c586a77."
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
  notes: "scripts/skill-matrix.js (Jaccard pairwise); skill-collision-allowlist.json; CI job. Commit 6d40af4."

- id: SP-017
  title: Slash command merge strategy
  category: skill-pack-fixes
  priority: P2
  effort: 1d
  agent_role: skill-pack-maintainer
  status: done
  depends_on: []
  unblocks: []
  completed_at: "2026-05-09"
  notes: "pk-focus/pk-ingest renamed in prometheus-knowledge; detect-command-conflicts.sh shipped to skill-pack; prefix convention documented in both CLAUDE.md files. Commits: f567e9f (skill-pack), ee611fc (knowledge)."

- id: SP-018
  title: End-to-end pipeline smoke test
  category: skill-pack-fixes
  priority: P1
  effort: 2-3d
  agent_role: hooks-engineer
  status: done
  depends_on: [SP-006, SP-012]
  unblocks: []
  completed_at: "2026-05-09"
  notes: "shared/scripts/tests/test-pipeline-smoke.sh — 7 integration tests (block + pass-through cases for both kbd-execute and kbd-reflect). All pass. Commit cba92ac (skill-pack, bundled with SP-012)."

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
  notes: "pk-event-store crate: EventRecord, EventStore (SurrealDB HTTP + JSONL fallback). pk events list/for-entry subcommands. Commit f041b11."

- id: SP-020
  title: Memory dual-store separation
  category: skill-pack-fixes
  priority: P1
  effort: 3-5d
  agent_role: rust-codegraph
  status: done
  depends_on: [SP-019]
  unblocks: []
  completed_at: "2026-05-09"
  notes: "dual_store.rs + migrate.rs + MigrateStores CLI subcommand. KG store (kg db) vs Episodic store (episode db). Commit f8dce14 (prometheus-knowledge)."

- id: SP-021
  title: mem0 compress_memories scheduled job
  category: skill-pack-fixes
  priority: P2
  effort: 1d
  agent_role: hooks-engineer
  status: done
  depends_on: []
  unblocks: []
  completed_at: "2026-05-09"
  notes: "mem0-compress.sh + launchd plist (Sunday 03:00) + cron snippet. Falls back to MCP HTTP if pk unavailable. Commit 79bb39b (skill-pack)."

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
  notes: "@quarantine retry + state machine in run-video-proof.ts. quarantine-state.json. Promote (5 clean) / escalate (10 retry) thresholds. Commit e15efa8."

- id: BDD-003
  title: IPFS pin sweep job
  category: bdd-testing
  priority: P2
  effort: 1d
  agent_role: bdd-engineer
  status: done
  depends_on: []
  unblocks: []
  completed_at: "2026-05-09"
  notes: "scripts/ipfs-pin-sweep.ts — compares ipfs pin ls vs docs/videos-manifest.json; --dry-run / --execute flags. Commit aa40765 (ssr-frontend)."

- id: BDD-004
  title: BDD video skill productization
  category: bdd-testing
  priority: P1
  effort: 3-5d
  agent_role: skill-pack-maintainer
  status: done
  depends_on: [BDD-001, BDD-002]
  unblocks: []
  completed_at: "2026-05-09"
  notes: "skills/testing/bdd-video-proof/ — SKILL.md + references/IPFS.md + references/SETUP.md. Passes validate:strict. Commit ff076fd (skill-pack)."

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
  notes: "validate-testid-coverage.ts + testid-patterns.json + testid-allowlist.json (52 baseline entries) + CI workflow testid-drift.yml. Commit bf4176d (ssr-frontend)."

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
  notes: "Added to ssr-frontend/CLAUDE.md and prometheus-skill-pack/CLAUDE.md. Commits: ba52048, 38f83e0."

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
  notes: "tests/features/drafts/ + promote-draft.ts + cucumber.js drafts profile. Commit f34fc58 (ssr-frontend)."

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
  notes: "scripts/codegraph-extract.ts produces codegraph.json (38 features, 351 scenarios, 80 testids, 37 source files). CI workflow codegraph.yml. pk codegraph extract CLI. Commits: 1564b18 (ssr-frontend), da2b120 (knowledge)."

- id: BDD-009
  title: pk-codegraph runtime coverage
  category: bdd-testing
  priority: P1
  effort: 1w
  agent_role: rust-codegraph
  status: done
  depends_on: [BDD-008]
  unblocks: [BDD-010]
  completed_at: "2026-05-09"
  notes: "scripts/merge-runtime-coverage.ts merges playwright reporter output into codegraph. Commit 2673719 (ssr-frontend)."

- id: BDD-010
  title: Impact-set hash test runner
  category: bdd-testing
  priority: P0
  effort: 1-2d
  agent_role: bdd-engineer
  status: done
  depends_on: [BDD-008, BDD-009]
  unblocks: [BDD-012]
  completed_at: "2026-05-09"
  notes: "scripts/run-impact-set.ts + pk:impact-run CI job. Runs only scenarios whose source files changed since last coverage snapshot. Commit 5b30a2a (ssr-frontend)."

- id: BDD-011
  title: Environment hash augmentation
  category: bdd-testing
  priority: P1
  effort: 1d
  agent_role: bdd-engineer
  status: done
  depends_on: [BDD-010]
  unblocks: []
  completed_at: "2026-05-09"
  notes: "scripts/compute-environment-hash.ts — SHA256 over Node version + pnpm lock + feature file set. Hash stored in test-run-meta.json. Commit f3e2a96 (ssr-frontend)."

- id: BDD-012
  title: Two-phase test gates
  category: bdd-testing
  priority: P1
  effort: 1d
  agent_role: bdd-engineer
  status: done
  depends_on: [BDD-010, BDD-011]
  unblocks: []
  completed_at: "2026-05-09"
  notes: "GitHub Actions workflow bdd-two-phase.yml: fast-bdd (impact set, <5 min) + selective-bdd (full suite, nightly + env hash change). Commit 65f950d (ssr-frontend)."

- id: BDD-013
  title: User-story to feature contract
  category: bdd-testing
  priority: P1
  effort: 1w
  agent_role: docs-writer
  status: done
  depends_on: [BDD-008]
  unblocks: [BDD-014]
  completed_at: "2026-05-09"
  notes: "STORY-FEATURE-CONTRACT.md + validate-change-ids.ts + @change-id: tag convention in codegraph. Commit 0b7bd60 (ssr-frontend)."

- id: BDD-014
  title: Feedback aggregation in docs site
  category: bdd-testing
  priority: P1
  effort: 3-5d
  agent_role: bdd-engineer
  status: done
  depends_on: [BDD-013]
  unblocks: []
  completed_at: "2026-05-09"
  notes: "DocsFeedbackWidget (thumbs up/down + comment) + /api/docs-feedback route + type definitions + running summary. Commit 5ac10d8 (ssr-frontend)."

- id: BDD-015
  title: Feedback record to draft-scenario emitter
  category: bdd-testing
  priority: P1
  effort: 3-5d
  agent_role: bdd-engineer
  status: done
  depends_on: [BDD-007]
  unblocks: []
  completed_at: "2026-05-09"
  notes: "scripts/feedback-to-draft.ts — --threshold (default 3) + --dry-run. Emits @feedback-sourced tagged drafts. Commit 2558a4a (ssr-frontend)."

# ── 03 CROSS-CUTTING ───────────────────────────────────────────────

- id: XC-001
  title: Bug-fix-ledger quarterly invariant promotion
  category: cross-cutting
  priority: P2
  effort: recurring
  agent_role: skill-pack-maintainer
  status: done
  depends_on: []
  unblocks: []
  completed_at: "2026-05-09"
  notes: "docs/BUG-FIX-LEDGER.md created with Q2 2026 first review (BF-001 through BF-005). Process documented. Commit dba2594 (skill-pack)."

- id: XC-002
  title: Cross-model QA loop (secondary model review)
  category: cross-cutting
  priority: P1
  effort: 2d
  agent_role: hooks-engineer
  status: done
  depends_on: []
  unblocks: []
  completed_at: "2026-05-09"
  notes: ".github/workflows/cross-model-qa.yml — workflow_dispatch with pr_number/model/focus inputs. Posts review as PR comment or stdout. Commit 27e04a3 (skill-pack)."

- id: XC-003
  title: Per-session SCRATCHPAD.md pattern
  category: cross-cutting
  priority: P2
  effort: 0.5d
  agent_role: docs-writer
  status: done
  depends_on: []
  unblocks: []
  completed_at: "2026-05-09"
  notes: "Session Scratchpad Pattern section added to CLAUDE.md. SCRATCHPAD.md added to .gitignore. Commit 7049920 (skill-pack)."

- id: XC-004
  title: prometheus doctor end-to-end loop test
  category: cross-cutting
  priority: P1
  effort: 2-3d
  agent_role: skill-pack-maintainer
  status: done
  depends_on: [SP-006, SP-012]
  unblocks: []
  completed_at: "2026-05-09"
  notes: "pk doctor subcommand — 5 checks (hooks-log, sycophancy binary, hooks.json symlink, pipeline-enforce, KB scoping). --json flag. Commit 7df2457 (prometheus-knowledge)."

- id: XC-005
  title: prometheus init project-scoped overlay
  category: cross-cutting
  priority: P1
  effort: 2-3d
  agent_role: skill-pack-maintainer
  status: done
  depends_on: [SP-008]
  unblocks: []
  completed_at: "2026-05-09"
  notes: "pk init subcommand — creates .prometheus/knowledge/, generates CLAUDE.md scaffold, updates .gitignore. --name/--stack/--yes flags. Commit 1dc65c0 (prometheus-knowledge)."
```

## Aggregate stats

```yaml
counts_by_status:
  ready: 0
  planned: 0
  in-progress: 0
  done: 41
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

2026-05-09 — All phases complete. 41/41 tasks done. Phases 1–6 fully executed.
Sessions: Phase 1 (f265e820), Phases 2–6 (f265e820 continued across compaction boundaries).
