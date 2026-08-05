# Bounded MCP event-page remediation

The unbounded-event finding was valid. `exec-events` previously returned every
event after the cursor in one result.

The event log, service, facade, and MCP layers now expose a bounded page read.
Each page is limited to 100 events and 8 MiB of serialized event data while the
reader validates the hash chain one event at a time. The MCP result contains
`events`, exclusive `nextAfter`, and `hasMore`; invalid caller limits are
rejected and the generated schema declares the 1–100 range.

The crate README and canonical Docusaurus MCP page document the continuation
contract.

Local verification:

- event-log count, continuation, final-page, and byte-bound assertions passed;
- `events_tool_returns_a_bounded_page_and_continuation_cursor` passed;
- generated contract readback reports `minimum: 1`, `maximum: 100`, and
  `default: 100`;
- warnings-denied clippy passed for the service and executable crates.

Task 6.3 remains the self-referential closure item described in prior feedback
and is completed only after a zero-finding review is recorded.
