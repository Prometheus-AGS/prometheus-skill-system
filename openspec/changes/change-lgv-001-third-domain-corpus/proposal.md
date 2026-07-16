# Proposal — change-lgv-001-third-domain-corpus

Build a fresh corpus JSON for a third eval domain, matching the exact
schema `content-grounding.sh` produces and `learn-grade` consumes:
`{concept_id, sources: [{source_ref, source_type, confidence,
is_misconception, content_summary, key_points?, misconceptions?}]}`.

Domain choice: something outside software/KBD (to avoid the eval
self-referentially testing the pack's own docs), e.g. general science
(cellular respiration, thermodynamics) or history (a specific well-documented
event). 10-15 sources, 3-5 explicit misconception entries.

Reuses `docs/learn/meta-corpus/kbd-lifecycle-corpus.json` and
`skill-pack-corpus.json` as the other 2 of 3 domains (no construction
needed for those — see plan.md open question #3 resolution).

## Goal
G-01 (partial — third domain only; first two domains already exist).
