---
id: boundary-authorization
title: Boundary authorization and early-execution evidence
description: Record when the owner authorized B4 or B5, and let phase closure report runner output that predates it.
---

# Boundary authorization and early-execution evidence

Under VLBV, generated code runs at exactly two points: the value-phase boundary
(**B4**, once) and an owner-authorized repair (**B5**). The full taxonomy, B0 to B6,
is defined once in
[`boundaries.md`](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/skills/process/kbd-process-orchestrator/references/boundaries.md).
This page covers two pieces that make that rule checkable afterwards:

- the **authorization event**, a timestamped record that the owner authorized the
  boundary, written before the first attempt;
- the **pre-authorization artifact scan**, which uses that timestamp at closure to
  list runner output that already existed when the boundary was authorized.

Neither one blocks anything. They are records and reports.

## Why record the authorization

"Tests ran only at the boundary" is a claim about ordering. Without a recorded
instant for the boundary there is nothing to order against: a report file written
during implementation looks the same as one written by the authorized run.

The pack's rules are instruction-only on every harness. No tool call is
intercepted, so an agent can run a test early and nothing stops it. What the pack
can do is make early execution **visible**. The authorization event fixes the
instant, and the closure packet compares file modification times against it.

The event also binds the authorization to a contract. It stores the digest of the
contract the phase was bound to at that moment, so a later reader can tell which
agreed scope the owner authorized.

## When to use it

Record an event in these two situations:

| Situation | Command |
|---|---|
| All planned production work is complete, the owner has said to run the boundary, and no attempt has started | `--boundary B4` (the default) |
| A B4 attempt produced a FAIL and the owner authorized a bounded repair | `--boundary B5` |

Do not record one in these situations:

- **Before the contract is adopted.** The script refuses a phase with no bound
  contract (exit 2). Adopt it first with `kbd-contract-adopt.sh`.
- **To grant yourself authority.** The event records a decision the owner made. An
  agent writes it only after the owner authorized the boundary, and names the
  owner and the instruction in `--authorizer`.
- **After the first attempt, for B4.** Once `boundary-results.json` exists the
  script refuses a B4 event (exit 3). An authorization written afterwards would
  date the boundary later than the run it is supposed to precede.
- **At a task end, checkpoint, commit or reviewer request.** None of those is B4.

## Record the event

Run it from anywhere inside the project. The script walks up to the nearest
`.kbd-orchestrator` directory, or takes `--project <dir>`.

```bash
node shared/scripts/kbd-boundary-authorize.mjs \
  --phase vlbv-r4-portability \
  --authorizer "owner (chat 2026-11-16: 'Authorize B4')" \
  --reason "B4 boundary run of the adopted R4 contract"
```

```text
AUTHORIZED phase=vlbv-r4-portability boundary=B4 at=2026-11-16T08:36:25.680Z contract=baa987b2…
```

For a repair, after the failed attempt is recorded:

```bash
node shared/scripts/kbd-boundary-authorize.mjs \
  --phase vlbv-r4-portability \
  --boundary B5 \
  --authorizer "owner (chat 2026-11-16: 'Stop and repair')" \
  --reason "Repair: scratch-home auth and not-run scoring"
```

| Option | Required | Meaning |
|---|---|---|
| `--phase <id>` | yes | Phase id in kebab-case; its directory must exist |
| `--authorizer <who>` | yes | Who authorized the boundary, in words a later reader can check |
| `--boundary B4\|B5` | no | Defaults to `B4` |
| `--reason <text>` | no | Stored with the event |
| `--project <dir>` | no | Start directory for locating `.kbd-orchestrator`; defaults to the working directory |

| Exit | Meaning |
|---|---|
| 0 | Event recorded, or B4 was already authorized (`ALREADY-AUTHORIZED` is printed and nothing is written) |
| 1 | Usage or I/O error: missing option, no `.kbd-orchestrator`, no phase directory, or a file that is not an authorization record |
| 2 | The phase has no bound contract |
| 3 | Ordering violated: B4 after results exist, B5 without a prior B4, or B5 without `boundary-results.json` |

