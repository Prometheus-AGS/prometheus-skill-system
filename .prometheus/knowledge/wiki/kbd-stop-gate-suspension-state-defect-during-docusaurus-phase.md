---
type: Reference
id: kbd-stop-gate-suspension-state-defect-during-docusaurus-phase
title: KBD Stop-Gate Suspension State Defect During Docusaurus Phase
tags:
- kbd-lifecycle
- stop-gate
- docusaurus
- github-pages
- state-machine
- session-parking
links:
- docusaurus-github-pages-site-phase-pull-summary
- docusaurus-github-pages-site-executor-completion
- docusaurus-github-pages-site-reflect-completion-at-2026-07-28
sources:
- stdin
- manual:docusaurus-github-pages-site
- https://claude.ai/code/artifact/4f30ad22-6e5a-4d70-8a0a-9089ad742291
- scratchpad/kbd-interruption-defect.md
timestamp: 2026-07-28T13:07:58.608803+00:00
created_at: 2026-07-28T13:07:58.608803+00:00
updated_at: 2026-07-28T13:07:58.608803+00:00
revision: 0
---

## Context

- **Phase:** `docusaurus-github-pages-site`
- **KBD root:** `/Users/gqadonis/Projects/prometheus/prometheus-skill-pack`
- **Captured:** `2026-07-28T13:07:03Z`
- **Source context:** `manual:docusaurus-github-pages-site`
- **Artifact:** `https://claude.ai/code/artifact/4f30ad22-6e5a-4d70-8a0a-9089ad742291`
- **Markdown source:** `scratchpad/kbd-interruption-defect.md`

## Phase Goals

The phase aimed to:

- Stand up a Docusaurus documentation site for the skill-pack documentation:
  - skills catalog
  - KBD lifecycle
  - learn domain
  - substrate crates
- Deploy the site to GitHub Pages through GitHub Actions on pushes to `main`.
- Migrate or link existing documentation from `docs/`, `README`, and `CLAUDE.md`-derived guides into the site without duplicating canonical sources.

Related phase tracking and completion records include [Docusaurus GitHub Pages Site Phase Pull Summary](/docusaurus-github-pages-site-phase-pull-summary.md), [Docusaurus GitHub Pages Site Executor Completion](/docusaurus-github-pages-site-executor-completion.md), and [Docusaurus GitHub Pages Site Reflect Completion at 2026-07-28](/docusaurus-github-pages-site-reflect-completion-at-2026-07-28.md).

## Stop-Gate Defect Summary

The central defect is not primarily a regex bug; it is a missing lifecycle state. The model distinguishes only between:

- `in progress`
- terminal states

It has no vocabulary for work that is **unfinished on purpose**. As a result, a deliberately suspended phase is indistinguishable from an agent quitting early. The stop-gate was designed to prevent early abandonment, so it correctly blocks the session, but it also forbids intentional suspension.

## Compounding Causes

### A. Architectural: no paused/suspended state

`_wr_is_terminal_status` has no `paused` or `suspended` vocabulary. This is the durable root cause: the state machine cannot represent a parked but intentionally unfinished waypoint.

### B. Detection: overly broad completion matching

The detector matches bare completion terms such as:

```text
done|complete|finished
```

Because these terms match anywhere in prose, the block message can contain the same words and cause the gate to re-fire on its own explanation.

### C. Retry cap cannot deduplicate

`CAP_KEY` includes the transcript byte size. Since transcript size grows on every turn, the deduplication key changes every turn. This violates the intended contract:

```text
one enforced retry, never a loop
```

The deduplication mechanism therefore cannot reliably prevent repeated blocks.

## Ordered Fixes

1. **Add `_wr_is_suspended_status`**
   - Durable architectural fix.
   - Allows the lifecycle model to distinguish intentional suspension from accidental early termination.

2. **Add `/kbd-pause` and `/kbd-resume` commands**
   - Provides an explicit interface for suspension and resumption.
   - Captures why the plan changed rather than relying on implicit status edits.

3. **Repair regex and cap behavior**
   - Tighten completion detection so ordinary prose and gate explanations do not retrigger the gate.
   - Remove transcript byte size from the retry cap key or otherwise stabilize the deduplication identity.
   - Restore the intended “one enforced retry, never a loop” behavior.

4. **Support an unconditional `PAUSE` file**
   - Provides an escape hatch when the waypoint itself is defective.
   - This case needed it because the waypoint recorded `7/29` while the driver reported `17/29`.

## Workaround Risks

Setting a waypoint to `status: complete` silences the gate, but it corrupts progress reporting when the phase is not complete. In this case, the phase was at `17/29`, so marking it complete would misrepresent remaining work.

A fresh session scoped to `prometheus-skill-pack` may hit the same gate if that repository has a non-terminal waypoint. Check repository waypoint status before starting the stop-gate fix session.

## Recorded Position

```text
Position: web-scaffold-and-c29-gate › letter-agreement-c29-gate | status: apply_ready
Progress: changes 4/7
Last: letter-agreement-c29-gate 17/29 — §1–4 done; §5–7 remain. Defect report written for the prometheus-skill-pack stop-gate
Next: fix the stop-gate in a fresh session scoped to prometheus-skill-pack; this session stays parked with the open flint-forge decision
```

# Citations

1. stdin
2. manual:docusaurus-github-pages-site
3. https://claude.ai/code/artifact/4f30ad22-6e5a-4d70-8a0a-9089ad742291
4. scratchpad/kbd-interruption-defect.md