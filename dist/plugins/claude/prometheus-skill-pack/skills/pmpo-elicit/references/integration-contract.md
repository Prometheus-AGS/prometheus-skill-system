# pmpo-elicit integration contract

How any KBD/PMPO stage calls `pmpo-elicit` when it detects missing information,
instead of guessing or silently marking a question implicit.

## Calling

A caller raises an elicitation by constructing a `request` (schema:
`references/schemas/elicitation.schema.json`) and invoking `/pmpo-elicit`:

```
/pmpo-elicit "<question>" --hints "<h1>;<h2>" --criticality <low|medium|high|blocking> \
             --caller <stage> --write-back <path>
```

`pmpo-elicit` writes `elicitations/<id>/request.json` and, on resolution,
`elicitations/<id>/result.json` in the caller's state dir. The caller reads
`result.json`, applies the `answer`, and records `provenance` + `elicitation_id`
wherever it stores the resolved value.

## The four option classes (always offered, in this order)

1. **Direct answers** — 2–4 concrete options the caller could already infer.
2. **"Here's the source"** — the user supplies a URL / file / doc / person;
   pmpo-elicit fetches and extracts the answer, recording `provenance: source`
   and `source_ref`.
3. **"Research it for me"** — ALWAYS present. Runs a bounded research loop seeded
   by the request's `hints[]`; returns `answer` + `confidence` + `evidence`,
   `provenance: research`. Budget: `max_sources: 6`, `max_minutes: 10` — on a
   cap, return partial evidence with low confidence rather than spin.
4. **"Decide for me (implicit)"** — records the AI's chosen default with
   rationale, `provenance: implicit`. This makes an implicit decision an
   explicit, audited user choice rather than a silent one.

## Consumers

- **kbd-analyze** — contested stack choice (score gap < 15%) routes here rather
  than silently picking.
- **zeespec-interrogate** *(wiring lands in a later phase)* — unanswered
  questions in below-threshold dimensions route here, batched per dimension,
  instead of being silently marked implicit. Answers gain `provenance` and
  `elicitation_id`.
- **kbd-capability** *(later phase)* — under-specified capability needs route
  the missing fields here.

## Modes

- **Inline-fallback** (this release): the research option runs in-session with
  the same budget guards. Contract files are identical.
- **Child-isolated** (child-loops phase): the research option spawns a child
  loop with read-only scoped permissions; the handoff artifact is `result.json`.
  Only the isolation differs — callers see the same contract either way.
