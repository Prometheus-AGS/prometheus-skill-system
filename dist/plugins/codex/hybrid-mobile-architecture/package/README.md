# KnowMe Builder

**Version 2.0.0-alpha.4 · governed agentic application generator and skill pack**

KnowMe Builder creates and adopts Flutter, Tauri/React, Axum, and Rust
applications without treating the consumer repository as disposable generator
output. It combines:

- a Rust CLI with ownership-aware generation and upgrades;
- explicit application profiles;
- typed UAR, A2UI, AG-UI, identity, policy, persistence, and native-bridge
  boundaries;
- 36 public, self-contained Agent Skills (one package skill and 35 companions);
- generated commands and advisory activation adapters for Claude Code, Codex,
  OpenCode, and Kimi Code;
- receipt-owned skill installation for Claude Code, Codex, OpenCode, Kimi Code,
  MiniMax Code CLI, and Zed; and
- conformance, security, documentation, and installation gates.

Public documentation:
[KnowMe Builder documentation](https://know-me-tools.github.io/hybrid-mobile-architecture-skill/)

Start with:

- [Installation](https://know-me-tools.github.io/hybrid-mobile-architecture-skill/reference/installation)
- [CLI reference](https://know-me-tools.github.io/hybrid-mobile-architecture-skill/reference/cli)
- [Generation profiles](https://know-me-tools.github.io/hybrid-mobile-architecture-skill/architecture/profiles)
- [All 36 public skills](https://know-me-tools.github.io/hybrid-mobile-architecture-skill/reference/skills)
- [Services and boundaries](https://know-me-tools.github.io/hybrid-mobile-architecture-skill/reference/services)
- [Utilities and automation](https://know-me-tools.github.io/hybrid-mobile-architecture-skill/reference/utilities)
- [Common use cases](https://know-me-tools.github.io/hybrid-mobile-architecture-skill/reference/use-cases)

## Authority model

Three systems have deliberately separate jobs:

| System | Authority |
|---|---|
| KnowMe Builder | Application architecture, templates, typed adapters, skills, and conformance |
| Prometheus | Project UUID identity, signed KBD events, CRDT claims/conflicts, typed lifecycle, audit, and durable learning |
| Universal Agent Runtime | Agent runs, providers, prompts, governed tools, cancellation, recovery, and A2UI/AG-UI events |

Builder activation hooks are advisory. They never own lifecycle or mutation
authority. Generated applications must not introduce a second agent loop beside
UAR.

## Install

Build and install the CLI from a trusted checkout on macOS, Linux, or Windows:

```bash
cargo install --locked --path tools/knowme-builder
knowme-builder --version
```

Install all 36 public portable skills from a trusted checkout:

```bash
npx skills add ./skills \
  --skill '*' -a claude-code -a codex -a opencode -g -y
```

Install the complete native harness package from a trusted checkout:

```bash
node scripts/install-global-harnesses.mjs
```

That registers the Git source through the supported Claude/Codex marketplace
CLIs; installs portable skills for OpenCode, Kimi Code, MiniMax Code CLI, and
Zed in their native discovery roots; installs the OpenCode advisory plugin; and
records an ownership receipt. CLI compilation and MCP configuration are opt-in
with `--with-cli` and `--with-mcp`. The installer never invokes shell or Python
bootstrap scripts.

Verify:

```bash
knowme-builder manifest check
knowme-builder --json doctor --path .
prometheus doctor --json
```

Check native prerequisites independently of the Prometheus control plane:

```text
knowme-builder doctor --native-only --target x86_64-pc-windows-msvc --target aarch64-pc-windows-msvc --json
```

This distinguishes Rust target availability from Windows SDK/linker and runtime
verification. See [portable tooling status](docs/portability/project-tools.md).

See [docs/global-harness-installation.md](docs/global-harness-installation.md)
for native plugin registration and per-harness verification.

## Generate a project

Profiles are explicit:

```bash
knowme-builder new my-app \
  --profile sovereign-hybrid \
  --mode runnable \
  --check

knowme-builder new my-app \
  --profile sovereign-hybrid \
  --mode runnable
```

Available profiles:

| Profile | Architecture |
|---|---|
| `sovereign-hybrid` | Flutter mobile, Tauri desktop, Rust adapters, embedded/service UAR |
| `governed-web-shell` | React, Axum BFF, service UAR, PEM/PGlite, optional legacy embed |
| `flutter-mobile` | Flutter, Rust, embedded UAR |
| `tauri-desktop` | Tauri/React, Rust, service or in-process UAR facade |
| `axum-web` | React, Axum, service UAR |

`runnable` emits complete locked application entrypoints and a persisted Notes
vertical slice through UI state, repository ports, Rust use cases and SQLite.
Certification remains target-specific: the generated source is runnable, while
the receipt or CI job for a host proves its native build and execution.
`skeleton` records missing surfaces and never receives runnable certification.

## Adopt an existing application

Do not regenerate an evolved application:

```bash
knowme-builder adopt existing-app \
  --profile governed-web-shell \
  --check

knowme-builder adopt existing-app \
  --profile governed-web-shell \
  --apply
```

Adoption records profile and package state and creates a project policy overlay
without claiming ownership of existing application files. Skill installation is
the separate `knowme-builder skills install --path existing-app` operation.
Adoption metadata does not implement missing surfaces or integrate new features.
Follow the [project evolution workflow](skills/hybrid-mobile-architecture/references/project-evolution.md)
for brownfield changes and continuing upgrades of previous outputs.

## Ownership-aware upgrades

```bash
knowme-builder upgrade existing-app --check
knowme-builder upgrade existing-app --apply
knowme-builder upgrade existing-app --rollback
```

`.knowme-builder/generated.lock.json` records the generated path, template,
source digest, last installed digest, ownership, and version. The Builder
replaces only managed files that still match their last installed digest.
Modified files produce a conflict report and proposed replacement.

Conflicts are discovered before managed application files are changed; a failed
JSON result also returns a nonzero process exit. New outputs retain their
rendering name across directory moves. Legacy state without that identity needs
`upgrade --app-name <original-name>`; unsupported schemas/versions require an
explicit migration. Managed-file upgrades preserve original bytes and output
hashes in a recovery journal; rollback refuses to overwrite later user edits.
An interrupted apply requires rollback before retrying. Explicit migration IDs
cover the supported `2.0.0-alpha.3` to `2.0.0-alpha.4` transition, including
added, removed and renamed files, dependency manifests and regenerated bridge
outputs. Unsupported histories fail with a migration diagnostic. Application
database migrations remain a separately reviewed operation.

## Add capabilities

```bash
knowme-builder add feature conversations --path .
```

Feature additions are written into the capability registries already consumed
by each generated surface, recorded in ownership state, included in previews,
and preserved by later upgrades. A compatible adopted project can use this path
when it exposes the recognized empty registry seam. Auth, module and
legacy-embed adapters are intentionally rejected before writes in this release;
they require their own identity, routing or message-boundary implementation and
runtime evidence.

## Skill bundle

`skills` is the canonical public tree. It contains the package-level
`hybrid-mobile-architecture` skill plus 29 project companions:

- quality and release: `a11y-gate`, `flutter-golden-ui`,
  `tauri-ui-review`, `reference-ui-fidelity`,
  `hybrid-runtime-verification`;
- agent UI and runtime: `a2ui-surface-contract`, `agui-event-contract`,
  `content-block-ui`, `local-inference-lanes`,
  `agent-runtime-security`, `axum-agent-gateway`,
  `persona-scoped-agent`;
- application architecture: `entity-graph-web-shell`, `mini-app-module`,
  `legacy-app-embed`, `pem-local-first`;
- data and privacy: `sync-doctrine`, `peer-profile-sync`, `client-rag`,
  `anonymized-replica`, `domain-glossary-service`;
- design: `hybrid-design-tokens`, `mobile-navigation`,
  `tauri-custom-titlebar`;
- build, delivery, and workflow: `dependency-pin-discipline`,
  `deploy-hybrid-agentic-stack`, `build-branded-docusaurus`,
  `orchestrate-prometheus-application`, `karpathy-progress-memory`.

Install or check project copies:

```bash
knowme-builder skills install --path .
knowme-builder skills check --path .
```

Generated harness trees must not be edited independently:

```bash
node scripts/sync-harness-skills.mjs
node scripts/sync-harness-skills.mjs --check
```

## Application runtime contract

A runnable profile must support:

```text
message
  → UAR run
  → model stream
  → governed tool
  → A2UI/AG-UI event
  → persisted projection
  → restart recovery
```

UI layers are projections. They do not own provider routing, prompts, tools,
policy, or run lifecycle.

Application-facing tool requests go through UAR governance:

1. trusted server/tool resolution;
2. JSON Schema validation;
3. independent effect classification;
4. verified identity and policy;
5. confirmation when required;
6. idempotent, bounded, cancellable execution;
7. result validation/redaction; and
8. immutable audit outcome.

## Verification

Core package checks:

```bash
node scripts/check-builder-authority.mjs --release
node scripts/check-skill-contracts.mjs
node scripts/check-runtime-security.mjs
node scripts/check-prometheus-boundary.mjs
node scripts/sync-skill-resources.mjs --check
node scripts/sync-harness-skills.mjs --check
node scripts/check-git-url-discovery.mjs
node scripts/test-harness-installer.mjs
cargo test --locked --manifest-path tools/knowme-builder/Cargo.toml
```

Documentation:

```bash
cd site
npm ci
npm run release:check
```

A build is not runtime evidence. Use `hybrid-runtime-verification` for a clean
checkout, production artifact, real launch, persistence, public workflow, and
physical-device proof when native bridges or local inference are involved.

## Repository map

| Path | Purpose |
|---|---|
| `builder.manifest.json` | Canonical package, profile, skill, template, target, and harness manifest |
| `tools/knowme-builder` | Rust CLI |
| `skills` | Canonical 30-skill public source |
| `templates/project-skills` | Generated 29-skill project/scaffold projection |
| `assets/templates` | Maintained profile, feature, adapter, native, build, and contract templates |
| `compatibility` | Prometheus and UAR external contract descriptors |
| `versions.toml` | Builder application-stack version authority |
| `scripts` | Bootstrap, generation, compatibility, audit, and installation utilities |
| `site` | Public Docusaurus source and publication gates |
| `docs` | Specifications, evidence, prompting guides, research, and documentation map |

Read [docs/documentation-map.md](docs/documentation-map.md) before treating a
dated assessment or research file as current authority.

## Release status

`2.0.0-alpha.2` adds standards-conformant Git-URL distribution and current
Prometheus 1.7 integration to the production-convergence architecture and
consumer adoption support. Stable `2.0.0` remains gated by the complete consumer CI
suites and current physical-device certification required by their profiles.
