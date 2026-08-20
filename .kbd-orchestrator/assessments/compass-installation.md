# ASSESSMENT: Compass installation in Prometheus Skill Pack

Project: prometheus-skill-system
Date: 2026-08-09
Codebase baseline: Skill Pack 1.7.0 is installed through a signed immutable
generation across 14 global skill roots; Compass 0.3.6 is installed locally,
while the inspected Compass source and latest local release tag are 0.3.7.
Cross-tool progress: none for this pre-phase decision

## Decision

Do **not** create a new KBD phase and do **not** add Compass to Skill Pack
installation now, either by default or as a managed optional binary.

Compass remains a credible external code-intelligence candidate, but the
current evidence proves capability and an installer-ownership conflict—not an
improvement in AI generation. A pre-phase, non-product pilot must establish that
benefit before the Skill Pack assumes adapter or binary-distribution ownership.
The pilot itself is not sufficient reason to create an implementation phase.

## What this rests on

- Compass 0.3.7 is a 32-crate Rust workspace under `MIT OR Apache-2.0`, with a
  single native `compass` binary. Documentation claims release support across
  macOS, Linux, and Windows, but this assessment did not verify every release
  artifact and architecture.
- The installed macOS ARM64 Compass 0.3.6 binary runs and exposes versioned
  machine capabilities, but is 139 MB. The source checkout is 55 MB (27.5 MiB
  tracked), so bundling source or compiling it during a normal Skill Pack
  installation would be a material cost.
- Compass creates deterministic, bounded structural code graphs and supports
  symbol search, callers/callees, impact, paths, CompassQL, history, semantic
  diffs, exports, MCP, and compact assistant-facing queries without requiring
  credentials, embeddings, a vector database, Python, or runtime parser
  downloads.
- Prometheus already has conceptual/episodic knowledge graphs in
  surreal-memory and task-context enrichment in forge-rs, but no comparable
  persistent source-structure graph, bounded call-path query engine, or
  revision-addressed code graph. The domains are complementary rather than
  duplicates: Compass models source evidence; surreal-memory models learned
  entities and relations.
- The existing immutable installer signs and verifies complete skill payloads
  across 14 roots. A real `compass install --user --all --dry-run --format json`
  fails for the shared Agent Skills, Claude, Kiro, Trae, and Trae CN targets
  because their Skill Pack-managed roots are symlinks. Cursor has no
  user-scoped Compass target. Directly invoking Compass's global assistant
  installer is therefore incompatible with the current ownership model.
- Compass project initialization writes `.compass/` and `compass-out/`, may
  consume substantial CPU, disk, and time, and can expose repository structure
  to any agent allowed to read the outputs. It must never run automatically as
  an installation side effect.

## Assumptions

- No Skill Pack-owned matched evaluation currently demonstrates that Compass
  improves answer quality, completion rate, or total task efficiency compared
  with the normal agentic-search baseline.
- Capability descriptions and external project tests are insufficient evidence
  for adopting a new installation dependency.
- The reproduced symlink-root failure can be avoided by leaving Compass
  external; it does not require Skill Pack-owned binary distribution.
- Existing toolchain-management and installer-pipeline decisions should own any
  future native-tool integration rather than creating a parallel release
  channel.
- Existing Compass users are believed to be able to use project-local Compass
  integration independently of the Skill Pack, but this assessment did not run
  `compass init`; the claim remains unverified and is not load-bearing for the
  no-integration decision.

## Falsifier

Revisit the no-phase decision only when all of the following external evidence
exists:

1. A pre-registered matched pilot names at least three materially different
   repositories and a task corpus containing architecture, impact, call-path,
   bug-localization, and feature-change tasks. It defines the normal
   agentic-search baseline, source/context bytes, elapsed time, and a blinded
   correctness/completeness rubric before execution.
2. The pilot shows a material benefit in at least two repositories: either a
   ten-percentage-point improvement in blinded correctness/completeness without
   increased median task time, or non-inferior correctness with at least 30%
   lower median context bytes and at least 20% lower median elapsed time. These
   are adoption-policy thresholds for accepting recurring distribution cost,
   not claimed external benchmarks.
3. The result holds across tasks both inside and outside Compass's strongest
   graph-query cases: no task category may regress blinded correctness by more
   than five percentage points, and the aggregate thresholds in condition 2
   must be computed across the complete mixed corpus rather than only favorable
   graph-query tasks.

If these conditions are met, a later decision must verify the actual release
artifact matrix and measured resource envelope before it can recommend binary
distribution.

No pilot is authorized, staffed, or scheduled by this decision. Re-evaluation
is triggered only by an explicit operator request or presentation of a
pre-registered pilot result meeting the conditions above; absent either event,
the no-phase decision intentionally remains in force.

An adapter-only prototype is a future engineering acceptance constraint, not a
value falsifier. It must prove detection and use without mutating
generation-owned roots before any managed binary channel is considered.

## Implementation status

- External Compass graph engine: **DONE externally** — released native CLI,
  local-first graph/query functionality, versioned capabilities, and extensive
  CLI tests exist in the Compass repository.
- Skill Pack code-graph capability: **MISSING** — no source-structure graph or
  Compass integration exists in canonical specs or installation docs.
- Default installation suitability: **NOT SUITABLE** — binary size, independent
  release cadence, and project-state creation violate a lightweight default
  skills install.
- Optional binary acquisition: **MISSING** — no pinned release manifest,
  checksum policy, rollback receipt, optional-tool flag, or platform matrix
  exists in Skill Pack.
