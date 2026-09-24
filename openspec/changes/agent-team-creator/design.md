## Context
The assessment and analysis identify existing KBD, platform, model and distribution foundations and eight distinct native contracts. The consistency prerequisite must complete before team implementation.

## Goals / Non-Goals
Provide a shared guided/expert team skill family with inspectable artifacts and handoffs. Do not replace native agent loops, modify UAR/BossFang/The Boss, install globally, invent unsupported APIs or claim skill-level Cedar enforcement.

## Decisions
Use a versioned portable manifest for task intent, role/skill/model policy and explicit ownership. Keep harness-native options and arbitrary native files as preserved staging inputs with collision diagnostics and source/version metadata. Generated config is a proposal; validation and native support are separately reported. Ship four sibling skills: agent-team-creator, agent-team-manage, agent-team-models, agent-team-handoff; one self-contained runtime inside creator, identical in both packages.

Use Node >=22, TypeScript 7.0.2, .mts→.mjs NodeNext, no runtime dependencies. Adapt mini platform/receipt patterns; no repository-root imports in shipped code. A guided flow explains roles/skills/models/harnesses and asks about outcomes, budget and review, defaulting to minimal teams. State mutation is lock-protected and revision-checked; cancellation, reassignment and handoff acceptance remain inspectable. KBD-linked tasks carry canonical identity and route KBD completion through its real CLI rather than projecting new authority.

Reuse liter-llm REST availability and catalog schema version 1, retaining provenance, per-token units and unknown metadata. Resolve policy defaults then role, skill and task overrides. Strength tier is declared policy, not inferred from model names. Memory is optional via real contracts/discovery and explicit mappings; local receipts/outbox support service outages. Karpathy canonical boundaries are distinct from arbitrary team events, and pk owns knowledge bundles.

Adapters respect native differences: Codex TOML, Claude/Copilot/OpenCode/Kimi Markdown with native fields, MiniMax user-data agent artifacts and limited CLI selection, DeepSeek Cordis presets/experimental teams. UAR and BossFang use separately verified registration formats, not one invented universal API. No per-harness command runs merely because an export is generated.

## Dependencies
kbd-consistency-before-agent-teams. Analyze candidates cand-002 through cand-015; cand-006 liter-llm is reused, cand-007 existing Docusaurus sites reused. UAR/BossFang exact structs/routes must be recorded before adapter implementation and certification.

## Risks
Version drift requires native escape hatches and honest validation status. Atomic local task ownership is not distributed lease enforcement. Native permissions remain the enforcing boundary; handoff content and memory are not authorization. Local macOS evidence cannot certify Windows. Unavailable native CLIs must be reported as unverified, not passed.

## Verification
After coherent production: compiled CLI real filesystem/process integration covers guidance, validation, all eight exports, opaque config preservation/collision failure, task conflicts, cancellation/reassignment, KBD identity, model constraints, ownership transfer and offline memory retry. Verify the packaged entry point, source/artifact parity, strict skill metadata and native schemas where installed. Regenerate distributions twice, validate drift, update root docs and both Docusaurus sites and run local builds. Deterministic QA and independent cumulative review precede commit/push and reflection.

## Resolved UAR/BossFang contracts

Read-only source inspection 2026-09-24. UAR commit ba12845138104d3c8c3b8bca8bc7c5be24004e91 (1.0.0); BossFang c719a4d683e4d3fb42e436f812e0193f865c9d2c (2026.7.11). No service registrations or live acceptance tests performed.

UAR POST /api/agents consumes src/uar/domain/artifact.rs AgentArtifact, not librefang-uar-spec AgentArtifact. Required objects: version,kind,id,metadata,runtime,policy,schemas,prompt,memory,tools,ui. runtime.entry=default; policy.provider.default={provider,model}; tools.allow and skills.prefer are nested policy fields. Empty provider/model inherit defaults. GET /api/agents/{id} returns persisted artifact; PUT upserts. POST /api/uar/runs consumes {artifact:<full artifact>,input:<task>}, not agent_id. Compiler compile-and-register accepts {content:<UAR-AGENT-MD>}; its converter is not lossless for v2 fields. Actor endpoints are execution primitives with owner authentication, not a persistent team registration API.

BossFang POST /api/agents consumes {manifest_toml:<native TOML>} and returns agent_id. PATCH /api/agents/{id} replaces the manifest. Manifest model settings live under [model], including system_prompt. Skills [] means all; skills_disabled=true means none. MCP [] means none and ["*"] means all. Standard GET is a projection, so retain the original manifest locally. POST /api/hands/install accepts {toml_content:<HAND.toml>,skill_content:""}; multi-agent [agents.<role>] supports coordinator and invoke_hint. Activation is separate POST /api/hands/{id}/activate with {config:{}} and may start autonomous schedules. POST /api/workflows accepts name,description,steps with agent_name or agent_id; run separately at /api/workflows/{id}/run. No verified HTTP UAR Markdown import route.

Require operator base URL and credential reference. UAR defaults JWT-required; API-key-only cannot be promised. BossFang supports Bearer or X-API-Key. Never infer mutation authorization from health/discovery success. Model discovery: UAR /api/uar/providers/{id}/models; BossFang /api/models. Native registration is not atomic across agents: receipts must preserve each returned native ID and outcome; no automatic activation or execution on export/install.

Primary local sources: UAR src/uar/domain/artifact.rs, api/discovery.rs, api/routes.rs, api/compiler.rs, compiler/to_artifact.rs, api/actors.rs, security/middleware.rs; BossFang crates/librefang-api/src/types.rs, routes/agents/lifecycle.rs, routes/skills/hands.rs, routes/workflows/workflow.rs; crates/librefang-types/src/agent.rs; crates/librefang-hands/src/lib.rs; crates/librefang-uar-spec/src/types.rs.
