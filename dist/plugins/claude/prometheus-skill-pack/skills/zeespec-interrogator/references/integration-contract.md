# Caller Integration Contract

This document specifies how external skills invoke `zeespec-interrogator` and
consume the constraint manifest it produces. All callers use the same underlying
skill; the `caller` parameter routes the output enrichment format.

---

## Invocation Interface

ZeeSpec is invoked via its slash commands. Callers pass context via arguments
or by writing an invocation request to the shared state directory.

### Direct Invocation (standalone, human-driven)

```bash
/zeespec-interrogate "<subject_name>" [--caller <caller>] [--change-id <id>]
```

### Programmatic Invocation (from another skill)

Write a request file to `.zeespec/pending/<subject_name>.request.json`:

```json
{
  "subject_name": "forge-rs-implementation-layer",
  "subject_description": "The Layer 4 code enrichment engine...",
  "caller": "kbd",
  "caller_context": {
    "phase": "assess",
    "change_id": "CHANGE-042",
    "proposal_path": "openspec/changes/CHANGE-042/specs/change.md"
  },
  "dimensions": ["why", "who", "when", "what", "how"],
  "coverage_threshold": 0.70
}
```

ZeeSpec reads this file on invocation and processes accordingly.

---

## Output Location

The manifest is always written to:
```
.zeespec/<subject_name>/manifest.json
```

ZeeSpec emits the path on final stdout line:
```
ZEESPEC_MANIFEST=.zeespec/<subject_name>/manifest.json
```

Callers should parse this line to locate the manifest.

---

## Caller: `kbd-process-orchestrator`

### When to Invoke

KBD invokes ZeeSpec from its Assess or Plan phase when:
- The `spec_coverage_score` for a change or area is below `kbd_coverage_threshold` (default: 0.70)
- A change proposal has undefined `Why` or `Who` dimensions
- The user explicitly requests interrogation with `/kbd-assess --interrogate`

KBD does NOT invoke ZeeSpec for:
- Routine implementation changes with well-understood scope
- Changes to existing, well-specified systems (unless the change introduces new scope)
- Changes where the calling user explicitly overrides with `--skip-zeespec`

### What KBD Reads from the Manifest

KBD reads `caller_enrichment.openspec_spec_addition` and appends it to the
OpenSpec proposal spec file at `caller_context.proposal_path`. This is the
only automated write KBD performs on the manifest.

KBD also reads:
- `go_recommendation` — GO proceeds, CAUTION proceeds with logged gaps, NO-GO blocks Plan phase
- `gaps.critical` — added to `execution.md` as blocking constraints
- `blocked_until` — tracked in `current-waypoint.json` until resolved

### KBD Integration Point in `execution.md`

```markdown
## ZeeSpec Gate

| Subject | Coverage | Recommendation |
|---|---|---|
| <subject_name> | <score>% | <GO|CAUTION|NO-GO> |

Critical gaps blocking this change:
<list from gaps.critical>

Implicit decisions (AI will decide):
<list from gaps.implicit>
```

---

## Caller: `iterative-evolver`

### When to Invoke

The evolver invokes ZeeSpec from its Assess phase when:
- The domain or goal area has never been formally specified
- The assess phase detects insufficient constraint coverage for a strategic area
- The user explicitly requests interrogation before planning

### What the Evolver Reads from the Manifest

The evolver reads `caller_enrichment.planning_constraints` and merges it into
`analysis.json` under the `zeespec_constraints` key.

It also reads:
- `go_recommendation` — influences whether the evolver proceeds to Plan
- `constraints.why` — feeds the evolver's goal validation
- `gaps.critical` — flagged in the evolver's gap analysis

---

## Caller: `standalone`

No automated integration. The user reads the manifest directly.

The manifest at `.zeespec/<subject_name>/manifest.json` contains:
- Full constraint inventory by dimension
- GO/CAUTION/NO-GO recommendation with rationale
- Human-readable summary block
- `blocked_until` list for manual resolution tracking

---

## Manifest Stability Contract

The constraint manifest schema is versioned at `references/schemas/constraint-manifest.schema.json`.

Callers may depend on:
- `coverage.aggregate_score` — stable, numeric
- `coverage.aggregate_status` — stable, enum
- `go_recommendation` — stable, enum
- `gaps.critical[].implicit_implication` — stable, string
- `blocked_until[]` — stable, array of gap IDs
- `caller_enrichment.openspec_spec_addition` — stable when caller=kbd
- `caller_enrichment.planning_constraints` — stable when caller=iterative-evolver

Fields added in future versions will not remove existing fields. Breaking changes
will increment the schema major version and require a migration.

---

## Error Cases

| Condition | ZeeSpec Behavior |
|---|---|
| Subject already interrogated | Offer to resume or re-interrogate |
| Caller requests non-existent subject | Create new interrogation |
| Interrogation timed out (user abandons) | Write partial manifest with `status: incomplete` |
| Coverage threshold not met | Write manifest with NO-GO, emit ZEESPEC_MANIFEST path |
| Schema validation fails | Do not write manifest, emit error to stderr |
