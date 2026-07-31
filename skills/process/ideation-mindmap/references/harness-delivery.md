# Harness delivery — what was actually run, and what the limits are

The ideation flow never renders its own prompts. It emits a `UiIntent` through
[`scripts/emit-ui-intent.sh`](../scripts/emit-ui-intent.sh), and `ui-surface`
resolves the tier. Tier logic lives in one place; a skill that prints its own
prompt works on its author's harness and silently degrades everywhere else.

## Verified by running it, not by asserting it

Both rows below were executed on 2026-07-31. The Codex row is the one that
matters — Tier 0 text is a floor, not evidence of harness delivery.

| Harness | Tier resolved | Mechanism | Result |
|---|---|---|---|
| `claude-code` | `tier1_structured` | inline structured prompt | prompt rendered; exit 0 |
| `codex` (non-Claude) | `tier1_structured` | file-pair handshake | **round trip completed in 2 s**; exit 0 |
| forced `tier0_text` | `tier0_text` | plain text | completed in text; exit 0 |

The Codex run was a genuine two-party handshake: an independent process, blind
to the flow, polled for `__ui_intent__.json`, read the title, and wrote
`__ui_response__.json`. The flow consumed that response and continued with the
selected value. Both files were removed afterwards. Confirmed under `bash -x`
that dispatch reached `HARNESS=codex → _render_tier1_file_pair` — not a Tier 0
fallback that happened to print something.

## Stated limits

**Tier 1 outside Claude Code requires the harness to poll.** The file-pair
handshake writes `~/.prometheus/learn/ui/__ui_intent__.json` and waits up to 30 s
for `__ui_response__.json`. Nothing in the mechanism can make a harness look at
that directory. A harness that does not poll produces a **timeout, not a
response** — and the flow reports it as such:

```
[emit-ui-intent] NO RESPONSE: the harness did not answer within the timeout.
[emit-ui-intent]   This is a stated limit, not delivery. Fall back to Tier 0 text.
```

`emit-ui-intent.sh` exits **3** on that path. This distinction is deliberate: a
caller that treated a timeout as a response could not tell a working round trip
from a silent fallback, and "some text appeared" would read as delivery. Verified:
with no responder running, the intent file was written and the flow exited 3
after 30 s.

**`zed` resolves to Tier 0, not Tier 1.** `render.sh` detects `zed` in
`_detect_harness` but its Tier 1 dispatch routes only `opencode|codex|kimi` to
the file-pair branch, so `zed` falls through to Tier 0. That is a working floor,
not a defect in this change — but it is not Tier 1 delivery, and should not be
described as such.

**Only `codex` was exercised.** `opencode` and `kimi` share the identical code
path and are expected to behave the same, but they were not run. Treat them as
unverified until someone runs them.

## Calling it

```bash
bash skills/process/ideation-mindmap/scripts/emit-ui-intent.sh \
  --title "Which idea to build?" \
  --body  "Three survived scoring." \
  --option "Standup generator" \
  --option "PR summariser"
```

Exit codes: `0` a response was obtained · `1` usage or environment error · `3` no response
within the timeout (a stated limit — degrade to Tier 0 rather than claiming
delivery).
