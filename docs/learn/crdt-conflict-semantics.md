# CRDT Conflict Semantics — learner-model

**Schema version:** 1.0.0
**CRDT engine:** automerge-rs (AutoCommit document per learner)
**Applies to:** `substrate/learner-model/` Rust crate

---

## Design Principle

The learner model is a compact CRDT document. Multi-device merge must never:
- Lose an observation
- Schedule less review than the conservative device
- Delete a gap that has not been explicitly resolved
- Produce a mastery estimate lower than the most-recent graded evidence supports

When two devices have divergent state, merge conservatively: prefer more practice, earlier review, and higher observation count.

---

## Field-Level Merge Strategy

### `concepts[id].mastery` — LWW with vector clock

**Merge rule:** last-write-wins, determined by vector clock comparison.

```
Device A: mastery=0.7, vector_clock={A:5, B:2}
Device B: mastery=0.6, vector_clock={A:4, B:3}

Concurrent update (neither dominates). Tie-break: higher mastery wins.
Result: mastery=0.7

Non-concurrent: the device with the strictly later vector clock wins.
```

**Rationale:** mastery is probabilistic. The most recent graded evidence is authoritative. Tie-break to higher mastery avoids regressive dampening from stale low-score observations.

---

### `concepts[id].observations` — Union append

**Merge rule:** union of all observations from both devices, deduplicated by `(timestamp, score, source_skill)` triple. Never delete.

```
Device A observations: [{t: T1, score: 0.8, src: learn-grade}]
Device B observations: [{t: T1, score: 0.8, src: learn-grade}, {t: T2, score: 0.9, src: learn-practice}]

Merged: [{t: T1, ...}, {t: T2, ...}]  — union, dedup by triple
```

**Rationale:** observations are the evidentiary record. Data loss in an append-only log is worse than a duplicate entry. Dedup by content triple (not by UUID) prevents near-duplicate accumulation.

---

### `concepts[id].fsrs_card.stability` — max(local, remote)

**Merge rule:** take the higher stability value from either device.

```
Device A: stability=14.2 (days)
Device B: stability=11.8 (days)
Result: stability=14.2
```

**Rationale:** higher stability = less forgetting = fewer required reviews. Conservative means preferring the device that produced stronger retention evidence. Downgrading stability arbitrarily would schedule unnecessary reviews.

---

### `concepts[id].fsrs_card.due` — min(local, remote)

**Merge rule:** take the earlier due date from either device.

```
Device A: due=2026-07-05T00:00:00Z
Device B: due=2026-07-08T00:00:00Z
Result: due=2026-07-05T00:00:00Z
```

**Rationale:** prefer more review, not less. A later due date on one device means that device has not yet done a review that the other has — or has a more optimistic stability estimate. Scheduling the earlier date is conservative and safe.

---

### `concepts[id].fsrs_card.state` — LWW (latest last_review wins)

**Merge rule:** the device with the later `last_review` timestamp has the authoritative state.

```
Device A: state=Review, last_review=2026-07-01T10:00:00Z
Device B: state=Relearning, last_review=2026-07-01T09:00:00Z
Result: state=Review (Device A is later)
```

**Rationale:** card state reflects the outcome of the most recent review session. Last-review timestamp is the ground truth for which device's state is current.

---

### `concepts[id].fsrs_card.reps` — max(local, remote)

**Merge rule:** take the higher rep count.

**Rationale:** reps is monotonically increasing. The device with more reps has done more reviews. Merging down would lose review history.

---

### `concepts[id].fsrs_card.lapses` — max(local, remote)

**Merge rule:** take the higher lapse count.

**Rationale:** lapses is monotonically increasing. Same reasoning as reps — data loss is worse than over-counting.

---

### `gaps` map — Union append, resolved_at from latest

**Merge rule:**
- All gaps from both devices are included (union by gap_id)
- If a gap exists on both devices, `resolved_at` is set to the non-null value (or null if neither has resolved it)
- A gap is only considered resolved if `resolved_at` is non-null on the resolving device

