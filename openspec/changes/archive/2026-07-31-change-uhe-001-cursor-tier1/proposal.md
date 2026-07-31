# Route cursor to Tier 1, or record why not

**Change:** `change-uhe-001-cursor-tier1`
**Phase:** uar-host-execution
**Goal:** S6

## Why

See `.kbd-orchestrator/phases/uar-host-execution/plan.md` for full rationale,
acceptance criteria, and the two-round adversarial review record.

## Outcome: ROUTED AND VERIFIED

The acceptance criteria offered two mutually exclusive outcomes. **The first was
taken**, so task 5 (the not-routed branch) is closed as **not applicable** — no
diagnostic was needed because `cursor` does reach Tier 1.

Evidence:

| Check | Result |
|---|---|
| Round trip under `cursor`, independent blind responder | **completed in 2 s**, exit 0, both files cleaned |
| Flow continued with the returned value | `Option A` from `{"selected":"Option A","source":"cursor-file-pair"}` |
| `bash -x` dispatch | `HARNESS=cursor` → `_render_tier1_file_pair` — not a Tier 0 fallback |

**Two causes, matching `zed` exactly** — which is why this generalises:

1. `detect-surface-tier.sh:76-83` hardcoded `TIER="tier0_text"`.
2. `render.sh` omitted `cursor` from **both** Tier 1 dispatch lists (direct and
   the Tier 2 → Tier 1 fallback). Fixing only the first would have left it
   degrading to text whenever surface-bridge was down.

**Every harness `_detect_harness` recognises now reaches Tier 1** —
`claude-code`, `opencode`, `codex`, `kimi`, `zed`, `cursor`. Only `unknown`
falls to Tier 0, which is correct. Verified by enumerating both the detector's
return values and the dispatch lists, not by assertion.
