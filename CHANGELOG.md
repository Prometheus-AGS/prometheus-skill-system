# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.7.0] - 2026-08-03

### Added

- Signed, exact-replay Sovereign Sync push resources with durable receipts,
  resumable events, shared REST/MCP execution, and Rust-generated OpenAPI 3.1.
- Private Unix-socket transport, durable P2P identity, secret pairing tickets,
  endpoint/signing-key enrollment, replay defense, and explicit token-authenticated
  loopback TCP.
- Signed folded-state checkpoints, frontier caches, immutable hash-linked journal
  archive segments, and rollback metadata.
- Ed25519-signed plugin generations, a separate trust store, one canonical skill
  index across host/agent/mobile surfaces, and 14 signed target receipts.
- Git-object protected-test certification with SSH-signed change approvals and
  cumulative local adversarial-review receipts.
- Deterministic `docs:sync`, generated route/schema/CLI/capability references,
  five architecture decisions, and a documentation-only main-branch sync bot.

### Changed

- Learner storage is Loro-only. Immutable uniquely keyed evidence is folded
  deterministically into mastery and conservative FSRS state after local writes
  and remote imports; legacy snapshots are preserved during migration.
- KBD pause is explicitly advisory. Journal transactions and causal-frontier
  validation remain the write-concurrency boundary; agent Bash and Python are
  unrestricted.
- Installation is strict by default, with explicit `--skills-only` and
  non-certifying `--best-effort` modes plus post-install artifact verification.
- Release validation is local-only. Hosted automation is limited to deterministic
  managed-document synchronization and GitHub Pages packaging/deployment.

### Removed

- The obsolete KBD voter/quorum facade, mutation-policing `PreToolUse` hooks,
  hosted test workflows, and stale Automerge/operator-ID documentation.

## [1.6.2] - 2026-08-03

### Fixed

- Generate distinct Claude Code and Codex hook manifests from one declarative contract, with every command pinned to a content-addressed runtime bundle.
- Bootstrap missing bundles only from the native plugin payload after hash verification; hook execution no longer follows mutable `stable` or `current` projections.
- Validate the source manifest, immutable generation, bundle index, fixed runner, and actual Codex cache in `prometheus doctor`.
- Install versioned hook runtime receipts and retain older bundle mappings so already-open sessions remain resolvable during cache turnover.

## [1.4.0] - 2026-06-28

### Added

**Learn Domain — Feynman-Spine Learning & Education Capability (12 skills)**

- `skills/learn/ui-surface` — Cross-harness UI rendering primitive; Tier 0 (text), Tier 1 (AskUserQuestion / file-pair), Tier 2 (surface-bridge MCP App)
- `skills/learn/learn-goal` — Learning desire intake with deep research scoping, honest feasibility gate (GREEN/YELLOW/RED), KB adapter integration (`--kb` flag)
- `skills/learn/learn-survey` — Diagnostic placement; 11 items (5 conceptual, 3 procedural, 3 misconception probes); sets recursion floor; seeds learner model
- `skills/learn/learn-plan` — Concept dependency DAG in surreal-memory; topological sort; time-budgeted curriculum with `--replan` support
- `skills/learn/feynman-loop` — Core PMPO learning loop (Spec=concept+depth, Plan=explanation structure, Execute=plain-language+analogies+skeptic, Reflect=grade+gaps); vertical recursion with floor guard (max depth 3); horizontal escalation (novice→peer→skeptic)
- `skills/learn/learn-grade` — External, source-grounded, sycophancy-corrected grader (S-02 pattern); pass = score ≥ 0.7 AND misconceptions_absent
- `skills/learn/learn-retain` — FSRS-6 spaced retrieval; reads due queue; four-tier rating mapping (Easy/Good/Hard/Again by score)
- `skills/learn/learn-practice` — Derivation/implementation/transfer modes; mastery-gated access; interleaved schedule
- `skills/learn/learn-certify` — OB 3.0 / W3C VC JSON-LD credential; integrity guardrail (Δmastery > 0.4 → integrityNote); self-issued via did-plc
- `skills/learn/learn-kb` — KB registry management; four adapter types (dify:, palace:, local:, web:); privacy guarantee — never forwards KB content to external APIs
- `skills/learn/learn-about-system` — Zero-friction adoption entry for the Prometheus stack; uses meta-corpus files for KBD and skill-pack self-teaching
- `skills/learn/learn-harness` — Harness auto-detection; 13-row capability map table (5 harnesses); per-harness orientation; `--map-only` flag

