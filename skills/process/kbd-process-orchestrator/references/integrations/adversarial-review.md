# Integration: adversarial-review

Cross-model, fresh-context adversarial review as a KBD pipeline stage.
Skill home: `skills/process/adversarial-review/` (standalone,
harness-agnostic — this document is the KBD-side wiring contract).

## Where it runs

| KBD stage | Mode | When | Skip |
|---|---|---|---|
| `kbd-assess` | `artifact assess` | after `assessment.md` is written, before `kbd_stage_handoff_write assess` | `--skip-adversarial-review` only |
| `kbd-analyze` | `artifact analyze` | after `analysis.md` + `library-candidates.json`, before handoff | `--skip-adversarial-review` only |
| `kbd-plan` | `artifact plan` | after `plan.md`, **before** change structures are emitted | `--skip-adversarial-review` only |
| `kbd-execute` | `diff <change-id>` | after `/refine-validate` passes, before archive | `--skip-adversarial-review`, or <3 files, or docs-only |

`kbd-assess` additionally runs the **multi-model preflight** as its step 0
(the KBD entry point), caching `.kbd-orchestrator/model-preflight.json`.

## Invocation contract

```bash
SKILL_DIR="${CLAUDE_PLUGIN_ROOT}/skills/process/adversarial-review"
REVIEW_DIR=".kbd-orchestrator/phases/$PHASE/review/$TARGET"

bash "$SKILL_DIR/scripts/preflight-models.sh"                       # step 0 / lazy
bash "$SKILL_DIR/scripts/build-review-packet.sh" \
  --mode "$MODE" --phase "$PHASE" --target "$TARGET" \
  --out "$REVIEW_DIR/packet.json"
bash "$SKILL_DIR/scripts/dispatch-judge.sh" \
  --mode "$MODE" --packet "$REVIEW_DIR/packet.json" \
  --out "$REVIEW_DIR/findings.json"
bash "$SKILL_DIR/scripts/check-findings-sycophancy.sh" \
  --findings "$REVIEW_DIR/findings.json" \
  --counter-key "adv-review-$PHASE-$TARGET" > "$REVIEW_DIR/rejection.md" \
  || { # exit 2: report rejected as theater — re-dispatch ONCE with feedback
    bash "$SKILL_DIR/scripts/dispatch-judge.sh" \
      --mode "$MODE" --packet "$REVIEW_DIR/packet.json" \
      --feedback "$REVIEW_DIR/rejection.md" \
      --out "$REVIEW_DIR/findings.json"
  }
```

`dispatch-judge.sh` exit codes the caller must honor:

- `0` — findings written (`isolation_mode: liter-llm`).
- `3` — liter-llm unavailable: dispatch a **harness-native fresh-context
  subagent** whose prompt is exactly the mode's mandate file
  (`assets/reviewer-mandate-<mode>.md`) + `packet.json`, nothing else;
  normalize its output to the findings schema with
  `"isolation_mode": "harness-native"`.
- `4` — no judge possible: record
  `adversarial_review: "SKIPPED (<reason>)"` in the phase `progress.json`
  and continue. Never fail the stage on missing review infrastructure.

## Gate semantics

Findings schema:
`skills/process/adversarial-review/assets/schemas/findings.schema.json`.
`verdict` is `BLOCK` iff ≥1 CRITICAL finding.

**Diff mode (kbd-execute):**
- `BLOCK` → `certification: BLOCKED` in `progress.json` (same field the QA
  gate uses); fix, then re-run `refine-validate` **and** `adversarial-review`.
- `PASS` with WARNINGs → warnings stay in `$REVIEW_DIR`, archive proceeds.

**Artifact mode (assess/analyze/plan):**
- `BLOCK` → revise the artifact, rebuild packet, re-dispatch. Max 2 vet
  rounds; on the second `BLOCK`, accept and append an
  **"Unresolved review findings"** section (the CRITICAL findings verbatim)
  to the artifact so the next stage inherits them explicitly.
- WARNING findings → summarized in the stage's `kbd_stage_handoff_write`.

## Model routing

| Phase key | Class |
|---|---|
| `adv-review-preflight` | small |
| `adv-review-packet` | small |
| `adv-review-judge` | frontier |

The judge must resolve to a model **different from `producer_model`** in the
packet; on collision the dispatcher walks frontier → medium → small aliases
and only proceeds same-model with a logged `JUDGE_MODEL_COLLISION` warning.
See `skills/process/adversarial-review/references/isolation-and-routing.md`.

## Relationship to the other gates

- `refine-validate` (artifact-refiner) — deterministic checklist; always runs
  **first** in kbd-execute. `--skip-qa` and `--skip-adversarial-review` are
  independent.
- `sycophancy-correction` — screens the **judge's report** for softening
  (anti-theater), via `check-findings-sycophancy.sh`. Complementary to the
  reflection/assessment tone gates in `shared/scripts/`.
- Multi-persona escalation is **not** part of this integration — see the
  Escalation section of the adversarial-review SKILL.md.
