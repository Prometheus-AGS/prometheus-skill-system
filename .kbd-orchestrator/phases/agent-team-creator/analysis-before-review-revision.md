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
