# Architectural Patterns

The patterns invoked during the session, with citations and how each maps to specific tasks in this pack.

## Karpathy context engineering taxonomy

Karpathy's framing (popularized in his October 2024 series and in subsequent LangChain/LangGraph documentation) decomposes the work of "preparing context for an LLM" into four operations:

- **Write** — persist information that will be needed later. Distinct from in-memory context. Examples: scratch pads, episodic memory, knowledge wikis. The act of *deciding what to write* is itself a context-engineering decision.
- **Select** — retrieve relevant prior writes when constructing the current context window. RAG is one mechanism; structured memory queries are another. Selection quality dominates effective context.
- **Compress** — when the relevant prior content exceeds the budget, lossy reduce it. Summarization is the canonical example, but compression also covers structured extraction (e.g. "the assertions from the conversation") and quotient operations (e.g. "everything except this user's PII").
- **Isolate** — keep contexts that should not co-mingle from co-mingling. Per-project knowledge bases. Per-tenant memories. Per-conversation scratchpads. Isolation is the structural integrity guarantee.

This pack's mapping:

| Pattern | Tasks |
|---------|-------|
| Write | SP-007 (trace capture), SP-019 (LibrarianEvent persistence), XC-003 (per-session SCRATCHPAD.md) |
| Select | SP-002, SP-003, SP-004 (pk-focus quality), SP-005 (--inject-as system-context), BDD-008 (codegraph for retrieval-by-impact) |
| Compress | SP-021 (mem0 compress_memories), parts of SP-019 (event aggregation in librarian) |
| Isolate | SP-008 (per-project KB scoping), SP-020 (memory dual-store separation), XC-005 (project-scoped overlay) |

The honest framing: a knowledge system that does only Select (retrieval) without doing Write/Compress/Isolate well is a leaky bucket. The tasks above are the structural reinforcements.

## Characterization tests before refactor

From Michael Feathers' *Working Effectively with Legacy Code*. Before refactoring code that lacks tests, write tests that capture its current observable behavior — even if that behavior is ugly. Then refactor under the green light of those tests. The refactor is judged not by whether the new code is "better" but by whether the characterization tests still pass.

Applied in this pack at:

- SP-002 (pk-focus extraction): before changing the keyword-extraction logic, capture its current input → output behavior across a representative set of prompts.
- SP-007 (trace capture): before adding the trace files, characterize what the existing self-learning architecture *thinks* it produces, then verify whether that's what's actually written.
- SP-019 (LibrarianEvent persistence): the librarian's current event flow must be characterized before being made persistent, or you'll persist wrong-state events.
- BDD-005 (testid drift detection): before declaring a testid usage patterns "the rule," capture which testids exist today and what step definitions reference them. The rule emerges from the characterization, not the other way around.

## Invariants vs regression-guards lifecycle separation

An **invariant** is a property that must always hold across all code in the project. Examples: "no `any` types," "no `ts-ignore` without a comment," "all PII fields go through redaction." Invariants are reviewed quarterly and rarely change.

A **regression-guard** is a test for a specific bug that occurred. Example: "after fix #4731, the date picker no longer renders past-dated valuations as future." Regression-guards accumulate and rarely retire.

Mixing these in the same review channel is the bug. If a CLAUDE.md file contains both "no `any` types" and "after fix #4731, ensure validations include the YYYY-MM-DD parser before the MM/DD/YYYY parser," the second rule expands without bound and the first rule gets buried.

This pack's mapping:

- The skill-pack `CLAUDE.md` should hold invariants only (SP-001 unification work touches this).
- A separate `BUG_FIX_LEDGER.md` (proposed in XC-001) holds the chronological regression-guards.
- Quarterly review: anything in BUG_FIX_LEDGER.md that has triggered ≥3 regression preventions becomes a candidate for promotion to invariant status. The promotion is a deliberate, reviewed event, not automatic.

## Broad-change threshold

Not every code change should trigger the full test/video pipeline. Most don't need to. The threshold for "this is a *broad* change" is met when *any* of the following holds:

- More than three files modified.
- A change crosses an entity boundary (e.g. modifies how `Acquisition` and `Buyer` relate).
- A document-generation command is added/modified/removed.
- A shared component's API or rendering contract changes.
- Server-state fetching or refetching logic changes (e.g. `useQuery` keys, SSE invalidation maps).

Broad changes require:

- Full BDD video re-recording for impacted scenarios (BDD-010 selectivity makes this affordable).
- Updated user-story or feature-file documentation (BDD-013 contract).
- Cedar review of any policy implications (SP-011 PEP).

