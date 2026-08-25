## 1. Mutation Initialization

- [x] 1.1 Repair the legacy-aware initializer to accept `NotInitialized`, preserve compatible waypoint state, and verify focused helper tests cover first initialization and idempotent reuse.
- [x] 1.2 Route every typed mutation precondition through automatic initialization while leaving read-only commands on replay-only behavior, and verify focused CLI unit tests pass.
- [x] 1.3 Replace the speculative migration status hint with automatic-initialization guidance and include the runtime path in initialization failures, verified by status and error assertions.

## 2. Process Contract

- [x] 2.1 Add an isolated compiled-CLI integration fixture proving a registered empty runtime remains empty after status, initializes on its first typed mutation, preserves compatible legacy state, and does not initialize twice.
- [x] 2.2 Extend the fixture with a rejected typed mutation and verify the process exits non-zero without recording the rejected command as committed.

## 3. Local Certification

- [x] 3.1 Run strict OpenSpec validation, focused `kbd-runtime` and `prometheus-cli` tests, affected-workspace formatting and Clippy, and record the observed results.
- [x] 3.2 Build the release CLI and run an installed-binary isolated proof for issue #265 before publishing the upstream commit.
