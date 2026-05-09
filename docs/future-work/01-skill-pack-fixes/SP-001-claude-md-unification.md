---
id: SP-001
title: Two CLAUDE.md files unification
status: ready
priority: P1
estimated_effort: 1d
agent_role: skill-pack-maintainer
depends_on: []
unblocks: []
related: [SP-016, SP-017]
created_from_conversation_turn: 3-4
---

# SP-001 — Two CLAUDE.md files unification

## Problem

Two `CLAUDE.md` files exist in the prometheus stack and define overlapping rules with no unification authority:

- `/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/CLAUDE.md` (the pack itself).
- `/Users/gqadonis/Projects/prometheus/prometheus-knowledge/CLAUDE.md` (the Karpathy wiki crate).

Each prescribes rules about KDD lifecycle, OpenSpec compliance, and library curation. Where they overlap they sometimes contradict. There is no precedence rule documented.

## Evidence

Read both files end-to-end. Look for:

- Sections that describe the KDD phases (assess/plan/execute/reflect). Both have language. Compare.
- Library/skill discovery rules. Both define rules; the prometheus-knowledge version focuses on curation, the skill-pack version on usage.
- The list of mandatory pre-commit checks. Likely diverges.

## Why it matters

When a Claude Code agent starts a session in either repository, it loads the local CLAUDE.md and derives rules from it. If a session in prometheus-skill-pack invokes a tool from prometheus-knowledge (or vice versa), the agent's behavior depends on which CLAUDE.md it happened to read first. This is non-deterministic.

The longer-term cost is documentation entropy: as either file evolves, divergence accumulates and contradictions grow undetected.

## Proposed fix

Establish a single canonical CLAUDE.md with a clear precedence rule:

1. **Promote** the skill-pack `CLAUDE.md` to canonical-for-pack-and-dependents. The pack is the meta-layer.
2. **Reduce** the prometheus-knowledge `CLAUDE.md` to crate-specific concerns only (Rust workspace conventions, librarian-specific behavior, model routing per-task). Remove anything that overlaps with general KDD/OpenSpec/skill-discovery rules.
3. **Add a short header** to the prometheus-knowledge `CLAUDE.md` stating: "For project-wide conventions (KDD, skill discovery, OpenSpec), see prometheus-skill-pack/CLAUDE.md. This file covers prometheus-knowledge-specific rules only."
4. **Document the precedence** in the skill-pack `CLAUDE.md` under a new section "Documentation hierarchy."

## Trade-offs and risks

- **Risk: prometheus-knowledge becomes incomplete** if its CLAUDE.md is reduced too aggressively. Some Rust-specific or librarian-specific rules genuinely belong only in the local file. The reduction should be additive removal of duplicates, not aggressive deletion.
- **Risk: someone adds a new project-wide rule to the prometheus-knowledge CLAUDE.md** without realizing it's the wrong place. Mitigation: add a `pre-commit` hook (or CI check) that flags additions to the prometheus-knowledge CLAUDE.md and prompts for review.
- **Cost: existing sessions in flight** may be running off the old files. The unification should be communicated to active developers.

## Acceptance criteria

- [ ] The skill-pack `CLAUDE.md` contains the canonical KDD lifecycle, skill discovery, and OpenSpec rules.
- [ ] The prometheus-knowledge `CLAUDE.md` contains *only* crate-specific rules and references the skill-pack file for everything else.
- [ ] A "Documentation hierarchy" section in the skill-pack `CLAUDE.md` states the precedence explicitly.
- [ ] No rule appears verbatim in both files.
- [ ] A grep for "KDD" and "OpenSpec" in the prometheus-knowledge CLAUDE.md returns only references-to-skill-pack lines, not rule definitions.

## Implementation steps

1. Read both files; produce a side-by-side diff of sections.
2. Categorize each rule: project-wide, pack-specific, or knowledge-specific.
3. Move project-wide rules to the canonical skill-pack CLAUDE.md. Delete duplicates.
4. Move pack-specific rules to the skill-pack CLAUDE.md (already there in most cases).
5. Reduce the prometheus-knowledge CLAUDE.md to knowledge-specific rules with a header pointing to canonical.
6. Add the "Documentation hierarchy" section to the canonical file.
7. Commit each repository with a message linking to this task.
8. Verify by running a fresh Claude Code session in each repo and checking that the agent loads the right file and references the canonical one when needed.

## Dependencies

None.

## Open questions

- Is there a third CLAUDE.md somewhere (e.g. in `prometheus-fabric` or another related repo) that should also be considered? Verify before declaring done.
- Should this work also produce an `AGENTS.md` reconciliation? The skill-pack has `AGENTS.md`; prometheus-knowledge does not. Out of scope for this task unless investigation shows divergence; track separately if so.
