<!--
Dossier template — the lossless handoff. Everything downstream reads this, so
anything compressed out here is gone for good.

Rules (references/thread-contracts.md):
  - Facts only. No title, no sections, no conclusions, no analysis.
  - As long as necessary; several pages is expected and correct.
  - Carry the context of each fact so it cannot be misattributed later.
  - Flag anything untrustworthy or contradictory. Stage 06 resolves it; you
    only have to surface it.
  - Cite every fact inline as [src:<sha8>], where <sha8> is an id in THIS
    thread's sources.json. A citation to anything else is a CRITICAL merge
    failure — that rule is what enforces the director's no-search constraint.
  - Default cap 6000 tokens.

Delete this comment when writing a real dossier.
-->

{{fact_with_its_context}} [src:{{sha8}}]

{{another_fact_stated_plainly}} [src:{{sha8}}]

{{a_statement_that_conflicts_with_the_above_flagged_as_such}} [src:{{sha8}}]