Narrow changes (single file, no contract change, no shared component) skip the broad-change tail and just need typecheck + lint + targeted unit/component tests.

## Subagents over monolithic for cross-cutting work

When a task crosses multiple concerns (e.g. "implement BDD-008: extract code graph") that touch the Rust workspace, the Node-side ts-morph tooling, the Surreal schema, and CI integration, attempting it in one Claude Code session will exhaust context before completion. The pattern is to hand off to specialized subagents:

- `rust-codegraph` agent owns the Rust crate and Surreal interaction.
- A `tooling` subagent owns the Node-side ts-morph extractor.
- A `ci-engineer` subagent owns wiring it into the CI pipeline.

Each subagent reads only its slice of the task doc and emits a single artifact. The parent session integrates.

This pack reflects that pattern in the `agent_role` field on every task. See `00-meta/parallel-agent-routing.md` for the matrix.

## Hooks over CLAUDE.md for deterministic enforcement

CLAUDE.md is *probabilistic compliance* — the model reads it and is more likely to follow the rules, but follow-through is not guaranteed. For rules that *must not* be violated, encode them as hooks (PreToolUse, PostToolUse, SubagentStop) that execute deterministic checks and block the operation on failure.

Concrete examples in this pack:

- SP-011 (Cedar gate at PostToolUse for SKILL.md): the gate is a hook, not a CLAUDE.md rule, because we cannot rely on the model to refuse a `Edit` to a SKILL.md file when the user (or a subagent) has asked for it.
- SP-013 (sycophancy correction in SubagentStop reflector): the correction *runs*, deterministically, before the reflector's output is accepted. CLAUDE.md saying "be honest" is not enough.
- BDD-006 (immutable-tests rule): the rule is in CLAUDE.md, but it's also enforced at PreToolUse on `Edit`/`Write`/`MultiEdit` targeting `tests/steps/*.steps.ts`, which would be the natural extension if behaviour drift is observed.

## The "category error" pattern

When a user request is internally inconsistent or rests on a faulty assumption, the right response is to surface the inconsistency, propose what should be built instead, and let them push back. The wrong response is to build the literal ask.

In this session, "auto-update tests when code changes" was the canonical category error. If the agent that writes the production code also writes the test for that code in the same operation, the test stops being a regression check. The reframing into BDD-005/006/007 produces what was actually wanted (faster feedback loops, less friction with code-gen agents) without breaking the regression-detection property.

Other tasks where this pattern applies:

- BDD-013: bidirectional user-story ↔ feature ↔ docs sync was originally requested as bidirectional. The doc explains why bidirectional is unstable and recommends one direction with the others as generated outputs.
- SP-008: per-project KB scoping was originally framed as a configuration option. The doc reframes it as a confidentiality requirement (because the global KB risks cross-project data leakage).

## Sycophancy as structural failure

Sycophancy is not just "the agent agrees too much." It includes:

- **S-01 unconditional acceptance** — agreeing without reasoning.
- **S-02 hedge softening** — adding "but maybe" caveats that lower the load on the requester.
- **S-03 completion without trade-offs** — declaring something done with no risks surfaced.
- **S-04 false certainty** — offering confident answers in domains the agent doesn't actually know.
- **S-05 user-drift compliance** — letting the user's framing rewrite the agent's prior position without acknowledgment.
- **S-06 effort minimization** — proposing the smallest possible work item to avoid pushback.
- **S-07 expertise theatre** — invoking expertise the agent doesn't have.
- **S-08 closure pressure** — declaring resolution prematurely to end the interaction.

This pack treats sycophancy as a *structural* failure mode, not a tone problem. SP-013 is the highest-leverage fix because it makes the correction operational at the architecture layer (the critic agent in the Reflect phase), not tonal.

The required `Trade-offs and risks` section in every task doc is also a sycophancy-elimination measure: a doc with empty trade-offs is, by structural definition, S-03.

## References

- Karpathy, A. (2024). *Software 3.0: context engineering*. Twitter thread series.
- Feathers, M. (2004). *Working Effectively with Legacy Code*. Prentice Hall.
- LangChain documentation on Write/Select/Compress/Isolate.
- The `sycophancy-correction` skill at `prometheus-skill-pack/skills/sycophancy-correction/`.
- Cedar policy language documentation (AWS).
- Cucumber.js and Playwright reference manuals.
- Karpathy guidelines as adapted in `ssr-frontend/CLAUDE.md` and replicated to other project CLAUDE.md files.
