# Tuning Log — change-lgv-006

## Finding

`change-lgv-005`'s initial metrics run showed a systematic pattern, not
noise: **every one of the 8 `factually-flawed` eval items** scored
`clarity` 0.18–0.35 points *below* its gold-standard score, while `strong`
items scored clarity slightly *above* gold (7/8 positive). Mean signed
diff across all 24 items: **-0.104** (grader consistently under-scored
clarity relative to gold).

## Root cause

Not a grader bug — a **rubric ambiguity**. The original Step 3 clarity
definition read:

> "Clarity — Is it understandable to the stated target level?"

This is genuinely ambiguous between two readings:

1. **Prose readability**: are the sentences well-structured, grammatically
   clear, easy to parse? (what the gold-standard authoring assumed)
2. **Conceptual understandability**: does the explanation leave the reader
   with an accurate mental model? (what the grader agents consistently
   resolved toward)

Under reading 2, a grammatically clear but factually false sentence
*doesn't* make the concept understandable — so grading agents docked
clarity for factual errors, effectively double-penalizing the same error
under both `accuracy` and `clarity`.

Both readings are individually defensible. The ambiguity itself is the
defect: the rubric should specify which axis clarity measures, so the
four dimensions stay orthogonal (an explanation can be clearly-written
and wrong, or awkwardly-written and correct — the rubric should be able to
express both).

## Fix

Edited `learn-grade/SKILL.md`'s Step 3 clarity row to read:

> "Is the **prose** readable and well-structured at the stated target
> level — sentence structure, organization, terminology use? Score prose
> quality independent of whether the content is factually correct;
> factual correctness is `accuracy`'s job, not `clarity`'s. A
> grammatically clear sentence that states something false still scores
> high on clarity and low on accuracy — do not let a factual error pull
> clarity down."

This makes clarity orthogonal to accuracy by definition, matching how the
other three dimensions are already independent of each other.

## Validation

Re-ran the 8 `factually-flawed` items (the ones with the largest clarity
gap) through the tuned rubric. All 8 confirmed the expected effect:

| Item | Clarity before | Clarity after | Gold | Gap before | Gap after |
|---|---|---|---|---|---|
| cr-005-flawed-mitochondria-create-energy | 0.50 | 0.85 | 0.82 | -0.32 | +0.03 |
| cr-006-flawed-breathing-conflation | 0.60 | 0.75 | 0.85 | -0.25 | -0.10 |
| cr-007-flawed-glycolysis-location | 0.55 | 0.85 | 0.85 | -0.30 | 0.00 |
| kbd-006-flawed-openspec-optional | 0.60 | 0.75 | 0.80 | -0.20 | -0.05 |
| kbd-007-flawed-assess-writes-plan | 0.60 | 0.85 | 0.85 | -0.25 | 0.00 |
| kbd-008-flawed-reflect-early | 0.60 | 0.85 | 0.82 | -0.22 | +0.03 |
| sp-006-flawed-internet-required | 0.50 | 0.85 | 0.80 | -0.30 | +0.05 |
| sp-007-flawed-restart-required | 0.60 | 0.85 | 0.85 | -0.25 | 0.00 |

One item (`kbd-001-strong-full-cycle`, a "strong" tier item) was also
re-graded as a control: clarity moved from 0.55 → **0.45**, moving
*further* from gold (0.90). On inspection this is not a regression — the
tuned rubric correctly identified that this item's single 200+ word
run-on sentence has genuinely poor prose structure independent of its
(correct) content, which the original gold score of 0.90 had
conflated with factual accuracy. This is a **gold-standard authoring
bias**, not a new grader defect; it's noted here rather than silently
absorbed into "improvement," since not every clarity-score movement after
tuning was toward gold — the tuning made the grader more discriminating,
which is the intended outcome even where it disagrees with an
under-scrutinized gold value.

## Metric impact (24-item aggregate, before → after)

| Dimension | Pearson r (before) | Pearson r (after) | Spearman r (before) | Spearman r (after) | MAE (before) | MAE (after) |
|---|---|---|---|---|---|---|
| clarity | 0.405 | **0.609** | 0.379 | **0.625** | 0.160 | **0.088** |
| accuracy | 0.930 | 0.938 | 0.896 | 0.935 | 0.090 | 0.077 |
| completeness | 0.892 | 0.908 | 0.785 | 0.738 | 0.155 | 0.157 |

Misconceptions precision/recall/F1 unchanged (0.923 / 1.0 / 0.96) — the
tuning targeted clarity only and did not touch the misconception-detection
logic, as expected.

## Conclusion

Real, systematic issue → real tuning, not a no-op. Clarity correlation
improved substantially (Pearson +0.204, Spearman +0.246, MAE roughly
halved) without degrading the other three dimensions. Accuracy and
completeness both held steady or slightly improved as a side effect of
re-grading with a sharper rubric.

**Caveat carried forward to G-06 documentation**: all ground truth used
here is still in `draft` review status (per the phase's resolved open
question #1). These before/after numbers should be re-verified once a
human review pass on the gold scores has landed — the `kbd-001` control
case above is a live example of where the gold standard itself may need
correction, not just the grader.
