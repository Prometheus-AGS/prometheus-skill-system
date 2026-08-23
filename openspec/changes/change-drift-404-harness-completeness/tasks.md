## 1. Implementation

- [x] 1.0 Record that no normalizer is needed: target source trees have no generated internal marker to restore
- [x] 1.1 Add mandatory `required | install-only` lifecycle metadata for every target and derive completeness checks from it
- [x] 1.2 NEGATIVE FIXTURES: omitted policy, missing required tree, and empty required tree fail naming the target
- [x] 1.3 C-04: run twice and assert byte-for-byte fixture-tree idempotency
- [x] 1.4 C-05: implement in portable Node.js; no shell-version dependency
- [x] 1.5 C-03: document the policy in the schema and change design

## 2. Verification

- [x] 2.1 Negative fixtures fail naming the omitted, missing, or emptied target
- [x] 2.2 Second consecutive validation leaves its fixture tree byte-identical
- [x] 2.3 Target list and lifecycle are derived from `skill-system.json`, not hardcoded in the validator
- [x] 2.4 No normalizer is shipped because there is no generated target-tree invariant to restore
