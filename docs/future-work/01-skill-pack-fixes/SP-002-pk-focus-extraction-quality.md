---
id: SP-002
title: pk-focus keyword extraction quality (stopwords, gating)
status: ready
priority: P1
estimated_effort: 1d
agent_role: hooks-engineer
depends_on: []
unblocks: [SP-003, SP-004]
related: [SP-007]
created_from_conversation_turn: 3-4
---

# SP-002 — pk-focus keyword extraction quality

## Problem

`shared/scripts/pk-focus-on-prompt.sh` extracts keywords from the user's prompt and calls an LLM to retrieve relevant `prometheus-knowledge` entries. The current extraction is naive:

- No stopword list.
- No relevance gate.
- No caching.

Consequence: the script fires the LLM call on every prompt, even when the prompt is "thanks" or a yes/no acknowledgment. Cost and latency accumulate.

## Evidence

Read `shared/scripts/pk-focus-on-prompt.sh`. Identify the section that derives the search query from the prompt. It will (per the conversation) be a single tokenization step with no filtering.

Compare against best-practice query construction:

- Stopword exclusion (the, a, please, can, you, will, …).
- Minimum content length gate (e.g. <8 unique non-stopword tokens → skip retrieval).
- Cache by hash of normalized query → result, with TTL.

## Why it matters

A skill-pack user with N hook invocations per session burns N × (LLM-call-cost + LLM-call-latency) on retrievals. For sessions with 50+ short turns, this is meaningful — both in wall-clock latency before each turn and in API spend.

Beyond cost: an LLM call on a non-knowledge-bearing prompt produces stochastic output that may be irrelevant or wrong. If injected into the agent's context, it's noise that crowds out signal.

## Proposed fix

Add three layers to `pk-focus-on-prompt.sh` in order:

1. **Stopword filter.** Use a static list (English; ~300 entries is sufficient — pull from any standard stopword corpus). Strip stopwords from the prompt before tokenizing.
2. **Relevance gate.** After stopword filtering, count unique content tokens. If <8, skip the retrieval entirely and exit 0 silently.
3. **Result cache.** Hash the normalized query string. Check `~/.cache/prometheus/pk-focus/<hash>.json` for a cached result with `ttl > now`. If hit, use cached. If miss, call LLM, write result with `ttl = now + 1h`.

The cache lives under `~/.cache/prometheus/pk-focus/`. TTL of 1 hour is a starting heuristic; tune later. The cache should be project-aware (cache key includes project root path) so cross-project queries don't share results.

## Trade-offs and risks

- **Risk: stopword list excludes legitimate technical terms.** "Will" is a stopword but also a verb in user stories. "Can" is a stopword but appears in security-relevant prompts. Mitigation: stopword filtering is for query-construction only; the original prompt is preserved and used for the LLM call when one fires.
- **Risk: relevance gate is too aggressive and skips queries that would have benefited.** The 8-token threshold is heuristic. If users complain that "I need help with X" doesn't trigger retrieval, lower the threshold. The gate is tunable.
- **Risk: cache holds stale results when knowledge entries change.** TTL bounds this. For knowledge entries that change frequently, the librarian should invalidate cache entries that reference them; that's an enhancement, out of scope for this task.
- **Cost: cache directory grows.** A periodic cleanup is needed. Out of scope for this task; SP-009 owns scheduled jobs.

## Acceptance criteria

- [ ] `pk-focus-on-prompt.sh` skips retrieval when the post-stopword token count is below 8.
- [ ] Cached results are read from `~/.cache/prometheus/pk-focus/<hash>.json` when valid.
- [ ] Cache writes happen only on cache miss.
- [ ] A characterization-test script (e.g. `tests/pk-focus.bats`) exercises three cases: short non-knowledge prompt (skipped), long knowledge-relevant prompt (LLM called), repeat of long prompt within TTL (cache hit).
- [ ] All three cases exit with status 0 and produce expected stdout.

## Implementation steps

1. Characterize current behavior: write `tests/pk-focus.bats` against the script as it stands today. Capture inputs and exact outputs (the test will be ugly but is necessary).
2. Add the stopword filter as a new function in the script.
3. Add the relevance gate (token count check after filtering).
4. Add the cache layer using `mkdir -p`, `sha256sum`, simple JSON read/write.
5. Re-run the characterization test suite. Two of the three cases should pass without modification (the long-prompt cases). The short-prompt case should now exit silently rather than calling the LLM.
6. Update the test suite to assert the new behavior on the short-prompt case.
7. Document the cache location and TTL in the script's header comment.

## Dependencies

None.

## Open questions

- Should the relevance gate be per-prompt or per-session (e.g. always call once per session)? Default: per-prompt; users with frequent retrieval needs can set `PK_FOCUS_FORCE=1`.
- What stopword list source? Recommend the NLTK English stopword list; it's well-curated and widely vetted. Vendor it as `shared/data/stopwords-en.txt`.
- Should the cache serialize the entire LLM response or just the entries IDs? Latter is smaller and lets the librarian re-render with current entry content. Default: store entry IDs + retrieval timestamps.
