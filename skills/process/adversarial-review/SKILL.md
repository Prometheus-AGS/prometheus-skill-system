---
license: MIT
name: adversarial-review
version: '1.0.0'
description: >
  Isolated, cross-model adversarial review of KBD artifacts and change diffs.
  Dispatches a fresh-context LLM judge over an OpenAI-compatible REST gateway
  (openai-proxy or `liter-llm api`) with an explicit
  mandate to find problems — the model that produced an artifact or change is
  never the model that reviews it. Runs as a pipeline stage inside
  kbd-assess/analyze/plan (artifact mode) and kbd-execute's per-change QA gate
  (diff mode). Findings are severity-bucketed (CRITICAL/WARNING/SUGGESTION)
  and the judge's own report is screened by sycophancy-correction before it
  is surfaced.
authors:
  - 'Prometheus AGS'
model_routing:
  policy_source: ".kbd-orchestrator/project.json → model_policy"
  phases:
    adv-review-preflight: small
    adv-review-packet: small
    adv-review-judge: frontier
  routing_reference: "references/isolation-and-routing.md"
triggers:
  keywords:
    - adversarial review
    - adversarial-review
    - cross-model review
    - llm as judge
    - review this diff adversarially
    - vet this plan
    - vet this assessment
  semantic: >
    Run an isolated find-problems review of a change diff against its spec,
    or of a KBD assessment/analysis/plan artifact against phase goals, using
    a different model than the one that produced the work.
metadata:
  tags: [process, review, quality, llm-as-judge, kbd]
---

# /adversarial-review

Run an **isolated, cross-model, mandate-to-find-problems review** of either a
change diff (post-implementation) or a KBD planning artifact
(pre-implementation).

## Progress Signals (MANDATORY)

Before building the review packet, emit:

```text
Starting adversarial-review — <mode> <target>
```

After the normalized findings are written, emit:

```text
Completed adversarial-review — <verdict> (<critical>/<warning>/<suggestion>)
```

Use the target and counts from the packet/findings files. Never guess them.

This is a different job from the existing gates:

- `refine-validate` is a **deterministic checklist** — it catches what was
  thought of in advance. Adversarial review hunts for what was **not**.
- `sycophancy-correction` corrects **existing text** for excess agreeableness.
  Adversarial review **generates** a finding set from scratch — and then has
  its own report screened by sycophancy-correction (see Anti-Theater Gate).
- `zeespec-interrogator` and the Darwin gates operate **pre-spec**. This skill
  operates on concrete artifacts and diffs.

## Isolation contract

The judge is a **fresh-context API call** to an OpenAI-compatible
`/v1/chat/completions` gateway. It
receives only a review packet — diff or artifact, acceptance criteria or
goals, file tree, constraints — and **never** the producing session's chat
history. Isolation is structural, not honor-system: the session reads only
the normalized findings JSON (the same pattern pmpo-evolver uses for its
isolated collection subprocesses).

**The producer never grades itself.** The packet records `producer_model`;
dispatch falls back from the `judge` role to the `critic` role when they would
match, and emits a `JUDGE_MODEL_COLLISION` warning when no alternative differs
(see `references/isolation-and-routing.md`). A packet with
`producer_model: unknown` makes that comparison pass trivially, so it is
recorded as `cross_model_check: unverified-producer-unknown` rather than
passed off as a clean cross-model review.

## Modes

### `--mode diff` — post-implementation (kbd-execute QA gate)

Reviews a completed change's diff against its acceptance criteria. Runs
**after** `refine-validate` passes (cheap deterministic checklist first,
expensive judgment second) and **before** archive.

Packet contents: change diff, acceptance criteria (`tasks.md` / OpenSpec spec
/ `verification.md`), repo file tree, blocking constraints from
`.kbd-orchestrator/constraints.md`, `producer_model`.

### `--mode artifact` — pre-implementation (assess / analyze / plan)

Vets a stage artifact before the stage hands off, so downstream stages run
against reviewed inputs:

| Stage | Artifact(s) | Mandate focus |
|---|---|---|
| `assess` | `assessment.md` | missed gaps; claims unsupported by the codebase |
| `analyze` | `analysis.md`, `library-candidates.json` | build-vs-adopt blind spots; uninspected candidates; stale landscape |
| `plan` | `plan.md` | wrong ordering; missing dependencies; untestable or ambiguous change criteria |

Packet contents: the artifact(s), phase `goals.md`, prior-stage handoff
summaries, constraints, `producer_model`.

## Workflow