The script reads the bound contract digest from
`prometheus kbd --path <root> phase disposition --id <phase> --json`, so the
`prometheus` CLI must be on `PATH`.

## What gets written

One file per phase, `.kbd-orchestrator/phases/<id>/boundary-authorization.json`.
Events are appended and never edited. The file is replaced atomically.

```json
{
  "schemaVersion": 1,
  "kind": "kbd.boundary-authorization",
  "phaseId": "vlbv-r4-portability",
  "events": [
    {
      "boundary": "B4",
      "authorizedAt": "2026-11-16T08:36:25.680Z",
      "authorizer": "owner (chat 2026-11-16: 'Authorize B4')",
      "contractDigest": "baa987b2956f8973c8e5047015d0b488e97411e0c42d46ee534f3cc8081def68",
      "reason": "B4 boundary run of the adopted R4 contract"
    },
    {
      "boundary": "B5",
      "authorizedAt": "2026-11-16T08:58:02.339Z",
      "authorizer": "owner (chat 2026-11-16: 'Stop and repair')",
      "contractDigest": "baa987b2956f8973c8e5047015d0b488e97411e0c42d46ee534f3cc8081def68",
      "reason": "Repair: claude-code auth in scratch home; harness-not-run scored FAIL"
    }
  ]
}
```

The ordering the script enforces is:

```text
contract adopted  <  implementation complete  <  B4 authorized  <=  first attempt
```

`authorizedAt` has millisecond precision because the scan compares it with file
modification times.

## Declare runner output in the contract

The scan needs to know which files count as runner output. Declare them in the
adopted contract (v2) under `boundaryPacket.runnerOutputGlobs`:

```json
{
  "boundaryPacket": {
    "runnerOutputGlobs": ["evals/reports/**", "evals/runs/**"]
  }
}
```

Globs are project-relative. `*` matches within one path segment, `**` matches
across segments, and `?` matches one character. `.git` and `node_modules` are
skipped.

List what the boundary packet's commands write: reports, coverage output, build
artifacts, recorded transcripts. Leave out files that are legitimately edited
during implementation, such as source and fixtures, or every phase will list them.

## Read the result at closure

`kbd-phase-close-eval.sh` writes `closure.json` and `closure-packet.md` for every
outcome, closed or refused. The packet has a section named
**Pre-authorization runner artifacts**, and `closure.json` carries the same data
under `preAuthorizationArtifacts`. The scan always compares against the first B4
event.

| Status | Packet line | What it means |
|---|---|---|
| `found` | One line per file, with its modification time | Runner output existed before B4 was authorized. Explain each file in the closure review |
| `none` | `none found (N runner artifact(s) under …)` | Files matched the globs and all were modified at or after authorization |
| `no-authorization` | `no B4 authorization event recorded` | The event was never written, so nothing can be compared |
| `no-globs` | `no runnerOutputGlobs declared in the contract` | The contract does not say what runner output is |

A `found` result is evidence to weigh. It does not prove a rule was broken: a
report committed by an earlier phase and left in place also predates this
authorization. It does not change the phase's disposition either, which comes only
from scenario results evaluated against the bound contract.

The two "scan not possible" statuses are the ones to fix. A phase that closes with
either has no ordering evidence at all.

## Limits

- The scan reads modification times. A tool that rewrites a file's timestamp, or a
  checkout that resets it, changes the result.
- It sees only files under the declared globs. Output written elsewhere, or
  deleted before closure, is not reported.
- Control level is instruction-only. Nothing here prevents early execution.

See also: [Quality gates](./quality-gates.md),
[Phase readiness contract](./phase-readiness-contract.md) and the
[value-led scenario workflow](/docs/guide/value-scenario-workflow).
