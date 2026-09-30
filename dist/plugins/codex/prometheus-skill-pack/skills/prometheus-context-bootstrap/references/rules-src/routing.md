# Skill routing

Layer 2 reference. The compact `trigger → skill` view in `CLAUDE.md` §F is generated from the tables below;
this file adds install commands and notes. **Route by name.** A skill invoked explicitly fires
deterministically; a skill left to self-activate is a coin flip, and a large installed set exceeds the
session-start description budget so later descriptions are silently dropped.

Rules for the library: one skill per role per platform; curate, don't accumulate; read a third-party
`SKILL.md` before it lands in a repo; **never install without the owner's confirmation** — an install
downloads and runs third-party code. If a skill named here is absent, say so, give the install command
from this file, and fall back to the Layer 1 rules. Never narrate what it "would have done" (F-2).

Set "Status" to `present` or `**absent**` for this machine; rows marked absent are flagged in `CLAUDE.md`.
Delete the rows for stacks this project does not use.

## Process
| When | Invoke | Status | Install / notes |
|---|---|---|---|
| planning or subagent-driven work | `superpowers` | check | `npx skills add obra/superpowers` |
| before a PR: interrogate the change | `grill-me` | check | `npx skills add mattpocock/skills@grill-me` |
| phase lifecycle or position | `kbd-status`, `kbd-assess`, `kbd-analyze`, `kbd-plan`, `kbd-execute`, `kbd-reflect` | check | Prometheus skill pack |
| phase completion, before delivery, before a lesson becomes a rule | `adversarial-review` | check | Prometheus skill pack; say which judge ran |
| any reflection or self-assessment | `sycophancy-correction` | check | Prometheus skill pack; no external equivalent |

## Design and UI
| When | Invoke | Status | Install / notes |
|---|---|---|---|
| new UI surface, anything "make it look good" | `impeccable` | check | `npx impeccable install`; supersedes `frontend-design` — do not install both |
| palette, type pairing, style for a stated audience | `ui-ux-pro-max` | check | `npx skills add nextlevelbuilder/ui-ux-pro-max-skill`; failed Agent Trust Hub — read before use in a client repo |
| UI review before delivery: a11y, focus, touch targets, reduced motion | `web-design-guidelines` | check | `npx skills add vercel-labs/agent-skills@web-design-guidelines` |
| animation, transitions, component feel | `emilkowalski/skill` | check | `npx skills add emilkowalski/skill` |

## React, Next.js and TypeScript
| When | Invoke | Status | Install / notes |
|---|---|---|---|
| any React or Next.js implementation | `react-best-practices` | check | `npx skills add vercel-labs/agent-skills@react-best-practices` |
| component API, composition, prop design | `composition-patterns` | check | `npx skills add vercel-labs/agent-skills@composition-patterns` |
| Next.js routing, RSC, caching | `next-best-practices` | check | `npx skills add vercel-labs/agent-skills@next-best-practices` |
| adding or changing shadcn components | `shadcn` | check | `npx skills add shadcn/ui@shadcn` |
| Tailwind v4 | `tailwindcss` | check | `npx skills add hairyf/skills@tailwindcss` |
| browser end-to-end | `webapp-testing` | check | `npx skills add anthropics/skills@webapp-testing` |

## Flutter and Dart
| When | Invoke | Status | Install / notes |
|---|---|---|---|
| any Flutter implementation | `flutter/skills` | check | `npx skills add flutter/skills --skill '*' --agent universal`; omit its `flutter-apply-architecture-best-practices` — the house variant replaces it |
| Dart tests, deps, analyzer fixes | `dart-lang/skills` | check | `npx skills add dart-lang/skills --skill '*' --agent universal` |

## Rust, WASM and Tauri
| When | Invoke | Status | Install / notes |
|---|---|---|---|
| any crate work and Cargo timing | `prometheus-rust-workspace` | present | house router; loads specialized skills only when relevant |
| general Rust implementation and review | `rust-best-practices` | check | `npx skills add https://github.com/apollographql/skills --skill rust-best-practices` |
| Tokio, async I/O, concurrency, cancellation | `rust-async-patterns` | check | `npx skills add https://github.com/wshobson/agents --skill rust-async-patterns` |
| Rust MCP servers and transports | `rust-mcp-server-generator` | check | `npx skills add https://github.com/github/awesome-copilot --skill rust-mcp-server-generator` |
| post-implementation review of Rust | `prometheus-rust-auditor` | check | Prometheus skill pack |
| Tauri 2 | `tauri-v2` | check | house skill; no community skill with traction |

## Building tools
| When | Invoke | Status | Install / notes |
|---|---|---|---|
| writing or fixing a skill | `skill-creator` | check | `npx skills add anthropics/skills@skill-creator` |
| building an MCP server | `mcp-builder` | check | `npx skills add anthropics/skills@mcp-builder` |
| library documentation | `context7` | check | `npx skills add intellectronica/agent-skills@context7` |
| no row matches | `find-skills` | check | `npx skills add vercel-labs/skills@find-skills` |

## Discovery protocol — when no row matches and the task is non-trivial
1. Invoke `find-skills` with the platform and the function as keywords.
2. Report the top result, its install base and its audit status.
3. Propose the install command. Do not install without confirmation.
4. If nothing credible exists, proceed on the Layer 1 rules and log the gap to `.prometheus/gotchas.md` as a
   candidate house skill.

Search order: skills.sh (install telemetry is a real usage signal) → agentskills.io (the standard;
vendor-official repos link from here) → agenticskills.io (curated, audit notes) → GitHub.

## UI and project teams
| When | Invoke | Status | Install / notes |
|---|---|---|---|
| rendered UI or UX code, tokens, motion or copy | `prometheus-ui-ux` | present | shared protocol; project .agents/UI_UX_PROTOCOL.md overrides bundled default |
| completed UI phase review | `prometheus-ui-review` | present | independent read-only context, no taste, respect user-only restrictions |
| code work in a project with an existing team | `agent-team-creator` | present | select relevant roles from .agent-team/project-routing.json and real manifests; install-project adopts sole team or preserves explicit choice |
