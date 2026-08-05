# Assessment — kimi-desktop-extensibility

_Generated 2026-08-05. Scope: what Kimi Desktop actually lets a third party extend,
and which of those points `prometheus-skill-pack` should use._

Local evidence: Kimi.app with daimon release **0.5.49**, 13 installed plugin
packages (12 vendor + ours). Kimi Code CLI **0.29.1**. Public evidence: Kimi Code
CLI plugin docs and the DeepWiki mirrors of `moonshotai/kimi-code`.

## The finding that governs everything else

**Kimi Desktop is a thin Electron shell. It has no UI extension surface of its own.**

Per the DeepWiki page for `apps/kimi-desktop`, the app embeds the `apps/kimi-web`
assets, ensures a local daemon is running, and points a Chromium instance at it.
It "does not expose custom views, panels, themes, or renderers as extension
points" — it delegates all rendering to a Vue 3 web app and confines itself to
window management and daemon lifecycle (discovering the port via `daemon.lock`,
enforcing single-instance).

This is corroborated locally. The union of **every** manifest key across all 13
installed packages is:

```
$schema author description homepage interface keywords license
mcpServers name sessionStart skillInstructions skills version
```

and every `interface` sub-key is:

```
category developerName displayName hostKind iconUrl
longDescription mcpOverrides platforms shortDescription websiteURL
```

There is no `views`, `panels`, `theme`, `renderer`, `ui`, or `commands` key
anywhere — not in the vendor packages, and not in the runtime's pin file.

**Consequence: the UI/UX customization goal in the request is not achievable
through the supported plugin API.** The only fields that touch presentation are
`interface.displayName`, `iconUrl`, `category`, and the description strings —
i.e. how the plugin appears in a list, not how the app looks or behaves.

Anything further would mean modifying `/Applications/Kimi.app` internals or
injecting into the embedded web app. That is out of scope and should stay out:
it breaks on every app update, is unsignable, and is the same class of trap as
editing a plugin cache (CLAUDE.md already forbids the analogue).

## Extension points that DO exist

Ranked by whether the pack should adopt them.

### E1 — `skills` — ADOPTED, shipping

`"skills": "./skills/"` names a directory, so one package carries the whole pack.
Already implemented in commit `751ff48`: 145 skills installed and verified.

### E2 — `mcpServers` — **ADOPTED, shipping** (kde-001)

Used by 4 of 12 vendor packages (`github`, `cloudflare`, `supabase`, `kimi-cu`).
Reuses the standard MCP schema; stdio servers may reference a command on PATH or
a `./`-relative path inside the plugin root.

This is the single highest-value unclaimed point, because the pack already
operates 7 MCP servers (surreal-memory, pk-cherry, forge-mcp, surface-bridge,
sovereign-sync, prometheus-research, liter-llm). Declaring them here would give
Kimi Desktop the same tool surface the other 14 harnesses already have. Today
Kimi Desktop gets skills but **no tools** — the skills can describe workflows
they cannot execute.

`interface.mcpOverrides` additionally lets a user enable/disable individual
servers per plugin, which suits an opt-in posture for the heavier daemons.

### E3 — `sessionStart` — **REJECTED for now** (kde-002 dropped)

Shape is minimal — `{"skill": "<name>"}`; `readSessionStart` reads only `skill`,
so no arguments can be passed.

**Supported by the runtime, but dropped: no suitable payload exists.** It runs on
EVERY session, and all three candidates failed — `kbd-status` presumes a KBD
project, `learn-harness` needs a script not shipped inside it, and
`learn-about-system` asks the operator a question. Re-openable the moment a
small, argument-free, manifest-only orientation skill exists. See
`session-start-finding.md`.

### E4 — `hooks` — **SUPPORTED** (verdict from kde-003)

