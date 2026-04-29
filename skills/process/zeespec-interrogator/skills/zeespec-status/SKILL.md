---
license: MIT
name: zeespec-status
description: >
  Show current ZeeSpec interrogation progress for a named subject. Displays
  which dimensions have been completed, current coverage scores if available,
  and the manifest path if the interrogation is complete. Read-only — does
  not modify state.
---

# /zeespec-status

Read-only status report for a named interrogation session.

## Usage

```
/zeespec-status "<subject_name>"
```

## Setup

1. Parse `subject_name`
2. Locate `.zeespec/subjects/<subject_name>/state.json`
3. If not found: report "No interrogation found for '<subject_name>'"
4. Read state and produce the status report below

## Output Format

```
ZeeSpec Interrogation Status — <subject_name>
─────────────────────────────────────────────
Status:     <running | complete | incomplete>
Caller:     <standalone | kbd | iterative-evolver>
Started:    <started_at>
Updated:    <updated_at>

Phases Completed:
  ✅ interrogate    (if complete)
  ✅ score          (if complete)
  ✅ manifest       (if complete)
  ✅ persist        (if complete)
  ⏳ <phase>        (if pending)

Dimensions Interrogated:
  Why:   <defined_count> defined, <partial_count> partial, <implicit_count> implicit
  Who:   ...
  When:  ...
  What:  ...
  Where: ...
  How:   ...

Coverage (if scored):
  Why:      <score>%  <sufficient|partial|insufficient>
  Who:      <score>%  <sufficient|partial|insufficient>
  When:     <score>%  <sufficient|partial|insufficient>
  What:     <score>%  <sufficient|partial|insufficient>
  Where:    <score>%  <sufficient|partial|insufficient>
  How:      <score>%  <sufficient|partial|insufficient>
  Aggregate: <score>%  → <GO|CAUTION|NO-GO>

Manifest: <path or "not yet generated">
Blocked Until Resolved: <count> gaps

Last Checkpoint: <checkpoint_id> at <timestamp>
```

## Rules

- Never modify state, scores, or manifest
- If coverage has not been scored yet, omit the Coverage block and note "Run /zeespec-score to compute"
- If manifest has not been generated, note "Run /zeespec-interrogate to complete"
