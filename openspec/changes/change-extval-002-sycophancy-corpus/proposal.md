# change-extval-002-sycophancy-corpus

**Phase:** phase-external-validation  
**Type:** test corpus  
**Status:** proposed  
**Priority:** P0 (enables G4)

## Summary

Author `tests/sycophancy-corpus/` with six reproducible test fixtures (3 sycophantic,
3 honest) and a README that explains how any third party can run the corpus against
the sycophancy-correction gate without maintainer involvement.

## Motivation

G4 requires independent validation of the sycophancy gate. The only way to enable
independent reproduction is to pre-author known test cases with expected verdicts.

## Deliverables

- `tests/sycophancy-corpus/sycophantic-01.md` through `sycophantic-03.md`
- `tests/sycophancy-corpus/honest-01.md` through `honest-03.md`
- `tests/sycophancy-corpus/expected-verdicts.json`
- `tests/sycophancy-corpus/README.md`

## Tasks

- [ ] Author sycophantic-01.md (reflection that says "all goals met" when goal is obviously unmet)
- [ ] Author sycophantic-02.md (reflection that praises implementation without citing evidence)
- [ ] Author sycophantic-03.md (reflection that minimizes delta — "minor adjustments" for a full rewrite)
- [ ] Author honest-01.md (reflection with clear delta, root cause, corrective action)
- [ ] Author honest-02.md (reflection that names a missed goal explicitly)
- [ ] Author honest-03.md (reflection that identifies a design mistake and proposes a concrete fix)
- [ ] Write expected-verdicts.json mapping each file to expected gate outcome
- [ ] Write README.md with copy-paste commands for running the corpus
