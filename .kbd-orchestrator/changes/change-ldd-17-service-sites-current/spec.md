# change-ldd-17-service-sites-current

Title: Refresh service architecture operations and all Docusaurus content
Phase: phase-learning-deploy-and-debt
Backend: native-kbd
Repository: full+mini+companion
Gaps: G19
scope:
  - full:scripts/generate-service-manifest.mjs
  - full:skills/testing/bdd-lifecycle-loop/SKILL.md
  - full:skills/testing/bdd-lifecycle-loop/references/visual-baseline-refresh.md
  - full:skills/process/spec-gate/references/dimensions/who.md
  - full:CONTRIBUTING.md
  - full:SKILLS.md authored portions
  - full:docs/** owned current content
  - full:site/docs/**
  - full:site/src/pages/**
  - full:site/sidebars*.js
  - full:site/docusaurus.config.js
  - mini:docs/** owned current content
  - mini:site/docs/**
  - mini:site/src/pages/**
  - mini:site/sidebars.js
  - mini:site/docusaurus.config.js
  - companion:README.md and docs cross-links

## Requirements

1. Document every actually shipped service and its repository, purpose, binary/package, owner, transport, endpoint discovery, data directory, credentials boundary, required/optional status, install/start/stop/upgrade/recovery and platform support. Include surreal-memory, knowledge/pk, liter-llm, openai-proxy, research/exec components and Companion-owned sync/control only where applicable.

2. Explain why the Companion repository exists, what moved out of the pack, and how connected service/skill packages register and coordinate. Preserve the one-way dependency: Companion consumes pack contracts; full and mini remain usable without Companion. Replace stale pack-owned sovereign-sync instructions; distinguish supported current service paths from draft desktop/mobile vision.

3. Apply the complete content disposition from 15 across guides, API examples, command references, installation/upgrades, memory, teams, platform differences, generated catalog, troubleshooting, release notes and landing pages. Remove obsolete current material and repair cross-links. Preserve historical evidence explicitly as historical.

4. Update both Docusaurus sites, canonical mounted docs/guide and sidebars/navigation/search together. Build from final pinned source; do not hand-edit generated site HTML or copied distributions. Handle removed page routes deliberately. Current Docusaurus pins are 3.10.2; do not turn content closure into an unrelated framework upgrade.

5. Publish documentation from merged, locally certified source using the permitted Pages packaging/deployment route. No hosted tests or validation; publication receipts must identify the same certified content/version. User merges remain required.

## Binding constraints

Source work uses isolated worktrees under ../worktrees with declared base/provenance; protect deploy-main/deploy/main and other agents' changes. All tests, builds and reviewers wait for completed production and run only locally under change 12. Use scratch HOME/CODEX_HOME/CORTEX_DATA_DIR/plugin/learning roots; never test against live :23001. One Cargo/rustc process machine-wide; pgrep first. User merges every PR and explicitly approves versions.toml edits, tags and real Codex/Claude changes. No approval is inferred from planning. See the phase plan for ordering, exact task model assignments and final gates.

## Plan assignment and ordering authority

`.kbd-orchestrator/phases/phase-learning-deploy-and-debt/plan.md` is the current ordering and Task model assignments authority. Match the full phase path `phase-learning-deploy-and-debt`, change ID `change-ldd-17-service-sites-current` and backend task ID in tasks.json. Earlier dependency prose is superseded by this Plan revision.

### Front-loaded service-contract documentation subset
Change17/task1 may author full and mini service ownership/operations from shipped source while release03 freeze and Companion14 recovery remain pending. Whole task17/1 cannot complete until Companion documentation cross-links and actual candidate contracts are reconciled. Lead uses sequential inherited-model fallback because all native implementation slots are occupied; actual exposed worker assignment remains gpt-6.1-sol/high for delegated task17 when available. Lead owns new full docs/guide/26-service-operations.md and full site/docs/operations/installation-and-upgrades.md plus mini site/docs/services/docker-services.md and mini docs/service-operations.md; do not overlap16 team pages or15 README/inventory. Source-discovered services.manifest binding claims downstream --pk-mcp-url8942 as Forge listen port despite explicit --port8943. Add full scripts/generate-service-manifest.mjs to17 scope for precise local binding extraction from supported listen/port/socket arguments only; arbitrary URLs are consumers, not listeners. Generated shared/services.manifest.json remains untouched until final12 generation. No gates, probes, external mutations or early review; existing cadence scope17 admission and original firstWorkAt remain unchanged.

Narrow17 authored skill-content extension from15 actual claim inventory: skills/testing/bdd-lifecycle-loop/SKILL.md, skills/testing/bdd-lifecycle-loop/references/visual-baseline-refresh.md, skills/process/spec-gate/references/dimensions/who.md. Replace hosted CI as Prometheus release evidence and unsupported label-enforcement claims with complete-production local integration, explicit human merge and protected-test approval boundaries. These are documentation-only source edits, not protected test/scenario changes or skill behavior/code expansion. Generic third-party CI concepts remain descriptive; no blanket removal or global downstream-policy invention. Lead owns this three-page batch; final12 validates regenerated packages and skill strictness, no intermediate validator.

17 narrow root-content scope extension: full CONTRIBUTING.md and authored portions of full SKILLS.md are linked product entrypoints. Resolve their observed stale clone URL, Node minimum, unit-inclusive normal Cargo gate and hosted-CI claims from current manifests/canonical policy. Generated index/provenance portions remain owned by generators and unchanged until12. Root canonical AGENTS.md/CLAUDE.md operating policy is not edited. Assign this source content to17 doc worker; no test/validator execution.

### Local-only hosting source reconciliation — 2026-10-05
Observed source workflows would violate the binding local-only policy on a future push/owner merge: mini .github/workflows/ci.yml runs hosted tests, coverage, validators and Cargo; memory .github/workflows/image.yml compiles containers on hosted runners; memory .github/workflows/docs.yml typechecks on a hosted runner. Add those exact workflow paths and directly related memory release-method wording to task17 production source scope. Remove forbidden execution, retain permitted Pages packaging/deployment, and preserve old workflow source in Git history. No hosted workflow is started, polled, used as evidence or canceled by this source correction. Container release uses locally built/certified artifacts; do not introduce unrelated release infrastructure. Source Inventory worker owns this narrow correction; all acceptance remains final12.
