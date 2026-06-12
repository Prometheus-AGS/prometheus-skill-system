---
id: change-003-nesting-docs
title: Document enter/descend verb + selected-vs-entered invariant
phase: outer-loop-and-ux
gaps: [U3]
priority: P2
effort: S
agent: claude-code
evolver_item_id: null
status: proposed
scope:
  - skills/process/kbd-process-orchestrator/SKILL.md
  - skills/process/kbd-process-orchestrator/references/schemas/current-waypoint.template.json
---

# change-003 — Nesting docs

## Context

Arbitrary-depth child loops work, but the descent verb (kbd-child-exit --enter)
and the selected-vs-entered invariant (path[] tail == childPointer → sibling
add; pointer cleared → nest) are implicit in two scripts. External tools
manipulating path[] need this documented.

## Scope

In:

- SKILL.md "Nested phases" section: document
  - the enter/descend operation (`/kbd-child-exit --enter`) and its pairing
    with `/kbd-new-child` / `/kbd-next-child` / `/kbd-child-exit`;
  - the selected-vs-entered invariant and what each state means for the next
    `/kbd-new-child`;
  - a note that descent = set path[] + clear childPointer.
- Add `/kbd-child-exit` (+ `--enter`) to Quick Start Commands.
- current-waypoint.template.json: a `__note` or comment on `path`/`childPointer`
  stating the invariant.
- Also document (one line) that scope-guard ask-flip is HELD pending live hook
  verification (records the deferred future step).

## Tasks

- [ ] 1. SKILL.md Nested phases: enter/descend + invariant + Quick Start entry
- [ ] 2. waypoint template note on the invariant
- [ ] 3. validate:strict still clean

## Verification

validate:strict clean; the invariant and enter verb are findable in SKILL.md.
