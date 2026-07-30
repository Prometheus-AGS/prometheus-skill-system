# Goals

- Ship an ideation skill that vets an idea through a **judge with a find-problems mandate**, not a persona round-table debating to consensus
- Make every decision artifact carry `cross_model_check: verified-distinct`, reusing the creation-review packet machinery from `adversarial-review-for-creation`
- Add explicit **automation-bias countermeasures** for personal and irreversible decisions: surface disconfirming evidence, state confidence honestly, and refuse to manufacture certainty
- Persist decisions and their outcomes into the Karpathy wiki so a later decision can be checked against what actually happened — the differentiator no surveyed competitor has
- Wire coach and reflector personas as **separate roles that cannot grade their own output**, following the producer≠judge rule this pack already enforces
- Deliver on at least Claude Code plus one non-Claude harness (Codex or Kimi), through `ui-surface` tier detection rather than harness-specific rendering
- Prove it end to end with committed fixtures: a weak idea must be blocked and a sound one must pass, both judged cross-model

---

_Seeded 2026-07-30 from `adversarial-review-for-creation/reflection.md`; goals
below the line are the seed and the constraints that shaped the list above._

## Seed

The assess stage of the previous phase captured a large vision — persona teams
with a judge, business-model vetting, coach/reflector personas, Feynman +
Karpathy loops for personal and business development, delivery across Claude
Desktop / Codex / Kimi, and hooks into the librefang/bossfang orchestrator. None
of it was in scope there, and the adversarial machinery that phase hardened is
exactly the substrate it needs: a real cross-model judge, manifest-level packets,
and a bounded, auditable rejection gate.

## Research constraints carried forward (assessment.md:118–165)

These are findings, not preferences. They should survive into planning, or be
explicitly overruled with a stated reason:

1. **A naive persona round-table is contraindicated.** The Multi-Agent Debate
   martingale proof (NeurIPS 2025 spotlight) shows debate alone **does not improve
   expected correctness**. What does work is a *targeted intervention biasing the
   belief update toward correction* — i.e. a judge with an explicit find-problems
   mandate, which is what this pack already builds. L-MAD measures debate gains at
   **up to 8%** — real but modest; Kraidia (Nature, 2026) documents
   *persuasion-driven adversarial influence*, where one confident wrong agent moves
   the group.

2. **Automation bias is the dominant risk for personal decisions.** Explainable AI
   *increased trust while promoting over-reliance*, producing **"False Confirmation"**
   errors — visible reasoning "may instead provide false assurance that errors have
   been checked for and ruled out." Skill degradation under high-control
   configurations is documented. For *"should I become a pro athlete"* and
   especially relationship questions, a confident persona panel is a machine for
   manufacturing false confidence about irreversible choices.

3. **Idea validation is commoditised.** ideaproof.io and 9–12 tool roundups already
   sell "validate in 120 seconds". Generic validation is not a differentiator. The
   three things the surveyed tools lack: a persisting Karpathy loop, structural
   sycophancy correction, and a recorded `cross_model_check`. (Deliberately narrow —
   that survey covered ideaproof.io plus two roundups, not the whole category.)

## Blockers to clear first (previous phase TD-02, TD-03)

Neither is optional; the second silently defeats the goals above.

1. **Commit the previous phase** — 150 changed paths, including a new
   `tools/openai-proxy` submodule and `.gitmodules`.
2. **`bash scripts/update-skill-pack.sh --force`** — plugin caches are stale (6
   drift findings). The producer-model guard is **not live in any installed copy**
   until this runs, so any creator or ideation skill running from a cache cannot
   make the judge≠producer guarantee these goals depend on.

## Instructions

Review and refine before running:

```
/kbd-assess ideation-and-decision-tools
```
