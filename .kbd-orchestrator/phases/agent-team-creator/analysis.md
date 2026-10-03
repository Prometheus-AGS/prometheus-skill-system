# Analyze: agent-team-creator
Date: 2026-09-24. Stack specified by operator: Node.js and TypeScript 7 for new team scripts/hooks. Existing Rust KBD projection code remains Rust; correcting that existing producer does not add a Rust dependency to the team skills.

## Authority and review disposition
The operator explicitly authorized modifications and eventual commit/push in both prometheus-skill-pack and prometheus-skills-mini, then directed “Follow the KBD process and do not skip steps” and “clear up all these conflicts and inconsistencies”. See operator-authorization.md, supplied verbatim in the review packet.
Therefore mini's reference-only default is overridden only for this phase, not removed globally. Its historic no-remote/no-commits prose is factually obsolete (both origin URLs and HEADs are recorded in assessment.md). Personal/team knowledge-log publication remains separately prohibited.
The assessment's unresolved QA finding is resolved by source precedence: full current kbd-execute says “File-count and documentation-only skips do not exist”; the current integration contract requires deterministic QA followed by adversarial review before archive. Mini constitution A-9 requires completed behavior before integration testing. Align stale constraints/OpenSpec context with these governing sources under the operator's explicit correction request. This strengthens evidence; it does not invent a passing review or erase the original findings.

## Research procedure and limits
Tier 1: inspected the local source repositories, their existing KBD helpers/runtime, packaging and native-harness research collected earlier in this session. One GitHub repository search for agent skills team coding orchestration returned no results; that narrow search is not proof that alternatives do not exist.
Tier 2: reused the already resolved TypeScript /microsoft/typescript/v7.0.2 identity and fetched NodeNext/.mts compile guidance from its primary sources. Earlier harness research used Context7 plus official source and remains separately attributed; no live harness compatibility certification is claimed.
Tier 3: npm view typescript@7.0.2 returned version 7.0.2, Apache-2.0, Node >=16.20.0. Our runtime baseline remains Node LTS >=22.
Tier 4: not needed for the observed KBD corrections. This analysis makes no ecosystem-completeness claim. Research stops within the 8-query-per-tier/20-minute budget.
Prior research is reference material; new queries used here: tier1 1, tier2 1, tier3 1, tier4 0, plus local source inspection.

## Build versus reuse
- Adapt the existing canonical KBD runtime projection, not a parallel state store or rewritten event history. Phase lacks its own evidence/certification/publication fields. Until a real phase-scoped contract exists, project those dimensions as NOT_TRACKED, and expose the existing run-wide values under an explicitly named runCompletion field with scope metadata. Phase-local implementation counts remain canonical. Correct stage/lifecycle labels and top-level parentPhase derivation in the same projection boundary.
- Adapt existing phase creation helpers so both runtime and legacy paths complete the same metadata and hook postconditions. The current phase was repaired manually; do not create it again or fire phase:before again. Existing hook dispatch and atomic writers are the reusable mechanisms.
- Adapt current source instructions instead of editing generated AGENTS.md or distribution payloads. Reconcile direct-progress edits, bare apply routing, predecessor-gate behavior, tests-first/CI text and publication prose. Regenerate owned surfaces only after source changes.
- Adopt TypeScript 7.0.2 locally for the new shared runtime. Emit .mjs from .mts using NodeNext and explicit import extensions; distributed execution requires Node, not ts-node or a package manager. Reuse/adapt existing atomic-write, exclusive lock, shell-free spawn, receipt and model resolution patterns with provenance; avoid importing repository-root modules from shipped skills.
- Reuse existing inventories and plugin generators. The four sibling skills are auto-discovered; full process-plugin roster is explicit. Mini copying remains constrained by its full-install detection. Do not imply a target declaration installs native teams.