**Substrate Crates (Layer A)**

- `substrate/storage-provider` — `StorageProvider` + `CrdtEngine` async traits; `LocalDirAdapter` (default); `AutomergeEngine` (automerge 0.5); `IrohDocsAdapter` stub
- `substrate/learner-model` — automerge-backed CRDT learner model; simplified FSRS-6 scheduler with Rating enum; JSON-RPC stdin/stdout shell interface; PFA mastery update at ≥5 observations; binary + lib crate
- `substrate/surface-bridge` — Axum 0.7 MCP App server on `127.0.0.1:7890`; `/health`, `/mcp/detect-surface-tier`, `/mcp/render-ui-intent`, `/mcp/collect-response`; macOS launchd plist included

**Schemas & Corpora**

- `docs/learn/schemas/learner-model.schema.json` — JSON Schema Draft-07 for LearnerModel, ConceptState, FSRSCard, GapRecord, LearnerModelSeed
- `docs/learn/schemas/grounding-corpus.schema.json` — JSON Schema for content-grounding output; six source_type values
- `docs/learn/schemas/kb-corpus.schema.json` — Extends grounding-corpus; adds `kb_source` and `privacy_mode` required fields
- `docs/learn/crdt-conflict-semantics.md` — Field-level CRDT merge rules; mastery=LWW+vc, observations=union-append, fsrs_card.due=min, fsrs_card.stability=max
- `docs/learn/surface-tier-detection.md` — Tier 0/1/2 detection signals per harness
- `docs/learn/kb-adapter-guide.md` — Privacy-safe KB adapter usage reference (195 lines)
- `docs/learn/meta-corpus/kbd-lifecycle-corpus.json` — 18 source entries + 8 misconceptions for KBD self-teaching
- `docs/learn/meta-corpus/skill-pack-corpus.json` — 15 source entries + 9 misconceptions for skill-pack self-teaching

**Shared Scripts**

- `shared/scripts/content-grounding.sh` — 4-tier source chain (Dify KB → palace RAG → MCP filesystem → Firecrawl web); `--include-misconceptions` flag
- `shared/scripts/content-grounding-kb.sh` — Privacy-safe KB adapter; NEVER forwards KB content to external APIs; warns on external API env vars
- `shared/scripts/detect-surface-tier.sh` — `default`/`--print`/`--json` modes; reads env vars + surface-bridge.pid check

**Integration Tests**

- `tests/learn/integration-basic-flow.sh` — write-goal → write-survey → write-artifact → write-grade pipeline
- `tests/learn/integration-full-loop.sh` — FSRS card mutation, practice-result fields, VC JSON-LD structure, integrity guardrail
- `tests/learn/integration-kb.sh` — local adapter, privacy guardrail, corpus schema validation, KB registry
- `tests/learn/integration-meta.sh` — KBD corpus, skill-pack corpus, detect-surface-tier, learn-about-system, learn-harness, all 12 skills validate

**Documentation**

- `docs/guide/10-learn-skills.md` — Operator guide chapter for the learn domain (274 lines)
- `CLAUDE.md` — New `## Learn Domain` section: four-layer architecture, substrate crates, surface tier degradation contract, KB adapter pattern, mastery criterion, anti-sycophancy mandate

**Infrastructure**

- `scripts/install-skills-flat.sh` — `install_learn_substrate` function: builds storage-provider, learner-model, surface-bridge Rust crates; installs learner-model binary to `~/.local/bin`; installs surface-bridge as macOS launchd service
- `shared/scripts/detect-toolchain.sh` — Added `learner-model` binary check and `surface-bridge` HTTP reachability check
- `.claude-plugin/plugin.json` — Added all 12 `./skills/learn/<skill>` paths
- `marketplace/marketplace.json` — Added `learn` domain entry with 12 skills

### Changed

- `skills/process/kbd-process-orchestrator/prompts/reflect.md` — inlined sycophancy-correction invocation contract and scoring thresholds directly into the "Sycophancy Self-Check (MANDATORY)" section.
- `skills/process/kbd-process-orchestrator/prompts/assess.md` — added invocation subsection mirroring reflect.md with Assess-appropriate thresholds.
- `skills/process/kbd-process-orchestrator/prompts/plan.md` — added Sycophancy Self-Check section (previously missing).
- `skills/process/kbd-process-orchestrator/references/integrations/sycophancy-correction.md` — Plan phase promoted from exclusion to first-class checked phase.
- `scripts/check-prerequisites.sh` — invokes smoke-test.sh after sycophancy-correction binary install to verify functionality.
- `scripts/validate-skills.js` — hardened against symlink loops and recursive structures (lstat + isSymbolicLink skip, realpath-based visited-set, expanded skip list).
- `CLAUDE.md` — directory structure diagram updated with `skills/learn/` subtree and `substrate/` crates; `## Learn Domain` section added
- `README.md` — learn domain added to directory structure and skills section

