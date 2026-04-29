# KBD Assessment — Compliance & Power-Multiplier Phase

> **Phase**: `phase-compliance-and-power-multiplier`
> **Date**: 2026-04-28
> **Tool**: Claude Code (Opus 4.7)
> **Skill pack version**: 1.2.0

---

## Phase Goals

1. Audit full compliance against the latest **agentskills.io** open-standard.
2. Audit full compliance against the **Claude Code plugin marketplace** schema.
3. Audit compatibility with **opencode** plugin/extension standards.
4. Audit **hooks** usage against current Claude Code hook semantics.
5. Evaluate the **Karpathy-toolchain** integration (forge-rs ↔ pk ↔ rustbpe ↔ nanochat) and ensure the *native-agent* generator first-classes the available Karpathy Rust libraries.
6. Ensure the agent-runtime environment **bootstraps its own dependencies** — Rust toolchain detection + auto-install, building of `surreal-memory-server`, `liter-llm`, `prometheus-knowledge`, and `forge-rs` if they are missing.
7. Determine how to wire the skill pack to the **librefang fork** at `/Users/gqadonis/Projects/references/librefang` so that generated agents can be **packaged as WASM and uploaded to a librefang/bossfang instance via URL**.
8. Recommend the *minimum coherent additions* that make this pack the most powerful **ideation → planning → implementation → dynamic-tool-creation** pipeline available.
9. Web-research-backed **impact report**: what does landing this skill library do to a developer team's velocity and quality.

---

## 1. Inventory Snapshot

### 1.1 Skills (29 SKILL.md files, plus 7 nested entity sub-skills)

