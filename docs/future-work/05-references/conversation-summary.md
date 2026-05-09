# Conversation Summary

The full architectural review session that produced this pack, distilled. Reading this document is the fastest way to load context if you are picking up cold.

## Session shape

The session ran across roughly seven turns with three logical arcs:

1. **Generic methodology framing** (turns 1–2). Travis pasted a regression-prevention methodology document. Claude validated the framing, mapped it to existing context engineering literature (Karpathy, LangChain Write/Select/Compress/Isolate), and built a generic "kit" of CLAUDE.md/AGENTS.md files, hooks, and subagent definitions. **Claude's first mistake**: building the kit before looking at what already existed in the skill-pack. This was scope creep dressed as helpfulness. The kit was never used; this pack is the corrective.

2. **Honest skill-pack evaluation** (turns 3–4). Claude read `prometheus-skill-pack` and `prometheus-knowledge` end-to-end, including all `shared/scripts/*.sh`, the Rust workspace, the Cedar policies, the change-006 hook plan, and the orchestrator state. Travis invoked `sycophancy-correction` against Claude's resulting summary at adversarial strictness; the tool returned a low score (0.125) with one critical pattern flagged: **S-03 — substantive completion with no trade-offs, risks, or alternatives surfaced.** Claude's evaluation was technically correct but had been delivered as a clean win with no friction. Subsequent turns surfaced trade-offs explicitly. This is the **sycophancy lesson** that informs the entire pack.

3. **SSR BDD/video/feedback analysis** (turns 5–6). Travis presented five high-level asks about the SSR test infrastructure. Claude read the SSR repo: `tests/`, `scripts/run-video-proof.ts`, `scripts/validate-video-coverage.ts`, `scripts/generate-video-run-report.ts`, `scripts/generate-bdd-docs.ts`, the cucumber.js profiles, the docs site, the feedback engine, and the Azure AI Foundry agent prompt. The honest finding: most of what Travis was asking for already existed in some form. Two of the five asks were largely solved and just needed productization. One was a category error. Two were real gaps justifying new work. The recommendations in this pack reflect that decomposition, not the surface ask.

## The 15 weaknesses identified in the existing skill-pack

In severity order:

