# KBD bottleneck advisory replay corpus, v1

Five recorded failure patterns from issue #256, replayed against
`prometheus kbd guard advise`. Each case seeds a phase, records the blockers
and (where stated) leaves a gate running, then states the advice expected.

The driver is `substrate/kbd-runtime/tests/bottleneck_advisory_replay.rs`. For
every case it also asserts that reading the advice appended no journal event,
created no phase or child, and left every phase status and change unchanged.

| Case | Pattern | Expected advice |
|---|---|---|
| `scope-dilution` | One product failure repeated while other changes wait | `architecture-child`, command printed, not run |
| `mixed-product-fixture` | A fixture failure repeated beside one product failure | `measurement-repair` for the fixture only |
| `approval-wait` | Waiting for an approval, recorded twice | none; listed as waiting |
| `release-setup-failure` | Release setup failed twice with the same evidence | `setup-repair` |
| `quiet-running-build` | A gate is still running after earlier failures | none; `in-flight: do not stop` |

## Case fields

| Field | Meaning |
|---|---|
| `pendingChanges` | Changes registered in the phase before any blocker |
| `blockers[]` | Recorded in order: `id`, `summary`, `fingerprint`, `class`, `evidenceRef`, optional `cleared` |
| `gateRunning` | When true, a gate is started and not finished |
| `expect.recommendations[]` | `kind`, `fingerprint`, `attempts`, and whether a command is suggested |
| `expect.waiting[]` | Blocker ids reported as waits |
| `expect.inFlight` | Number of running gates reported |
| `expect.withheld` | Recommendations held back while a gate runs |
