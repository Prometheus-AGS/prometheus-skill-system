# Future Work — Knowledge Pack

This directory contains the full output of an architectural review session held on **2026-05-09** between Travis James (Prometheus AGS) and Claude (Opus 4.7) covering:

1. **`prometheus-skill-pack`** — the agent skill management system at `/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/`.
2. **`prometheus-knowledge`** — the Karpathy-pattern Rust wiki at `/Users/gqadonis/Projects/prometheus/prometheus-knowledge/`.
3. **`ssr-frontend`** — the San Saba Royalty Next.js app at `/Users/gqadonis/Projects/sansaba/ssr-frontend/` and its BDD test infrastructure.
4. **`document-generation-agent`** — the related Mastra-based AI agent at `/Users/gqadonis/Projects/sansaba/document-generation-agent/` (referenced but not directly modified).

The session produced a structured inventory of:

- **15 specific weaknesses** in the existing skill-pack/knowledge stack.
- **6 targeted Karpathy/memory improvements** that build on what exists.
- **15 BDD-testing tasks** decomposed from five high-level user asks, with one of those asks (auto-update tests) reframed as a category error to avoid.
- **5 cross-cutting tasks** (bug-fix ledger review, cross-model QA, scratchpad pattern, doctor loop test, init overlay).
- **A build order with time estimates** prioritizing the highest-leverage work first.

This pack converts that inventory into 54 atomic, agent-consumable task documents that parallel Claude Code instances can pick up and execute independently.

## How to use this pack

If you are a **human reading this for the first time**:

1. Start with `INDEX.md` for the full file map.
2. Read `05-references/conversation-summary.md` for the full session context.
3. Read `04-build-order/execution-roadmap.md` for the recommended priority sequence.
4. Pick the highest-priority document from `01-skill-pack-fixes/`, `02-bdd-testing-evolution/`, or `03-cross-cutting/` and assign it to a Claude Code session.

If you are a **Claude Code agent picking up work**:

1. Read `00-meta/execution-protocol.md` to understand the contract.
2. Read `STATUS.md` to find a task whose `status` is `ready` and whose `agent_role` matches yours.
3. Mark its status `in-progress` in STATUS.md before starting work.
4. Read the task document in full, including `Trade-offs and risks`.
5. Implement, verify against `Acceptance criteria`, and update STATUS.md to `done`.
6. Read `00-meta/parallel-agent-routing.md` if you're unsure whether a task is yours.

## Memory tracking

This session was supposed to use the `surreal-memory-server` MCP to track task status, but that MCP did not surface during the session. As a substitute, `STATUS.md` at this directory's root holds the task graph in YAML format. The schema mirrors what `surreal-memory` entities would have been (see `00-meta/memory-schema.surql` for the intended Surreal schema). When you bring `surreal-memory` online via Claude Code, run the bootstrap script described in `00-meta/memory-bootstrap.md` to hydrate Surreal from `STATUS.md`.

## A note on honest framing

The conversation that produced this pack used the `sycophancy-correction` skill to keep Claude's analysis honest. One ask in particular — "tests should auto-update when code changes without reminding the AI tool" — was identified as a **category error** and explicitly is not implemented. See `02-bdd-testing-evolution/BDD-006-immutable-tests-rule.md` for the reframing and what should be built instead. If you (the human) disagree with this framing after reading the rationale, that is the right place to push back.

Other known framings in this pack that may invite pushback:

- **Bidirectional sync of user stories ↔ tests ↔ docs is unstable.** This pack picks one direction (tests-tagged-with-OpenSpec-change-IDs) and recommends the others be *generated outputs* of that direction, not parallel inputs. See `02-bdd-testing-evolution/BDD-013-story-feature-contract.md`.
- **The Karpathy KB at `~/.prometheus/knowledge/` is currently global and that is a confidentiality risk** (e.g. SSR session data leaking into a hypothetical Brius healthcare project). See `01-skill-pack-fixes/SP-008-per-project-kb-scoping.md` for the proposed per-project scoping.
- **Sycophancy correction not being wired into the PMPO Reflect phase is the single highest-leverage fix in the skill pack.** See `01-skill-pack-fixes/SP-013-sycophancy-reflector-hook.md`.

## Repository convention

All paths in task documents are absolute as authored on the Mac at `/Users/gqadonis/Projects/...`. When run on Linux, mentally substitute `/home/gqadonis/Projects/...`. The Mac username is `gqadonis` (note the `i`).

## Last updated

2026-05-09 — initial generation.
