# Proposal — change-lgv-002-eval-explanations

Write 20+ Feynman-style explanations spanning the 3 domains (kbd-lifecycle,
skill-pack, the new third domain from change-lgv-001). Mix of:
- Strong explanations (should score high, no misconceptions)
- Partially incomplete explanations (should score low on completeness)
- Factually flawed explanations (should score low on accuracy)
- Misconception-containing explanations (should score 0.0 on
  misconceptions_absent — pulled directly from each corpus's
  is_misconception:true entries)

Each explanation gets a draft ground-truth annotation: per-dimension gold
score (completeness/accuracy/clarity/misconceptions_absent) plus a list of
which specific misconceptions (if any) are present.

**Human review required before treating drafts as ground truth** — this
change produces drafts; a follow-up review step (documented in tasks.md)
confirms or corrects them.

## Goal
G-01.
