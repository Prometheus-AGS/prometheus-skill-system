# Change identity diagnosis and repair

A phase-scoped change ID names exactly one canonical change. This guide covers
how duplicate or mismatched identities are prevented, how to inspect them, and
how an operator repairs a confirmed canonical record without losing history.
Scenario references (IDR-001 to IDR-003) point at the `kbd-194-change-identity-repair`
OpenSpec change.

## Canonical repair versus projection repair

| Layer | Owner | Repair |
|---|---|---|
| Canonical journal/state | Signed `kbd-runtime` | Only `change repair-identity`, signed by an operator. Append-only; source records stay in the journal. |
| Projection (`progress.json`, `tasks.md`, waypoint) | Derived from canonical state | `kbd-apply.sh reconcile <phase> --repair` after the canonical record is correct. Never edit a projection to "fix" canonical state, and never edit a journal. |

Reconcile reports a canonical identity problem and stops; it does not repair one.
A projection-only repair uses confirmed canonical state and creates neither a
canonical merge nor a completion receipt.

## Prevention (IDR-001)

- Registering a change ID that already exists is refused with the existing
  record untouched. An equivalent retry is idempotent; reusing a `commandId`
  for different change content is refused.
- Registering an ID already present under a different map key, or one retired
  by a repair, is refused rather than recreated.
- Legacy import refuses duplicate change or task IDs and key/embedded-ID
  mismatches, names every competing identity and leaves the input bytes alone.

## Inspect first (IDR-001)

```sh
prometheus kbd --path . change inspect-identity --phase <phase-id>
```

Inspection changes nothing and grants no repair authority. It prints, for the
phase: each change map key, its embedded ID, the JCS SHA-256 of the record, the
full record, identity `issues` (duplicate ID, key/embedded-ID mismatch, task key
mismatch) and the outstanding boundary obligations.

`kbd-apply.sh reconcile <phase> --json` reports the same families from the
projection side: repeated progress IDs, key/ID mismatches, a qualified task whose
key is absent, and an ambiguous title, each naming the phase and competing
identities. A missing qualified task key is **not** proof of a missing start
receipt, and a repeated ID is not proof of canonical duplication.

## Repair (IDR-002)

Write a resolution file from the inspection output:

```json
{
  "sources": { "alias-a": "<sha256 of alias-a>", "target": "<sha256 of target>" },
  "survivorKey": "target",
  "targetKey": "target",
  "taskMappings": [
    { "sourceKey": "alias-a", "sourceTaskKey": "1", "targetTaskKey": "1" }
  ],
  "receiptMappings": [
    { "sourceKey": "task:alias-a/1", "targetSubject": "target/1" }
  ]
}
```

```sh
prometheus kbd --path . change repair-identity \
  --command-id <unique-id> --phase <phase-id> \
  --expected-revision <revision from inspection> \
  --resolution-file resolution.json --reason "<why>"
```

The command is operator-signed and bound to the current revision and to each
source digest. It is refused, with the unresolved identities named and no
mutation, when:

- the revision or any source digest changed since inspection;
- the event is not signed by an enrolled, active operator device key, the phase is closed, or the reason is empty;
- a source's embedded ID does not equal `targetKey`, or the target key exists
  outside the listed sources;
- any source task lacks an explicit mapping, or two tasks with different
  content map to one target task (resolve outcome conflicts separately with real
  evidence);
- an outstanding receipt for a source is unmapped, maps to a missing
  change/task, or collides with an existing obligation;
- the survivor is complete while merged work is unfinished (completion is never
  fabricated; choose an unfinished survivor instead);
- no damaged identity exists (ordinary transitions use their normal commands).

One signed event records the original change and obligation snapshots plus the
result. Replay re-derives the same effective result from the preceding state, so
a doctored payload is rejected. Repeating the same `--command-id` with the same
content is idempotent; the same id with different content is refused.

## Continuation after repair (IDR-003)

A retired key stays recorded as an alias of its target. The normal entry points
resolve it:

- `kbd-apply.sh begin-task|end-task` accept the backend change name or a retired
  alias, never register the alias as a new change, and bind the signed guards to
  the surviving change and registered task key (so remapped receipts match).
- Backend task ordinals that differ from a registered task key still map by
  title, then sequence; an unmatched or ambiguous task is refused, not duplicated.
- `kbd-apply.sh reconcile <phase> [--repair]` inspects an alias progress row
  through its surviving change and converges without a second change or task.

An unmatched start, uncertain prior effect or unmappable receipt stays a blocking
obligation. Nothing is synthesized from an implementation-complete flag.

## Authority model and limits

`repair-identity` is signed as an operator action, but the runtime can only check the
signing key and the self-declared actor kind; it cannot tell a person from a harness
session running as the same user with the same device key. Treat the command as
operator-only by policy: keep it out of agent allowlists and review the signed
repair record. The public CLI cannot produce a non-operator repair.

## Supported incident families and limits

Supported: duplicate change IDs from legacy import or registration; map key
differing from embedded ID; alias keys carrying tasks and outstanding receipts;
task key differing from embedded task ID.

Not supported: merging conflicting task outcomes, inventing receipts, or
repairing a closed phase. Historical events remain readable; invalid new input is
rejected when a new command is prepared, not retroactively.

The original issue #194 journal and the 1.11.3 binary that produced it are not
available, so no root-cause claim is made about that incident and no live
incident repair is performed here. An opaque legacy completion baseline is
preserved as recorded; if its aggregate count is ambiguous, the repair record
must state that limitation. Evidence for these paths comes from representative
fixtures at the phase's final verification boundary and proves only that scope.
