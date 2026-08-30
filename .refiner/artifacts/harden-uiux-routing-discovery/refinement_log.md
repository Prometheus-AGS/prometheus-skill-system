# Refinement log — harden-uiux-routing-discovery

Artifact-refiner deterministic QA for the completed existing-target-first,
capability-aware UI/UX routing change.

The installed artifact-refiner adapter contains only its entry `SKILL.md` and
omits the canonical controllers, domain adapters, and schemas it delegates to.
KBD therefore used its documented local fallback contract: inspect the complete
change diff against `.kbd-orchestrator/constraints.md`, persist every finding
and refinement here, then require local integration and strict validation before
OpenSpec verification and archival.

## Specification

- Artifact type: skill code, managed content, documentation, and integration evidence
- Content type: `direct:code`
- Target: the `uiux-routing` pack resolves an existing incumbent surface before
  Impeccable context loading, treats future paths only as destinations, and
  selects UX-review skills from the installed catalog without invented vendor
  provenance.
- Blocking criteria: truthful target/capability semantics; exact managed-fence
  preservation; second-run idempotence; dry-run target immutability; accurate
  documentation; read-only UAR evidence; strict local validation.

## Changed surfaces

- `skills/kbd-inject-agent-rules/references/template-uiux-routing.md` —
  existing-target-first context authority and capability-aware fallback.
- `skills/kbd-inject-agent-rules/references/cache-uiux-routing.md` — optional,
  catalog-sourced `ux-designer` provenance.
- `skills/kbd-inject-agent-rules/SKILL.md` — pack-aware behavior, flags,
  defaults, references, and dry-run semantics.
- `skills/kbd-inject-agent-rules/kbd-inject-agent-rules.sh` — truly read-only
  previews for missing targets.
- `shared/lib/tests/test-agent-rules-injector.sh` — 14 production-entry-point
  filesystem scenarios, including exact UI/UX fence preservation,
  idempotence, and missing-target dry-run behavior.
- `skills/kbd-apply/kbd-apply.sh` — prerequisite canonical-title boundary
  subjects so repeated OpenSpec numeric task IDs do not corrupt receipts.
- OpenSpec task and execution evidence for all six tasks.

## Constraint check (`.kbd-orchestrator/constraints.md`)

| Constraint | Status | Evidence |
|---|---|---|
| C-01 generated artifacts in sync | IN-PHASE BATCH / OPEN | The parent change `reconcile-kbd-control-plane-projections` is the named owner for deterministic double generation, tracked-hash identity, `validate:codex`, and installed-surface parity before parent-phase certification or commit/push. This change claims source validation only, not distribution certification. |
| C-02 no committed secrets | PASS | Focused scan found no private-key material or credential-like assignments in change surfaces. Templates contain only skill names, local paths, and public source references. |
| C-03 docs updated with surface changes | PASS / N/A | The injector skill documentation now matches both packs and the real flag/default contract. No plugin manifest, marketplace, MCP, hook registration, or installation surface changed. |
| C-04 generators stay idempotent | N/A | Generator behavior did not change. Managed-fence execution itself is byte-idempotent and covered by the real injector integration suite. |
| C-05 Bash 3.2 compatibility | PASS | `/bin/bash -n` passed for the injector and its integration harness; neither changed script is launchd-reachable. The full suite executed under `/bin/bash` successfully. |

## Refinement iterations

### Iteration 1 — BLOCKING finding corrected

Diff inspection found that documentation promised `--dry-run` would not write
target files, but the production script created an empty file when the target
was absent before rendering the preview. An isolated production probe confirmed
the mismatch.

Refinement:

- missing targets now use `/dev/null` as the preview source;
- normal non-dry-run creation behavior is unchanged;
- integration scenario 7 now proves an existing target remains byte-identical
  and an absent target remains absent.

### Iteration 2 — PASS

- `/bin/bash -n` passed for both touched shell scripts.
- `/bin/bash .../test-agent-rules-injector.sh` passed 14/14 full-integration
  scenarios after the refinement.
- `npm run validate:strict -- skills/process/kbd-process-orchestrator` passed
  23 skills with zero errors. Its 42 longstanding trigger/exclusion warnings
  are non-blocking and outside this routing change.
- `git diff --check` passed after the refinement and evidence updates.
- The temporary dry-run target retained identical SHA-256 before and after.
- UAR-local Impeccable resolved the real incumbent A2UI page, and the UAR
  porcelain-status digest remained byte-identical before and after all reads.

### Iteration 3 — PASS after adversarial remediation

The first distinct-model adversarial pass found a stale-waypoint concern, a
non-hermetic installed-memory-service dependency, an overstatement about task
title uniqueness, and an IPv6 no-proxy mismatch. Deterministic refinement:

- replayed derived KBD projections through the detector without manual edits;
- retained the missing historical boundary receipt as explicit audit evidence
  and cleared only its typed review-time blockers after verifying canonical
  task transitions at revisions 193 and 195;
- made the installed-service scenario an explicit live integration probe while
  preserving the hermetic production-entry-point suite;
- replaced bracketed `[::1]` no-proxy entries with libcurl's bare `::1` host
  token; and
- separated entity-search transport failure, HTTP failure, and invalid-response
  stubs; and
- corrected the KBD apply comment to document fail-closed duplicate-title
  resolution.

The hermetic memory suite passed 10/10, the explicit live suite passed 11/11, the
injector suite passed 14/14, Bash syntax checks passed, strict skill validation
passed 23 skills with zero errors, and `git diff --check` passed. Source
inspection disproved the review claims that `title` can be uninitialized and
that memory logging curls an empty URL.

## Verdict

PASS — the dry-run mismatch and every actionable adversarial finding were
corrected. The final implementation, documentation, integration evidence, and
KBD constraints now agree. Proceed to the bounded adversarial re-review,
OpenSpec verification, and archival; retain generated distribution and
installed-surface certification for `reconcile-kbd-control-plane-projections`
as planned.
