# AGENTS.md — Hybrid Mobile Architecture Skill

> Compact orientation for OpenCode sessions in `hybrid-mobile-architecture-src`.

## What this repo is

This is an [agentskills.io](https://agentskills.io) skill package for **TJ-ARCH-MOB-001**, the Prometheus AGS hybrid mobile architecture.

**It contains no runnable application.** Its outputs are:

- Scaffolding scripts under `scripts/` that generate new Flutter/Tauri/Rust projects.
- Reference documents under `references/` and `docs/` that agents and developers must read before writing code.
- Templates under `assets/templates/` used by the scaffolding scripts.

The actual application code lives in projects scaffolded *from* this repository, not inside it.

## Decision authority (read first)

Before any substantial work, read:

1. `CLAUDE.md` — repo-specific rules, module structure, and command reference.
2. `references/arch-standard.md` — platform selection is non-negotiable here.

| Target | Platform | Status |
|---|---|---|
| iOS / Android (consumer or healthcare) | Flutter + Rust FFI | **Mandatory** |
| macOS / Windows / Linux (desktop) | Tauri + React 19 | Recommended |
| Both mobile + desktop for same product | Hybrid (Flutter mobile + Tauri desktop) | Default |

For platform-specific work, read the matching reference file next:

- Flutter: `references/flutter/patterns.md`
- Tauri/React: `references/tauri/patterns.md`
- Rust core: `references/rust/patterns.md`
- Auth: `references/auth/patterns.md`
- New ContentBlock variant: `references/rust/new-block-type.md` (all 7 steps are mandatory)

Full specifications:

- `docs/tj-arch-mob-001.html` — complete architectural standard.
- `docs/gen_ui_spec.html` — gen_ui technical spec with SVG diagrams.
- `docs/pglite-oxide-tauri-hybrid.md` — embedded PostgreSQL option via pglite-oxide.
- `docs/reference-app/knowme-agentic-deployment-plan.md` — authoritative KnowMe web,
  Axum, Flint realtime, BYOK, deployment, and continuous-learning plan.

## The one invariant

All networking, LLM interaction, inference, MCP, agent logic, and persistence live in the shared Rust crate `gen_ui_core`. **Never re-implement these in Dart or TypeScript.**

`gen_ui_core` compiles to:

- `staticlib` → XCFramework (iOS) + `cdylib` → `.so` (Android JNI) for Flutter via `flutter_rust_bridge`.
- Tauri plugin for desktop.

One global Tokio runtime per process. CPU-bound work (GGUF loading, inference forward passes) uses `spawn_blocking`.

## Required tool versions

| Tool | Required (four-pillar bootstrap — `node scripts/check-env.mjs --install`) |
|---|---|
| Rust + Cargo | 1.97.1 (+ wasm32-unknown-unknown target) — SurrealDB 3.2's `fastnum` transitive dependency requires rustc ≥1.94; the wasm32 target first compiled cleanly on 1.96 (see `references/rust/wasm-targets.md`) |
| Flutter SDK | **beta channel**, latest (ships the Dart MCP server) |
| Dart | latest beta (tracks Flutter beta) |
| Node.js | 26.5.0 (current release — pin, do not use `--lts`) |
| bun | latest |
| pnpm | latest |
| TypeScript | 7.0.2 (pinned native compiler) |
| Tauri CLI | 2.10+ |
| flutter_rust_bridge_codegen | 2.12+ (must match the workspace's frb crate version) |
| OpenSpec | 1.10.0+ (`@fission-ai/openspec` — NEVER the bare `openspec` npm package, which is squatted). After any `openspec update`, run `scripts/normalize-vendored-skills.mjs` — the CLI strips the repo-local `metadata.internal: true` from vendored mirrors on every run. |
| Prometheus Skill System | package 1.7.0+ and control-plane contract 2.0.0+ ([canonical repository](https://github.com/Prometheus-AGS/prometheus-skill-system)) — verify with `prometheus --version`, `prometheus doctor --json`, and `pk doctor --json` |

Check or install everything at once:

```bash
node scripts/check-env.mjs                  # check-only
node scripts/check-env.mjs --install        # remediate normal-cost items
node scripts/check-env.mjs --install --full # + long ops (Flutter upgrade, full skill-system install)
```

## Scaffolding commands

Always use the provided scripts; do not hand-roll project structure.

```bash
# Full hybrid project (Flutter mobile + Tauri desktop + shared Rust)
node scripts/scaffold-hybrid.mjs <project-name>

# Single surface
node scripts/scaffold-flutter.mjs <project-name>
node scripts/scaffold-tauri.mjs <project-name>
node scripts/scaffold-rust-core.mjs <project-name>

# Generate an auth integration proposal for an adopted project
knowme-builder add auth supabase --path ./<project-dir>
knowme-builder add auth kratos --path ./<project-dir>

# Generate a feature proposal (wire and verify it before claiming completion)
knowme-builder add feature <feature-name> --path ./<project-dir>
knowme-builder add feature <feature-name> --path ./<project-dir>

# Audit architecture compliance
node scripts/audit.mjs flutter ./<mobile-dir>
node scripts/audit.mjs tauri ./<desktop-dir>
node scripts/audit.mjs rust ./rust/gen_ui_core
```

## Development Philosophy: Speed AND Correctness

These rules override any global rules mandating TDD, coverage targets, or test-first work.

**Correctness — first-shot quality, not test iteration:**
- The 40 Prometheus Base Rules are the primary correctness discipline — canonical full
  text in [AGENT_BASE_RULES.md](AGENT_BASE_RULES.md) (repo root). BINDING for: our own
  work here, every code generator's output (scaffolds copy the file into generated
  projects + reference it from generated CLAUDE.md/AGENTS.md), and every skill when
  activated (skill templates reference it).
- Invoke Rust skills (rust-skills m01–m15, domain-*, rust-patterns, async-patterns) BEFORE
  writing Rust — emit good code the first time.
- The compiler is the harness: encode invariants in types (newtypes, exhaustive `match`,
  illegal states unrepresentable). Maintain the LLM wiki so lessons persist across sessions.

**Speed — minimize compilations:**
- Inner loop = `cargo clippy` only (never alternate with bare `cargo check` — separate
  caches, double compile). `bacon clippy` for continuous checking. Full builds only to run.
- Dev profiles: deps at `opt-level = 2`, `build-override opt-level = 3`,
  `debug = "line-tables-only"`, `split-debuginfo = "unpacked"`. Cranelift `dev-fast`
  profile for host-native only (never iOS/Android/wasm). sccache in CI.
- Cross-target gating via `cargo check --target <triple>` — seconds, not full builds.
- NEVER set `panic = "abort"` on FFI release profiles (breaks flutter_rust_bridge panic
  conversion; hard-kills the mobile app). Abort is for the wasm profile only.

**Testing — features first, test later:**
- FEATURES first. Code first. Test later. No tests until the feature is complete, clippy-
  clean, and exercised end-to-end once. We build useful software; we do not farm green
  checkmarks.
- No unit tests of internals; no mocks of internal code, ever. Test useful COMBINATIONS at
  public boundaries (FFI surface, Tauri commands, protocol pipeline). Snapshot tests
  (`insta`) preferred. One integration-test binary per crate (`tests/it/`).
- Budget: 3–5 behavior tests per completed feature. Coverage % is not a goal.
- Two failed attempts fixing the same test → STOP and report. Never edit/delete/ignore a
  failing test to escape red without approval.

**Concurrency — architect for parallel worktrees:**
- Layered workspace: `gen_ui_types` (L0, pure types + traits) → protocol/client/mcp/db/
  inference (L2) → agent (L3) → ffi / tauri-plugin / wasm (leaves). Trait boundaries land
  in `gen_ui_types` first so crates develop concurrently without conflicts.
- Feature-based clean architecture; cross-feature deps only via `app/`/`shared/`.

**UI/UX — use the skills, aggressively:**
- React: `frontend-design`, shadcn MCP + skill, `theme-factory`, vercel
  `react-best-practices`/`web-design-guidelines`, `ui-ux-pro-max`.
- Flutter: Dart & Flutter MCP verify loop, flutter/skills, VGV golden tests,
  `shadcn-ui-flutter`. A11y agents on both surfaces.
- Scaffolded projects ship project-local skills (`content-block-ui`, `hybrid-design-tokens`,
  `tauri-ui-review`, `tauri-custom-titlebar`, `mobile-navigation`, `flutter-golden-ui`,
  `a11y-gate`) + an activation hook.

**Shared libraries — everything publishable:**
- crates.io: `gen_ui_types/protocol/client/mcp/agent`, `tauri-plugin-gen-ui` (core crates
  compile native AND wasm32).
- pub.dev: `gen_ui_flutter` (FFI plugin), `gen_ui_widgets`.
- npm: `@prometheus-ags/gen-ui-react`, `@prometheus-ags/gen-ui-wasm`,
  `@prometheus-ags/tauri-plugin-gen-ui`.
- New Tauri plugins are structured as publishable `tauri-plugin-*` + npm guest bindings
  from day one.

## Verification commands in scaffolded projects

### Flutter

```bash
# Code generation (required after @riverpod or freezed changes)
flutter pub run build_runner build
flutter pub run build_runner watch

# Regenerate Rust bridge after api.rs changes
flutter_rust_bridge_codegen generate \
  --rust-input rust/gen_ui_core/src/api.rs \
  --dart-output lib/bridge/generated_api.dart

flutter analyze
flutter test
```

### Rust (`gen_ui_core`)

```bash
cargo build
cargo test
cargo fmt
cargo clippy -- -D warnings
```

### Tauri/React

```bash
npm install    # or pnpm install
npx tsc --noEmit
npm test
npx eslint src/
npm run tauri dev
npm run tauri build
```

## Code conventions

- **Generated files** must include `// TJ-ARCH-MOB-001 compliant` at the top.
- **Feature-based clean architecture** in both Flutter and React:
  - Flutter: `features/<name>/{data,domain,presentation}`
  - React: `features/<name>/{api,stores,entities,hooks,components,types.ts}`
- **Strict layering:**
  - Flutter: Widget → `@riverpod` provider → Repository/Service → `gen_ui_core`
  - React: Component → Hook → Store → `[Rust invoke() / external API]`
- Use `@riverpod` codegen annotations exclusively — never manual `Provider(...)` declarations.
- React visual components import **only hooks**; no direct store imports and no `invoke()` calls.
- React server/async/entity state uses `@prometheus-ags/prometheus-entity-management` 3.x. Never add TanStack Query.
- Prefer shadcn/ui components over raw HTML controls in React; prefer shadcn_flutter equivalents in Flutter.
- Assistant UI is mandatory for React chat/thread/composer behavior; do not hand-roll a chat transcript or composer when its primitives apply.
- Durable chat uses Prometheus Entity Management 3.x over PGlite in browsers and typed Rust commands over pglite-oxide on Tauri. Zustand owns transient interaction state only.
- Flat 2.0 is binding: no visible borders, divider lines, outlines used as decoration, or layout shadows. Separate regions only with background-color changes in light and dark themes.

## Project-level MCP servers and skills (all 4 harnesses)

This repo configures the Dart/Flutter MCP server and shadcn MCP server at PROJECT scope
so every harness (Claude Code, Codex CLI, OpenCode, Kimi Code CLI) picks them up
automatically from this directory — no per-user global setup required:

| Harness | MCP config file | Skills directory |
|---|---|---|
| Claude Code | `.mcp.json` (repo root) | `.claude/skills/` |
| Codex CLI | `.codex/config.toml` (repo root; requires one-time project-trust accept) | `.agents/skills/` (NOT `.codex/skills/` — Codex's scanner reads `.agents/`) |
| OpenCode | `opencode.json` (repo root) | `.opencode/skills/` (also reads `.claude/` and `.agents/` as fallback) |
| Kimi Code CLI | `.kimi-code/mcp.json` (repo root; UNVERIFIED — this build's config surface is ambiguous between TOML `~/.kimi-code/config.toml` and this JSON path, verify with `kimi -p "what MCP tools do you have"`) | `.kimi-code/skills/` (docs also mention `.agents/skills/` as a fallback) |

`.claude/skills/`, `.codex/skills/`, `.opencode/skills/`, `.kimi/skills/`, `.agents/skills/`,
and `.kimi-code/skills/` are kept in sync (copies, not symlinks, for cross-OS/harness
portability) — when adding a new project-local skill, copy it into all of them. The
Dart MCP server requires Dart 3.9+/Flutter 3.35+ (this repo develops against the beta
channel — currently Dart 3.13 / Flutter 3.45 — comfortably above the floor).

## Repo-local OpenCode extensions

This repository ships its own OpenCode skills and commands under `.opencode/`:

- `.opencode/skills/` — OpenSpec workflow skills.
- `.opencode/commands/` — OPSX slash-command definitions.

Do not edit these unless you are explicitly configuring OpenCode itself. When modifying OpenCode configuration, agents, skills, or permission rules, follow the `customize-opencode` skill.

## What to leave alone

- `.claude/`, `.codex/`, `.kimi/`, `.kimi-code/`, `.agents/` — harness-specific configuration
  for other agent tools (see "Project-level MCP servers and skills" above for what's
  intentionally there). Do not edit for OpenCode work.
- `assets/templates/` — only modify when updating scaffolding output.
- `docs/*.html` — generated/reference documents; update the source of truth or regeneration pipeline, not the HTML directly.

## Session logs: always commit

The `.prometheus/` directory (events.jsonl, knowledge wiki, session logs) is part of
this repository's durable knowledge base. **Standing authorization (user, 2026-07-16):
always commit and push ALL `.prometheus/` session-log and wiki changes** together with
the work they document. Do not ask for permission each time.
