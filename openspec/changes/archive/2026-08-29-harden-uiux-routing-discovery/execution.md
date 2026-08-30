# Execution Evidence: harden-uiux-routing-discovery

## Prerequisite boundary correction

Before task 1 began, the bottleneck guard correctly rejected the bare numeric subject `1` because task ordinals repeat across OpenSpec changes. The KBD apply adapter now evaluates task boundaries by canonical task title; the guard still folds the resolved title into typed phase/change/task receipt fields. The transient ambiguity blocker was cleared through a signed typed command, all six tasks were registered before the first boundary, and task 1 started at canonical revision 167 with an accurate denominator.

## Task 1.1 — Existing-target-first routing

Updated the managed `uiux-routing` template so bounded UI/UX context has an explicit authority boundary before Impeccable runs:

- The workflow must name a file, route, component, or surface and confirm that it exists.
- When a proposed destination is a future absent path, the agent must locate the incumbent implementing surface and record an explicit `Context target` to `Planned destination` mapping.
- The incumbent is context authority only; the approved specification and plan continue to own the eventual destination, preventing discovery from silently changing scope.
- If no incumbent exists, the unresolved state must be recorded and the workflow cannot claim a concrete Impeccable analysis.
- UI/UX Pro Max and Impeccable now operate on the resolved existing target, and Impeccable context loading is explicitly ordered after target resolution.

The managed-fence rendering and idempotency integration scenarios remain deferred until tasks 2.1 and 2.2, after the coherent template, capability, roster, and documentation implementation is complete, as required by the repository's implementation-first policy.

## Task 1.2 — Capability-aware UX review

Removed the unconditional `frontend-design + ux-designer` attribution from the managed routing template and replaced it with an explicit active-catalog decision:

- Named review skills are consulted only when installed.
- Installed `ux-designer` use must record the source actually reported by the catalog rather than inferring a vendor from the name.
- Missing `ux-designer` selects the named `UI/UX Pro Max + frontend-design` fallback; if `frontend-design` is also absent, the workflow truthfully records the reduced UI/UX Pro Max fallback.
- Missing optional capabilities are not reported as unfinished requirements.
- The default roster now classifies `ux-designer` as optional and community/project-provided, requires an installed catalog entry, and explicitly forbids inferred Anthropic provenance.

The project has no local roster override, so the corrected default roster is the effective source for this repository. Rendering and catalog-branch integration assertions remain deferred until tasks 2.1 and 2.2 under the implementation-first policy.

## Task 1.3 — Injector documentation

Updated the injector skill contract to match the implemented command surface:

- Documented `--target CLAUDE.md|AGENTS.md|both`, `--path <root>`,
  `--pack agent-rules|uiux-routing`, `--refresh`, `--dry-run`, and
  `-h|--help`, including their actual defaults.
- Documented how each pack resolves its template, cache, and marker prefix,
  including the project-local UI/UX roster override.
- Defined the `uiux-routing` pack's existing-target-first Impeccable boundary
  and capability-aware `ux-designer` fallback behavior.
- Corrected the dry-run guarantee to cover target files specifically; when
  combined with `--refresh`, the selected cache date can still change because
  cache refresh precedes target rendering in the script.
- Replaced stale single-pack reference paths with the actual pack-specific
  template and cache paths plus the implemented legacy fallbacks.

Static comparison against the script confirms the documented accepted values,
defaults, refusal cases, and refresh/dry-run ordering. Full integration
verification remains deferred to tasks 2.1 through 2.3.

## Task 2.1 — Managed-fence integration

Extended the production-entry-point integration harness with a dedicated
`uiux-routing` refresh scenario. The scenario constructs a real project
`CLAUDE.md` containing exact prefix and suffix fixtures (including tabs and
trailing spaces), replaces a stale managed fence through the real injector,
and compares the complete result against `prefix + production template +
suffix`. It then runs the production injector a second time and requires the
result to be byte-identical to the first-run snapshot.

The signed integration gate was attempted first:

