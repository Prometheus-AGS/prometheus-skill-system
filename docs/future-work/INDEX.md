# Index

Full map of every document in this pack, grouped by category. Each row shows the task ID, title, priority, estimated effort, and a one-line summary. `STATUS.md` at the root has the live status of each. See `04-build-order/dependencies-graph.md` for which tasks block which.

## Meta (`00-meta/`)

| File | Purpose |
|------|---------|
| `memory-schema.surql` | Surreal schema for tracking — tasks, doc plans, dependencies, agent roles |
| `memory-bootstrap.md` | How to seed `surreal-memory` from `STATUS.md` once Surreal is online |
| `execution-protocol.md` | The contract a Claude Code agent follows when picking up a task |
| `parallel-agent-routing.md` | Which `agent_role` handles which task families and why |

## Skill-pack fixes (`01-skill-pack-fixes/`)

These are the 15 honest weaknesses identified in the existing `prometheus-skill-pack` plus 6 targeted Karpathy/memory improvements. Total: 21 tasks.

| ID | Title | Priority | Effort |
|----|-------|----------|--------|
| SP-001 | Two CLAUDE.md files unification | P1 | 1d |
| SP-002 | pk-focus keyword extraction quality (stopwords, gating) | P1 | 1d |
| SP-003 | pk-focus result caching | P2 | 0.5d |
| SP-004 | pk-focus context-sensitive multi-turn extractor | P1 | 2d |
| SP-005 | `pk focus --inject-as system-context` flag | P2 | 0.5d |
| SP-006 | Stop hook observability log (`~/.prometheus/hooks.log`) | P0 | 1d |
| SP-007 | Trace capture file existence verification + implementation | P1 | 2d |
| SP-008 | Karpathy KB per-project scoping (confidentiality) | P0 | 1-2d |
| SP-009 | `pk lint --fix` scheduled job | P2 | 0.5d |
| SP-010 | `compile_user_prompt` strict JSON parser | P1 | 1d |
| SP-011 | Cedar gate at PostToolUse for SKILL.md edits | P1 | 1d |
| SP-012 | 4-layer pipeline enforcement hook | P1 | 2-3d |
| SP-013 | **Sycophancy correction in SubagentStop(reflector) hook** | P0 | 1-2d |
| SP-014 | SubagentStop fallback matcher verification | P2 | 0.5d |
| SP-015 | hooks.json symlink fix | P2 | 0.5d |
| SP-016 | Skill description collision detection (skill-matrix.js) | P1 | 1d |
| SP-017 | Slash command merge strategy (skill-pack vs pk) | P2 | 1d |
| SP-018 | End-to-end pipeline smoke test | P1 | 2-3d |
| SP-019 | LibrarianEvent first-class persistence | P0 | 1w |
| SP-020 | Memory dual-store separation (KG vs episodic) | P1 | 3-5d |
| SP-021 | mem0 compress_memories scheduled job | P2 | 1d |

## BDD testing evolution (`02-bdd-testing-evolution/`)

The five SSR asks decomposed into 15 atomic tasks. Note BDD-006 reframes the "auto-update tests" ask as a category error; what is actually implemented is selector-drift detection plus an immutable-tests rule.

| ID | Title | Priority | Effort |
|----|-------|----------|--------|
| BDD-001 | Manifest dual-key cleanup migration | P0 | 0.5d |
| BDD-002 | Flake quarantine system (@quarantine tag + retry policy) | P0 | 1d |
| BDD-003 | IPFS pin sweep job | P2 | 1d |
| BDD-004 | BDD video skill productization | P1 | 3-5d |
| BDD-005 | testid drift detection (`validate-testid-coverage.ts`) | P0 | 1d |
| BDD-006 | Immutable-tests CLAUDE.md rule | P0 | 0.5d |
| BDD-007 | Candidate test drafts (`tests/features/drafts/`) | P1 | 1d |
| BDD-008 | pk-codegraph extraction (ts-morph + Surreal) | P0 | 1-2w |
| BDD-009 | pk-codegraph runtime coverage (Playwright trace ingestion) | P1 | 1w |
| BDD-010 | Impact-set hash test runner | P0 | 1-2d |
| BDD-011 | Environment hash augmentation (prisma + .env + migrations) | P1 | 1d |
| BDD-012 | Two-phase test gates (PR fast / release thorough) | P1 | 1d |
| BDD-013 | User-story to feature contract (OpenSpec change-id tagging) | P1 | 1w |
| BDD-014 | Feedback aggregation in docs site | P1 | 3-5d |
| BDD-015 | Feedback record to draft-scenario emitter | P1 | 3-5d |

## Cross-cutting (`03-cross-cutting/`)

| ID | Title | Priority | Effort |
|----|-------|----------|--------|
| XC-001 | Bug-fix-ledger quarterly invariant promotion | P2 | recurring |
| XC-002 | Cross-model QA loop (Codex/GPT review) | P1 | 2d |
| XC-003 | Per-session SCRATCHPAD.md pattern | P2 | 0.5d |
| XC-004 | `prometheus doctor` end-to-end loop test | P1 | 2-3d |
| XC-005 | `prometheus init` project-scoped overlay | P1 | 2-3d |

## Build order (`04-build-order/`)

| File | Purpose |
|------|---------|
| `execution-roadmap.md` | Recommended sequence with rationale |
| `parallel-work-streams.md` | Which streams can run concurrently with which |
| `dependencies-graph.md` | Mermaid graph of blocks/unblocks edges |

## References (`05-references/`)

| File | Purpose |
|------|---------|
| `conversation-summary.md` | Full distilled session, including the sycophancy lesson |
| `existing-system-inventory.md` | What was found to already exist (so don't rebuild it) |
| `architectural-patterns.md` | Patterns invoked: Karpathy context engineering, characterization tests, broad-change threshold, etc. |
| `methodology-validations.md` | External references that informed the analysis |

## Counts

- Meta: 4 files
- Skill-pack fixes: 21 files
- BDD testing evolution: 15 files
- Cross-cutting: 5 files
- Build order: 3 files
- References: 4 files
- Top-level: README.md, INDEX.md, STATUS.md
- **Total: 55 files**
