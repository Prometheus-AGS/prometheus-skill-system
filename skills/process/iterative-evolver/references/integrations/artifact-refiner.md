# Evolver Integration: artifact-refiner

The iterative-evolver integrates with `artifact-refiner` in two ways:

1. **Indirect QA** — When KBD handles execution, artifact-refiner runs per-change
   QA as part of the KBD inner loop (see `references/integrations/kbd-orchestrator.md`)
2. **Direct report refinement** — The `/evolve-report` sub-skill can invoke
   artifact-refiner to polish evolution reports

**Skill location**: `skills/imported/artifact-refiner/SKILL.md`
**Entry commands**: `/refine-validate`, `/refine-content`, `/refine-code`
**State backend**: `.refiner/artifacts/<artifact-name>/state.json`

---

## 1. Indirect QA (via KBD)

When the evolver delegates execution to KBD, artifact-refiner runs
automatically per completed change:

```
evolve-execute → kbd-execute → per-change QA → artifact-refiner
```

The evolver does not invoke artifact-refiner directly for code QA.
It reads the results back through the KBD reflection and evolver bridge.

### What the Evolver Reads

From `.kbd-orchestrator/phases/<phase>/reflection.md`:
- Artifact Quality Summary section
- Pass rate, recurring violations, refinement iteration count

From `.refiner/artifacts/<change-id>/refinement_log.md`:
- Per-change pass/fail history
- Constraint violation details

---

## 2. Direct Report Refinement

The `/evolve-report` sub-skill generates a markdown evolution report.
When artifact-refiner is available, the report can be refined for quality:

### Invocation Contract

```yaml
artifact_name: "evolution-report-<timestamp>"
artifact_type: content
content_type: direct:markdown
constraints:
  - "Report must have all required sections"
  - "Numbers must be internally consistent"
  - "Executive summary must be under 200 words"
  - "Recommendations must be actionable and specific"
target_state:
  description: >
    A clear, concise evolution report that accurately summarizes the
    cycle's assessment, analysis, plan, execution, and reflection.
```

### When to Refine

- Default: refine when artifact-refiner is available
- Skip with `--no-refine` flag on `/evolve-report`
- Skip when the report has fewer than 3 sections (partial report)

### Output

- Original draft preserved at `reports/evolution-report-<timestamp>-draft.md`
- Refined version at `reports/evolution-report-<timestamp>.md`
- Refinement log at `.refiner/artifacts/evolution-report-<timestamp>/refinement_log.md`

---

## When NOT to Use

- For non-software evolutions where code QA is irrelevant — report
  refinement is still useful in any domain
- When artifact-refiner skill is not installed — degrade gracefully,
  generate the report without refinement
- When iterating rapidly (max_iterations > 3) — refinement adds latency;
  only refine the final report
