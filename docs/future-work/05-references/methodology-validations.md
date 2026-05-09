# Methodology Validations

External references that informed the analysis. Where the conversation made a claim with structural weight, this is where to look for the supporting literature.

## Karpathy context engineering taxonomy

The Write/Select/Compress/Isolate framing is widely attributed to Andrej Karpathy's 2024 commentary on agent context management. The taxonomy as used in this pack is the formalization adopted by LangChain/LangGraph documentation in early 2025.

The framing's value is descriptive, not prescriptive: it gives names to operations engineers were already doing ad hoc, which makes it possible to reason about which operations are missing or weak in a given system.

Applied to the skill-pack and prometheus-knowledge:

- **Write is partial.** Conversations are written into surreal-memory and the librarian wiki, but `LibrarianEvent` records are in-memory only (SP-019).
- **Select is naive.** `pk-focus-on-prompt.sh` is a single-pass keyword extraction that does not gate on relevance, leading to LLM calls on every prompt regardless of whether the prompt benefits from KB context (SP-002, SP-004).
- **Compress is reactive.** `mem0 compress_memories` exists but is not scheduled (SP-021).
- **Isolate is broken.** The Karpathy KB at `~/.prometheus/knowledge/` is global across projects (SP-008).

## Feathers, *Working Effectively with Legacy Code* (2004)

Specifically the chapter "I Need to Make a Change. What Methods Should I Test?" and the "characterization tests" pattern. The relevant adaptation: when refactoring code without tests, write tests against current observable behavior first, refactor under their green light, then update tests as part of a separate, review-gated step.

Applied in this pack to:

- SP-002, SP-007, SP-019 (refactoring or extending existing systems where current behavior must be characterized first).
- BDD-008 (extracting a code graph requires a baseline characterization of which symbols exist before the extractor runs, so the extractor's output can be validated).

## Probabilistic vs deterministic enforcement

The framing that CLAUDE.md is "probabilistic compliance" while hooks are "deterministic" reflects empirical observation of LLM-driven workflows: a model reading a CLAUDE.md rule is more likely to follow it, but follow-through is not 100%. For invariants that must not be violated, the rule must be enforceable at the tool layer, not the prompt layer.

This is a structural argument, not a tonal one. SP-011 and SP-013 are the canonical applications.

## The Pareto distribution of broad-change cost

Not every change requires the full test pipeline. The pareto observation (well-trodden in CI/CD literature) is that ~80% of changes are narrow (single file, no contract change), and the cost of running the full pipeline on every change is roughly 5–10× the value of the marginal regression caught. The broad-change threshold encoded in this pack (>3 files, entity boundary, doc-gen command, shared component, state/refetch) operationalizes that pareto.

References: martinfowler.com/articles/continuousIntegration.html (general CI principles); Nx and Turborepo documentation on "affected" computations (modern incarnations of the same idea).

## Cedar policy language

The Cedar policy language (open-sourced by AWS in 2023) is used in `prometheus-skill-pack/policies/`. The policy at `policies/skill-mutation.cedar` gates programmatic mutations of skill state through a PEP (Policy Enforcement Point).

Relevant for SP-011: Cedar PEPs sit at the API or RPC layer. They do not automatically intercept filesystem operations performed via the Edit/Write/MultiEdit tools. Wiring Cedar to the PostToolUse hook on those tools is the natural extension and what SP-011 implements.

Reference: docs.cedarpolicy.com.

## TypeScript Compiler API and ts-morph

The `pk-codegraph` extraction (BDD-008) is intended to use ts-morph (a higher-level wrapper around the TypeScript Compiler API). ts-morph provides:

- AST traversal with type information.
- Symbol resolution across imports.
- Source-position-accurate references for testid extraction.

The choice of ts-morph over raw `typescript` API or tree-sitter is deliberate:

- Raw `typescript`: requires you to manage the program/compiler-host lifecycle yourself; verbose for the use case.
- tree-sitter: language-agnostic but lacks TypeScript's type semantics. Works for syntax-level extraction, fails for "find all components that import this hook."
- ts-morph: TypeScript-specific, exposes type info, manages program lifecycle. Right tool for this job.

Reference: ts-morph.com.

## Playwright traces

BDD-009 ingests Playwright traces to map test runs to source files. The trace format is a `.zip` containing JSON event streams. Each trace captures network requests, DOM events, console messages, and (when configured) JavaScript coverage data.

Coverage data is what enables file-level mapping. The relevant Playwright API is `page.coverage.startJSCoverage()` and `page.coverage.startCSSCoverage()`. Traces with coverage produce a mapping from each scenario to a list of "files that executed JS during this scenario" with byte-level granularity.

Reference: playwright.dev/docs/trace-viewer-intro and playwright.dev/docs/api/class-coverage.

## SurrealDB 3.x notes

The session referenced specific SurrealDB 3.x compatibility notes:

- `type::thing()` was removed; use `type::record($table, $key)`.
- Content objects must not contain an `id` field at insert time.
- UPDATE operations require the two-arg form with separate bindings.

These notes informed the schema in `00-meta/memory-schema.surql` and any SurrealQL elsewhere in the pack.

Reference: surrealdb.com/docs/surrealdb/3.x/migration.

## IPFS pinning and dedup

The video pipeline uploads `.webm` files to IPFS via `scripts/upload-videos-to-ipfs.ts` and stores CIDs in `docs/videos-manifest.json`. The "unchanged" status returned by the upload service is the dedup signal — when content matches an existing CID, the upload is a no-op and the existing CID is returned.

The pin sweep (BDD-003) deals with the long tail: CIDs that were pinned in earlier runs but whose content is no longer referenced by the latest manifest. Pin tracking is the IPFS gateway's responsibility; the sweep query is "for every CID we ever uploaded, is it referenced in the current `videos-manifest.json` for any scenario?" CIDs answering "no" can be unpinned.

Reference: docs.ipfs.tech for pinning concepts; the gateway-specific docs for the unpin RPC.

## Sycophancy correction pattern catalog

The 8 patterns enumerated by the `sycophancy-correction` skill (S-01 through S-08) come from Anthropic's research on sycophantic failure modes in LLM outputs (publicly cited in Anthropic constitution discussions and in the safety literature).

This pack treats them not as tone defects to filter out post hoc, but as structural failure modes to engineer around. SP-013 is the architectural intervention; the required `Trade-offs and risks` section in every task doc is the structural reinforcement at the documentation level.

## What to consult these references for

If, in implementing a task in this pack, you discover the underlying assumption is wrong, do not silently work around it. Instead:

1. Note the discrepancy in the task's STATUS.md `notes:` field.
2. If the discrepancy invalidates the task entirely, set `status: blocked` and reference the citation that contradicted the assumption.
3. If the citation is more recent than the original analysis (most of the references above are stable, but Karpathy commentary, Anthropic constitution updates, and SurrealDB version notes evolve), prefer the more recent citation.

The point of having these references is so that disagreements about *what* to build can be resolved by appealing to *why*, with sources.
