## 1. Runtime Contract

- [x] 1.1 Add successor initialization to the command/event/reducer path and verify terminal success, non-terminal rejection, field reset/preservation, idempotency, and immutable audit with `kbd-runtime` tests
- [x] 1.2 Update project-document folding for causally ordered run IDs and verify sequential replay plus concurrent successor conflict behavior

## 2. Operator Surfaces

- [x] 2.1 Add `prometheus kbd run start` with durable projection and PAUSE ordering, and verify CLI success/failure behavior with focused tests
- [x] 2.2 Update `/kbd-new-phase` and documentation to roll terminal runs exactly once, and verify the shell smoke suite

## 3. Certification

- [x] 3.1 Run strict OpenSpec, formatting, Clippy, runtime, CLI, Sovereign Sync, and skill tests; build the affected release binaries and record a clean review diff
