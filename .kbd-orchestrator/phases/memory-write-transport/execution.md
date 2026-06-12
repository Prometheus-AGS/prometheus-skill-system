# Execution — memory-write-transport

Backend: **native-kbd** (consistent with all prior changes; no openspec pin in
project.json). Dispatched by claude-code, driven one task per turn.

- change-001-rest-write-path (T1) — ACTIVE. Re-point `_mem_call` at the REST API.
  QA: <3 files, code+test, live round-trip is the gate (skip artifact-refiner).
- change-002-outbox-flush-and-compress (T2) — next. Fix flush + mem0-compress.

Verification is a LIVE round-trip against the running surreal-memory server,
not just fake-curl. Each change commits on green.
