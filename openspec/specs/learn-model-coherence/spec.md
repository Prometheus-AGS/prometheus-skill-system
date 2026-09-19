# learn-model-coherence Specification

## Purpose

The learn skills (`learn-goal`, `learn-kb`, `feynman-loop`, `learn-grade`,
`learn-practice`, `learn-retain`, `learn-certify`) hand work to one another
through files and one substrate binary. This capability states the contracts
those hand-offs rest on: one artifact path, one corpus shape produced by one
grounding script, artifact provenance, the learner-model write paths, and the
rule that the grader's eval baseline is captured only against reviewed truth.
Each requirement names the change that introduced it
(`change-rah-005`, `change-rah-007`, `change-rah-008`, `change-rah-009`, phase
`research-agent-hardening`).

## Requirements

### Requirement: One artifact path

feynman-loop, learn-retain, and learn-certify SHALL resolve the same file for a
given goal, concept, and artifact id:
`<learn-home>/goals/<goal-id>/artifacts/<concept-id>/<artifact-id>.json`, where
`<learn-home>` is `~/.prometheus/learn` or `PROMETHEUS_LEARN_HOME`.
`write-artifact.sh` SHALL write that path and refuse an artifact whose
`concept_id` is missing or whose ids contain a path separator or dot-segment.
"Most recent" for a concept is the artifact with the latest `closed_at`.

#### Scenario: Round trip

- **WHEN** `write-artifact.sh` writes an artifact for concept `c1`
- **THEN** the glob learn-retain documents (`artifacts/<concept-id>/*.json`) and
  the directory learn-certify documents (`artifacts/<concept-id>/`) both resolve
  that file

#### Scenario: Ids are path components

- **WHEN** an artifact carries `concept_id: "../escape"` or no `concept_id`
- **THEN** `write-artifact.sh` exits non-zero and writes nothing outside the
  goal's artifact store

### Requirement: The corpus carries key points and misconceptions

Every source emitted by `content-grounding-kb.sh` SHALL carry `key_points[]`
and `misconceptions[]` alongside `source_ref`, `source_type`, `confidence`,
`is_misconception`, and `content_summary` (`schema_version` 1.1.0).
`key_points` are the sentences of `content_summary` unless the KB entry authored
its own list, which is kept verbatim; `misconceptions` is `[content_summary]`
for a source flagged `is_misconception` and `[]` otherwise. A corpus at the
older shape SHALL be brought to this shape by the same script's `--normalize`
mode, never by a second derivation.

#### Scenario: Local KB

- **WHEN** the script runs against the fixture KB with `--include-misconceptions`
- **THEN** `jq '.sources[] | has("key_points") and has("misconceptions")'` is
  true for every source
- **AND** a misconception source's `misconceptions[]` holds its text and every
  other source's is empty

#### Scenario: Normalize an older corpus

- **WHEN** `--normalize` is given a schema 1.0.0 corpus whose sources have
  `is_misconception` and `content_summary` only
- **THEN** the output carries both arrays on every source, keeps any authored
  `key_points[]`, and keeps the corpus identity fields

### Requirement: One grounding script

`shared/scripts/content-grounding-kb.sh` is the only implementation. The two
skill-local entry points (`learn-goal`, `learn-kb`) SHALL exec it and contain no
adapter logic.

#### Scenario: Wrapper

- **WHEN** either wrapper is run with the same arguments as the shared script
  (and the same `CONTENT_GROUNDING_BUILD_AT`)
- **THEN** it produces byte-identical output, under `/bin/bash` 3.2 as well

### Requirement: Artifacts carry verification and provenance

A feynman artifact SHALL carry one `verification` entry per transfer score with
a label in `verified | unverified | blocked | inferred` and non-empty evidence,
and a `provenance` block naming the grade file and corpus path.
`write-artifact.sh` SHALL refuse an artifact lacking either (change-rah-005).

#### Scenario: Refused without provenance

- **WHEN** an artifact omits `provenance.grade_file`
- **THEN** `write-artifact.sh` exits non-zero with the reason

### Requirement: Gaps and sessions persist

WHEN `add_gap` or `add_session` is called over the learner-model binary's
JSON-RPC interface, THEN the record SHALL be returned by `get_concept` for that
concept after a fresh `load`, keyed by the id the call returned. `add_gap`
SHALL refuse an unknown `concept_id` and a `label` outside
`verified | inferred` with an `error` reply. `add_session` with a `session_id`
the model already holds SHALL replace that record, so an open and a close call
produce one session (change-rah-009).

