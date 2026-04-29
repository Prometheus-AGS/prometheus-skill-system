# iterative-evolver Integration

This document specifies the integration contract between
`iterative-evolver` and `zeespec-interrogator`.

---

## When the Evolver Invokes ZeeSpec

The evolver invokes ZeeSpec from its **Assess phase** when a domain or goal
area is under-constrained before strategic planning begins.

**Trigger condition:**
```
domain is newly defined (no prior evolution for this subject)
OR domain_coverage_estimate < evolver_coverage_threshold (default: 0.65)
OR user explicitly requests interrogation
```

The evolver uses a lower default threshold (0.65) than KBD (0.70) because
strategic-level planning tolerates slightly more ambiguity than tactical-level
change execution.

---

## Invocation Protocol

### Writing the Request File

```json
// .zeespec/pending/<subject_name>.request.json
{
  "subject_name": "<evolution_name>-domain",
  "subject_description": "<goals and domain summary from evolver state>",
  "caller": "iterative-evolver",
  "caller_context": {
    "phase": "assess",
    "evolution_name": "<evolution_name>",
    "domain": "<software|business|product|...>"
  },
  "dimensions": ["why", "who", "when", "what", "how"],
  "coverage_threshold": 0.65
}
```

### Invoking the Skill

```
/zeespec-interrogate "<subject_name>" --caller iterative-evolver
```

The evolver waits for ZeeSpec to complete before loading the domain adapter
and proceeding to Analyze.

---

## Reading the Manifest

The evolver reads these fields from the manifest:

| Field | Evolver Action |
|---|---|
| `go_recommendation` | `GO` → proceed to Analyze. `CAUTION` → log gaps in `assessment.json`. `NO-GO` → surface to user at approval gate before Analyze. |
| `caller_enrichment.planning_constraints` | Merge into `analysis.json` under `zeespec_constraints` key |
| `constraints.why` | Feed into goal validation in Reflect phase |
| `gaps.critical` | Add to `assessment.json` gap analysis |
| `blocked_until` | Surface at human approval gate as recommended resolution items |

---

## Merging into analysis.json

The evolver writes `planning_constraints` into `analysis.json`:

```json
{
  "external_landscape": { ... },
  "opportunities": [ ... ],
  "threats": [ ... ],
  "zeespec_constraints": {
    "subject": "<subject_name>",
    "coverage_status": "partial",
    "go_recommendation": "CAUTION",
    "constraints_by_dimension": {
      "why": [
        { "id": "c-why-1", "statement": "The system MUST NOT use cloud inference.", "confidence": "high" }
      ]
    },
    "critical_gaps": [
      { "id": "gap-why-4", "dimension": "why", "implicit_implication": "No compliance constraints defined..." }
    ],
    "blocked_until": ["gap-why-4"]
  }
}
```

---

## Human Approval Gate Behavior

When ZeeSpec returns NO-GO or CAUTION with non-empty `blocked_until`, the evolver
surfaces these at its human approval gate (post-Assess, pre-Analyze):

```
⚠️  ZeeSpec Constraint Gate

The following gaps are recommended for resolution before planning:

  [gap-why-4] Why Q4 — Compliance constraints undefined
  Implication: System assumes no regulatory requirements.
  Suggestion: Confirm whether HIPAA, GDPR, or SOC 2 apply.

ZeeSpec recommendation: CAUTION

Options:
  1. Resolve gaps and re-run /zeespec-interrogate before planning
  2. Acknowledge gaps and proceed to Analyze (gaps logged in analysis.json)
  3. /evolve --skip-zeespec to bypass entirely
```

---

## Overriding the ZeeSpec Gate

```
/evolve --skip-zeespec
```

Logs the override in `decisions.md`. Does not prevent execution.
