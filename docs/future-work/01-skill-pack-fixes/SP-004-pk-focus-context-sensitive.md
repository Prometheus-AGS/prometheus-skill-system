---
id: SP-004
title: pk-focus context-sensitive multi-turn extractor
status: planned
priority: P1
estimated_effort: 2d
agent_role: hooks-engineer
depends_on: [SP-002]
unblocks: [SP-005]
related: []
created_from_conversation_turn: 3-4
---

# SP-004 — pk-focus context-sensitive multi-turn extractor

## Problem

Even with stopword filtering and gating from SP-002, `pk-focus` extracts query terms from the *current single prompt*. In a multi-turn session, the most relevant context for retrieval is often the *accumulated* conversation: the user said "the pk lint problem" referring to a topic mentioned 5 turns ago. Single-prompt extraction misses this entirely.

## Evidence

Capture a session of 10+ turns where the user uses pronouns and short references in later turns. Run the current `pk-focus` and observe what's retrieved — likely garbage for those later turns because the query is "the X problem" with no entity resolution.

## Why it matters

The librarian's value is highest in long, evolving sessions where context drifts. The current extractor degrades exactly when the agent needs help most.

## Proposed fix

Promote the extractor to a small LLM call (cheap model, e.g. local Qwen via litellm) that takes:

- The current prompt.
- A sliding window of the last 3-5 turns (assistant + user).

And outputs a structured query — entities and topics — for the librarian to retrieve against.

The cheap-LLM call replaces the naive tokenization. It runs inside `UserPromptSubmit` hook, returns within ~1 second on a local model, and produces a richer query than tokens alone.

The flow:

1. UserPromptSubmit fires.
2. SP-002's stopword filter and gate run first; if the gate passes, proceed.
3. The cheap-LLM extractor runs over the windowed context, producing `{entities: [...], topics: [...], free_terms: [...]}`.
4. The librarian retrieves against the structured query.
5. SP-002's cache stores the structured query → result mapping.

## Trade-offs and risks

- **Latency cost.** Adds ~500-1000ms per qualifying prompt. Mitigations: (a) run the extractor only when the cheap model is local (no network); (b) skip extractor entirely on first turn (no prior context); (c) parallelize the extractor call with whatever else `UserPromptSubmit` is doing.
- **Cheap-model quality.** A local Qwen may produce inconsistent structured output. Mitigation: strict-JSON output mode with schema validation; fall back to SP-002's tokenization on parse failure.
- **Privacy.** The extractor sees prior turns including any sensitive content. Stays local (no network); never sent to cloud.

## Acceptance criteria

- [ ] Extractor produces structured `{entities, topics, free_terms}` for typical multi-turn prompts.
- [ ] Falls back to SP-002 tokenization on extractor failure.
- [ ] Latency p95 under 1.2s on local model.
- [ ] On a 10-turn session with mid-conversation pronoun references, retrieval is meaningfully better than SP-002 alone (measured by relevance-rated retrieval count).

## Implementation steps

1. Define the extractor schema (JSON) in `shared/scripts/lib/pk-focus-schema.json`.
2. Write the extractor prompt in `shared/scripts/lib/pk-focus-extractor.prompt.md`.
3. Add `pk_focus_extract_context` function to `pk-focus-on-prompt.sh` that calls the cheap model with the windowed context.
4. Wire the structured query into the librarian retrieval call.
5. Write characterization tests with synthetic multi-turn fixtures.
6. Measure latency.

## Dependencies

SP-002 (stopword filter and gate must run first).

## Open questions

- How big is the window? Default: 3 prior turn pairs. Larger may help for some queries but bloats the extractor input.
- Does the extractor need persistent memory beyond the window? Possibly later — for now, the window plus the cache covers most cases.
