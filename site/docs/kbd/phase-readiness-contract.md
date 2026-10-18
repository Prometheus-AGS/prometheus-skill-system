---
title: Phase readiness contract
description: Adopted exit contracts, the Plan→Execute readiness gate, evidence-bound phase closure, and budget accounting for KBD phases.
---

# Phase readiness contract

The phase readiness contract makes phase completion an explicit, auditable
decision instead of an impression. A KBD phase adopts a versioned,
content-digested exit contract **before** execution; the Plan→Execute gate
refuses to dispatch work against a phase that has no valid adopted contract;
and a terminal evaluator closes the phase only when every adopted scenario has
valid PASS evidence bound to the current run and the adopted contract digest.

This page documents the shipped behavior, installation, a practical example,
and the exact limits of what is enforced.

## The planning contract

A phase contract is a JSON document at
`.kbd-orchestrator/phases/<phase>/adopted-contract.json` declaring:

- **Identity** — project, run, and phase IDs the contract is bound to.
- **Goal and functional scope** — what the phase must deliver, plus explicit
  exclusions.
- **Scenario matrix** — the acceptance surface. Every scenario names a
  production entry point, its real collaborators, an observable oracle, and at
  least one negative control, plus the task that owns it.
- **Budgets** — declared ceilings for time, tool calls, repair attempts, and an
  absolute no-progress limit. Token budgets may be `UNKNOWN`; the declared
  fallback (time + tool-call bounds) governs in that case.
- **Verification packet** — the post-implementation integration commands,
  expected effects, receipts, and stop conditions.

Contracts are adopted through the canonical writer,
`shared/scripts/kbd-contract-adopt.sh`, which records the adoption as an
immutable canonical decision (visible in `prometheus kbd audit`) keyed to the
contract's SHA-256 digest. Material changes require a new adoption that
supersedes the previous one (`--supersedes`), so the full lineage stays in
history. Re-adopting byte-identical content is an idempotent no-op
(`ALREADY-ADOPTED`); history is never rewritten.

## The readiness gate

`shared/scripts/kbd-phase-readiness-check.sh` is the read-only checker. It
verifies, in order:

1. an adoption decision exists and is unambiguous for the contract path,
2. the contract identity matches the current project/run/phase,
3. the recorded digest still matches the recomputed digest (a drifted contract
   is `REJECT-STALE`, exit 4 — fix by re-adopting, never by editing history),
4. every scenario's owning task exists in the phase `tasks.json`,
5. contract quality: the scenario matrix has real production entries, oracles,
   and negative controls; budgets are declared with a token fallback; a
   verification packet exists.

The checker exits `0 READY` only when all of these hold (`2 BLOCKED`,
`3 REJECT-IDENTITY`, `5 MALFORMED` otherwise). The Plan→Execute stage gate in
`skills/process/kbd-process-orchestrator/shared/lib/stage-gate.sh` runs this
same validation and refuses dispatch when it fails — and it refuses to dispatch
any phase that is already closed.

## Execution flow

1. Create and activate the phase; write its goals.
2. Author the contract and adopt it with the canonical writer.
3. Confirm `READY` with the checker.
4. Execute the phase's tasks. Implementation completes before verification:
   the boundary packet runs once, after production changes are coherent.
5. Collect scenario evidence keyed to the adopted contract digest and current
   run.

## Evidence-bound closure

`shared/scripts/kbd-phase-close-eval.sh` is the terminal evaluator. It closes a
phase only when the evidence packet shows every adopted scenario PASS, with:

- evidence bound to the **current run** and the **adopted contract digest**
  (evidence from an older contract or a different run is not inherited);
- source freshness recomputed at evaluation time;
- budget accounting present and within the declared ceilings (see below).

On success it records `PHASE-CLOSED` as a canonical decision bound to
phase + run + contract digest + evidence digest. A replay of an unchanged,
still-valid closure is an idempotent `ALREADY-CLOSED` no-op. On unmet or
unverifiable criteria it records `PHASE-CLOSURE-REFUSED` with `FAIL` (a real
oracle is unmet) or `BLOCKED` (the verification itself cannot run) and stops —
it never invents follow-up work and never converts a failure into a pass.

