# learner-model

Authoritative runtime for learner state in the Feynman learning loop: a CRDT-backed
store of concepts, observations, gaps, sessions, and spaced-repetition cards, driven
by learn-* skills through a newline-delimited JSON-RPC binary.

```
learner-model [data-dir]        # default ~/.prometheus/learn/learner-model
```

Every request is one JSON line on stdin; every response is one JSON line on stdout.
A response is either the result or `{"error": "..."}`; the process never exits on a
bad request.

## Methods

| Method | Params | Result |
|---|---|---|
| `load` | `learner_id` | the whole `LearnerModel` |
| `seed_from_survey` | `seed` (learn-survey output) | `{ok, learner_id}` |
| `get_concept` | `learner_id`, `concept_id` | the `ConceptState` plus `gaps[]` (this concept's) and `sessions[]` (those that touched it) |
| `add_observation` | `learner_id`, `concept_id`, `score`, `source_skill` | `{ok}` — PFA mastery update from the fifth observation on |
| `review` | `learner_id`, `concept_id`, `score`, `rating` (again/hard/good/easy), `source_skill`, `timestamp?` | `{ok, fsrs_card}` — advances the FSRS card |
| `add_gap` | `learner_id`, `concept_id`, `description`, `source_skill`, `severity?` (minor/major/misconception), `source_evidence?`, `label?` (verified/inferred) | `{ok, gap_id}` (change-rah-009) |
| `resolve_gap` | `learner_id`, `gap_id`, `resolved_at?` | `{ok, gap_id, resolved_at}` (change-rah-009) |
| `add_session` | `learner_id`, `skills_called[]`, `concepts_touched[]`, `session_id?`, `session_type?`, `started_at?`, `ended_at?` | `{ok, session_id}` (change-rah-009) |
| `set_certified` | `learner_id`, `concept_id`, `certified_at?` | `{ok, concept_id, certified_at}` (change-rah-009) |

Timestamps are RFC 3339; omitted ones default to now (UTC).

`gaps` is a map keyed by `gap_id` and `sessions` is a list deduplicated by
`session_id` and ordered by `started_at` on every load, save, and CRDT import, so two
devices that recorded the same session converge. Concept state (`mastery`,
`fsrs_card`) is always re-derived from the immutable observation set
(`store::fold_concept`), never trusted from the document.

## FSRS dependency decision

Recorded 2026-09-06 (change-rah-009, analysis D-10) from `cargo tree -e normal`
on scratch manifests, run while no other Cargo build was active:

| Candidate | Version resolved | Unique crates in the tree (root excluded) | What is under it |
|---|---|---|---|
| `rs-fsrs` | 1.2.1 | 7 | `chrono` (`iana-time-zone`, `core-foundation-sys`, `num-traits`) and `serde` (`serde_core`); all already in this crate's tree |
| `fsrs` (fsrs-rs) | 6.6.2 | 41 | the `burn` machine-learning stack (parameter optimisation), which nothing here needs |

**Decision: `rs-fsrs` 1.2.1 is adopted** for `next_review()`. Its tree is scheduler-only
and adds nothing new to the dependency set. `fsrs` (fsrs-rs) is the reference
implementation and the optimiser, and is not adopted.

`rs-fsrs` 1.2.1 ships the FSRS-5 parameter set (19 weights, `type Weights = [f64;
19]`, default `request_retention` 0.9, `maximum_interval` 36500, short-term
scheduler enabled, fuzz disabled). It updates `difficulty` on every review through
FSRS's mean-reversion rule, which is the property the previous stub lacked (it
stored `difficulty` and never read it). The 21-weight FSRS-6 set is what fsrs-rs
6.x implements; adopting it would mean the 41-crate tree. The card fields this crate
persists (`stability`, `difficulty`, `due`, `state`, `reps`, `lapses`,
`last_review`) are the same under both, so a later switch does not change the
schema. Docs in this repository that say "FSRS-6" describe the card model; the
scheduling arithmetic is FSRS-5 until that switch.

`src/fsrs.rs` is the adapter: it maps this crate's `FSRSCard` to `rs_fsrs::Card`,
calls `FSRS::default().next(card, now, rating)`, and maps the result back. The
`difficulty` delta is asserted by `tests/rpc_roundtrip.rs` over the built binary
(a Good review followed by a Hard review produce different `difficulty`), not by
inspection.

## Tests

```bash
cargo test -p learner-model --test rpc_roundtrip   # drives the built binary over stdin/stdout
```

The round trip seeds from a survey, records five observations (mastery moves only
from the fifth), adds a gap and a session, certifies the concept, reviews it twice,
then reloads and asserts every record is present with its id.
