## Why

An observed Agent Fabric Convergence delivery is frozen at an older C08 source set. Its operation failed, while a later source set passed a separate 122-step integration gate. Cadence correctly refuses to adopt that different-source evidence, but its admission check also rejects the documented corrective path after a failed delivery. This strands later independent work without improving the evidence for the old candidate.

## What Changes

- Permit a corrective delivery only when its evidence identifies the immediately preceding failed iteration.
- Provide an explicit, audited disposition for a finalized failed candidate when the operator elects to continue independent work without claiming that candidate succeeded. Preserve its failed outcome, receipts, and publication obligations.
- Surface the disposition in reports and copy the same skill bytes to mini with the exact full-pack source revision and payload digest.
- Advance The Boss's mini source pin only after the mini commit exists and regenerate its packaged skill payload. Keep native CLI source pins tied to the native binaries they describe.

## Capabilities

### Modified Capabilities
- delivery-cadence: A failed delivery may be corrected or explicitly retired with evidence while remaining failed in the audit history.

## Impact

The shared Cadence CLI, state/report schema, full and mini skill source provenance, and The Boss mini payload pin. No UAR runtime change, no reinterpretation of the C08 gate, and no automatic publication credit.