#### Scenario: Round trip

- **GIVEN** a model seeded from a survey with concept `c1`
- **WHEN** the binary is driven over stdin with `add_gap` for `c1`, `add_session`
  touching `c1`, then `load` and `get_concept`
- **THEN** `get_concept` returns the gap under `gaps[]` and the session under
  `sessions[]`, each with the id the write returned
- **AND** `load` returns the gap under `gaps.<gap_id>` and the session in
  `sessions`

#### Scenario: Session update is not a duplicate

- **WHEN** `add_session` is called twice with the same `session_id`, the second
  with a later `ended_at`
- **THEN** the model holds one session with that id and the later `ended_at`

#### Scenario: Gap resolution

- **WHEN** `resolve_gap` is called with a `gap_id` returned by `add_gap`
- **THEN** `get_concept` returns that gap with a non-null `resolved_at`
- **AND** an unknown `gap_id` produces an `error` reply, not an exit

### Requirement: Certification is recorded

WHEN `set_certified` is called for a concept, THEN `certified_at` SHALL be set on
that concept and returned by `get_concept` and `load`, so `learn-certify`'s
concept gate reads a value the checkpoint step wrote. A concept never certified
SHALL carry `certified_at: null`.

#### Scenario: Certify

- **WHEN** `set_certified` is called for `c1` with an RFC 3339 `certified_at`
- **THEN** `get_concept` for `c1` returns that `certified_at`
- **AND** `load` returns it under `concepts.c1.certified_at`
- **AND** an uncertified sibling concept still returns `certified_at: null`

### Requirement: Mastery moves only from the fifth observation

WHEN observations are added to a concept, THEN mastery SHALL stay at the seeded
prior until the fifth observation, and from the fifth SHALL follow
`mastery_new = mastery_old + 0.3 × (score − mastery_old)`.

#### Scenario: Five observations

- **GIVEN** `c1` seeded at mastery 0.4
- **WHEN** four observations with score 1.0 are added
- **THEN** `get_concept` still returns mastery 0.4
- **WHEN** a fifth observation with score 1.0 is added
- **THEN** `get_concept` returns mastery 0.58

### Requirement: Difficulty is live

WHEN a review is recorded, THEN `next_review()` SHALL read the card's current
`difficulty` and update it through the FSRS scheduler, so two reviews with
different ratings produce different difficulties. The scheduler dependency and
its dependency weight are recorded under "FSRS dependency decision" in
`substrate/learner-model/README.md`.

#### Scenario: Review

- **WHEN** a `review` with rating `hard` follows a `review` with rating `good`
  on the same concept
- **THEN** `difficulty` differs between the two returned `fsrs_card` values
- **AND** the later difficulty is the higher one

### Requirement: Skills call the write paths

`learn-grade` SHALL call `add_gap` for each gap it identifies, `learn-practice`
SHALL call `add_session` when a practice session opens and closes, and
`learn-certify` SHALL call `set_certified` at checkpoint issuance and read
`certified_at` and practice sessions from the binary rather than from a file it
does not write.

#### Scenario: Skill instructions name the methods

- **WHEN** the three SKILL.md files are read
- **THEN** `learn-grade` names `add_gap`, `learn-practice` names `add_session`,
  and `learn-certify` names `set_certified`, each with the JSON-RPC call

### Requirement: Ground truth is reviewed before it is used

No eval baseline SHALL be recorded as reviewed unless every item in the
learn-grade eval dataset carries `review_status: reviewed` with a reviewer and
date (change-rah-007). A baseline captured while any item is `draft` SHALL say
so in `metrics-summary.json` and `EVAL-RESULTS.md`.

#### Scenario: Count

- **WHEN** `index.json` is read
- **THEN** `metrics-summary.json` reports the same `reviewed` and `draft` counts,
  and a baseline is called reviewed only when `draft` is 0

### Requirement: The pre-change baseline is captured

`baseline-snapshot.json` SHALL be regenerated from the reviewed truth before a
change that alters the corpus shape re-runs the eval, and a post-change
baseline SHALL be recorded beside it with the metric movement, never tuned
away.

#### Scenario: Snapshot

- **WHEN** the metrics script runs against the reviewed set
- **THEN** the snapshot's timestamp is later than every review date and
  `EVAL-RESULTS.md` cites it
- **AND** a post-change run that could not happen (truth still draft) is
  recorded as blocked with the reason rather than as a baseline