Documented for Kimi Code CLI plugins ("declare hook rules in its manifest that run
on lifecycle events … using the same fields as config.toml hooks: event, matcher,
command, timeout"). **Not present in any of the 12 installed vendor packages**,
so it is unproven on the desktop daimon path.

**Resolved by `kde-003`** via source inspection of the shipped loader
(`agent-core/dist/index.mjs`, `parseManifest`). `hooks` is parsed and returned.

Contract — **differs from Claude Code's shape**: a flat ARRAY of
`{event, matcher?, command, timeout?}`, `.strict()`, where `event` is one of
`PreToolUse PostToolUse SessionStart Stop SubagentStop UserPromptSubmit
Notification`. `timeout` is an integer **seconds** value capped at 600; the
pack's `hooks.json` uses milliseconds and would be rejected.

Caveat: parsing is not execution. See `probe-verdict.md`.

### E5 — `systemPrompt` / `systemPromptPath` — **NOT SUPPORTED** (verdict from kde-003)

**Resolved by `kde-003`: REJECT for this runtime.** Neither field appears in
`parseManifest`'s returned manifest object on daimon 0.5.49. The CLI docs that
described them do not reflect this build.

Worse than unsupported: neither is in `UNSUPPORTED_RUNTIME_FIELDS`, so declaring
one produces **no diagnostic** — it is silently ignored. That is precisely the
inertness failure mode this phase exists to avoid.

`skillInstructions` (E0, adopted) already covers routing guidance and IS
consumed, so nothing is lost.

### E6 — `agents` — DEFER

Documented ("a plugin can ship its own agents by declaring directories in the
manifest's `agents` field, or by placing an `agents/` directory at the plugin
root"). Absent from installed packages.

The pack has agents, but they are Claude-Code-shaped. Porting them is a larger
effort than E2/E3 and should follow, not precede, tool availability.

### E0 — `skillInstructions` — ADOPTED, already shipping (resolved)

Flagged as an unresolved gap in three consecutive handoffs (assess, analyze,
spec). Owned and resolved by change `kde-000-skillinstructions-decision`, so the
decision has a spec, a verification, and an archive record rather than living
only as a note here.

**Status: adopted and already emitted.** `install-kimi-desktop-plugin.sh` writes
a `skillInstructions` block naming the skill families (kbd-*, learn-*,
adversarial-review, language patterns, bdd-*) and instructing the agent to read a
SKILL.md in full before following it.

**Rationale for adopting rather than expanding it:** it is the routing hint that
tells the model which of 145 skills to reach for, and it costs one string. It is
also the reason E5 (`systemPrompt`) is held at CONSIDER rather than adopted —
the two compete for the same context budget, and `skillInstructions` is the
supported, already-working one. No further change is required.

### E8 — `commands` — **SUPPORTED, UNOWNED** (discovered by kde-003)

Not in any of the 12 vendor packages and in no documentation reviewed during
assess or analyze — found only by reading the loader. `parseManifest` returns
`commands: await readCommands(...)`, accepting a string or string[].

This may be the Kimi Desktop equivalent of the 147 slash commands the pack ships
to Claude Code and Codex. No change owns it. Plan should decide whether to
create one.

### E9 — `interface` — ADOPTED, shipping

Consumed by `parseManifest` via `readInterface`. The pack emits `displayName`,
`shortDescription`, `longDescription`, `developerName`, `websiteURL`, and
`category` — how the plugin presents in a list. `iconUrl` and `mcpOverrides` are
available and unused; neither is load-bearing.

Recorded for completeness: goal 2 asks for a verdict on **every** extension
point, and this one was being used without ever being decided — the same gap
that produced `kde-000`.

### E7 — Marketplace publication — OUT OF SCOPE for now

Distribution is a CDN-backed `plugins/marketplace.json`, with trust tiers
`official` / `curated` validated against `code.kimi.com` paths. **No submission
process is documented.** Local installation works and is sufficient; revisit only
if Moonshot publishes a submission path.

## Rejected: MiniMax Desktop

Recorded so it is not re-investigated. MiniMax Agent's support directory holds
only Electron state plus a UI/auth config; its `workingDirectory`
(`~/.minimax-agent/projects`) holds only projects; there is no `SKILL.md`, plugin
directory, or MCP config anywhere beneath it. `~/.minimax/skills` (178 skills)
belongs to **MiniMax Code**, the CLI, which the flat installer already serves.
Building a desktop integration there would ship dead files.

## Open questions for analyze/plan

1. ~~Does the desktop daimon honour `hooks` and `systemPrompt`?~~ **CLOSED by
   `kde-003`**: `hooks` supported (array shape, seconds timeout ≤600);
   `systemPrompt` not supported and silently ignored. See `probe-verdict.md`.
   New open question in its place: does a parsed hook actually SPAWN? Parsing is
   not execution.
2. **Do `~/.kimi/plugins` (CLI, per docs) and desktop `plugin-packages` share a
   loader?** Locally `~/.kimi/plugins` does not exist and our package is not
   visible to the CLI, so they appear separate — but the CLI is 0.29.1 while the
   daimon is 0.5.49, and the docs may describe a newer CLI than is installed.
3. ~~Is there a catalog budget?~~ **CLOSED by `kde-005`.** Kimi does NOT behave
   like Codex: the limit is a **per-skill** cap (`LISTING_DESC_MAX = 250`), not
   a shared budget, and no cap on skill *count* exists. Adding a skill costs the
   others nothing, so **no curation is needed** — the Codex
   `config/codex-catalog.txt` remedy solves a problem this runtime does not
   have.

   Measured separately, though: **89 of 145 descriptions (61%) exceed 250 chars**
   (median 278, max 662) and are truncated, losing the trailing trigger guidance
   the model selects on. `whenToUse` is emitted UNtruncated and is the escape
   hatch. Rewriting those descriptions is out of scope — see
   `catalog-budget-finding.md`.

## Where the manifest lives (corrected after adversarial review)

The first draft said "declare the pack's MCP servers in `kimi.plugin.json`"
without stating which file that is. Both reviewers flagged it; the judge raised
it as CRITICAL. Verified:

- The **only** `kimi.plugin.json` tracked in this repo is
  `tools/liter-llm/plugin/kimi.plugin.json` — a different component's plugin,
  not the pack's.
- The pack's manifest is **generated**, not stored: it is written at install
  time by `scripts/install-kimi-desktop-plugin.sh` (the Python heredoc around
  line 149) directly into the staging directory.

**Therefore every change below edits the GENERATOR, never a manifest file.**
Hand-editing the installed manifest under `plugin-packages/` would be overwritten
on the next install and is invisible to git — the same rule constraint C-01
already applies to `.codex-plugin/plugin.json`.

## Suggested change order

1. `change-kde-001` — emit `mcpServers` from `install-kimi-desktop-plugin.sh` (E2).
   **Blocked on a prerequisite the first draft missed:** the 7 MCP servers are
   Rust binaries in `~/.local/bin`, and the manifest requires a command on PATH
   or a `./`-relative path inside the plugin root. Whether `~/.local/bin` is on
   the daimon's PATH is unverified. Establish that first; if it is not, the
   options are absolute paths (if accepted) or shipping launcher shims.
2. `change-kde-002` — emit `sessionStart` (E3). Target is **`kbd-status`**,
   confirmed present in the 145 installed skills. The first draft named no skill
   and left an unverified dependency.
3. `change-kde-003` — probe package testing whether `hooks` **and**
   `systemPrompt` are honoured by the daimon. Widened from hooks-only: OQ-1
   covers both, and one throwaway package can answer both.
4. `change-kde-004` — depending on 003, wire the hook bundle or record it inert.
5. `change-kde-005` — measure catalog/description budget at 145 skills (OQ-3).

Changes 1–2 are additive to a package that already ships. Change 3 is a
throwaway experiment whose only output is a verdict.

### Reinstall-durability, per change (goal 4)

The critic correctly noted goal 4 was argued only for the rejected UI path and
E1. Explicitly, for each adopted point: all of E2/E3/E4/E5 are **manifest fields
emitted by the generator**, so they inherit E1's durability — the installer
rebuilds the package atomically on every run and is invoked from
`install-skills-flat.sh`. None of them writes to app-managed state outside the
package directory. E4 is the only one that could introduce an external artifact
(a hook command path); if adopted, that path must live inside the plugin root.

## Corrections made after adversarial review

Two independent reviewers ran against this assessment: judge `k3` (Moonshot,
verdict BLOCK) and critic `MiniMax-M3` (verdict PASS). Both are cross-vendor
relative to the producer (Claude), so neither is a self-grade.

| Finding | Severity | Resolution |
|---|---|---|
| Change order targeted an unlocated `kimi.plugin.json`; the only one in-repo belongs to liter-llm | CRITICAL | Verified and corrected — the manifest is generated; changes edit the generator |
| `sessionStart` named no concrete skill | WARNING | Verified `kbd-status` exists among the 145; named explicitly |
| MCP servers assumed launchable without checking PATH/plugin-root reachability | WARNING | Recorded as an explicit prerequisite blocking change-kde-001 |
| Goal-4 durability argued only for rejected path and E1 | WARNING | Added a per-change durability paragraph |
| `systemPrompt` in OQ-1 but absent from the probe | SUGGESTION | Probe widened to cover both |
| "the other 14 harnesses already have" is unsupported by the packet | SUGGESTION | Accepted; the claim is true of the pack's install targets but is not evidenced in the packet, so it is not load-bearing for any change |

## Verified healthy (no action)

- Package installs, is idempotent, and is restored by `install-skills-flat.sh`
- 145/145 skills carry a `SKILL.md`; manifest structurally matches the vendor
  `github` package apart from `mcpServers`/`sessionStart`, both addressed above
