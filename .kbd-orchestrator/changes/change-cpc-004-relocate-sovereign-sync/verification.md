# Verification — change-cpc-004-relocate-sovereign-sync

Repository: `prometheus-companion`  
Depends on: change-cpc-003-companion-workspace

## Acceptance criteria

- `--test integration_tests` = 22 passed; `--test domain_sync` = 4 passed; `kbd-mobile --test wire_compat` passed.
- `--test cli_socket_contract` passes using the installed `/Users/gqadonis/.local/bin/prometheus` binary (`PROMETHEUS_CLI_TEST_BINARY`).
- `cargo clippy ... --lib -- -D clippy::unwrap_used` passes for all four crates; `bash scripts/audit-all.sh` exits 0.
- Pack tree untouched by this change (deletion is change-cpc-009).

## Verify commands

Every acceptance criterion above maps to a command here; run from the repository named above, locally, after the edit batch.

```verify
cargo clippy -p sovereign-sync -p sovereign-client -p kbd-mobile -p iroh-docs-adapter --lib -- -D clippy::unwrap_used
cargo test -p sovereign-sync --test integration_tests && cargo test -p sovereign-sync --test domain_sync && cargo test -p kbd-mobile --test wire_compat
cargo test -p sovereign-sync --test cli_socket_contract
bash scripts/audit-all.sh
```

## Evidence

Executed 2026-09-02/03 locally. No hosted CI cited.

### Passing gates

- **Ported test targets** (all in the Companion workspace, no assertion changes):
  - `cargo test -p sovereign-sync --test integration_tests` — **22 passed**
  - `cargo test -p sovereign-sync --test domain_sync` — **4 passed**
  - `cargo test -p kbd-mobile --test wire_compat` — **1 passed**
- **`cargo test -p sovereign-sync --test cli_socket_contract` — 1 passed.** The
  boundary proof: the pack's **installed** CLI (`~/.local/bin/prometheus`,
  15.5 MB) submits a signed typed mutation over a Unix socket to the
  **Companion-built** daemon (`CARGO_BIN_EXE_sovereign-sync`, a separate
  process), and the commit is read back through the same socket. Asserts the
  runtime path carries the fixture's project id, the revision advances past
  zero, and the committed phase appears in canonical state.
- **`cargo clippy -p sovereign-sync -p sovereign-client -p kbd-mobile -p iroh-docs-adapter --lib -- -D clippy::unwrap_used` — exit 0.**
- `cargo check` passes for all four relocated crates, resolving
  `kbd-runtime`, `storage-provider`, `learner-model`, and
  `prometheus-skill-index` from the git rev `cfbc262b...`.
- `bash scripts/audit-all.sh` — **PASS 7 / FAIL 0 / SKIP 3**, identical to the
  pre-change baseline despite adding ~11k LOC.

### Task 3 was a verification, not a rewrite

The spec assumed `unwrap()` had to be removed from library code. Analysis of
all 218 occurrences across the four crates found **zero in library paths**: every
one is inside a `#[cfg(test)]` module, which Companion AGENTS.md §17 permits.
Spot-checked before trusting the parser (`iroh_docs.rs` first unwrap at line 399,
`#[cfg(test)]` at 391; `sovereign-client` unwraps at 261-262, test module at 243).
The clippy gate then confirmed it independently at exit 0. No typed-error rewrite
was needed, and none was invented to look busy.

### Defects found and fixed during relocation

1. **`iroh-docs-adapter` was missing `serde_json`.** 18 compile errors, all the
   same cause: the file inherited the dependency from `storage-provider` when it
   was a module there. Added at the workspace-pinned version.
2. **`use crate::traits::...` → `use storage_provider::traits::...`.** The
   adapter referenced its former parent crate's internals; the trait stayed in
   the pack, so the path had to become an external one.
3. **`thiserror` version split preserved, not "fixed".** kbd-mobile uses `2`,
   the other three use `1` — that mix is pre-existing in the pack, verified at
   the source. Silently upgrading during a relocation would conflate two
   changes and put an error-type major bump inside a move.

### What was deliberately not copied

`.DS_Store`, per-crate `Cargo.lock` files (the workspace owns one lock),
`com.prometheusags.sovereign-sync.plist` (a stale service definition superseded
by the Companion's own), and `.prometheus/` session logs. Verified: 0 such files
under `crates/`.

### Dependency direction held

`grep 'path = "../'` across the relocated manifests returns only intra-Companion
members (`kbd-mobile` ↔ `sovereign-sync`, both dev-only or local). No path
dependency into the pack; the four pack crates are `{ workspace = true }`
resolving to the git rev. The only textual match for "prometheus-skill-pack" was
sovereign-sync's `description` string, which was updated to name its new owner.

### Build discipline

One Cargo build at a time throughout; no competing process was running at any
check (verified with `ps -eo pid,command`).