```
1. preflight   scripts/preflight-models.sh          [MODEL_ROUTING] class=small
2. packet      scripts/build-review-packet.sh       [MODEL_ROUTING] class=small
3. judge       scripts/dispatch-judge.sh            [MODEL_ROUTING] class=frontier
4. gate        scripts/check-findings-sycophancy.sh (anti-theater screen)
5. surface     findings JSON → caller gate semantics
```

Concretely:

```bash
SKILL_DIR="${CLAUDE_PLUGIN_ROOT}/skills/process/adversarial-review"

# 0. Load the gateway credential and declare the producer.
#    KBD_PRODUCER_MODEL is REQUIRED for the guarantee this skill exists to make.
#    The judge's collision check compares candidate != producer, so an unknown
#    producer makes it pass trivially — which is exactly what happened to all 8
#    historical reviews. Set it to the model running THIS session.
set -a; . ~/.prometheus/kbd/secrets.env 2>/dev/null || true; set +a
export KBD_PRODUCER_MODEL="${KBD_PRODUCER_MODEL:-claude-opus-5}"

# 1. Preflight (cached 24h at .kbd-orchestrator/model-preflight.json)
#    Reports the gateway, the model per role, and WHICH config layer supplied it.
bash "$SKILL_DIR/scripts/preflight-models.sh"

# 2. Build the packet
bash "$SKILL_DIR/scripts/build-review-packet.sh" \
  --mode diff --phase "$PHASE" --target "$CHANGE_ID" \
  --out ".kbd-orchestrator/phases/$PHASE/review/$CHANGE_ID/packet.json"

# 3. Dispatch the judge
bash "$SKILL_DIR/scripts/dispatch-judge.sh" \
  --mode diff \
  --packet ".kbd-orchestrator/phases/$PHASE/review/$CHANGE_ID/packet.json" \
  --out    ".kbd-orchestrator/phases/$PHASE/review/$CHANGE_ID/findings.json"

# 4. Anti-theater gate (exit 2 = rejected: re-dispatch once with feedback)
bash "$SKILL_DIR/scripts/check-findings-sycophancy.sh" \
  --findings ".kbd-orchestrator/phases/$PHASE/review/$CHANGE_ID/findings.json" \
  --counter-key "adv-review-$CHANGE_ID"
```

Artifact mode replaces `--target "$CHANGE_ID"` with `--target assess|analyze|plan`.

## Dispatch fallback chain

Per the liter-llm-bridge and pmpo-evolver conventions — warn, never silently
degrade, never block the pipeline:

1. **REST gateway** — an OpenAI-compatible `POST /v1/chat/completions` at the
   resolved gateway (openai-proxy on `:8181`, or `liter-llm api`). Full isolation,
   true cross-model. `dispatch-judge.sh` exit 0, and the findings record
   `isolation_mode: rest-gateway:<url>` plus `cross_model_check`.

   There is **no `liter-llm complete`** — the binary ships only `api` and `mcp`
   subcommands (it is a proxy *server*). Earlier revisions of this doc and of
   `dispatch-judge.sh` called that non-existent subcommand; because the guard only
   checked that the *binary* existed, the failure surfaced as "liter-llm
   unavailable" rather than as the CLI-contract mismatch it was. Speak REST.
2. **Harness-native fresh-context subagent** — when no gateway is reachable
   (`dispatch-judge.sh` exit 3), the calling session dispatches a subagent
   (Agent tool / equivalent) whose prompt is exactly: the mode's mandate file
   + the packet JSON. Nothing else. Findings are logged with
   `"isolation_mode": "harness-native"` — a weaker guarantee (same model
   family), stated, not hidden.
3. **Skip with warning** (exit 4) — no judge available at all. Record
   `adversarial_review: SKIPPED (<reason>)` as a canonical KBD decision; never
   edit the phase progress projection.
   Never fail the phase because review infrastructure is missing.

## Output contract

`findings.json` (schema: `assets/schemas/findings.schema.json`):

```json
{
  "mode": "diff",
  "verdict": "BLOCK",
  "judge_model": "openai/gpt-4o",
  "isolation_mode": "rest-gateway:http://localhost:8181/v1",
  "producer_model": "claude-opus-5",
  "cross_model_check": "verified-distinct",
  "findings": [
    {
      "severity": "CRITICAL",
      "file": "src/auth/session.rs",
      "line": 142,
      "claim": "Token expiry is checked before refresh, allowing a replay window",
      "evidence": "diff hunk @@ -138,+142 removes the pre-refresh revocation check required by AC-3",
      "suggested_fix": "Re-check revocation after refresh, before issuing the new token"
    }
  ]
}
```

