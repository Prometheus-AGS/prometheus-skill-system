# Per-target request replay remediation

The multi-target replay finding was valid. Request replay protection used only
the canonical execution `requestId`, which incorrectly treated a distinct
dispatch to a second enrolled target as a duplicate.

The queue now keys request replay protection by `(requestId,
targetEndpointId)`. Dispatch-ID/hash replay and conflict behavior is unchanged.
One canonical signed request can therefore create one durable record per target,
while a different dispatch ID for the same request and same target is still
rejected.

Local verification:

- `queue::tests::one_request_is_accepted_once_per_target` passed and asserts
  two target records plus same-target replay rejection;
- all 8 transport scenarios passed;
- warnings-denied clippy passed.

Task 6.3 remains the self-referential closure item described in prior feedback
and is completed only after a zero-finding review is recorded.
