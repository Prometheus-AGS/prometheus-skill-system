# Self-Teaching Loop

The Prometheus Skill Pack teaches itself using its own learning infrastructure.
This document describes the full cycle from entry to retention.

## The Core Proposition

When an operator runs `/learn-about-system`, they are not entering a
documentation reader or a tutorial system. They are entering the same Feynman
learning loop that the skill pack uses for any domain — Rust ownership, cancer
immunotherapy, linear algebra, or the KBD lifecycle itself.

The loop is domain-agnostic. The pre-built meta-corpora (`kbd-lifecycle-corpus.json`
and `skill-pack-corpus.json`) are the only meta-specific artifact. Everything
downstream — survey, feynman, grade, retain — operates identically regardless
of subject.

## Full Cycle

```
learn-about-system      — entry point, corpus selection, area routing
       ↓
learn-goal              — goal artifact, feasibility gate, corpus path recorded
       ↓
learn-survey            — placement assessment, knowledge level established
       ↓
feynman-loop            — teach-back cycle: explain, gap-detect, re-explain
       ↓
learn-grade             — grade artifact written, gap list produced
       ↓
learn-plan              — curriculum generated from gaps (optional, for multi-session)
       ↓
learn-practice          — exercises and retrieval practice
       ↓
learn-retain            — spaced repetition scheduling, retention artifact
```

### Entry: learn-about-system

Selects the corpus appropriate for the operator's area of interest. For
`--area kbd` and `--area skills`, the corpus is pre-built and bypasses the
live assembly step in `learn-goal`. For `--area harness` or freeform input,
standard `learn-goal` corpus assembly runs.

### Goal: learn-goal

Records the subject, target proficiency level, time budget, and corpus path in
`~/.prometheus/learn/goals/<goal-id>/goal.json`. The feasibility gate runs a
sycophancy-corrected assessment of whether the operator's time budget matches
the domain's realistic learning curve. The gate never inflates a RED to YELLOW
to please the operator.

When `learn-about-system` pre-loads a corpus, `learn-goal` skips Steps 1–2
(intake and corpus assembly) and begins at Step 3 (time-to-mastery research).

### Survey: learn-survey

Presents concept-understanding questions drawn from the corpus `sources` and
`misconceptions` arrays. The goal is placement, not testing: learn-survey
establishes what the operator already knows so the Feynman loop starts at the
right level rather than wasting time on known concepts.

Misconception probes are surfaced here. Examples for the KBD area:
- "kbd-plan writes code" (it does not)
- "position-reminder.txt is optional" (it is not)

Survey results are written to `~/.prometheus/learn/goals/<goal-id>/survey.json`.

### Feynman: feynman-loop

The operator explains a concept back in their own words. The loop detects gaps
between the explanation and the grounded corpus content, then asks the operator
to re-explain the gaps. This continues until the operator can explain the
concept accurately without prompting.

The loop is non-sycophantic: it does not praise incomplete explanations. It
names the gap and asks for another attempt.

### Grade: learn-grade

Produces a grade artifact summarizing which concepts were mastered, which are
partial, and which are absent. The grade drives curriculum generation in
`learn-plan`.

### Plan: learn-plan

Generates a sequenced curriculum from the grade's gap list. The curriculum is
written to `~/.prometheus/learn/goals/<goal-id>/curriculum.json`. For single-
session learning (KBD orientation, skill pack overview), this step may be
skipped if all gaps were resolved in the Feynman loop.

### Practice: learn-practice

Exercises and retrieval practice targeting the gaps identified in the grade.
For the KBD area, practice exercises might include:
- "Write the progress signal for starting kbd-execute on step 4 of 12 of the
  slli-integration phase"
- "What file does a kbd-* skill read before any other action?"
- "What are the three sections required in a PMPO reflection?"

### Retain: learn-retain

Schedules spaced repetition reviews based on the grade and practice results.
Writes a retention artifact with review dates. For meta-subjects (KBD, skill
pack), retention is typically short: concepts are reinforced naturally during
daily use of the skills.

## Why This Matters for Adoption

A new operator who runs `/learn-about-system --area kbd` does not read
documentation passively. They:

1. Assess their prior knowledge (they may know nothing, or may have used KBD
   before without understanding the position-reminder contract)
2. Encounter the specific misconceptions that cause operators to use the system
   incorrectly
3. Explain concepts back and have gaps corrected with grounded content
4. Leave with verified understanding, not just exposure

This is more expensive than reading a README. It is also more effective. The
skill pack's own adoption experience demonstrates the learning infrastructure
in action — which is itself a form of teaching.

## Meta-Corpus Structure

Both pre-built corpora (`kbd-lifecycle-corpus.json`, `skill-pack-corpus.json`)
follow the standard `learn-goal` corpus schema:

```json
{
  "corpus_id": "string",
  "subject": "string",
  "sources": [
    {
      "source_ref": "string",
      "title": "string",
      "content": "string",
      "authority": "high|medium|low",
      "type": "concept|procedure|example|misconception"
    }
  ]
}
```

Entries with `"type": "misconception"` are surfaced by `learn-survey` as
probes. Entries with `"type": "concept"` or `"procedure"` drive the Feynman
loop. Entries with `"type": "example"` are used in `learn-practice` exercises.

The corpora live at:
- `docs/learn/meta-corpus/kbd-lifecycle-corpus.json`
- `docs/learn/meta-corpus/skill-pack-corpus.json`

They are committed to the repository and versioned with the skill pack.