```text
prometheus kbd gate run --kind integration --scope harden-uiux-routing-discovery:managed-fence -- /bin/bash skills/process/kbd-process-orchestrator/shared/lib/tests/test-agent-rules-injector.sh
BLOCKED at revision 184: implementation is incomplete for the active phase
```

That policy evaluates completion at the whole active-phase boundary, so the
gate receipt remains deferred until all phase implementation is complete. The
same bounded local full-integration command was then run directly for this
change-level task:

```text
/bin/bash skills/process/kbd-process-orchestrator/shared/lib/tests/test-agent-rules-injector.sh
PASS: 14/14 full-integration scenarios
```

The new scenario passed both exact surrounding-byte preservation and
second-run idempotence without mocks, hosted CI, or a Rust build.

## Task 2.2 — Isolated dry-run and incumbent UAR context

Ran the updated `uiux-routing` pack in `--dry-run` mode against an isolated
temporary `CLAUDE.md`. The target remained byte-identical before and after
(`5db236adbdf021082c517c247aec1b45beb9b4a8826ec1f7170cc4fa76362f98`),
while the proposed diff contained all three required routing signals:

- resolve an existing context target before bounded context loading;
- consult `ux-designer` only when that exact capability is installed; and
- record the named `UI/UX Pro Max + frontend-design` fallback when absent.

The UAR-local Impeccable 4.1.1 adapter was then loaded read-only with the
explicit incumbent source target:

```text
Context target: frontend/src/features/skills/ui/a2ui-library-page.tsx
Current route: /admin/skills/a2ui
Planned destination: frontend/src/features/presentation/
Planned route: /admin/presentation
```

Impeccable reported `targetExists: true`, `hasVisualImplementation: true`, and
resolved the owning project root to UAR's `frontend/`. No `PRODUCT.md`,
`DESIGN.md`, or surface brief overrides the incumbent implementation, so the
existing A2UI page, its Skills shell, and `frontend/src/shared/theme/tokens.css`
are the read-only visual authority. The development-only
`/admin/a2ui-testing` route remains a separate protocol surface and was not
selected. The planned `frontend/src/features/presentation/` directory does not
yet exist, so it was retained solely as the approved future destination.

The pre-existing UAR worktree was dirty before inspection. Its complete
porcelain-status digest was captured before Impeccable context loading and
again after all UAR reads:

```text
before: 6bd1c0f8fdecd3803df7c89fbf4e24b952a95c4b558081740baa85db6e7e3e44
after:  6bd1c0f8fdecd3803df7c89fbf4e24b952a95c4b558081740baa85db6e7e3e44
```

The incumbent target itself resolved to SHA-256
`e974bd90649f0d19073bedcfe6f2bba1ab121844d531c655b482f3268601d598`.
No UAR files, status entries, builds, tests, hooks, detector state, or visual
artifacts were changed.

## Task 2.3 — Strict local validation

Ran the final source-tree gates locally after implementation and integration
evidence were complete:

```text
npm run validate:strict -- skills/process/kbd-process-orchestrator
PASS: 23 skills validated, 0 errors, 42 pre-existing description-quality warnings

git diff --check
PASS: no whitespace or conflict-marker errors
```

The warnings concern missing trigger/exclusion prose across longstanding KBD
skills and do not contradict this change's routing contract. No hosted CI,
Rust build, unit-test command, or generated-distribution refresh was run.
Distribution regeneration and installed-surface parity remain explicitly owned
by the later parent change `reconcile-kbd-control-plane-projections` so all
source changes are batched before that expensive gate.

## Artifact-refiner QA

The installed artifact-refiner adapter lacked its delegated canonical
controllers and schemas, so KBD used the repository's persisted deterministic
fallback at
`.refiner/artifacts/harden-uiux-routing-discovery/refinement_log.md`.

The first QA iteration found one blocking mismatch: `--dry-run` created an
empty target when the requested file did not exist, contradicting the documented
target-file immutability guarantee. The injector now renders missing targets
against `/dev/null` during preview, and integration scenario 7 requires that an
absent target remain absent.

