## Why

Issue #196 needs useful scenario selection, BDD authoring and real-value verification as separate responsibilities. Existing runners and adopted-contract fields already supply execution and identity foundations, while current BDD guidance mixes authoring with per-change failing-test-first execution.

## What Changes

- Add independently invocable `value-scenario-discovery`, called by Assess/Spec/Plan, with a finite owner-calibrated frontier.
- Add explicit author-only routes to existing JS/Rust BDD skills, consuming adopted scenario IDs and oracle sources.
- Add a thin `value-scenario-verify` procedure for the complete authorized value-phase boundary, reusing actual runners and canonical writers.
- Replace conflicting BDD lifecycle/integration guidance while retaining higher-priority host rules, original evidence and failure lineage.

## Capabilities

### New Capabilities

- `value-led-scenario-verification`: bounded discovery, authoring and real-path observation tied to existing contract authority.

### Modified Capabilities

None; #195 owns shared method-authority semantics.

## Impact

scope: testing skills, KBD stage procedure calls/BDD integration references, new small skill registrations, docs/site. Reuses cand-002/004/005/006; no new runner or dependency upgrade. Depends on kbd-195-vlbv-policy-propagation. Generated-output owner: kbd-194-196-delivery-qualification.