- Assistant payload ownership: **BLOCKED AS CURRENTLY DOCUMENTED** — Compass's
  global installer refuses generation-owned symlink roots, correctly avoiding
  unsafe traversal. The pack must own any globally distributed adapter.
- Project opt-in and privacy boundary: **MISSING** — no Skill Pack command or
  policy distinguishes CLI presence from explicit per-project graph creation.
- Health and lifecycle verification: **MISSING** — no doctor check covers
  Compass version, capabilities, graph freshness, disk use, or orphaned watch
  processes.
- Value qualification: **MISSING** — Compass documents assistant workflows, but
  this assessment found no Skill Pack-owned matched evaluation proving improved
  generation quality or reduced context cost.

## Spec gap summary

- No OpenSpec contract owns optional native tool installation, capability
  negotiation, Compass graph lifecycle, or privacy/consent behavior.
- The current immutable generation manifest covers skills, not an independently
  versioned 139 MB native binary.
- Existing global installation paths cannot safely delegate to
  `compass install --user`; doing so conflicts with signed root ownership.
- No evidence currently justifies installing Compass for every user or
  initializing every repository.

## Build health

- Skill Pack baseline: **PASS** — clean `main` after merged PR #53; its signed
  generation and 14 receipts were previously verified locally.
- Installed Compass executable: **PASS for inspection** — `compass 0.3.6`, help,
  capability JSON, and installation dry-run execute successfully.
- Compass 0.3.7 source build: **UNKNOWN** — the Compass checkout contains
  unrelated user changes and was not built or modified for this assessment.
- Test coverage: **PARTIAL for this decision** — Compass has broad CLI and domain
  tests, including installation tests; Skill Pack has no cross-product tests or
  usefulness benchmark.

## Constraint check

- Skill Pack AGENTS.md violations: **NONE** — assessment was local-only and did
  not use hosted validation.
- Immutable-generation violation: **PRESENT in the naive design** — direct
  global Compass guidance installation cannot traverse managed symlink roots.
- Compass AGENTS.md violations: **NONE** — its dirty working tree was read only;
  no build, graph, or generated state was created.

## Goal progress for a possible phase

- Prove measurable code-generation/context benefit: **NOT MET**.
- Define an optional, checksum-verified, rollback-safe binary channel:
  **NOT MET**.
- Preserve signed Skill Pack ownership while exposing Compass guidance:
  **NOT MET**, with a reproduced integration blocker.
- Require project consent and local privacy/scope controls: **NOT MET**.
- Add doctors, version negotiation, tests, docs, and uninstall behavior:
  **NOT MET**.

## Alternatives considered

1. **Mandatory default installation** — rejected because of binary size,
   platform/release coupling, and automatic cost imposed on users who may not
   need code graphs.
2. **Git submodule plus source build** — rejected for normal installation;
   Rust 1.97 and a 32-crate build increase time, disk use, and failure surface.
3. **Run `compass install --user` after Skill Pack installation** — rejected by
   reproduced symlink-root failures and signed-generation ownership.
4. **Documentation-only recommendation** — safe but leaves capability/version
   detection, adapter ownership, and uninstall behavior fragmented.
5. **Pack-owned adapter for an externally installed Compass** — the cheapest
   future prototype, but not justified for product integration until a matched
   pilot demonstrates user value.
6. **Optional managed binary plus pack-owned adapter** — deferred. It adds
   recurring release tracking, checksum, rollback, architecture, and drift work
   without evidence that Skill Pack ownership improves adoption or outcomes.

## Prior-decision reconciliation

The memory search surfaced `cowork-cli-integration-planning-phase-goals`, which
already covers toolchain management, plugin management, and installer-pipeline
concerns. Any future Compass work must extend that installation architecture or
explicitly supersede it; this assessment does not authorize a parallel native
binary channel. Codex verify-and-publish decisions likewise remain authoritative
for generated payload and distribution verification.

## Sycophancy review

The optional sycophancy MCP detector is unavailable in this session. Manual
self-check found substantial negative evidence: no measured generation benefit,
large binary cost, installed/source version drift, and a reproduced installer
conflict. The recommendation is conditional and falsifiable rather than an
endorsement of mandatory inclusion.

## Assessment complete

Final candidate verdict after first adversarial review: **NO NEW PHASE and no
Skill Pack installation integration at this time.** Compass is promising, but
the missing matched usefulness evidence is the decision, not merely a task for
an implementation phase to discover later.

## Adversarial review findings addressed

- The first review found that context compression on Compass-favored tasks did
  not measure improved AI output. The revised reversal conditions define the
  baseline, mixed task corpus, blinded quality metric, and efficiency metrics.
- The first review found that the assessment asked the reviewer to supply
  evidence it admitted was missing. The revised decision declines phase
  creation until a pre-phase pilot supplies that evidence.
- Installer safety and capability drift were moved out of the falsifier; they
  remain future acceptance constraints.
- Adapter ownership and binary distribution are now separate decisions, with
  adapter-only qualification required first.
- Existing installer/toolchain decisions are explicitly acknowledged.

## Round-two adversarial warnings retained

- Project-local Compass initialization was not empirically exercised; the
  assessment relies only on installed CLI inspection, capability output,
  installer dry-run evidence, and source/docs review.
- The 139 MB cost observation applies specifically to the installed macOS ARM64
  Compass 0.3.6 executable. Other release artifact sizes and architectures were
  not checked and cannot be inferred from that number.
- This decision deliberately does not assign a pilot owner or schedule. That is
  an explicit deferral, not an implicit implementation backlog.
