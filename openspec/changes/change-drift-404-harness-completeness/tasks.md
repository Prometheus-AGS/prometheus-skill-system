## 1. Implementation

- [ ] 1.0 Answer with evidence: does this repo need a normalizer? It enforces no internal:true invariant. If not, record that and ship the assertion alone
- [ ] 1.1 Add the per-harness completeness assertion, deriving the set from skill-system.json
- [ ] 1.2 NEGATIVE FIXTURE FIRST: empty one harness in a scratch copy; the gate must fail naming it
- [ ] 1.3 C-04: run twice, assert the second run leaves the tree clean
- [ ] 1.4 C-05: bash 3.2 compatible if shell
- [ ] 1.5 C-03: document the gate

## 2. Verification

- [ ] 2.1 Negative fixture fails naming the emptied harness
- [ ] 2.2 Second consecutive run leaves `git diff --exit-code` clean
- [ ] 2.3 Harness list derived from skill-system.json, not hardcoded
- [ ] 2.4 The normalizer question is answered in the change, either way