### Submodule bumps
- `skills/imported/sycophancy-correction` → d973ef370fe238ceeed72c1b462e85ee83144734

[1.4.0]: https://github.com/Prometheus-AGS/prometheus-skill-system/releases/tag/v1.4.0

## [1.1.0] - 2026-04-15

### Added

**Skills (61 total, 0 errors, 0 warnings)**

- React entity management suite (27 skills): setup, CRUD, GraphQL, Prisma, realtime, optimization
- Process orchestration (20 skills): KBD orchestrator, iterative evolver, PMPO skill creator
- GitOps CI/CD (4 skills): bootstrap, transform, ArgoCD multi-cloud, Kustomize overlays (TJ-CICD-001)
- BDD testing (1 skill): Cucumber.js + Playwright with video recording
- Imported: artifact-refiner (9 skills) via git submodule

**Nested PMPO Pipeline**

- Iterative-evolver outer loop delegates to KBD inner loop for software domain execution
- KBD auto-detects OpenSpec for structured change management
- Artifact-refiner QA gate per completed change
- Evolver bridge file (`evolver-bridge.json`) maps plan items to KBD changes
- KBD reflect reports back to evolver with artifact quality metrics

**Rust CLI (tools/prometheus-cli/)**

- 4-crate workspace: prometheus-cli, prometheus-agents, prometheus-learn, prometheus-cedar
- 15 subcommands: install, uninstall, list, search, audit, verify, doctor, status, generate, validate, build, memory, evolve, learn, optimize
- 10-platform adapter library (Claude Code, OpenCode, Cursor, Codex, Gemini CLI, Roo Code, Windsurf, Amp, Cline, Kilo Code)
- Cross-platform `TraceCapture` protocol for self-learning pipeline
- Cedar Skill Mutation PEP: gates skill.mutate/generate/promote/trace.capture
- Self-learning pipeline: trace capture, evaluation, knowledge compilation scaffolding, dspy-rs optimization scaffolding

**Surreal-Memory Integration**

- Root `.mcp.json` with surreal-memory, tavily, sequential-thinking servers
- Entity mapping patterns for all skill domains (evolver, KBD, GitOps, artifact-refiner)
- Comprehensive integration reference (`shared/references/surreal-memory-integration.md`)
- Graceful degradation when surreal-memory unavailable

**OpenCode Support**

- 3 TypeScript tool definitions (`.opencode/tools/`): evolve, kbd, gitops
- `.opencode/package.json` for auto-dependency installation
- Compatibility declared for 8 platforms in plugin.json

**Distribution**

- Marketplace with 5 granular plugin entries (full pack + domain-specific)
- TypeScript multi-platform installer (`scripts/install-platforms.ts`)
- GitHub Actions CI: validate skills, check formatting, cargo check + clippy
- Skills.toml/Skills.lock format (cowork-compatible)

**Governance & Architecture**

- Cedar default policies: development (permit all), staging (require validation), production (deny mutations)
- Self-learning architecture reference (`shared/references/self-learning-architecture.md`)
- UAR-embeddable library design (prometheus-learn as library, CLI as thin wrapper)
- Unified hooks.json with 5 events: SessionStart, PreToolUse, PostToolUse, SubagentStop, Stop

### Changed

- Recursive skill validator now scans sub-skills at any nesting depth
- Validator excludes backslashes inside code blocks from path separator warnings
- Validator adds line count checks (warning at 500, error at 800)

### Fixed

- Missing SKILL.md in prometheus-entity-skills container directory
- Duplicate pmpo-skill-creator in skills/creation/ (removed, kept skills/process/)
- Backslash path separators in bdd-testing, entity-crud-table, entity-realtime-channel
- pmpo-skill-creator sub-skill directory names mismatched frontmatter (clone→clone-skill, etc.)
- Empty skill categories (rust, ui-ux, devops, documentation) removed from manifests
- Plugin.json upgraded to full schema (author object, mcpServers, hooks path)

[1.1.0]: https://github.com/Prometheus-AGS/prometheus-skill-system/releases/tag/v1.1.0
