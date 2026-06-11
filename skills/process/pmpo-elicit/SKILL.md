---
license: MIT
name: pmpo-elicit
version: '1.0.0'
description: >
  Ask-or-research elicitation primitive for the PMPO lifecycle. When any stage
  detects missing information, pmpo-elicit asks the user for the answer or its
  source — and always offers to research it autonomously from minimal hints —
  recording the answer with provenance instead of silently guessing.
metadata:
  tags: [process, orchestration, automation]
---

# /pmpo-elicit

Resolve a missing piece of information the way the user prefers: answer it,
point to where the answer lives, or have it researched — never silently
assumed.

## When to use

Any KBD/PMPO stage that hits an unknown it cannot responsibly default. Replaces
two bad habits: asking the user everything upfront, and silently marking an
unanswered question "implicit." See `references/integration-contract.md` for the
caller protocol and consumers.

## The four option classes (always offered, in this order)

1. **Direct answers** — 2–4 concrete options inferred from context (use
   `AskUserQuestion`; multi-select when the choices are not exclusive).
2. **"Here's the source"** — the user names a URL / file / document / person;
   this skill fetches and extracts the answer (firecrawl / Read), recording
   `provenance: source` and the `source_ref`.
3. **"Research it for me"** — ALWAYS present. Runs a bounded research loop seeded
   by the request `hints[]`, returns answer + confidence + evidence
   (`provenance: research`). Budget: `max_sources: 6`, `max_minutes: 10`; on a
   cap, return partial evidence at low confidence rather than spin.
4. **"Decide for me (implicit)"** — records the chosen default + rationale
   (`provenance: implicit`) so the implicit decision is explicit and audited.

## Modes

- **Inline-fallback** (current): option 3 runs in-session under the same budget.
- **Child-isolated** (arrives in the child-loops phase): option 3 spawns a
  read-only scoped child loop returning `result.json`. Callers see the identical
  contract; only isolation differs.

## Progress Signals (MANDATORY)

Before any other action, emit:

```
Starting pmpo-elicit — <short question>
```

When the answer is resolved and `result.json` is written, emit:

```
Completed pmpo-elicit — <short question> (provenance: <user|source|research|implicit>)
```

Emit to plain response text — no tool call needed.

## How to invoke

1. **Read/construct the request** — `{question, context, hints[], criticality,
   caller, write_back_path}` (schema:
   `references/schemas/elicitation.schema.json`). Write
   `elicitations/<id>/request.json` under the caller's state dir.
2. **Present the four option classes** via `AskUserQuestion`, with option 3
   ("research it for me") and option 4 ("decide for me") always included.
3. **Resolve** per the chosen option:
   - source → fetch + extract, record `source_ref`.
   - research → bounded loop (max 6 sources / 10 min), record evidence +
     confidence; stop at the cap with partial results.
   - implicit → record the default + rationale.
4. **Write `elicitations/<id>/result.json`** with `provenance`, `confidence`,
   `evidence`, and `cost`.
5. **Return** the result path to the caller, which applies the answer and
   records `elicitation_id`.

## Examples

```
/pmpo-elicit "Which session store backend?" --hints "axum;redis-or-pg" --criticality high --caller kbd-analyze
/pmpo-elicit "Target Node version?" --criticality blocking --caller kbd-spec
```

## Budget & termination guards

The research option never loops unbounded: max 6 sources, max 10 minutes, then
returns what it has at lowered confidence. This mirrors the proven
2-rejection sycophancy-gate cap — bounded effort over unbounded thoroughness.
