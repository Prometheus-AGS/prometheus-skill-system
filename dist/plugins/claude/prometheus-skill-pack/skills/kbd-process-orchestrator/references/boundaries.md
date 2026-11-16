# Boundaries (B0–B6) and the value-phase boundary sequence

This is the **only** definition of a gate, checkpoint or boundary in the pack.
Every skill, prompt, integration note and harness protocol that mentions one links
here and defines none of its own. VLBV is the method for every phase (see
`method-context.md`); a signed `KBD-METHOD` deviation may name another execution
cadence for one phase, and the closure packet discloses it.

Generated code executes at **B4**, once, and — only after an authorized repair —
at **B5**. Nowhere else. Editor/LSP diagnostics read while writing code are static
reading, not execution.

| ID | Boundary | Trigger | Allowed | Not allowed |
|---|---|---|---|---|
| B0 | Edit batch | Any edit | Reading, static reasoning, editor/LSP diagnostics | Any execution of generated code: tests, builds, compiler checks, imports, lint, typecheck, probes |
| B1 | Task end | `kbd-apply end-task` | Journal the transition | QA, review, tests, build |
| B2 | Change end | Last task of a change | Record implementation complete; `openspec validate --strict` (a static check of spec documents only) | Refiner, adversarial review, OpenSpec verify, archive, tests, build |
| B3 | Checkpoint | Cadence tick, time limit, compaction, pause, harness handoff | Save state, stop admitting new scope, write the handoff | Any execution; refreshing the pack or OpenSpec |
| B4 | Value-phase boundary | All planned production complete, contract amended to `implementationComplete`, separate current authority | The boundary packet, once, in order (below) | Commands outside the packet; retries |
| B5 | Repair boundary | B4 produced FAIL and the owner authorizes a bounded repair within `budgets.repairAttempts` | Complete the agreed repair, then rerun only the affected scenarios and their negative controls; invalidate affected evidence | New scope; weakening an oracle without a signed amendment |
| B6 | Release and publication | Phase CLOSED and release authority given | Delivery-cadence publication procedures | Treating publication as acceptance |

## The B4 sequence

One normative sequence. `kbd-execute`, `value-scenario-verify`, `delivery-cadence`,
`kbd-reflect` and `cross-tool-protocol.md` reference it; none restates it differently.

1. **Admit.** `kbd-phase-readiness-check.sh` confirms the latest non-superseded
   contract digest, complete production, method resolution and current boundary
   authority.
2. **Freeze the candidate.** Record source, build, configuration, fixture and
   reference identities. Delivery-cadence `candidate freeze` *is* this step.
3. **Run the boundary packet once, in order.** Commands are typed `build`, `run`,
   `scenario` and `negativeControl` (contract v2; v1 contracts list them as
   strings). The cadence checkpoint's build and functional run are the packet's
   `build` and `run` entries, not a separate execution.
4. **Record.** Write `boundary-results.json` with one verdict per scenario —
   `PASS`, `FAIL`, `BLOCKED` or `NOT_EXERCISED` — and `executedBy`
   (`harness`, `version`, `model`). Keep the original runner output beside it.
5. **Cumulative static QA and adversarial review.** One artifact-refiner pass and
   one cross-model adversarial review over the whole phase diff. The refiner
   triggers no build.
6. **Close.** `kbd-phase-close-eval.sh` evaluates against the bound contract and
   writes the closure object and `closure-packet.md`.
7. **After CLOSED only.** OpenSpec verify and bulk archive for every change in the
   phase (archiving earlier would write unverified behavior into `openspec/specs/`).
8. **Reflect.** Plan-versus-outcome differences and contract-accuracy metrics
   (`kbd-reflect`).

If step 3 yields any FAIL, steps 5–7 do not run: the owner either authorizes B5 or
closes the phase unmet. A later green receipt never overwrites an earlier failure.

## Closure states

The four states are separate and never inferred from each other:
`implementationComplete`, `behaviorAccepted` (B4/B5 passed every mandatory
scenario), `releaseEligible`, `releaseAuthorized` (a separate owner decision).

## Control level

The rules on this page are **instruction-only** on every harness. Contract digest,
staleness and amendment are enforced for callers of the canonical commands
(`kbd-contract-adopt.sh`, `kbd-phase-readiness-check.sh`, `kbd-phase-close-eval.sh`).
No tool interception is provided or claimed.
