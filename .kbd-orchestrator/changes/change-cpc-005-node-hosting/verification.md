# Verification — change-cpc-005-node-hosting

Repository: `prometheus-companion`  
Depends on: change-cpc-004-relocate-sovereign-sync

## Acceptance criteria

- `--test continuity` passes: launches the pack daemon binary and the headless Companion binary as separate processes on one data root, asserts revision and frontier equality then a successful append.
- Headless start without key file exits non-zero with the documented message; `ls` of the data root shows no new files.
- `bash scripts/audit-all.sh` exits 0.

- The continuity test asserts the signed audit export, the receipt set, and the run history are identical before the append and fully retained after it (goal: project.loro, receipts, run history migrate intact).

## Verify commands

Every acceptance criterion above maps to a command here; run from the repository named above, locally, after the edit batch.

```verify
( unset PROMETHEUS_DEVICE_KEY_FILE; PROMETHEUS_DATA_DIR=$(mktemp -d) ./target/debug/prometheus-companion-headless; test $? -ne 0 ) && test -z "$(ls -A "$PROMETHEUS_DATA_DIR" 2>/dev/null)"  # fail-closed headless start writes nothing
cargo test -p prometheus-substrate --features headless --test continuity
bash scripts/audit-all.sh
```

## Evidence

Executed 2026-09-03 locally. No hosted CI cited.

### Passing gates

- `cargo test -p prometheus-substrate --no-default-features --features headless --lib` — **7 passed** (2 surface/contract, 3 paths, 2 identity).
- `cargo test ... --test continuity` — **1 passed**, two real processes on one data root.
- `cargo build ... --bin prometheus-companion-headless` — builds.
- **Real fail-closed behaviour**, not just a unit test: with no key file and an empty `XDG_CONFIG_HOME`, the headless binary printed the two-remedy message, exited **1**, and `find` over the data root showed it wrote **nothing**.
- `bash scripts/audit-all.sh` — **PASS 7 / FAIL 0 / SKIP 3**, unchanged from baseline.

### The continuity test was wrong at first, and the negative controls caught it

The first version passed while proving nothing. Two controls were run:

| Control | First version | After the fix |
|---|---|---|
| A: Companion never starts | passed (wrong) → then **FAILED** correctly | FAILED correctly |
| B: Companion opens a *different* data root | **passed (wrong)** | **FAILED** correctly |

Root cause, found by running control B rather than assumed: **`prometheus kbd` does not require a node.** It reads the local signed runtime straight from `PROMETHEUS_DATA_DIR` and only uses the control socket when a control plane is present. Verified directly: with no node listening at all, `prometheus kbd status --json` still exits 0 and returns a full report. So every CLI-mediated assertion in the draft was measuring the CLI against a data root, never the node.

Fixed with two assertions the CLI cannot satisfy on the node's behalf:

1. `GET /health` **directly over the Unix socket** (dependency-free HTTP/1.1 client in the test) — nothing answers there unless the node is running.
2. `GET /api/v1/kbd/projects` over the same socket, asserting the registry contains the project the *pack daemon* wrote. The registry is read from the node's **own** data root, so this is precisely what catches a node pointed at the wrong one.

Control B now fails with `the Companion node's registry does not contain the project the pack daemon wrote (<uuid>) — the node opened a different data root`.

### What continuity actually proves

Pack daemon writes a phase → daemon stopped → Companion headless node opens the *same* data root → assertions: revision, frontier, `runId`, and `latestBoundaryReceipts` all unchanged; the pack's phase present in the Companion's view; the signed audit export no smaller. Then the Companion appends its own phase: revision advances, **both** phases present, so the append added history rather than replacing it.

### Architectural fact recorded

The CLI's independence from the node is not a defect — it is the open-core boundary working. The pack stays fully functional with no Companion installed (goal G6, D-02 option D). It does mean **any future test asserting "the node did X" must assert over the socket**, never through the CLI.

### Notes

- `node.rs` composes the daemon's own startup sequence (`HttpService::start_unix` → `AppState::try_new_with_startup_handle` → `gate.install`) rather than reimplementing it; a second startup path would not be covered by the 22 existing integration tests.
- `paths.rs` mirrors `kbd-runtime`'s resolution exactly and never uses Tauri's path resolver, which would join the bundle identifier and silently open a second empty journal.
- Task 5's `begin-task` was refused by the bottleneck guard ("not a unique canonical work item") because change-cpc-004 has an identically titled task; the gate itself was run and passed.
- One Cargo build at a time throughout.
