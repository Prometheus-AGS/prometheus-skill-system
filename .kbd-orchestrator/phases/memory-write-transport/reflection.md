# Reflection — memory-write-transport

Gate: sycophancy-correction analyze_reflect_phase — score 0.018 (PASS); S-08 not detected; one Low S-07 (length).

## Delta

1. The assessment + my earlier deploy memory confidently concluded "bash CANNOT write to surreal-memory — SSE-only, outbox+agent-drain by design." WRONG. Analyze found a plain REST API (POST /api/v1/memory → 201) the server exposes alongside SSE, used by the in-repo Rust client. The "hard transport problem" was self-inflicted (_mem_call POSTed JSON-RPC to the GET-only SSE stream). I shipped a wrong GLOBAL lesson to memory twice before catching it.
2. The Phase 4 memory write-back — the headline "learn across projects" feature — was NON-FUNCTIONAL from when it was built until this phase. Every bash write 405'd to an outbox that never drained (flush used the same broken call). ~6 phases recorded nothing to the server from bash.
3. The fix only fully covers add_memory. Task-streams + compress have NO REST route — unwritable from bash, now DROPPED on flush (telemetry). The Phase 4 execute:before task-stream hook still does nothing durable; I made that honest, not fixed.
4. REST AddMemoryRequest is narrower than the MCP tool (no categories/metadata/importance passthrough in the bridge's {content,user_id} body). Scoping still works (user_id), but richer fields are unused.
5. I added create_entity/create_relation mappings "for completeness" — mapped but NOT live-tested (no caller). Unverified speculative generality.

## Root Cause

1. The assessment probed the SSE endpoint without checking the server ALSO exposed REST, and without reading the in-repo Rust client that already answered it. Anchored on the first transport found. The analyze tier-1 "read the existing client" step corrected it — process caught the error, but after I'd committed the wrong conclusion to memory.
2. Phase 4's fake-curl test returned 200 for any POST, so the broken endpoint was never exercised. It validated the outbox mechanics, not a real write. A live round-trip (added only THIS phase) would have caught it 6 phases earlier.
3. Task-streams genuinely have no REST route; drop is honest for telemetry but means a Phase 4 feature is removed, not delivered. Chose drop over unbounded outbox.
4/5. create_entity/create_relation + struct-match assumption added "for completeness" without a caller or live test — YAGNI.

## Corrective Actions

1. House rule (proven twice this session: SSE + the deploy health-probe): before concluding a service can't do X over transport Y, enumerate ALL its transports/routes (read contracts/router source) AND check how the project's own client talks to it. Memory corrected (wrong entries deleted; recall index now "REST + SSE; bash CAN write").
2. Every memory/transport feature needs a LIVE round-trip test vs the running service, not just fake-curl. Added this phase.
3. Decide deliberately on per-phase task-streams: if they matter, build the agent-drain path (agent replays outbox task-stream lines via mcp__surreal-memory__* before drop); if not, remove the task-stream bridge functions rather than leaving dead calls.
4. If categories/importance matter, widen the REST body (the server's AddMemoryRequest has a categories field). Optional.
5. Drop the unverified create_entity/create_relation mappings unless a live test backs them.

## Recommended Next Phase

None required — focused 2-change fix, complete + live-verified. Honest follow-ups if pursued: (a) agent-driven outbox drain so task-stream/compress lines reach the server via MCP tools instead of being dropped; (b) YAGNI decision on create_entity/create_relation mappings; (c) the standing 6-phase-plan item: a LIVE reload + /loop-tick end-to-end run (this memory fix is now one verified piece of it).
