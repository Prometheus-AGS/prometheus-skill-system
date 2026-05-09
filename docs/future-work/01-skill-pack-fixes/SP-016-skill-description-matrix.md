---
id: SP-016
title: Skill description collision detection (skill-matrix.js)
status: ready
priority: P1
estimated_effort: 1d
agent_role: skill-pack-maintainer
depends_on: []
unblocks: []
related: [SP-001, SP-017]
created_from_conversation_turn: 3-4
---

# SP-016 — Skill description collision detection

## Problem

The pack ships 64 skills, each with a `description` field in its frontmatter that the agent uses to decide when to invoke that skill. With 64 entries, near-miss descriptions are statistically guaranteed — two skills that look semantically equivalent to the agent's matcher, leading to ambiguous invocation.

There is no current automated check for description collisions.

## Evidence

```
$ find skills -name SKILL.md | wc -l   # ~64
$ for f in skills/*/SKILL.md; do head -10 "$f" | grep -A1 '^description:'; done | sort | head -50
```

Inspect the descriptions. Many will be similar. A measure of similarity (cosine on embeddings, or BM25 between descriptions) will surface the candidates.

## Why it matters

The agent's skill-matching is non-deterministic when descriptions collide. Two skills with descriptions like "create BDD tests for behavior validation" and "generate Cucumber tests for behavior testing" might both fire on the same prompt, producing inconsistent agent behavior.

This is a *pack quality* problem. Users adding skills don't know to check.

## Proposed fix

Add a `scripts/skill-matrix.js` (or `.ts`) that:

1. Reads all `skills/*/SKILL.md` frontmatter.
2. Extracts `name` and `description` per skill.
3. Computes pairwise similarity using a lightweight method (start with: shingle-based Jaccard on lowercase tokens; upgrade later if needed).
4. Reports pairs above a similarity threshold (e.g. 0.6 Jaccard) with both descriptions side-by-side.
5. Optionally computes an embedding-based similarity for higher-fidelity ranking (require an embedding model only if available; fall back to lexical).

Wire it into CI:
- On PR: report pairs above threshold; fail if a *new* pair appears that wasn't already on the allow-list.
- Allow-list at `scripts/skill-collision-allowlist.json` for known acceptable overlaps.

## Trade-offs and risks

- **Risk: embedding-based similarity requires running a model in CI.** Mitigation: lexical Jaccard is sufficient for first version; embeddings are an upgrade.
- **Risk: false positives flag legitimate skill families.** Mitigation: allow-list entries with reasoning notes.
- **Risk: thresholds need tuning.** Start at 0.6 Jaccard; observe.

## Acceptance criteria

- [ ] `scripts/skill-matrix.js` exists and runs `node scripts/skill-matrix.js` from pack root with no error.
- [ ] Output lists existing pairs above threshold with both descriptions.
- [ ] CI check fails on PRs that introduce new pairs above threshold without allow-list addition.
- [ ] Allow-list mechanism documented.
- [ ] Run on the current 64 skills produces a manageable list (initial expectation: 5-15 pairs to review).

## Implementation steps

1. Write the script (Node, simple Jaccard).
2. Run against current skills; capture output.
3. Triage: each pair → genuine duplicate (rename one), genuine overlap (allow-list with reasoning), or false positive (allow-list).
4. Add the CI check.
5. Document in pack README under "Adding a new skill: avoiding description collisions."

## Dependencies

None.

## Open questions

- Should the matrix also detect *missing* coverage (areas where no skill matches typical user prompts)? Out of scope here; possible follow-up.
- Should it include `name` collisions as well? Trivial to add — yes.
