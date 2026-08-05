# Event-page cursor-progress remediation

The stalled-cursor finding was valid. If the first eligible event exceeded the
page byte budget, the initial implementation could return an empty page with
`hasMore: true` and an unchanged cursor.

The page reader now returns an explicit error when the first eligible event
cannot fit. A later event that does not fit still ends a nonempty page with
`hasMore: true`, so every successful continuation advances `nextAfter`.
Canonical documentation states the same invariant.

Local verification:

- the event-log regression now asserts an explicit byte-limit error instead of
  a stalled empty page;
- the MCP continuation test passed;
- warnings-denied clippy passed for the service and executable crates.

Task 6.3 remains the self-referential closure item described in prior feedback
and is completed only after a zero-finding review is recorded.