| Domain | Skills |
|---|---|
| **process/** (orchestration) | `native-agent`, `zeespec-interrogator`, `iterative-evolver`, `kbd-process-orchestrator`, `pmpo-skill-creator`, `liter-llm-bridge` |
| **architecture/** | `clean-architecture` |
| **rust/** | `axum-patterns`, `error-handling`, `async-patterns`, `workspace-structure`, `mcp-server`, `actor-model`, `performance` |
| **react/** | `react-vite-stack`, `prometheus-entity-skills` (+6 nested entity-graph sub-skills) |
| **flutter/** | `flutter-rust-ffi` |
| **tauri/** | `tauri-react-vite` |
| **htmx/** | `htmx-alpine-lit` |
| **typescript/** | `base-patterns` |
| **go/** | `base-patterns` |
| **python/** | `pyo3-bridge` |
| **devops/** | `gitops-bootstrap`, `gitops-transform`, `argocd-multicloud`, `kustomize-overlay` |
| **testing/** | `bdd-testing` |
| **imported/** (submodules) | `artifact-refiner`, `sycophancy-correction` |
| **(empty stubs)** | `documentation/`, `ui-ux/` |

### 1.2 Tools (Rust workspaces / submodules)

| Tool | Type | Role |
|---|---|---|
| `tools/forge-rs/` | In-repo Rust workspace (6 crates) | Layer 4 enrichment engine, MCP server :8943 |
| `tools/prometheus-cli/` | In-repo Rust workspace (4 crates) | Skill-management CLI + Cedar governance |
| `tools/surreal-memory-server/` | Submodule | Knowledge graph + distributed state |
| `tools/liter-llm/` | Submodule | LLM proxy + 22 MCP tools, multi-provider routing |
| `tools/prometheus-knowledge/` | Submodule | Karpathy-method wiki (`pk` CLI + `pk-cherry` MCP :8942) |

### 1.3 Hooks

`hooks/hooks.json` — 5 lifecycle events wired:
`SessionStart`, `PreToolUse(Bash)`, `PostToolUse(Write|Edit|MultiEdit)`,
`SubagentStop` (5 phase matchers), `Stop`.

### 1.4 Distribution surfaces

- `.claude-plugin/plugin.json` v1.2.0 (8 platforms declared in `compatibility.platforms`)
- `marketplace/marketplace.json` v1.0.0 (5 plugins: full pack + 4 sub-packs)
- `.opencode/package.json` + `.opencode/tools/{evolve,gitops,kbd}.ts` (TypeScript opencode tools, `@opencode-ai/plugin` 1.14.28)
- `scripts/install-platforms.ts` — symlinks into 8 AI-tool directories
- `scripts/register-slash-commands.sh` — registers slash-commands in opencode + codex

---

## 2. Compliance Gap Report

### 2.1 agentskills.io v2026 — **Largely compliant, 4 fixes needed**

The current spec ([agentskills.io/specification](https://agentskills.io/specification), [DeepWiki: Agent Skills Specification](https://deepwiki.com/anthropics/skills/6.1-agent-skills-specification)) requires:

| Requirement | Status | Evidence |
|---|---|---|
| `SKILL.md` exists per skill | ✅ | All 29 skills validated by `scripts/validate-skills.js` |
| YAML frontmatter `name` (≤64 chars, kebab-case `^[a-z0-9]+(-[a-z0-9]+)*$`) | ✅ | Schema enforced in `validate-skills.js:14-22` |
| YAML frontmatter `description` (1–1024 chars) | ⚠️ | Schema allows 1024; **spec now caps at 200** for the discovery summary. Several SKILL.md files exceed 200 chars |
| Forward-slash paths only | ✅ | Validator strips code blocks, then checks for `\` |
| Standard sub-dirs `scripts/`, `references/`, `assets/` | ✅ | Used throughout |
| Progressive disclosure (main < 500 lines) | ⚠️ | `kbd-process-orchestrator/SKILL.md` and a few entity skills exceed 500 lines |
| `name` matches dir | ✅ | Warning emitted otherwise |

**Gap #A1 — Description-length violation.** Cap descriptions at 200 chars (the new discovery limit) and use a separate `summary` or move long-form into the body. Tighten validator: `maxLength: 200` for `description`.
**Gap #A2 — Add optional first-class fields the spec now blesses.** `version`, `license`, `metadata.tags[]`, and `metadata.category` are present in some SKILL files (e.g. `prometheus-entity-skills`) but not standardized. Add to schema as optional.
**Gap #A3 — `documentation/` and `ui-ux/` are empty directories.** These violate marketplace listing integrity (the marketplace lists them as plugins). Either delete or populate with a stub skill before publish.
**Gap #A4 — License field absent at the SKILL.md level for most skills.** The new spec recommends per-skill `license` since redistribution paths can detach a skill from the umbrella pack.

### 2.2 Claude Code plugin marketplace — **Compliant, 2 enhancements suggested**

Reference: [anthropics/claude-plugins-official marketplace.json](https://github.com/anthropics/claude-plugins-official/blob/main/.claude-plugin/marketplace.json) and [Plugin marketplace docs](https://code.claude.com/docs/en/plugin-marketplaces).

| Requirement | Status |
|---|---|
| `.claude-plugin/marketplace.json` at repo root with `marketplace_version`, `name`, `owner`, `plugins[]` | ✅ |
| Each plugin has `source.{type, url, path}` | ✅ |
| Frontmatter on `marketplace.json` (rare but valid) | ✅ |
| Plugins reference a real `.claude-plugin/plugin.json` for component resolution | ✅ |
| `hooks` declared at the plugin level | ✅ |
| `mcpServers` field | ⚠️ Declared in `plugin.json` but `.mcp.json` referenced does not exist in the repo |
| `commands/` directory for slash-commands | ⚠️ The pack ships slash-commands via `register-slash-commands.sh` rather than the plugin-native `commands/` directory |

**Gap #B1 — Missing `.mcp.json`.** `plugin.json` line 41 declares `"mcpServers": "./.mcp.json"` but the file does not exist. Either create it (forge-rs MCP, pk-cherry MCP, surreal-memory MCP, liter-llm MCP) or remove the field.
**Gap #B2 — Adopt native plugin-commands directory.** Migrate slash-commands into `.claude-plugin/commands/*.md` so Claude Code auto-discovers them on plugin install — eliminates the bash-registration step.

### 2.3 Hooks usage — **Solid; 2 robustness fixes**

`hooks/hooks.json` correctly uses the modern format (`hooks: [{matcher, hooks: [{type, command, timeout}]}]`).

| Concern | Status |
|---|---|
| `SessionStart` runs `detect-project-context.sh` | ✅ — but no `timeout` is too generous; 15s is fine, but the hook should `exit 0` even on detection failure (it does, via `|| true`) |
| `PreToolUse` Bash guard against direct deploys | ✅ |
| `PostToolUse` runs `validate-state.sh` + `validate-gitops-write.sh` | ✅ |
| `SubagentStop` matches sub-agent names | ✅ — but there is **no fallback matcher** for unrecognized sub-agent names |
| Hook scripts use `${CLAUDE_PLUGIN_ROOT}` | ✅ (correct convention) |

**Gap #C1 — Add a `UserPromptSubmit` hook** to inject Karpathy-focus context (`pk focus "<latest user query keywords>"`) into the system prompt automatically when a project has a `prometheus-knowledge` instance running. This is the missing closed-loop link between *user intent* and *learned context*.
**Gap #C2 — Add a `Stop` hook step** that calls `forge reflect` on the just-completed task so the Karpathy learning loop runs without manual prompting.

### 2.4 OpenCode compatibility — **Functional but underutilized**

Current state: `.opencode/package.json` declares `@opencode-ai/plugin@1.14.28` and `.opencode/tools/{evolve,gitops,kbd}.ts` define typed OpenCode tools.

OpenCode plugin spec ([Plugins | OpenCode](https://opencode.ai/docs/plugins/)) requires:

- A JS/TS module that **exports a Plugin function** (named or default), receiving a context object and returning a hooks object.
- `package.json` with `@opencode-ai/plugin` + `@opencode-ai/sdk` (the SDK is **missing** from current package.json).
- Plugins are listed in `opencode.json` under `plugin: [...]`.

| Requirement | Status |
|---|---|
| `package.json` deps | ⚠️ Has `@opencode-ai/plugin` but **lacks `@opencode-ai/sdk`** |
| Plugin function export | ❌ The TS files in `.opencode/tools/` are *tool definitions*, not Plugin functions |
| `opencode.json` registration | ❌ No `opencode.json` in the repo |
| Hooks via OpenCode plugin API | ❌ No PostToolUse/PreToolUse hooks expressed in opencode-native form |

**Gap #D1 — Add a real `.opencode/plugin.ts`** that exports a default Plugin function. The function should register the same lifecycle hooks (in opencode terminology) that `hooks/hooks.json` defines for Claude Code, plus the existing tools.
**Gap #D2 — Add `@opencode-ai/sdk` to `.opencode/package.json` dependencies** (per the docs, both packages are required for typed plugins).
**Gap #D3 — Generate an `opencode.json`** at install time via `scripts/install-platforms.ts --platform opencode` so the plugin auto-loads.

---

## 3. Karpathy-Toolchain Evaluation

### 3.1 What is currently wired

- `tools/prometheus-knowledge/` → `pk` CLI (`focus`, `ingest`, `lint`, `search`) + `pk-cherry` MCP on :8942.
- `tools/forge-rs/` → calls `pk focus` during `forge enrich` and `pk ingest` during `forge reflect`.
- The native-agent generator does **not** currently first-class `prometheus-knowledge` as a default companion service — it's only included in the docker-compose if explicitly opted in.

### 3.2 What's available upstream that we're missing

The Karpathy ecosystem has shifted significantly since the pack was last revised. Per [Karpathy's nanochat announcement](https://x.com/karpathy/status/1977755427569111362) and [karpathy/rustbpe](https://github.com/karpathy/rustbpe):

| Library | Language | Status |
|---|---|---|
| `karpathy/nanoGPT` | Python | **Deprecated** — superseded by nanochat |
| `karpathy/nanochat` | Python (8k LOC, full ChatGPT clone) | **Active**; replaces nanoGPT |
| `karpathy/rustbpe` | **Rust** | **Active**; lightweight BPE tokenizer training (the Python `minbpe` was too slow; HF tokenizers too bloated) |
| `karpathy/minbpe` | Python | Maintenance only |
| `karpathy/llm.c` | C/CUDA | LLM training reference |
| `ToJen/llm.rs` | **Rust** | Community Rust port of llm.c |

**Gap #E1 — The native-agent template should optionally generate a `crates/agent-tokenizer/` that depends on `rustbpe`** so the agent has a built-in, blazing-fast tokenizer for context window estimation, prompt-budgeting, and any local fine-tuning workflows. This makes the generated agent first-class with the modern Karpathy stack.
**Gap #E2 — Add a `skills/rust/karpathy-tokenizer/` skill** that teaches the LLM how to use `rustbpe` correctly (training BPE, exporting to tiktoken format).
**Gap #E3 — Wire `pk` as a default service in the native-agent docker-compose.** Right now it's an opt-in companion. It should be on by default — the Karpathy learning loop is the strongest differentiator of this pack and should run by default, not by opt-in.

### 3.3 Bootstrap & environment readiness — **Half-done**

`scripts/check-prerequisites.sh` correctly:

- Detects Node.js (≥18), offers `nvm`/`brew`/`apt` install paths.
- Detects Rust + Cargo, offers `rustup` install via `curl https://sh.rustup.rs | sh -s -- -y`.

**Gaps:**
- It does **not** detect or build the four submodule binaries (`pk`, `pk-cherry`, `liter-llm`, `surreal-memory-server`, `forge`). A user can pass the prereq check and still have a non-functional pipeline.
- It does **not** verify Docker / Docker Desktop / Compose v2 (the native-agent template uses these).
- It does **not** install the `wasm32-wasip2` target needed for the upcoming WASM packaging path (see §4).

**Gap #F1 — Extend `check-prerequisites.sh` with `--build-tools`** that, when combined with `--install`, runs the four `cargo build --release` steps and copies binaries to `~/.local/bin` (or `/usr/local/bin` if writable). Make this idempotent (skip if `command -v pk` already resolves).
**Gap #F2 — Add `rustup target add wasm32-wasip2`** to the rust-toolchain detection step.
**Gap #F3 — Add Docker detection** mirroring the logic in the existing `docker-detect.sh` template the native-agent generator emits.
**Gap #F4 — Add a `npm run doctor` command** that runs the prereq check + tool build + a smoke test (`forge --version`, `pk --version`, `liter-llm --version`).

---

## 4. LibreFang Fork — WASM Packaging & URL-Based Upload

This is the **single highest-leverage opportunity in the assessment**. The librefang fork already has every piece needed; we just have to wire the native-agent generator to target it.

### 4.1 What librefang provides (verified by direct source inspection)

- `crates/librefang-runtime-wasm/` — Wasmtime-44-based skill sandbox with capability-based permissions, fuel limits, memory limits, epoch-based timeouts, and a documented Guest ABI:
  - Exports: `memory`, `alloc(size)->ptr`, `execute(input_ptr, input_len)->packed_i64`
  - Host imports: `librefang::host_call(req_ptr, req_len)->packed_i64`, `librefang::host_log(level, msg_ptr, msg_len)`
- `crates/librefang-skills/` — full skill loader with **6 runtimes**: `Python`, `Wasm`, `Node`, `Shell`, `Builtin`, `PromptOnly` (default).
- `crates/librefang-skills/src/publish.rs` — `PreparedLocalSkill` packager: validates a skill, scans for security warnings, and zips it (via the `zip` crate) for upload.
- `crates/librefang-skills/src/skillhub.rs` — Skillhub marketplace client (search/install/download), API-format-compatible with ClawHub.
- `crates/librefang-api/src/routes/skills.rs` — REST endpoints:
  - `POST /skills/install` (from local registry or remote)
  - `POST /skills/create`
  - `POST /skills/{name}/evolve/{update,patch,rollback,delete,file}` — full evolution lifecycle
  - `GET /skills/{name}` / `GET /skills/{name}/file`
  - `POST /skills/reload`
- `crates/librefang-api/src/routes/agents.rs` — agent CRUD + bulk operations + sessions + streaming + trajectory export.

### 4.2 The packaging path (proposed)

```
native-agent project
    │
    ▼  /create-native-agent --target librefang-wasm
    │
    ├── Cargo.toml: adds wasm32-wasip2 target binary "agent-skill"
    ├── crates/agent-skill/                 ← new crate, exports the WASM ABI
    │     ├── lib.rs                        ← #[no_mangle] alloc/execute
    │     └── host_bridge.rs                ← thin wrapper around librefang::host_call
    ├── skill.toml                          ← LibreFang manifest:
    │     [skill]
    │     name = "<agent-name>"
    │     [runtime]
    │     type = "wasm"
    │     entry = "agent-skill.wasm"
    │     [tools]                           ← lifted from agent's MCP tool list
    │     [requirements]
    │     capabilities = ["network.outbound", "memory.read"]
    └── scripts/package-for-librefang.sh    ← cargo build --target wasm32-wasip2 --release
                                              + zip agent-skill.wasm + skill.toml + README
                                              + curl -X POST <bossfang-url>/skills/install
                                                   -H "X-Skill-Source: zip"
                                                   --data-binary @<zip>
```

### 4.3 Concrete additions

**Gap #G1 — Add a new skill `skills/rust/librefang-wasm-skill/`** with templates that produce a WASM-ABI-compliant skill crate. Templates needed:

- `Cargo.toml.tera` — sets `crate-type = ["cdylib"]`, target `wasm32-wasip2`
- `lib.rs.tera` — `#[no_mangle] alloc / execute`, JSON in/out, `host_call` wrapper
- `skill.toml.tera` — librefang manifest with capabilities, tools, requirements

**Gap #G2 — Add a new top-level command `/package-as-librefang-skill`** to the native-agent generator. It should:

1. Detect whether the current project was generated by `/create-native-agent`.
2. Generate the `agent-skill` crate alongside the existing `agent-server` crate, sharing the domain types from `agent-core`.
3. Emit `skill.toml`, `package-for-librefang.sh`, and a `librefang.md` deployment guide.

**Gap #G3 — Add `/upload-to-bossfang <url>`** as a slash-command:

```bash
forge package-librefang ./my-agent          # produces my-agent.lf-skill.zip
curl -X POST <url>/skills/install \
     -H "Content-Type: application/zip" \
     --data-binary @my-agent.lf-skill.zip
```

**Gap #G4 — Add a sub-package to `marketplace/marketplace.json`** named `prometheus-librefang-skills` so this capability is discoverable as a focused install.

**Gap #G5 — Document the host-call surface**: `references/librefang-host-abi.md` listing every `librefang::host_call` method the WASM agent can invoke (HTTP, memory, tool-dispatch, kernel handle), keyed off `librefang-runtime-wasm/src/host_functions.rs`.

This converts every native agent the pack generates into a **portable, sandboxed, capability-restricted, OCI-distributable** artifact — aligning with the broader industry direction documented by [Microsoft Wassette](https://opensource.microsoft.com/blog/2025/08/06/introducing-wassette-webassembly-based-tools-for-ai-agents/) and [Server-Side WASM as the Motherboard of Agentic AI](https://sriram-narasim.medium.com/server-side-wasm-the-motherboard-of-agentic-ai-27be7e86ae35).

---

## 5. Power-Multiplier Analysis (Ideation → Plan → Implement → Dynamic Tools)

The pack already has every layer; the problem is **discoverability and a single onramp**. Below is the existing capability mapped to the canonical funnel, plus the missing connectors.

| Stage | Current Tooling | Gap |
|---|---|---|
| **Ideation** | None — user begins cold | **Gap #H1** Add `skills/process/ideation-mindmap/` that uses surreal-memory `generate_ideation_mindmap` to expand a one-line concept into a structured exploration tree |
| **Constraint capture** | `zeespec-interrogator` (Zachman 5W1H, GO/NO-GO manifest) | ✅ |
| **Strategic planning** | `iterative-evolver` (Assess→Analyze→Plan→Execute→Reflect) | ✅ |
| **Tactical planning** | `kbd-process-orchestrator` | ✅ |
| **Spec change-management** | `openspec/` (Layer 3) | ⚠️ The pack documents this but doesn't enforce it via a hook |
| **Code enrichment** | `forge-rs enrich` (Layer 4) | ✅ |
| **Implementation** | Native AI agents (Claude/Cursor/Codex via skills) | ✅ |
| **Dynamic tool creation** | `pmpo-skill-creator` + `forge template new skill` | ⚠️ Disconnected — there is no command that takes "I need a tool that does X" and emits a runnable WASM/native skill plus marketplace listing |
| **Cross-session memory** | `surreal-memory-server` + named PMPO state | ✅ |
| **Learning loop** | Karpathy `pk ingest` on every reflect | ✅ |
| **Distribution** | `marketplace/marketplace.json` (5 plugins) | **Gap #H2** No story for distributing user-created skills back to a private/public hub |
| **Deployment to runtime** | Docker via native-agent | **Gap #H3** No WASM/librefang path (covered in §4) |

### 5.1 The single missing onramp

**Gap #H4 — Add `/start-business-build`**, a top-level orchestrator that:

1. Runs `ideation-mindmap` (new — see Gap #H1) to expand the user's concept.
2. Pipes the mindmap into `zeespec-interrogator` to capture constraints.
3. Hands constraints to `iterative-evolver` which produces an OpenSpec change set.
4. For each change, calls `forge enrich` then dispatches to the implementing AI tool of choice (Claude/Codex/Cursor).
5. On completion, runs `forge reflect` → `pk ingest` (closing the loop).
6. If the user accepts the build, `forge package-librefang` (new — Gap #G2) and offers `/upload-to-bossfang <url>` (Gap #G3).

This single command is the pack's *headline experience*. Everything else is plumbing under it.

### 5.2 Dynamic tool creation as a first-class flow

The path from "I need a tool that does X" to a working LibreFang WASM skill installed in a running bossfang instance should look like:

```
User → /create-tool "scrape weekly competitor pricing"
  ↓
pmpo-skill-creator (existing) generates a SKILL.md + skill.toml
  ↓
forge template new skill rust competitor-scraper (existing)
  ↓
NEW: emit librefang-wasm-skill template (Gap #G1)
  ↓
cargo build --target wasm32-wasip2 --release
  ↓
forge package-librefang (new) → competitor-scraper.lf-skill.zip
  ↓
curl POST $BOSSFANG_URL/skills/install (existing librefang endpoint)
  ↓
curl POST $BOSSFANG_URL/skills/reload
  ↓
Skill is live in the agent OS, callable as a tool
```

Five of seven steps already exist. The two missing steps are the librefang-wasm template (G1) and the package script (G3). **This is roughly a one-week build for one engineer.**

---

## 6. Cross-Tool Progress (from `progress.json`)

No `progress.json` exists for this phase yet — this is the first assessment for `phase-compliance-and-power-multiplier`. There is no prior cross-tool work to incorporate.

---

## 7. Web-Research Impact Report

### 7.1 The productivity context (mid-2025 → 2026)

Independent research is converging on a clear pattern: AI coding assistants help the most when they sit on top of strong process, and hurt when they replace it.

- **Adoption is saturated.** [Stack Overflow 2025 Developer Survey](https://survey.stackoverflow.co/2025/ai/) and [Second Talent's 2026 stats](https://www.secondtalent.com/resources/ai-coding-assistant-statistics/) put daily AI-tool usage above 50% of professionals, generating 41% of code worldwide.
- **Raw productivity gains are modest.** The [METR randomized controlled trial](https://metr.org/blog/2025-07-10-early-2025-ai-experienced-os-dev-study/) — a strong study of experienced OSS developers — actually found a **19% slowdown** when AI was permitted, despite developers reporting they felt faster. Faros AI's [AI Productivity Paradox report](https://www.faros.ai/blog/ai-software-engineering) shows 75% of engineers using AI tools while most orgs see no measurable performance gain.
- **Quality concerns are real.** Independent code analyses cited in the Faros report find ~1.7× more issues in AI-coauthored PRs and 48% of AI-generated code still has security flaws.
- **The differentiator is structure.** Faros explicitly: *"In well-structured organizations, AI acts as a force multiplier… In struggling organizations, AI tends to highlight existing flaws rather than fix them."*

### 7.2 What this skill pack changes

Three properties of this pack address every weak spot in the research literature:

1. **Adversarial reflection at every phase** (the `sycophancy-correction` skill is a nuclear option for the *"AI agreed with my bad plan"* failure mode that the productivity-paradox studies repeatedly identify). When an evolver phase says "the execution completed successfully, with minor edge cases remaining" the sycophancy detector forces it to restate as "the execution deviated in these three ways…".
2. **Mandatory grounding** (zeespec → openspec → forge enrich) injects a constraint manifest into every implementation step, addressing the *"AI generates plausible-but-wrong code"* failure mode that drives the 1.7× issue-rate finding.
3. **Persistent learning** (Karpathy `pk ingest` after every reflect) means the next iteration starts from a corrected base. Most teams using AI assistants today have **no learning loop at all** — every session starts from zero.

### 7.3 Quantitative impact projection

Based on the research base above, a team that adopts this pack can credibly expect:

- **Defect rate**: closer to baseline (forge constitution-check + sycophancy-correction at the reflect phase catch the most common AI-generated bug classes before they reach review).
- **Cycle time**: net positive only if the team adopts the full PMPO loop. Teams that use only the skills (and skip the orchestration) will likely reproduce the METR slowdown.
- **Onboarding time for new engineers**: dramatic reduction once `/start-business-build` exists (Gap #H4) — the same concept-to-WASM-deploy path is the same regardless of language or framework.
- **Knowledge retention across staff turnover**: this is the strongest claim. The Karpathy wiki (`prometheus-knowledge`) is a *permanent, human-readable, lint-checked* corpus that survives team churn. Today most teams' AI institutional knowledge is locked in private GitHub Copilot sessions and Slack threads, neither of which survive turnover.

The pack's headline value-proposition therefore is not "writes code faster" — it's **"prevents the failure modes the productivity-paradox research has now documented"**.

---

## 8. Prioritized Gap Punch List

| # | Gap | Effort | Impact | Priority |
|---|---|---|---|---|
| **G1** | `librefang-wasm-skill` skill + templates | M | **Massive** — unlocks WASM packaging | **P0** |
| **G2** | `/package-as-librefang-skill` command on native-agent | S | Massive | **P0** |
| **G3** | `/upload-to-bossfang <url>` slash-command | S | Massive | **P0** |
| **H4** | `/start-business-build` top-level orchestrator | M | Massive — single onramp | **P0** |
| **F1** | `check-prerequisites.sh --build-tools` | S | Critical — closes silent-failure gap | **P0** |
| **F2** | Add `wasm32-wasip2` target | XS | Critical for G1 | **P0** |
| **A1** | Description-length cap to 200 chars | XS | Compliance | **P1** |
| **A3** | Remove or populate empty `documentation/` and `ui-ux/` dirs | XS | Marketplace integrity | **P1** |
| **B1** | Create `.mcp.json` or remove the dangling reference | XS | Plugin install correctness | **P1** |
| **D1** | Real `.opencode/plugin.ts` exporting Plugin function | S | OpenCode compatibility | **P1** |
| **D2** | Add `@opencode-ai/sdk` dep | XS | OpenCode compatibility | **P1** |
| **C1** | `UserPromptSubmit` hook → `pk focus` injection | S | Closes Karpathy loop on entry | **P1** |
| **C2** | `Stop` hook → auto `forge reflect` | S | Closes Karpathy loop on exit | **P1** |
| **E1** | `agent-tokenizer` crate using `rustbpe` in native-agent | S | Modern Karpathy stack | **P2** |
| **E2** | `karpathy-tokenizer` skill | S | Teaches LLM to use rustbpe | **P2** |
| **E3** | Default-on `prometheus-knowledge` in native-agent compose | XS | Karpathy loop on by default | **P2** |
| **B2** | Migrate slash-commands to native `commands/` dir | S | Fewer install steps | **P2** |
| **F3** | Docker detection in prereqs | XS | Developer experience | **P2** |
| **F4** | `npm run doctor` smoke test | S | Reliability | **P2** |
| **G4** | `prometheus-librefang-skills` marketplace package | XS | Discoverability of new path | **P2** |
| **G5** | LibreFang host-ABI reference doc | S | Skill author enablement | **P2** |
| **H1** | `ideation-mindmap` skill | M | Stage-zero onramp | **P3** |
| **A2** | First-class `version`/`license`/`metadata.tags` in validator | XS | Forward-compat | **P3** |
| **A4** | Per-skill license fields | XS | Forward-compat | **P3** |
| **D3** | Auto-generate `opencode.json` at install | S | OpenCode UX | **P3** |
| **C** (overall) | Fallback `SubagentStop` matcher | XS | Robustness | **P3** |
| **H2** | Private/public skill-hub story | L | Out of scope here | Backlog |

**Sizes**: XS (≤2h), S (≤1d), M (1–3d), L (>3d).

---

## 9. Verification Plan (post-implementation)

After P0 work lands:

1. `npm run validate` — green across all skills with new 200-char description cap.
2. `bash scripts/check-prerequisites.sh --install --build-tools` — exits 0 with `forge`, `pk`, `liter-llm`, `surreal-memory-server` in `$PATH`.
3. `/create-native-agent --target librefang-wasm test-agent` — produces a project that builds with `cargo build --target wasm32-wasip2 --release`.
4. `forge package-librefang ./test-agent` → `test-agent.lf-skill.zip` exists.
5. Spin up local librefang: `librefang start &` then `curl -X POST http://localhost:4545/skills/install --data-binary @test-agent.lf-skill.zip`. Returns 200 + skill manifest.
6. `curl http://localhost:4545/skills/test-agent` returns the installed manifest with `runtime.type = "wasm"`.
7. `/start-business-build "I want to track shipping-cost trends"` produces a complete chain through to a deployable WASM skill, end-to-end in under 10 minutes of human attention.

---

## Sources

- [Specification — Agent Skills (agentskills.io)](https://agentskills.io/specification)
- [Agent Skills | Claude API Docs](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/overview)
- [Agent Skills Specification (DeepWiki)](https://deepwiki.com/anthropics/skills/6.1-agent-skills-specification)
- [What Is the Agent Skills Open Standard? (2026)](https://www.agensi.io/learn/agent-skills-open-standard)
- [Create and distribute a plugin marketplace — Claude Code Docs](https://code.claude.com/docs/en/plugin-marketplaces)
- [anthropics/claude-plugins-official marketplace.json](https://github.com/anthropics/claude-plugins-official/blob/main/.claude-plugin/marketplace.json)
- [Plugins — OpenCode Docs](https://opencode.ai/docs/plugins/)
- [@opencode-ai/plugin (npm)](https://www.npmjs.com/package/@opencode-ai/plugin)
- [Measuring the Impact of Early-2025 AI on Experienced Open-Source Developer Productivity — METR](https://metr.org/blog/2025-07-10-early-2025-ai-experienced-os-dev-study/)
- [The AI Productivity Paradox Research Report — Faros AI](https://www.faros.ai/blog/ai-software-engineering)
- [State of Developer Ecosystem 2025 — JetBrains](https://blog.jetbrains.com/research/2025/10/state-of-developer-ecosystem-2025/)
- [AI | 2025 Stack Overflow Developer Survey](https://survey.stackoverflow.co/2025/ai/)
- [AI Coding Assistant Statistics & Trends 2026 — Second Talent](https://www.secondtalent.com/resources/ai-coding-assistant-statistics/)
- [Introducing Wassette — Microsoft](https://opensource.microsoft.com/blog/2025/08/06/introducing-wassette-webassembly-based-tools-for-ai-agents/)
- [Server-Side WASM: The Motherboard of Agentic AI](https://sriram-narasim.medium.com/server-side-wasm-the-motherboard-of-agentic-ai-27be7e86ae35)
- [karpathy/nanochat](https://github.com/karpathy/nanochat)
- [karpathy/rustbpe](https://github.com/karpathy/rustbpe)
- [ToJen/llm.rs (community Rust port of llm.c)](https://github.com/ToJen/llm.rs)
- [Karpathy on nanochat (X)](https://x.com/karpathy/status/1977755427569111362)