## Team architecture consequences, to specify after process repairs
Separate role responsibility, reusable skill, model and harness execution authority. Guided intake asks about outcome, scope, deliverables, review and cost; small sequential work defaults to one agent, editable before generation.
A portable manifest stores intent, ownership and policy; native role/config artifacts retain harness semantics. Native option passthrough preserves bytes but must be paired with source/version metadata and installed validation. No finite common schema can certify every future native option.
KBD-linked work references canonical phase/change/task identity rather than inventing competing completion. Standalone team tasks need explicit local ownership and handoff acceptance. Handoff packets carry task/evidence/Git identity/memory references, never treat another harness's session ID or permissions as portable authority.
Model discovery must preserve unknown cost/capability as unknown. Explicit policies can bind team/role/skill/task tiers; unsupported native overrides require a visible diagnostic, not ignored fields.
Memory is optional: use current Karpathy canonical receipts only for real KBD boundaries, pk for knowledge bundles, separate team receipts/outbox otherwise. Services unavailable must not prevent local work.

## Native contract evidence carried into Spec
- Codex: standalone .codex/agents TOML; see https://learn.chatgpt.com/docs/agent-configuration/subagents . Plugin skill support does not imply an agents manifest field.
- Claude Code: .claude/agents Markdown, distinct subagent and experimental team semantics; https://code.claude.com/docs/en/sub-agents and https://code.claude.com/docs/en/agent-teams .
- Copilot: .github/agents and Fleet; https://docs.github.com/en/copilot/reference/custom-agents-configuration and https://docs.github.com/en/copilot/concepts/agents/copilot-cli/fleet .
- OpenCode: deployed singular agent/permission/prompt, not development v2 plural keys; https://opencode.ai/docs/agents/ and https://opencode.ai/docs/plugins/ .
- Kimi: current MoonshotAI/kimi-code, not archived kimi-cli; https://github.com/MoonshotAI/kimi-code/blob/main/docs/en/customization/agents.md . Model is not a supported role frontmatter field.
- MiniMax: actual mcode CLI; canonical agents in active user data directory; no custom-agent selection flag in exec; https://github.com/MiniMax-AI/minimax-code/blob/main/packages/tui/src/cli/contract.ts and https://github.com/MiniMax-AI/minimax-code/blob/main/packages/local-runtime-v2/src/service/agent/storage/canonical-agent-config.ts .
- DeepSeek: Cordis presets and experimental flat teams, not invented Markdown profiles; https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/experimental/agent-team/README.md .
- UAR/BossFang: existing discovery/registration is not proof of a portable team API. Resolve versioned local contracts before claiming live registration; no product kernel modifications in this phase.

## Alternatives rejected
Editing progress.json would be overwritten and corrupt authority; resetting run-wide completion would erase real history; skipping reviews because changes are documentation-only contradicts current gates; global host installation would violate the two-repository scope; forcing all harnesses into identical fields would silently drop options; adding a new orchestration daemon would duplicate existing execution ownership and violate mini's service budget.

## Required ordering and remaining uncertainty
First correct KBD source attribution and workflow instructions, then finish the team contract and implementations through reviewed changes. Each correction gets source-level and real integration evidence; no protected tests are modified. The installed prometheus binary may predate the source correction. Validate through the locally built runtime path without replacing the user's global binary; report that deployment boundary honestly.
Windows/Node22 and live harness evidence must come from actual local execution, not mock tests or macOS results relabeled as cross-platform. A narrow discovery search did not identify a reusable end-to-end team creator; that is a bounded conclusion, not a claim of novelty.

