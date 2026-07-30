# Goals

- Wire /adversarial-review into pmpo-skill-creator's Reflect phase so a generated skill is judged by a model that did not generate it
- Wire /adversarial-review into native-agent generation so a generated agent workspace is reviewed before it is declared ready
- Define artifact-mode review packets for the two new artifact kinds: a generated SKILL.md tree and a generated Cargo workspace
- Enforce KBD_PRODUCER_MODEL at both creator entry points so cross_model_check can never record unverified-producer-unknown
  - **RATIFIED 2026-07-30 (change-arc-001).** "Can never record" is satisfied by **failing
    closed**, not by masking the symptom: when `KBD_PRODUCER_MODEL` is unset, the creator
    **refuses to dispatch a review at all** (exit 2, no findings file written, refusal on
    stderr). The value is **never synthesized**.
  - Why this reading: a default such as `${KBD_PRODUCER_MODEL:-claude-opus-5}` would make
    `unverified-producer-unknown` unreachable by *fabricating* a producer identity — the
    review would record `verified-distinct` for a comparison that never happened. That is
    the exact failure class this phase exists to eliminate, reintroduced through the fix.
    An honest refusal is correct; a fabricated pass is not.
  - Rejected alternatives: (a) synthesize a default — fabricates identity; (b) tolerate an
    unknown producer and record `unverified-producer-unknown` — contradicts the goal text.
- Make the review gate blocking on CRITICAL findings, with the existing 2-rejection cap so it cannot loop forever
- Promote the sycophancy pass in execute.md/reflect.md from prompt instruction to an enforced gate in validate-skill.sh
  - **RATIFIED 2026-07-30 (change-arc-001).** `validate-skill.sh` is the **single enforced
    gate**, and it **shells out to** the existing `check-findings-sycophancy.sh` as an
    additional check group — propagating that helper's exit into its `FAIL` counter and
    surfacing the feedback in the existing `=== RESULT ===` block. Creators invoke
    `validate-skill.sh`; they **never** call the sycophancy helper directly.
  - Why this reading: the goal names `validate-skill.sh` as the enforcement point, so
    wiring creators straight to the helper would not satisfy it. But reimplementing the
    screen inside `validate-skill.sh` would duplicate logic — including the rejection cap —
    that would then drift from the copy used by adversarial review. Invoking the existing
    helper satisfies the goal's location requirement with one implementation.
  - Scope note: this ratification concerns *where enforcement lives*. The **value** of the
    rejection cap is goal 5's concern, and `change-arc-007` (a user-requested extension)
    makes that value overridable.
- Prove the loop end to end: a deliberately flawed generated skill and agent must each be caught by the judge and recorded verified-distinct
