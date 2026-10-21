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

Closure has **one authority**: the typed canonical runtime (issue #192).
`prometheus kbd phase close` records a phase disposition bound to project,
run, phase, the **bound contract digest** and the evidence digest, and a PASS
moves the phase to `complete` in the same event. Every reader — the terminal
evaluator, the Plan→Execute gate, Reflect, child-exit and next-phase — reads
`prometheus kbd phase disposition`, which reports one token:
`UNBOUND`, `OPEN`, `CLOSED`, `REFUSED_BLOCKED` or `REFUSED_FAIL`. Decision
summary text is never interpreted as closure, so a recorded string cannot fake
or hide one.

Adoption binds the phase (`prometheus kbd phase bind-contract`) and writes a
byte-exact snapshot `phases/<id>/adopted/<digest>.json` (its sha256 is the
digest). From then on the runtime refuses `phase transition --status
complete|cancelled` for that phase: it closes only through a PASS disposition.

`shared/scripts/kbd-phase-close-eval.sh` is the terminal evaluator. It reads
the required scenarios and budget ceilings from the **bound snapshot**, never
from the mutable working contract, and records through `phase close`:

- every bound scenario must have PASS evidence (an entry id `S` or `S-<sub>`);
  evidence entries for scenarios outside the bound contract are reported as
  `ignored-unadopted` and change nothing;
- evidence must name the current project, run and bound digest;
- source freshness is recomputed at evaluation time;
- budget accounting must be present and within the bound ceilings (below).

A PASS prints `CLOSED`. Re-running with byte-identical evidence is
`ALREADY-CLOSED` and appends nothing; different evidence after closure exits 6
(`CLOSED-DIFFERENT-IDENTITY`) because a closed phase is immutable. Unmet or
unverifiable criteria record `REFUSED_FAIL` (exit 3) or `REFUSED_BLOCKED`
(exit 2); the phase stays open. If the canonical writer rejects the write, the
evaluator prints `WRITER-FAILED` and exits 5 on both the PASS and the refusal
paths — it never claims a disposition it did not record.

### Closing is not continuing

A closed phase starts no successor on its own. `/kbd-next-phase` refuses a
bound phase that is not CLOSED (exit 2) and refuses a CLOSED one without an
operator-signed **start-successor** manifest (exit 4, nothing written). With
the manifest, one runtime event defines and activates exactly the named
successor; a rerun reports `ALREADY-STARTED`. While a closed phase awaits its
successor, the runtime admits activation only of an ancestor (so child-exit
works) or the authorized successor. Reflection text becomes `goals.md`
proposals for the successor, never adopted scope.

### Scope amendments need signed operator authority

Changing a bound contract is a material amendment. `kbd-contract-adopt.sh`
needs `--supersedes <decision> --amendment <manifest> --signature <sig>`, and
the runtime verifies the signature with `ssh-keygen -Y verify` under the
namespace `prometheus-kbd-scope-amendment` against
`~/.config/prometheus/kbd-scope-allowed-signers` (no flag or environment
override; a missing file fails closed). The manifest names the project, run,
phase, previous and new digests and the exact scenario diff; each manifest is
single-use. Only scenarios the amendment adds or modifies need fresh
evidence; unchanged scenarios may carry evidence (`scenarioDigest` plus
`carriedFrom`). Memory, reviewer findings, imported contracts and fields such
as `"approved": true` have no path into this check.

`shared/scripts/kbd-scope-authority-template.sh amend-contract|start-successor`
prints the canonical manifest from canonical state and the exact signing
command. It never signs. See `shared/config/kbd-scope-allowed-signers.example`.

### Budget accounting and the missing-accounting rule

The evidence packet must carry a `budgetConsumption` header naming its source
and classing each value as `measured`, `estimate`, or `unknown`. The evaluator
compares self-reported cumulative consumption against the bound snapshot's
ceilings (base budgets plus extensions; the latest `resultingCeilings` wins):

- **exceeded** → `BLOCKED (budget-exceeded)`;
- **unknown or missing on an enforceable dimension** (time, tool calls,
  repairs) → `BLOCKED (budget-unmeasured)`; tokens may stay `unknown` under the
  declared fallback;
- **at ceiling** → work already complete closes, labelled
  `budget-note=budget-exhausted`; a ceiling never substitutes for evidence;
- **missing header** → `BLOCKED (budget-consumption-missing)`.

The runtime keeps a high-water mark per dimension: a later disposition that
reports less is refused (`budget-regressed`), so counters never reset across
refusals, amendments or restarts. Budget extensions are part of the
digest-bound contract, so adding one to a bound contract is a signed
amendment.

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

# 5. continue only with operator authority
bash shared/scripts/kbd-scope-authority-template.sh start-successor \
  --phase my-phase --successor my-next-phase --title "My next phase" \
  --reason "operator approves the next scope" > successor.json
ssh-keygen -Y sign -f ~/.ssh/id_ed25519_sk -n prometheus-kbd-scope-amendment successor.json
bash skills/process/kbd-process-orchestrator/skills/kbd-next-phase/scripts/kbd-next-phase.sh \
  --authorize-successor successor.json --signature successor.json.sig
```

If `evidence.json` omitted `budgetConsumption`, step 4 would instead print
`BLOCKED … reason=budget-consumption-missing` and record a refusal.

## Limits and disclosures

Read these before relying on the mechanism:

- **Where enforcement lives.** The runtime enforces the closure invariants for
  every caller of `prometheus kbd`: a bound phase cannot be completed by
  `phase transition`, a closed phase cannot be re-opened, re-activated or
  given new work, and a successor or amendment needs a verified signature.
  The evaluator's scenario and budget analysis is still a shell convention
  invoked through the skill flow. Nothing here intercepts arbitrary tools or
  edits outside `prometheus kbd` (host enforcement is #185).
- **What a signature proves.** Any key whose private half a local agent can use
  unattended records intent, not human approval. Use a FIDO (`sk-*`) key or a
  passphrase the agent cannot supply; the allowed-signers file itself is
  readable and writable by local processes.
- **Disclosed test seams.** `KBD_READINESS_STATUS_FILE`,
  `KBD_PHASE_DISPOSITION_FILE`, `--status-file`, `--disposition-file` and
  `--decision-sink` override where state is read and dispositions are
  written. They exist for fixtures and are visible in every receipt that uses
  them; a decision sink can never close a phase.
- **Legacy phases.** Phases that were never bound keep their previous
  behaviour, including direct `phase transition --status complete`. Journals
  written before typed closure replay unchanged; older `prometheus` binaries
  cannot read journals that contain the new events, so upgrade the CLI,
  control plane and both packs together.
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