```
Device A: gap G1 {resolved_at: null}
Device B: gap G1 {resolved_at: 2026-07-02T00:00:00Z}
Result:   gap G1 {resolved_at: 2026-07-02T00:00:00Z}  — take the resolution

Device A: gap G2 {resolved_at: 2026-07-01T00:00:00Z}
Device B: gap G2 {resolved_at: 2026-07-03T00:00:00Z}
Result:   gap G2 {resolved_at: 2026-07-03T00:00:00Z}  — take later resolution
```

**Rationale:** gap records are the grader's evidentiary output. A gap should only be considered resolved when learn-grade has explicitly confirmed it. The absence of a resolution on one device does not cancel a confirmed resolution on another.

---

### `sessions` — Append-only, dedup by session_id

**Merge rule:** union by session_id. Never delete. No field-level merge within a session record (sessions are immutable once written).

---

## PFA Update Rule (≥5 observations)

After 5 or more observations exist for a concept, the mastery estimate transitions from LLM-seeded Bayesian prior to PFA-style incremental updates:

```
mastery_new = mastery_old + α × (score - mastery_old)
```

Where:
- `α` = learning rate = 0.3 (configurable; higher = faster response to new evidence)
- `score` = observation score [0,1] from learn-grade
- `mastery_old` = current mastery estimate

This is applied on the observing device only. The result becomes the new mastery value, which then participates in the LWW merge protocol on the next sync.

**Transition condition:** count of observations for the concept ≥ 5 (across the merged observation set, not per-device).

---

## Worked Conflict Examples

### Example 1: Concurrent mastery updates (LWW tie-break)

**Scenario:** Two devices independently grade the same concept on the same day.

```
Device A state: mastery=0.7, observations=[{T1, 0.7, learn-grade}], vector_clock={A:3}
Device B state: mastery=0.6, observations=[{T2, 0.6, learn-grade}], vector_clock={B:2}

Neither clock dominates (concurrent). Apply tie-break: max(0.7, 0.6) = 0.7.
Merged observations: [{T1, 0.7, learn-grade}, {T2, 0.6, learn-grade}] (union).
Merged mastery: 0.7.
```

---

### Example 2: FSRS due date conflict

**Scenario:** Device A reviewed and pushed due forward; Device B hasn't synced yet.

```
Device A: stability=14, due=2026-07-15T00:00:00Z, last_review=2026-07-01
Device B: stability=7,  due=2026-07-08T00:00:00Z, last_review=2026-06-28

stability: max(14, 7)    = 14
due:        min(July-15, July-8) = July-8   ← conservative: review sooner
state: Device A wins (later last_review): Review
reps: max(local, remote)
```

**Outcome:** device B's earlier due date overrides. The learner reviews on July 8 instead of July 15 — conservative. After that review, stability updates from the current state and FSRS reschedules correctly.

---

### Example 3: Gap dedup across devices

**Scenario:** Both devices detect the same gap independently (same concept, same session).

```
Device A gaps: {G-abc: {concept: "attention", description: "missed positional encoding", resolved_at: null}}
Device B gaps: {G-abc: {concept: "attention", description: "missed positional encoding", resolved_at: "2026-07-02"}}

Merge result: G-abc resolved_at = "2026-07-02"  ← Device B resolved it; Device A's null doesn't cancel
```

**Outcome:** the gap is marked resolved. The learner does not see it resurface on the next sync.

---

## automerge-rs Implementation Notes

The `LearnerModel` struct is backed by an `automerge::AutoCommit` document:

```rust
// Each concept's mastery field is an automerge::ScalarValue::F64
// Each observations list is an automerge::ObjType::List (append-only via put)
// FSRSCard fields are individual scalar values in an automerge::ObjType::Map
// Gaps map is an automerge::ObjType::Map keyed by gap_id (UUID string)
// Sessions list is an automerge::ObjType::List (append-only)
```

Merge is performed via `doc.merge(&mut other_doc)` — automerge handles concurrent edits via its internal CRDT. The field-level semantics above are enforced by the application layer:

- After merge, apply `mastery` tie-break logic
- After merge, compute `due = min(local_due, remote_due)` and write back
- After merge, compute `stability = max(local_stability, remote_stability)` and write back
- After merge, compute `reps = max(local_reps, remote_reps)` and write back
- After merge, compute `lapses = max(local_lapses, remote_lapses)` and write back

These post-merge fixups run in `LearnerModel::merge_from(delta: &[u8]) -> Result<()>`.
