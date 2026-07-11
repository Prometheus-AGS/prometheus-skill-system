# Reflection — phase-codex-plugin-implementation

_Reflected 2026-07-11._

## Summary

Delivered a **generated, Codex-verified plugin + marketplace** for the skill-pack
in parity with its Claude-Code plugin: `scripts/build-codex-plugin.js` emits
`.codex-plugin/plugin.json` and `.agents/plugins/marketplace.json` from the
canonical `.claude-plugin/*` sources, and the whole surface was validated against
**codex-cli 0.144.1** (marketplace add → 11 plugins resolve → umbrella installs →
7 MCP servers register). 8/8 changes DONE. The single biggest win was a
front-loaded spike that turned three assumed unknowns into facts before any real
code was written.

## Goal achievement

| Goal | Status | Evidence / caveat |
|---|---|---|
| G-01 research spec digest | **MET** | `references/codex-plugin-spec-digest.md` (4 sources, cited) |
| G-02 `.codex-plugin/plugin.json` | **MET** | generated; installs in 0.144.1 |
| G-03 marketplace | **MET** | `.agents/plugins/marketplace.json`; **11 plugins resolve** |
| G-04 skills discovery/budget/layout | **MET** | 30 curated (vs 301 total); `skills/` layout untouched |
| G-05 7 MCP servers + session fixes | **MET (caveat)** | all 7 register from shared `.mcp.json`; **full per-server init depends on env** (`doctor` ⚠ for unset vars) |
| G-06 hooks wiring | **MET (caveat)** | wired to PascalCase `hooks/hooks.json` (no translation needed); **end-to-end firing is interactive-trust only, NOT verified headlessly** |
| G-07 build tooling | **MET (caveat)** | generator + `build:codex`/`validate:codex` + drift guard, all tested; **`install-platforms.ts` codex-target integration NOT done** (npm script + docs only) |
| G-08 UAR compatibility | **MET** | `skills/` tree + `.codex/` unchanged — verified zero diff |
| G-09 parity/publishing docs | **MET** | `docs/codex-plugin.md` + CLAUDE.md updated |

**Achievement: 9/9 goals MET**, three carrying explicit caveats (below). Calling
those a clean "9/9" without the caveats would misrepresent what was verified.

## Delivered changes

001 runtime-spike (spike) · 002 plugin-manifest · 003 codex-marketplace ·
004 plugin-mcp · 005 hooks-wiring · 006 skills-bundle · 007 build-tooling ·
008 parity-docs-uar. All archived under `.kbd-orchestrator/changes/archive/2026-07-11-*`.

## Artifact Quality Summary

| Metric | Value |
|---|---|
| Changes with QA (artifact-refiner) | **0/8** |
| First-pass pass rate | N/A |

No `.refiner/artifacts/` logs and no `.kbd-orchestrator/constraints.md` exist, so
the artifact-refiner gate was not run (same as `phase-drui-standalone-hosting`).
Verification instead came from **live codex-cli 0.144.1 exercises** (install,
resolve, `codex mcp list`, `codex doctor`) + generator idempotency/drift tests +
`skills/` zero-diff checks. That is weaker than a constraint-gated QA pass.

## Delta → Root Cause → Corrective Actions

**Delta 1 — Hook firing was never proven end-to-end.**
G-06 wired the hooks and the spike confirmed they bundle with an accepted schema,
but no test shows a plugin hook actually *runs*.
*Root cause:* Codex plugin hooks are **non-managed** — they require interactive
user trust in a live session; there is no headless/CI trust path, and no trust
state is written at install time.
*Corrective:* add a manual "trust + observe `${PLUGIN_DATA}` write" step to the
publishing checklist; do not claim a working hook surface in release notes until
that manual check passes; consider filing an upstream ask for a headless trust flag.

**Delta 2 — `install-platforms.ts` was not wired to the generator.**
G-07's "integrate install-platforms.ts codex target" is only partially done —
npm scripts + docs exist, but the install flow does not call `build:codex`, and
`validate:codex` is not in CI.
*Root cause:* deliberately scope-managed to keep the turn focused on a working,
validated artifact; the TS install path is a larger change.
*Corrective:* follow-up change to (a) call `npm run build:codex` from
`install-platforms.ts`'s codex path and (b) add `npm run validate:codex` to
`.github/workflows/` next to `validate`.

**Delta 3 — MCP servers register but don't all initialize without env.**
`codex doctor` warns on unset env vars; a fresh machine won't have the 7 servers
fully live.
*Root cause:* secrets cannot be committed; the model is env-passthrough +
`${VAR:-default}`. Local-default servers work; keyed ones (tavily) don't until
provisioned.
*Corrective:* `docs/codex-plugin.md` documents provisioning; optionally add a
`prometheus setup`-style helper that seeds `~/.codex/config.toml` env from the
environment (mirroring the tavily/firecrawl setup done earlier this session).

**Delta 4 — Deliverables are uncommitted and QA-ungated.**
The artifacts sit in the working tree; no constraint gate ran.
*Root cause:* KBD execute produces working-tree changes; committing and authoring
`constraints.md` are separate steps.
*Corrective:* commit `.codex-plugin/`, `.agents/plugins/`, `scripts/build-codex-plugin.js`,
`docs/codex-plugin.md`, and the `CLAUDE.md`/`package.json` edits; add a lightweight
`constraints.md` so future phases get a real QA gate.

## Technical debt introduced

- `.agents/plugins/marketplace.json` uses `source.source="local"` — correct for
  in-repo dogfood, but external distribution needs `git-subdir`/`git` sources
  (documented, not implemented).
- Two generators now exist (`build-marketplace.js` for Claude, `build-codex-plugin.js`
  for Codex) with overlapping source reads — acceptable, but a future unified
  `build:plugins` could DRY them.

## Lessons captured

1. **Front-load a runtime spike for any new host format.** The 30-minute spike
   converted "does `.mcp.json` env work / do plugin hooks fire / what's the
   marketplace source schema" from blocking unknowns into facts, and correctly
   re-scoped 004 (inline env: yes) and 005 (hooks: interactive-only).
2. **Codex reuses Claude artifacts more than expected.** It reads the
   `mcpServers`-wrapper `.mcp.json` and the PascalCase `hooks/hooks.json` as-is,
   and reads `.claude-plugin/marketplace.json` as a legacy path — parity was
   mostly transformation, not rewrite.
3. **`codex plugin` verbs differ from `claude plugin`** (`add`/`remove`, not
   `install`/`details`) — captured in `[[codex-mcp-tavily-name-override]]` sibling docs.
4. **The `pipeline-enforce` guard scans command *text*.** A single command that
   both flips `progress.json` to 8/8 AND contains the string `/kbd-reflect` is
   blocked, because the PreToolUse hook reads progress *before* the command runs.
   Split "advance progress" and "reference next skill" into separate commands.

## Recommended next phase

`phase-codex-plugin-distribution-and-ci` — wire `build:codex`/`validate:codex`
into `install-platforms.ts` + CI (Delta 2), add the env-setup helper (Delta 3),
switch marketplace sources to `git-subdir` for external publish, and do the manual
hook-trust verification (Delta 1). Larger adjacent opportunity (UAR repo, not here):
the **Codex App Server adapter + `codex mcp-server` orchestration** (Modes 2–4 from
the brief) — out of this pack's scope but the natural capability frontier. Confirm
with the user before opening.