Severity semantics (matches the OpenSpec reporting convention):

| Severity | Diff mode | Artifact mode |
|---|---|---|
| `CRITICAL` | record a KBD blocker and set certification `blocked`; fix, then re-run refine-validate **and** adversarial-review | revise artifact and re-vet (max 2 rounds, then accept with an "Unresolved review findings" section appended so the next stage sees them) |
| `WARNING` | logged in the change's review dir; proceed to archive | appended to the stage handoff summary |
| `SUGGESTION` | informational | informational |

`verdict` is `BLOCK` iff at least one CRITICAL finding exists.

## Anti-theater gate

The judge's findings report is itself screened through sycophancy-correction
(`scripts/check-findings-sycophancy.sh`, reusing `shared/scripts/lib/sycophancy.sh`):
a report scoring ≥ 0.4 or matching high/critical sycophancy patterns —
e.g. zero findings on a large multi-file diff wrapped in hedged praise — is
**rejected**, and the judge is re-dispatched once with the rejection feedback
appended to its mandate (`dispatch-judge.sh --feedback <file>`). A
2-rejection soft cap accepts the third report with a logged warning,
mirroring the reflector gate. When the sycophancy binary is absent the gate
degrades gracefully (exit 0, warning) — it never blocks the chain.

## Skip rules

- Diff mode inherits the QA gate heuristics: skip when the change touches
  fewer than 3 files or is documentation-only, or when the caller passes
  `--skip-adversarial-review`.
- Artifact mode has **no** size heuristic — planning artifacts are always
  judgment-heavy. Skipping is explicit-flag-only (`--skip-adversarial-review`).
- `--skip-qa` and `--skip-adversarial-review` are independent flags; neither
  implies the other.

## Multi-model preflight

`scripts/preflight-models.sh` runs at KBD method start (kbd-assess step 0)
and lazily before any dispatch without a fresh cache. It:

1. Checks the `liter-llm` binary — missing → run `/liter-llm-bridge install`.
2. Detects provider keys from the canonical env vars (delegates to
   liter-llm-bridge's `detect-providers.sh`; see its
   `references/provider-env-vars.md`).
3. Resolves each role through `shared/scripts/lib/kbd-model-resolve.sh` and
   reports which config layer supplied it. Two files own this, and neither is a
   script:
   - `~/.prometheus/kbd/models.toml` — role → model NAME (KBD owns)
   - `~/.config/liter-llm/liter-llm-proxy.toml` — NAME → provider + base_url +
     `${KEY}` (liter-llm owns)

   Repair or extend both with `/liter-llm-bridge configure` (`repair`,
   `add-provider`, `verify`). It merges and never clobbers.

   The older `~/.config/liter-llm/config.toml` with a flat `[aliases]` table is
   **retired** — that shape is not a schema liter-llm can load, which is why the
   judge silently fell back to passing a class name through as a model id.
4. Verifies ≥ 2 distinct **dispatchable** models exist (so judge ≠ producer is
   always possible). Exactly 1 → `status: degraded`. It also reports
   `config_broken` with named defects when the liter-llm config exists but cannot
   serve a request — missing `[general] master_key` (401 on everything) or a
   localhost `base_url` with no `[security] outbound_policy` (`deny_private`
   blocks loopback).
5. When **no** keys are found: ask the user which providers to configure
   (list the env-var name per provider) and instruct them to export the key.
   **This skill never collects, stores, or writes API keys** — config.toml
   holds aliases only; keys stay in the environment.

Cache: `.kbd-orchestrator/model-preflight.json` — re-run on `--force`,
config change, or age > 24 h.

## Escalation — Party Mode is not built here

Multi-persona debate is **deliberately out of scope**. The research record
shows a single isolated adversarial reviewer captures most of the value,
while fixed-round multi-agent debate can launder confidence into consensus —
the exact failure mode sycophancy-correction exists to fight. Reserve
multi-persona escalation for pre-implementation, hard-to-reverse decisions
(zeespec-interrogator NO-GO/CAUTION, Darwin Gate 3). If it is ever built,
personas must be adversarial on *priorities* (security vs shippability vs
maintainability) — not the same reviewer run three times.

## References

- `references/output-contract.md` — findings schema and gate semantics
- `references/isolation-and-routing.md` — fresh-context judging rationale,
  model resolution, collision handling, fallback chain, preflight contract
- KBD wiring: `skills/process/kbd-process-orchestrator/references/integrations/adversarial-review.md`
