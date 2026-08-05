# Reflection — kimi-desktop-extensibility

_2026-08-05. 5/5 changes. Goals: **2 MET, 1 PARTIAL, 1 MET-but-unobserved**._

## Goal verdicts

### Goal 1 — map every extension point, separating verified from marketing — **MET**

`parseManifest` — in
`/Applications/Kimi.app/Contents/Resources/resources/daimon-bundle/app/daimon/node_modules/@moonshot-ai/agent-core/dist/index.mjs`,
an installed-app artifact that is NOT in this repo and is version-pinned to
daimon 0.5.49 — returns the complete consumed field set:
`name version description keywords homepage license author skills sessionStart
mcpServers hooks commands interface`. Thirteen fields, read out of the artifact
that enforces them.

The "separating verified from marketing" clause earned its place. Two
documented-for-CLI fields turned out **not** to exist on daimon 0.5.49:

- `systemPrompt` / `systemPromptPath` — absent from the returned manifest, AND
  absent from `UNSUPPORTED_RUNTIME_FIELDS`, so declaring one produces **no
  diagnostic** and is silently ignored.
- `agents` — same absence.

And one field nobody knew about was found: **`commands`**, in no vendor package
and no documentation.

### Goal 2 — explicit adopt/reject rationale per extension point — **PARTIAL**

All seven live points now carry a verdict: E1 `skills` adopted, E0
`skillInstructions` adopted, E2 `mcpServers` adopted, E3 `sessionStart` rejected
for now, E4 `hooks` supported-unadopted, E8 `commands` supported-unowned, E9
`interface` adopted.

Two were fixed **at reflect**, both found by checking rather than assuming:
`interface` was in use with no decision ever recorded, and E2/E3 still carried
pre-execution language ("ADOPT NEXT", "ADOPT") contradicting what actually
happened.

**Why PARTIAL, not MET:** E8 `commands` is recorded as "SUPPORTED, UNOWNED" and
handed to the next phase. That is a deferral, not a verdict. The goal asks for a
rationale per extension point; six of seven have one, and the seventh has a
placeholder. Marking this MET would be exactly the failure this phase kept
catching elsewhere — recording a question as if recording it were answering it.
`commands` was discovered late (by `kde-003`, after planning), which explains the
gap but does not close it.

### Goal 3 — is UI/UX customization achievable — **MET (the answer is no)**

Kimi Desktop is a thin Electron shell: it embeds `apps/kimi-web`, ensures a
daemon, points Chromium at it. There is no `views`, `panels`, `theme`,
`renderer`, or `ui` key in any installed package or in the loader. (Counts across
stages differ because ours was added mid-phase: 12 vendor packages at assess, 13
once `prometheus-skill-pack` was installed.)
The only presentation surface is `interface` — how the plugin appears in a list.

**This was the operator's headline request, and the answer is that it cannot be
done through the supported API.** Recorded so it is not re-investigated.

### Goal 4 — reinstall-durable, no app-managed-state traps — **MET**

Re-verified at reflect, not carried from execute: package deleted, reinstalled,
145 skills and 3 `mcpServers` restored. `mcpServers` is generated from
`scripts/mcp-port-table.json` rather than hardcoded, so a machine with different
ports gets a correct manifest from one reinstall.

Nothing was written into app-managed state beyond the package directory: the
`hooks`/`systemPrompt` question was answered by reading the loader, so the probe
package `kde-003` specified was never created.

**Scope of this MET:** goal 4 is about DURABILITY — that an adopted integration
survives a reinstall — and that is directly tested and passing. It is not a claim
that the integrations FUNCTION; see Honest limits. A field that survives
reinstall and never connects would still satisfy goal 4 and fail the operator.
The two are independent, and only the first was in scope here.

## What actually shipped

| Change | Outcome |
|---|---|
| `kde-000` | `skillInstructions` decision recorded — closed a CRITICAL carried across three handoffs |
| `kde-003` | `hooks` SUPPORTED (array shape, seconds ≤600); `systemPrompt` silently ignored; found `commands` |
| `kde-001` | 3 MCP servers emitted; forge auth removed at source via new `--no-auth`. **Emitted ≠ confirmed:** `prometheus-knowledge` and `forge-rs` each answered a real MCP `initialize`; `surreal-memory` was only observed emitting an SSE `endpoint` event, never a completed handshake |
| `kde-002` | **Dropped** — no suitable `sessionStart` payload exists |
| `kde-005` | Per-skill cap (250), not a shared budget → no curation needed |

## Delta — what did not go to plan

**Two of five changes did not produce the artifact they were specified to
produce, and both were right.** `kde-002` shipped nothing; `kde-003` shipped a
verdict instead of a probe package. A phase where every change produces a diff
would have been the worse outcome here.

**Method shift mid-phase.** `kde-003`, `kde-001` and `kde-005` were all answered
by reading `agent-core/dist/index.mjs` rather than by black-box probing. That was
not the plan. It is stronger: a probe shows one path failing, the loader shows
the whole contract and cannot yield a false negative from a wrong guess — which
the `kde-003` spec had itself named as the probe's central weakness.

**Every blocking assumption in `kde-001` was wrong.** Loopback URLs, SSE support,
and auth expressibility were all treated as risks; `McpServerConfigSchema`
answered all three in the affirmative. The three vendor packages being remote
HTTPS was a **biased sample**, and the spec generalised from it.

**Adversarial review carried real weight.** Assess went BLOCK→PASS after a
CRITICAL that a second reviewer (MiniMax-M3) had passed. Spec took three rounds.
The `skillInstructions` CRITICAL had been recorded as a warning in three
consecutive handoffs — a warning with no owner is unowned, which is why
`kde-000` exists.

## Root causes worth keeping

1. **A residual/absent field is not a decision.** `interface` was in use and
   undecided; `skillInstructions` was flagged three times and never owned. Both
   needed a change to own them.
2. **Vendor packages are a biased sample.** Three remote-HTTPS servers looked
   like a rule. The schema was the ground truth.
3. **Parsing is not execution.** Stated repeatedly and still true — see below.
4. **A gate that has never been run is not a gate.** The goal-4 gate found a
   latent defect (stale release manifest after the temperature fix) that would
   have failed every install on every machine.

## Honest limits

**The central claim of this phase is unverified.** Kimi Desktop has never been
observed connecting to the three MCP servers, and no hook has been observed
firing. Everything shipped satisfies the schema the loader enforces — necessary,
not sufficient. A phase that ends "schema-valid" while the operator's question
was "does it work" is not finished, it is staged.

Also unproven: whether `commands` (E8) does anything, and whether the 89
truncated descriptions measurably degrade selection.

## Recommended next phase — `kimi-desktop-runtime-verification`

The one thing this phase could not do from the loader: **observe it running**.

1. Launch Kimi Desktop and confirm the three MCP servers connect and expose tools.
2. Declare one `hooks` entry and confirm the command spawns (parsing ≠ execution).
3. Decide `commands` (E8) — supported, unowned, possibly the equivalent of the
   147 slash commands shipped to Claude Code and Codex.
4. Front-load the 89 over-cap descriptions into their first 250 chars.

Items 1–2 close the phase's stated limit. Items 3–4 are the concrete work it
surfaced but deliberately left.
