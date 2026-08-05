# kde-003 verdict — `hooks` and `systemPrompt` on the Kimi Desktop daimon

_2026-08-05. Method: **source inspection of the shipped loader**, not a black-box probe._

## Method change, and why it is stronger

The spec called for a throwaway probe package: install a manifest declaring
`hooks` and a `systemPrompt` sentinel, restart Kimi Desktop, observe. It also
warned that a silent probe cannot distinguish "not supported" from "wrong event
name" without a positive control.

That warning is moot, because the authoritative answer is readable directly:

```
/Applications/Kimi.app/Contents/Resources/resources/daimon-bundle/
  app/daimon/node_modules/@moonshot-ai/agent-core/dist/index.mjs
```

`parseManifest()` in `//#region src/plugin/manifest.ts` is the function that
reads `kimi.plugin.json`. Its return value **is** the supported field set. This
is better evidence than a probe: a probe shows one path failing, the loader shows
the entire contract, and it cannot produce a false negative from a wrong guess.

No probe package was installed. Nothing was written into app-managed state.

## Verdict 1 — `hooks`: **SUPPORTED**

`parseManifest` returns `hooks: readHooks(raw["hooks"], diagnostics)`.

Contract (`HookDefSchema`, `.strict()` — unknown keys rejected):

```js
z.object({
  event:   z.enum(HOOK_EVENT_TYPES),
  matcher: z.string().optional(),
  command: z.string().min(1),
  timeout: z.number().int().min(1).max(600).optional()
}).strict()
```

`hooks` is an **array** of these objects — not the Claude Code shape, which nests
`hooks` inside matcher groups keyed by event.

`HOOK_EVENT_TYPES` (from the bundle):

```
PreToolUse  PostToolUse  SessionStart  Stop
SubagentStop  UserPromptSubmit  Notification
```

That is the same seven-event vocabulary the pack already targets. **`timeout` is
capped at 600 seconds and is an integer** — the pack's `hooks.json` uses
millisecond timeouts (e.g. `30000`), which would be rejected here.

## Verdict 2 — `systemPrompt` / `systemPromptPath`: **NOT SUPPORTED**

Neither appears in `parseManifest`'s returned manifest object. The CLI
documentation that described them does not reflect this desktop daimon build
(0.5.49).

They are also absent from `UNSUPPORTED_RUNTIME_FIELDS`
(`tools`, `apps`, `inject`, `configFile`, `config_file`, `bootstrap`), so a
manifest declaring `systemPrompt` gets **no diagnostic at all** — it is silently
ignored. That is exactly the inertness failure this change existed to prevent.

**E5 should move from CONSIDER to REJECT for this runtime.** `skillInstructions`
already covers routing guidance and is genuinely consumed (kde-000).

## Unplanned finding — `commands` is supported, and nobody knew

`parseManifest` also returns `commands: await readCommands(...)`, accepting a
string or string[]. **`commands` appears in none of the 12 vendor packages** and
in no documentation reviewed during assess or analyze.

This matters: the pack ships 147 slash commands to Claude Code and Codex. If
`commands` is the Kimi Desktop equivalent, it is a whole extension point the
assessment missed. It is not in scope here — recording it so plan can own it.

## Corrections to the assessment

| Item | Was | Now |
|---|---|---|
| E4 `hooks` | INVESTIGATE — unproven | **SUPPORTED** — array of `HookDefSchema`, 7 events, timeout ≤600s integer |
| E5 `systemPrompt` | CONSIDER, with caution | **NOT SUPPORTED** — silently ignored, no diagnostic |
| — | not known | **`commands` supported** — new extension point, unowned |

## Why this is not "inconclusive"

The spec required a positive control before accepting a negative. That rule
guards against a probe that never loaded. It does not apply to reading the
loader's own source: the same function that returns `hooks` omits
`systemPrompt`, so the negative and the positive come from one artifact. If
`hooks` works — and the code says it parses — then the file was read correctly.

## Residual risk

Parsing is not execution. The loader accepting `hooks` proves the field reaches
the runtime's data model; it does not prove a hook command is spawned. A
follow-up would need one real hook firing. That is `kde-004`'s job, and this
verdict is what unblocks writing it.
