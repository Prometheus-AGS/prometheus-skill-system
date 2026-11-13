## ADDED Requirements

### Requirement: Canonical close admission validates the evidence facts
The canonical phase-close admission SHALL refuse a new PASS closure, without appending an event,
unless the evidence names the phase being closed, names this project when it names one, declares
at least one source digest and every declared digest equals the file's current sha256, and every
scenario entry owned by a bound scenario has non-empty `attempted`, `observed` and `evidence`.

#### Scenario: Direct close and evaluator refuse the same evidence
- **WHEN** evidence with a wrong phase, a wrong project, no source digests, a stale source
  digest, or an empty observation field is offered through `prometheus kbd phase close` and
  through `kbd-phase-close-eval.sh`
- **THEN** both refuse with the same reason code and the phase is not CLOSED

#### Scenario: Sub-scenario entries satisfy a bound scenario
- **WHEN** evidence carries entries `S-a` and `S-b` for bound scenario `S`, each complete
- **THEN** the phase closes; an empty field on `S-b` is refused as `empty-evidence:S-b`

#### Scenario: A valid receipt closes once
- **WHEN** a valid receipt is offered twice, and again after its source file changed
- **THEN** one disposition event exists, repeats are no-ops, and no successor starts

### Requirement: Recorded closures replay with their recorded semantics
The reducer SHALL apply the evidence fact checks only to disposition events stamped
`admissionVersion >= 2` by the admitting runtime, and SHALL NOT read files during replay.

#### Scenario: Pre-change journal
- **WHEN** a journal whose PASS closure was written before this change is replayed
- **THEN** it replays to the state the writing binary reported, whatever the current files hold