### Budget accounting and the missing-accounting rule

The evidence packet must carry a `budgetConsumption` header naming its source
and classing each value as `measured`, `estimate`, or `unknown`. The evaluator
compares self-reported consumption against the latest extension's
`resultingCeilings`:

- **exceeded** → closure refused;
- **at ceiling** → closure proceeds with an exhaustion note;
- **unknown class** → that dimension is unenforced;
- **missing header** → **`BLOCKED` (`budget-consumption-missing`)**. This is
  unambiguous current behavior: absent accounting is a refusal, not a waived
  check. Historical closures recorded before this rule remain labelled in
  history and do not claim budget satisfaction.

Budgets are cumulative across extensions and never self-renew. Amendments must
declare each field as add/replace/unchanged. The no-progress limit is an
absolute value, not additive. Exhaustion checkpoints and reports `BLOCKED`
truthfully; work completed within limits is not invalidated by consuming the
last permitted unit.

## Installation

The checker, adoption writer, evaluator, and gate ship in both distribution
payloads (`dist/plugins/{claude,codex}/prometheus-skill-pack`):

```bash
node scripts/install.js --scope user          # per-user install
node scripts/generate-skill-system-distribution.js --check   # payload consistency
```

Set `HOME`/`XDG_DATA_HOME` to isolate a scratch install. The installed payload
carries identical file digests to the source tree for these scripts.

## A practical example

```bash
# 1. adopt the phase contract (canonical, digest-bound)
bash shared/scripts/kbd-contract-adopt.sh \
  .kbd-orchestrator/phases/my-phase/adopted-contract.json
# → ADOPTED path=… sha256=… planRevision=… decision=contract-adoption-…

# 2. confirm readiness before Plan→Execute
bash shared/scripts/kbd-phase-readiness-check.sh --phase my-phase \
  --phase-dir .kbd-orchestrator/phases/my-phase
# → READY phase=my-phase contract=… digest=… planRevision=…

# 3. execute the phase's tasks (implementation first), collect evidence

# 4. close with an evidence packet including budgetConsumption
bash shared/scripts/kbd-phase-close-eval.sh --phase my-phase \
  --evidence-file evidence.json
# → CLOSED phase=my-phase verdict=PASS evidence=sha256:… entries=N
# a byte-identical replay → ALREADY-CLOSED (zero new records)
```

If `evidence.json` omitted `budgetConsumption`, step 4 would instead print
`BLOCKED … reason=budget-consumption-missing` and record a refusal.

## Limits and disclosures

Read these before relying on the mechanism:

- **Shell-layer enforcement.** The gate and evaluator are shell conventions
  invoked through the orchestrator skill flow. The compiled `prometheus kbd`
  CLI does **not** itself invoke them, and direct CLI or state manipulation
  bypasses them entirely. This does not establish automatic enforcement inside
  every harness.
- **Disclosed test seams.** `KBD_READINESS_STATUS_FILE`, `--status-file`, and
  `--decision-sink` override where status is read and decisions are written.
  They exist for fixtures and are visible in every receipt that uses them.
- **Self-reported accounting.** Budget consumption values are agent-authored
  self-reports, not independent telemetry and not tamper-proof. The budget
  predicate proves only the declared-limit comparison and the refuse/admit
  behavior.
- **Demonstration scope.** The installed-payload lifecycle was demonstrated on
  a scratch install (readiness READY → gate pass → CLOSED with sourced
  accounting → missing-accounting BLOCKED → bound replay ALREADY-CLOSED). That
  qualifies the installed shell workflow as demonstrated; host-side automatic
  enforcement (#185) and the broader #184 successor scope remain open future
  work.
- **Local-only validation.** All checks run locally. Hosted CI output is never
  acceptance evidence.

## Evidence and history

The candidate qualification for this feature, including the full scenario
receipts (RC-01–RC-07), boundary results, and reconciliation records, is
preserved at
[`docs/releases/1.12.2-candidate-qualification.md`](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/docs/releases/1.12.2-candidate-qualification.md)
in the repository. Issue [#184](https://github.com/Prometheus-AGS/prometheus-skill-system/issues/184)
remains open for the separately scoped successor corpus.
