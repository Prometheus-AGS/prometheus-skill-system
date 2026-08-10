# Harness delivery — what was actually run, and what the limits are

The ideation flow never renders its own prompts. It emits a `UiIntent` through
[`scripts/emit-ui-intent.sh`](../scripts/emit-ui-intent.sh), and `ui-surface`
resolves the tier. Tier logic lives in one place; a skill that prints its own
prompt works on its author's harness and silently degrades everywhere else.

## Verified by running it, not by asserting it

Every row below was executed on 2026-07-31. The five non-Claude rows are the ones
that matter — Tier 0 text is a floor, not evidence of harness delivery.

| Harness | Tier resolved | Mechanism | Result |
|---|---|---|---|
| `claude-code` | `tier1_structured` | inline structured prompt | prompt rendered; exit 0 |
| `codex` (non-Claude) | `tier1_structured` | file-pair handshake | **round trip completed in 2 s**; exit 0 |
| `zed` (non-Claude) | `tier1_structured` | file-pair handshake | **round trip completed in 2 s**; exit 0 |
| `opencode` (non-Claude) | `tier1_structured` | file-pair handshake | **round trip completed in 3 s**; exit 0 |
| `kimi` (non-Claude) | `tier1_structured` | file-pair handshake | **round trip completed in 2 s**; exit 0 |
| `cursor` (non-Claude) | `tier1_structured` | file-pair handshake | **round trip completed in 2 s**; exit 0 |
| forced `tier0_text` | `tier0_text` | plain text | completed in text; exit 0 |

Every non-Claude run was a genuine two-party handshake: an independent process,
blind to the flow, polled for `__ui_intent__.json`, read the title, and wrote
`__ui_response__.json`. The flow consumed that response and continued with the
selected value; both files were removed afterwards. Each was confirmed under
`bash -x` to reach `_render_tier1_file_pair` — `HARNESS=codex`, `HARNESS=zed`,
`HARNESS=opencode`, `HARNESS=kimi`, `HARNESS=cursor` — not a Tier 0 fallback that
happened to print something.

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

**`zed` is now Tier 1** (`change-msp-002`, 2026-07-31). It previously resolved to
Tier 0 for two independent reasons, both fixed:

1. `detect-surface-tier.sh` hardcoded `TIER="tier0_text"` for zed.
2. `render.sh` omitted `zed` from **both** Tier 1 dispatch lists — the direct one
   and the Tier 2 → Tier 1 fallback. Fixing only the first would have left zed
   degrading to text whenever surface-bridge was down.

Neither was a mechanism limit: the file-pair handshake is two files on disk and
asks nothing of the harness but reading one and writing the other. Verified by an
executed round trip with an independent blind responder, and confirmed under
`bash -x` that dispatch reached `HARNESS=zed → _render_tier1_file_pair`.

**All four file-pair harnesses have now been run** (`change-msp-003`,
2026-07-31). Nothing in this reference rests on "expected to behave the same"
any more: `codex`, `zed`, `opencode`, and `kimi` each completed a round trip
with an independent responder and each was confirmed under trace.

**`cursor` is now Tier 1** (`change-uhe-001`, 2026-07-31), and it had the
**identical two causes `zed` did** — which is why the lesson generalises rather
than being a one-off:

1. `detect-surface-tier.sh:76-83` hardcoded `TIER="tier0_text"`.
2. `render.sh` omitted `cursor` from **both** Tier 1 dispatch lists — the direct
   one and the Tier 2 → Tier 1 fallback.

Fixing only the visible one would have left it degrading to text whenever
surface-bridge was down. Verified by an executed round trip with an independent
blind responder, confirmed under `bash -x`.

**Every harness `_detect_harness` recognises now reaches Tier 1.** There is no
remaining harness held at the text floor by omission rather than by mechanism.

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