After refinement:

```text
/bin/bash -n <injector> <integration-harness>
PASS

/bin/bash skills/process/kbd-process-orchestrator/shared/lib/tests/test-agent-rules-injector.sh
PASS: 14/14 full-integration scenarios

npm run validate:strict -- skills/process/kbd-process-orchestrator
PASS: 23 skills, 0 errors, 42 non-blocking pre-existing warnings

git diff --check
PASS

focused sensitive-material scan
PASS: no private key or credential-like assignment
```

Artifact-refiner verdict: **PASS after one corrective iteration**. Generated
distribution and installed-surface certification remains assigned to
`reconcile-kbd-control-plane-projections`.

## Adversarial review remediation

The first distinct-model diff review returned BLOCK with two critical findings,
five warnings, and one suggestion. Sycophancy correction passed with score
`0.01785714365541935`, so the findings were handled without regeneration.

The actionable findings were resolved as follows:

- The detector replayed every writable derived projection and preserved the
  canonical audit trail. Its review-time retry could not reconstruct the
  historical task-6 start receipt, so it correctly recorded blockers instead
  of inventing evidence. Canonical task transitions at revisions 193 and 195
  prove start/completion; the two retry blockers were cleared using typed
  commands at revisions 201 and 202 with the missing receipt retained in
  history. There are now zero unresolved blockers. `/kbd-apply
  harden-uiux-routing-discovery` remains the correct resumable command until
  required review, verification, and archival finish.
- The installed-service memory scenario is now an explicit
  `KBD_MEMORY_LIVE_PROBE=1` integration probe. The default full-integration
  suite is hermetic; the live local service path remains separately certifiable.
- Curl no-proxy matching now uses the bare IPv6 loopback token `::1` across
  detection, logging, and recall.
- Recall now distinguishes a transport failure from a reachable entity-search
  HTTP error and from an invalid successful response; each writes a different
  atomic diagnostic stub.
- The KBD apply comment now states the real contract: titles are canonically
  resolved and duplicates fail closed. Composite subject support remains a
  separate runtime enhancement rather than an unsupported assertion.

Two warnings were disproved by direct source inspection: `end-task` assigns
`title="$*"` before guard evaluation, and memory logging exits when the
resolved REST URL is empty before invoking curl. The cumulative uncommitted
memory diff already has its own archived OpenSpec evidence and artifact QA.
Distribution refresh remains deliberately batched in
`reconcile-kbd-control-plane-projections`.

Post-remediation local evidence:

```text
/bin/bash -n <all touched memory, injector, KBD apply, and integration scripts>
PASS

/bin/bash skills/process/kbd-process-orchestrator/shared/lib/tests/test-memory.sh
PASS: 10/10 enabled hermetic full-integration scenarios; live probe explicitly skipped

KBD_MEMORY_LIVE_PROBE=1 /bin/bash skills/process/kbd-process-orchestrator/shared/lib/tests/test-memory.sh
PASS: 11/11 enabled full-integration scenarios, including the installed local service

/bin/bash skills/process/kbd-process-orchestrator/shared/lib/tests/test-agent-rules-injector.sh
PASS: 14/14 full-integration scenarios

npm run validate:strict -- skills/process/kbd-process-orchestrator
PASS: 23 skills, 0 errors, 42 pre-existing description-quality warnings

git diff --check
PASS
```

The final non-recursive adversarial packet received `PASS` from distinct judge
model `k3` against producer `gpt-5.6-sol`, with
`cross_model_check: verified-distinct` and sycophancy score `0.0`. It reported
zero critical findings, three warnings, and zero suggestions. The final two
hardening edits clarify C-01's parent-phase certification boundary and make the
source diff packet fail closed on missing identifiers or leaked review receipts.
The canonical next command now points at `add-kbd-registry-prune`; verification
and archival below close this change before that command is executed.
