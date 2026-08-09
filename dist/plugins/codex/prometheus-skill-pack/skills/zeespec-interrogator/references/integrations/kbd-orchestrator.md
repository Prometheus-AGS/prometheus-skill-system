# kbd-process-orchestrator Integration

This document specifies the exact integration contract between
`kbd-process-orchestrator` and `zeespec-interrogator`.

---

## When KBD Invokes ZeeSpec

KBD invokes ZeeSpec from **two points** in its lifecycle:

### 1. Assess Phase — Domain-level coverage check

During `kbd-assess`, after loading the project context, KBD evaluates a
**spec coverage estimate** for the overall project or for the phase's goal area.

**Trigger condition:**
```
spec_coverage_estimate < kbd_coverage_threshold (default: 0.70)
AND (no prior zeespec manifest for this subject OR manifest is stale)
```

**Staleness**: A manifest is stale when the `subject_description` has materially
changed or when > 30 days have elapsed since `manifest_generated_at`.

**How KBD estimates coverage without running ZeeSpec:**
KBD's assess phase reads existing spec files (AGENTS.md, CLAUDE.md, OpenSpec
proposals) and heuristically scores them against the six dimensions. If Why
and Who are not addressed in any spec file, coverage is estimated as < 0.60.
This fast-path avoids unnecessary ZeeSpec invocations for well-specified projects.

### 2. Plan Phase — Change-level coverage check

During `kbd-plan`, for each proposed change, KBD evaluates whether the change
is sufficiently specified across Why, Who, and When. Changes that introduce
new system behavior or cross architectural boundaries trigger ZeeSpec.

**Trigger condition:**
```
change.introduces_new_behavior == true
AND change.why_defined == false OR change.who_defined == false
```

---

## Invocation Protocol

### Writing the Request File

KBD writes a request file before invoking ZeeSpec:

```json
// .zeespec/pending/<subject_name>.request.json
{
  "subject_name": "<project-or-change-slug>",
  "subject_description": "<one paragraph from AGENTS.md or change proposal>",
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

`where` is typically omitted for change-level interrogations unless the change
introduces new deployment topology. KBD passes all six dimensions for domain-level
interrogations.

### Invoking the Skill

```
/zeespec-interrogate "<subject_name>" --caller kbd --change-id <id>
```

KBD waits for ZeeSpec to complete before proceeding to Plan.

### Capturing the Output

ZeeSpec emits on its final stdout line:
```
ZEESPEC_MANIFEST=.zeespec/<subject_name>/manifest.json
```

KBD parses this line to locate the manifest.

---

## Reading the Manifest

KBD reads these fields from the manifest:

| Field | KBD Action |
|---|---|
| `go_recommendation` | `GO` → proceed to Plan. `CAUTION` → proceed, log gaps. `NO-GO` → block Plan, require resolution. |
| `gaps.critical` | Write to `execution.md` as blocking constraints |
| `blocked_until` | Write to `current-waypoint.json` as unresolved blockers |
| `caller_enrichment.openspec_spec_addition` | Append to the OpenSpec proposal spec at `proposal_path` |
| `constraints.why` | Use as goal validation criteria in Reflect phase |

---

## Enriching the OpenSpec Proposal

When `proposal_path` is provided, KBD appends `openspec_spec_addition` to the
end of the spec file at that path:

```markdown
<!-- Appended by zeespec-interrogator — do not edit manually -->
## ZeeSpec Constraint Manifest

**Coverage**: 76% aggregate | **Recommendation**: CAUTION

### Defined Constraints
**Why**
- The system MUST achieve sub-200ms p99 latency on L4 hardware. (confidence: high)
- The system MUST NOT call external cloud inference APIs. (confidence: high)

**Who**
- Travis James MUST be the sole administrator. (confidence: high)
- Cedar policy MUST govern all authorization decisions. (confidence: medium)

### Open Gaps (resolve before implementation)
- `why.4`: No compliance constraints defined. [IMPLICIT: system assumes no regulatory requirements]
- `who.8`: Data retention ownership undefined. [IMPLICIT: no deletion guarantee]

### Implicit Decisions (AI will decide)
- `where.7`: Failover topology not defined. System will assume single-node, no failover.
```

---

## KBD Waypoint Update

After ZeeSpec completes, KBD updates `current-waypoint.json`:

```json
{
  "zeespec_gate": {
    "subject": "<subject_name>",
    "recommendation": "CAUTION",
    "manifest_path": ".zeespec/<subject>/manifest.json",
    "blocked_until": ["gap-why-4", "gap-who-8"],
    "evaluated_at": "<timestamp>"
  }
}
```

When `blocked_until` is non-empty, KBD's `kbd-status` reports the blockers
and the `exact_next_command` is set to guide the user toward resolving them
before Plan proceeds.

---

## Overriding the ZeeSpec Gate

The user can bypass ZeeSpec in KBD with:

```
/kbd-plan --skip-zeespec
```

This writes a note to `execution.md`:
```
[ZeeSpec gate bypassed by user at <timestamp>]
```

The override is logged but never prevents execution. ZeeSpec recommends; KBD governs.
