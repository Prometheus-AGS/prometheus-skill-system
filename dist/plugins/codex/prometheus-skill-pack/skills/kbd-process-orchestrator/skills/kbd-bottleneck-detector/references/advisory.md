# Advisory recommendations

`advise` reads canonical state and proposes one next step when the same
blocker has been attempted again without new evidence. It is a reading, not a
gate: it appends no journal event, stops nothing, creates no child, and changes
no disposition or contract. It exits 0 whenever canonical state can be read.

```bash
prometheus kbd --path . guard advise --json
```

## Record a blocker so it can be counted

Only blockers recorded with a fingerprint take part.

```bash
prometheus kbd blocker record --command-id <id> --id <blocker-id> \
  --summary "<what failed>" \
  --fingerprint <stable-problem-id> --class <class> --evidence <reference>
```

| Field | Use |
|---|---|
| `--fingerprint` | The same value for every attempt on the same underlying problem |
| `--class` | What kind of obstacle it is; see below |
| `--evidence` | What this attempt produced: a log path, receipt id or digest |

| Class | Counted as an attempt | Recommendation at two attempts |
|---|---|---|
| `product` (default) | yes | `architecture-child` |
| `setup` | yes | `setup-repair` |
| `measurement` | yes | `measurement-repair` |
| `authority` | yes | `authority-handoff` |
| `approval-wait` | no | none; listed under `waiting` |
| `resource-wait` | no | none; listed under `waiting` |

## When a recommendation appears

All of these hold:

1. Two or more attempts share one fingerprint.
2. At least two of them carry the same evidence reference, so an attempt
   learned nothing new.
3. At least one of them is still unresolved.
4. No gate is running.

While a gate started by `prometheus kbd gate run` has not finished, the output
says `in-flight: do not stop` and holds every recommendation back
(`withheldWhileInFlight`). A quiet build is not a stalled one.

## What to do with it

| Recommendation | Proposal to put to the operator |
|---|---|
| `setup-repair` | Repair the setup once, apart from the product change, then run the same check again |
| `measurement-repair` | Repair the fixture, oracle or runner; report the product result as NOT_EXERCISED, not FAIL |
| `authority-handoff` | Hand the one decision to its owner and continue with work that does not depend on it |
| `architecture-child` | Propose a scoped child phase; the command is printed in `suggestedCommand` and never run |

The agent proposes; the operator decides. A repair after a failed boundary
still needs the owner's authorization (B5), and a child phase is created only
when the operator runs `/kbd-new-child`.

## Replay corpus

Five recorded failure patterns live in
`shared/references/conformance/kbd-bottleneck-advisory/v1/`: scope dilution,
mixed product and fixture failure, approval wait, release setup failure, and a
quiet running build.
