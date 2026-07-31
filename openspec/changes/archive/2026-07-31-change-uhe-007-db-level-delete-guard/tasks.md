# Tasks

- [x] Add the constraint/trigger so DELETE of a Builtin skill fails at the database
- [x] ~~Test calls the storage provider DIRECTLY and is refused~~ — **PARTIAL: test written, never executed.** The crate does not compile (pre-existing rmcp/sse-stream break, present at 563ecc2). Guard proven at SQL instead.
- [x] Confirm the existing service.rs:374 guard still returns 409 on the normal path