## Review-driven additions: liter-llm and both Docusaurus sites
liter-llm is an ADAPT candidate, not a replacement to build. Both repositories pin c5c6caac617eb931cd5009146a70831422ec236c. The isolated full worktree submodule is uninitialized; read-only inspection used the existing canonical checkout at that exact SHA with clean status. Cargo.toml declares 1.18.2/MIT. schemas/catalog.json has $schema_version 1 and provenance (models.dev, fetched 2026-08-25, library_version 1.18.1); provider models carry optional pricing.input_cost_per_token/output_cost_per_token, capabilities, modalities and limits. Adapt REST /v1/models for available aliases plus a versioned catalog input for capability/cost metadata. Do not assume /v1/models alone supplies prices; convert per-token units explicitly and retain catalog provenance/freshness. Model strength tiers remain operator policy, not a property inferable from names. Catalog age and provider pricing drift are explicit risks. No liter-llm source changes or new resident service are needed.

Both Docusaurus sites use 3.10.2. Full site/sidebars-guide.js discovers docs/guide pages; site/scripts/generate-skills-catalog.mjs generates the catalog. Full source docs flow through npm run docs:sync; site prebuild checks public docs/OpenAPI/examples/contracts and generates the catalog before npm --prefix site run build. Mini site/sidebars.js needs an explicit narrative entry and its catalog generator needs category assignments; generate catalog then run the local site build (build:deploy combines them). Update README/usage docs and source guides, regenerate owned catalogs, and run both site builds locally only after implementation. Verify generated outputs twice for idempotence and record pre-existing or environment failures separately; do not hand-edit generated catalog pages or claim a successful site build without running it.

## Unresolved review findings (two-round artifact limit)
Findings remain recorded; subsequent structured evidence corrections are for Spec review, not a retroactive PASS.

### CRITICAL
The native-harness contract candidate collapses seven distinct harnesses into one candidate but only records Claude evidence, leaving most required harness integrations unevaluated in the structured analysis artifact.

Evidence: Goal requires teams across "UAR, Codex, Claude Code, Copilot CLI, Kimi Code, MiniMax CLI, OpenCode and DeepSeek Harness". In library-candidates.json, cand-005 is named "Official harness agent and plugin contracts" with fit_for_gap "native-fidelity", but its only evidence row cites "https://code.claude.com/docs/en/sub-agents" and claims generally that "Native roles and team semantics vary". There are no structured evidence rows or per-harness verdicts for Codex, Copilot, Kimi, MiniMax, OpenCode, DeepSeek, or UAR/BossFang.

Suggested fix: Split cand-005 into per-harness candidates or add evidence/verdict/risks for each required harness, including explicit unresolved status for UAR/BossFang if no versioned contract is available.

### WARNING
The analysis identifies UAR/BossFang as unresolved but does not carry that unresolved contract gap into the structured candidate verdicts or build-required work.

Evidence: analysis.md says: "UAR/BossFang: existing discovery/registration is not proof of a portable team API. Resolve versioned local contracts before claiming live registration; no product kernel modifications in this phase." However library-candidates.json has no candidate or build_required entry for resolving UAR/BossFang contracts; build_required only lists "Guided role discovery and a portable team/handoff contract" and "Native adapters and team model-policy mapping over liter-llm".

Suggested fix: Add a build_required item such as "Resolve UAR/BossFang versioned local team/registration contract" with blocking acceptance criteria, or mark UAR support explicitly out of scope until that research is complete.

### WARNING
The adopted Docusaurus framework candidate lacks a maintenance/license check even though it is marked adopt and is part of the requested documentation-site work.

Evidence: cand-007 has "name": "Existing Docusaurus 3.10.2 sites and catalog generators", "kind": "framework", "registry": "npm", "verdict": "adopt", but unlike cand-002 and cand-006 it records no license, package metadata, maintenance status, or source_url in its evidence row; the only evidence claim is "Both site/package.json files pin 3.10.2".

Suggested fix: Record Docusaurus package/license/maintenance evidence or change the verdict wording to "reuse existing pinned sites" and explicitly state that no new framework adoption is being made.

Disposition: per-harness candidates cand-008 through cand-015 now preserve the cited native evidence. UAR/BossFang has an explicit blocking contract-discovery item; no fake registration will be emitted. The Docusaurus candidate is explicitly reuse of existing pinned sites rather than new framework adoption.