1. Two `CLAUDE.md` files (one in `prometheus-skill-pack`, one in `prometheus-knowledge`) define overlapping rules with no unification authority. SP-001.
2. `pk-focus-on-prompt.sh` keyword extraction is naive — no stopword list, no relevance gate, no caching. Fires the LLM on every prompt regardless of whether the prompt has knowledge-base-relevant content. SP-002, SP-003, SP-004.
3. The Stop hook chain (`forge-reflect-on-stop.sh`, `pk-lint-cron.sh`, `pk-focus-cleanup.sh`, `mem0-compress-on-stop.sh`) runs four scripts with `|| true` everywhere. Failures are silently swallowed; there is no observability log. SP-006.
4. The self-learning architecture document describes trace capture writing to `.prometheus/traces/`, but no hook actually writes there. The capability is undocumented at the hook layer. SP-007.
5. The Karpathy wiki at `~/.prometheus/knowledge/` is global across projects. A hypothetical Brius healthcare project's session traces would write into the same KB as SSR sessions. **Confidentiality risk.** SP-008.
6. `pk lint` has DUPLICATE detection but `pk-lint-cron.sh` exists unwired — no scheduled job runs it. SP-009.
7. `compile_user_prompt` strips ` ```json ` fences but does not handle preamble or strict-mode validation. Brittle to LLM whim. SP-010.
8. Cedar PEP gates `skill.mutate` programmatically but `Edit`/`Write`/`MultiEdit` to a `SKILL.md` bypasses Cedar entirely. SP-011.
9. The 4-layer pipeline (ZeeSpec → PMPO → OpenSpec → forge-rs) is documented but not enforced — nothing checks that a multi-step task actually descended through the layers. SP-012.
10. **The single highest-leverage fix in the pack:** the `sycophancy-correction` skill is available but not invoked in the PMPO Reflect phase. Wiring it into the SubagentStop(reflector) hook is a few hours of work and structurally eliminates a class of "completion without trade-offs" output. SP-013.
11. The change-006 plan asserts a SubagentStop fallback matcher works without verifying it. SP-014.
12. Two `hooks.json` files exist (in `.claude-plugin/hooks/` and `hooks/`) committed as identical content rather than symlinks. Drift is a foot-gun. SP-015.
13. With 64 skills, no skill-description collision detection exists. Near-miss descriptions are statistically guaranteed at this count. SP-016.
14. Slash commands shipped from `prometheus-skill-pack` and slash commands shipped from `prometheus-knowledge` (`focus`, `ingest`) have no merge strategy when both are installed. SP-017.
15. There is no end-to-end smoke test of Layer 1 → Layer 4 of the pipeline. The pieces exist; the integration is asserted by inspection. SP-018.

## Targeted Karpathy/memory improvements (additive, beyond the 15)

- Promote `LibrarianEvent` to first-class persistence in surreal-memory. Currently events are in-memory in the librarian process; they should be `event` entities with relations to `WikiEntry`. SP-019.
- Separate the knowledge-graph store from the episodic-memory store inside Surreal — different table prefixes, different lifecycles, different access patterns. SP-020.
- Run `mem0 compress_memories` on schedule rather than reactively. SP-021.
- Make `pk focus` context-sensitive (multi-turn keyword extractor, not single-prompt). SP-004.
- Add `pk focus --inject-as system-context` flag so the librarian can inject knowledge as a system message rather than a user-visible response. SP-005.

## The five SSR BDD asks, honestly evaluated

### Ask 1: "Validate complex systems with video evidence at 250+ user-stories scale"

**Status: largely solved already.** SSR has a per-scenario video runner with state file tracking, IPFS upload with stable CIDs, three-phase coverage validation, and a generated docs site with `▶ Watch` pills. The remaining gaps are dual-keyed manifest cleanup (UUIDs and slugs both present), flake quarantine system (currently `failFast` kills throughput), IPFS pin sweep for storage cost, and productizing the pipeline as a reusable skill. BDD-001, BDD-002, BDD-003, BDD-004.

### Ask 2: "Tests automatically updated by code-generation agents without reminding the AI tool"

**Category error.** If an agent edits production code and then edits the tests for that code in the same operation to make them pass, tests stop being a regression check and become a tautology. What is genuinely useful is three different things: (a) selector/locator drift detection so a build fails when a `data-testid` is renamed without the corresponding step update; (b) test-needs-review flagging so that production-code changes mark scenarios as `pending-revalidation` until a human or runner re-records and re-uploads the video; (c) candidate test generation, where the agent drops drafts into `tests/features/drafts/` for human review rather than committing them ready-to-run. The CLAUDE.md rule that locks this down is BDD-006. The detection script is BDD-005. The drafts directory is BDD-007.

### Ask 3: "Code graph in surreal-memory to know which tests need updating when code changes"

**Tractable. The architecture is straightforward; maintenance is the hard part.** Build a new Rust crate `pk-codegraph` (in `prometheus-knowledge` as a sibling crate) with a Node-based ts-morph extractor that emits JSON, and the Rust crate ingests into Surreal with per-commit namespacing. Node types: `File`, `Component`, `Hook`, `Store`, `ApiFn`, `Type`, `TestId`, `FeatureFile`, `Scenario`, `StepDef`. Edge types: `imports`, `calls`, `defines`, `references_testid`, `exercises_scenario`. Static analysis only gets you so far — runtime test-to-code mapping needs Playwright trace ingestion. BDD-008 covers static extraction; BDD-009 adds runtime coverage from traces.

### Ask 4: "Run only the tests that need running between turns; skip ones whose code hasn't changed"

**Tractable, combines with Ask 3, has a hidden correctness trap.** Each scenario gets an `impact_set_hash` covering its transitive code closure. On a new run, recompute, compare, skip if matched. The trap: pure source-code closure is not enough for correctness. `prisma/schema.prisma` changes invalidate every DB-touching scenario. `.env` changes affect runtime behavior. Migrations and `package.json` matter. BDD-011 covers env-hash augmentation. BDD-010 is the runner refactor. BDD-012 separates the per-PR fast gate from the per-release thorough gate (where a full re-record happens at least every N days regardless of impact-set state, to catch environmental drift the hashing missed).

### Ask 5: "Use cases ↔ tests ↔ docs in sync, with users able to view in admin/docs site, validate assumptions, and submit feedback"

**Partially built. The structural issue is bidirectionality.** The docs site at `docs/site/` (mirrored to `public/docs/`) is generated from feature files via `generate-bdd-docs.ts`. The feedback engine collects bugs/UX/feature-requests/questions, triages via Supabase Edge Function and Azure AI Foundry, persists structured records. **What's missing is the loop closure**: feedback records don't automatically become draft scenarios; user stories in `docs/user-stories/` have no enforced relationship to feature files. The recommendation is to pick one direction (tests-tagged-with-OpenSpec-change-IDs, since OpenSpec already exists) and treat user-story documents as a *generated output* of test metadata, not a parallel input. BDD-013 covers the contract; BDD-014 wires feedback aggregation into the docs site; BDD-015 emits draft `.feature` files from triage records with `confidence > 0.7`.

## The build order (recommended priority sequence)

1. BDD-001 + BDD-002 (manifest cleanup + flake quarantine) — half day to one day, immediately reduces noise.
2. BDD-005 + BDD-006 + SP-013 — one day to two days, locks down boundary conditions and addresses the highest-leverage skill-pack fix.
3. BDD-008 (pk-codegraph) — one to two weeks. Foundation that enables BDD-009/010/011/012/013.
4. BDD-009 + BDD-010 — one week, payoff for BDD-008.
5. BDD-013 + BDD-014 + BDD-015 — one to two weeks, closes the use-case ↔ test ↔ feedback loop.
6. SP-019 (LibrarianEvent) + SP-020 (memory dual-store) — one to two weeks, the architectural memory work.

Other tasks (cleanups, observability, scheduled jobs) interleave at any priority slot they fit.

## Themes I'd reach for to explain this pack

- **Karpathy context engineering taxonomy.** The Write/Select/Compress/Isolate framing is what the pk-focus, librarian, and surreal-memory work is operationalizing. Knowing this gives you the vocabulary for why per-project KB scoping matters (Isolate) and why event-driven persistence matters (Write).
- **Characterization tests before refactor.** Several SP tasks rest on this: don't change the script until you have a test that captures its current observable behavior. That way, when the refactor lands, you can prove behaviour preservation.
- **Invariants vs regression-guards lifecycle separation.** Invariants are properties that must always hold (e.g. "no `any` types in TypeScript code"). Regression-guards are tests for specific bugs that have happened. They live in different files and review channels because mixing them buries the truly invariant rules.
- **Broad-change threshold.** Any change touching more than three files, crossing an entity boundary, hitting a doc-gen command, modifying a shared component, or changing state/refetch flow is a *broad change* and triggers full BDD video recording. This is what BDD-001/002/010/012 protect.
- **Sycophancy as a structural failure.** The lesson Travis surfaced — that even technically correct evaluations can be sycophantic if they don't surface trade-offs — is enacted in the pack at SP-013 (wiring the correction skill into Reflect) and reflected in every task doc's required `Trade-offs and risks` section.

## Process learnings worth keeping

- **Read what exists before proposing what to build.** Claude's turn-1 kit was unnecessary because the existing skill-pack covered most of it. The corrective is to inventory first, propose second.
- **Trade-offs are not optional.** A response with no trade-offs surfaced is sycophancy regardless of correctness. Every task doc in this pack includes them.
- **Honest reframings.** When the user's literal ask is unwise (e.g. auto-update tests), the right response is to surface the reasoning, propose what should be built instead, and let them push back. BDD-006 is the canonical example.
- **Productization vs rebuild.** When a system already works, the right next step is often to lift its patterns into a reusable skill, not rebuild it. BDD-004 is this for the video pipeline.
