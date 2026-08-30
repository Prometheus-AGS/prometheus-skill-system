# Execution Evidence: add-kbd-registry-prune

## Task 1.1 — Structured dry-run inventory

Added a read-only registry-maintenance boundary to
`substrate/kbd-runtime/src/registry.rs`:

- `MissingRegistration` records the exact path, project ID, replica ID, replica
  kind, and read-only classification for each absent registration.
- `RegistryPruneReport` records evaluation time, registry path, candidates,
  removals, apply state, and optional backup/checksum/receipt evidence so the
  apply implementation can return the same stable contract.
- `ProjectRegistry::inventory_missing` acquires a shared registry lock and never
  creates or rewrites `registry.json`. A missing registry produces an empty
  report without creating a backup.
- Candidate discovery follows the registry's deterministic `BTreeMap` order and
  explicitly sorts by path. `Path::try_exists` errors propagate instead of being
  treated as absence, so inaccessible paths fail closed.
- Existing paths are omitted; absent paths retain their exact registered
  project/replica identity in the report.

Per the repository's highest-precedence implementation-first policy, no Rust
build or test command was run and no test was authored during this production
edit. Full production-entry-point integration coverage for existing/missing
classification, mutation, reappearance, idempotence, shared replicas, and
rollback evidence remains batched in task 1.3 after tasks 1.1 and 1.2 complete
the coherent registry implementation. Formatting and compiler/test gates remain
owned by task 2.3.

## Task 1.2 — Locked, evidence-preserving apply

Completed the apply half of the registry-maintenance boundary in
`substrate/kbd-runtime/src/registry.rs`:

- `ProjectRegistry::prune_missing(true)` acquires the exclusive registry lock
  and re-evaluates every registered path under that lock. It does not reuse a
  prior dry-run result, so a path that reappears before apply is retained.
- An apply with no registry or no still-missing entries is non-mutating and
  creates no backup. A mutating apply removes only exact absent-path keys from
  the in-memory registry document; it never traverses or deletes the KBD
  `projects/` runtime tree.
- Before the atomic registry replacement, the runtime creates a unique,
  timestamped `registry-maintenance-backups/<operation-id>/` directory containing
  the original registry bytes, a SHA-256 checksum, a structured JSON receipt,
  and exact rollback guidance. Each evidence file and both evidence directories
  are synced before the live registry changes.
- The write-ahead receipt records the source backup hash and the SHA-256 of the
  exact planned registry bytes. This makes crash state distinguishable without
  claiming that an interrupted replacement committed.
- Registry serialization is shared between the planned hash and the existing
  same-directory temporary-file, fsync, and atomic-rename write path. Repeating
  apply after removal therefore produces no further mutation or backup.

This completes the coherent registry library implementation batch. Consistent
with the immutable implementation-first policy, no Rust build or test was run
and no test was authored during task 1.2. Full production-entry-point integration
scenarios begin in task 1.3; formatting and final compiler/integration gates
remain reserved for task 2.3.

## Task 1.3 — External registry-prune integration scenarios

Added `substrate/kbd-runtime/tests/registry_prune.rs` as an external Cargo
integration target. It exercises the public registry-maintenance entry point
through the real filesystem, registry flock, atomic persistence, and evidence
files without mocks or stubs:

- A dry run observes a removed registered checkout, reports its exact identity,
  leaves `registry.json` byte-for-byte unchanged, and creates no backup.
- The checkout is recreated between dry run and apply. The apply-time locked
  re-evaluation retains it, reports no mutation, and still creates no backup.
- Two registered replicas share one project UUID; only the removed checkout is
  pruned while the existing replica and a real retained `events.jsonl` runtime
  journal remain intact.
- Applied output is checked against the original bytes, checksum file, decoded
  JSON receipt, exact planned-registry hash, and rollback instructions.
- A repeated apply reports no candidates or removals, leaves registry bytes
  unchanged, and does not create another backup.

`git diff --check` passed after adding the target. No Cargo command was run in
this task: compiling and executing this target is intentionally consolidated
with the CLI integration path in task 2.3, after the remaining production and
documentation implementation is complete. This avoids an extra Rust build and
lock cycle while retaining the full integration scenario as the acceptance
gate.

## Task 2.1 — Explicit CLI authority and structured output

Extended the incumbent `prometheus kbd projects` action without changing its
ordinary list mode:

- `--prune-missing` selects the shared `ProjectRegistry::prune_missing`
  maintenance boundary; omission continues to print the existing registry list.
- `--apply` has a clap-level `requires = "prune_missing"` constraint, so apply
  authority cannot be inferred from another flag or accidental argument order.
- `--json` emits the complete camelCase `RegistryPruneReport`, including exact
  candidates, removals, retention count, and optional backup/checksum/receipt
  evidence.
- Human dry-run output names the mode, registry, each candidate, zero removals,
  and the exact apply invocation. Human apply output reports removals and all
  rollback evidence; idempotent apply states that no registry change is needed.

Added one external `prometheus` binary integration scenario to
`tools/prometheus-cli/crates/prometheus-cli/tests/kbd.rs`. Against an isolated
real registry it verifies rejected apply authority, human dry-run output and
byte immutability, JSON dry-run and apply contracts, materialized recovery
artifacts, and idempotent human apply output. It uses the real executable and
registry implementation without mocks or stubs.

`git diff --check` passed. Per the phase build policy, Cargo execution remains
deferred to task 2.3 so library and CLI scenarios compile and run in one
serialized machine-wide build cycle after operator documentation is complete.

## Task 2.2 — Operator runbook and command-reference parity

Updated the incumbent KBD identity/registry runbook at
`site/docs/kbd/tokens-and-authentication.md` with:

- the exact dry-run and explicit-apply commands;
- candidate review guidance for temporarily unavailable mounts and fail-closed
  metadata errors;
- locked apply-time re-evaluation and the guarantee that runtime data and
  existing shared-project replicas are never deleted;
- the timestamped backup directory contents and purpose of each artifact;
- idempotent repeat behavior; and
- an ordered rollback procedure that distinguishes planned/live/backup hashes,
  fails closed on an unknown live hash, requires the exclusive registry lock,
  and verifies registry, runtime, and doctor health after restart.

Added the command to both exhaustive CLI indexes in
`docs/guide/16-cli-and-scripts.md` and `docs/guide/13-tools-reference.md`.
Static comparison with the clap definitions confirms the documented hierarchy:
`prometheus kbd --path <project> projects --prune-missing [--apply] [--json]`.
`--path` is owned by `kbd`; the three maintenance/output flags are owned by
`projects`; and the source declares `--apply` with
`requires = "prune_missing"`.

`git diff --check` passed. No Cargo command was needed or run for this
documentation task; the consolidated Rust and documentation certification
remains task 2.3.

## Task 2.3 — Consolidated local certification

All required validation ran locally after the complete production and external
integration implementation. Cargo/rustc activity was checked machine-wide
before every Rust gate. External `universal_agent_runtime` and Compass builds
caused three fail-closed refusals; no competing Rust process was launched by
this task. The signed blocked receipts remain audit evidence at revisions 237
and 241. Successful Rust gates used workspace-local target identities and the
repository's configured sccache wrapper.

Exact commands and results:

1. `cargo fmt --manifest-path substrate/kbd-runtime/Cargo.toml` — passed.
2. `cargo fmt --manifest-path tools/prometheus-cli/Cargo.toml --all` — passed.
3. `cargo fmt --manifest-path substrate/kbd-runtime/Cargo.toml -- --check` and
   `cargo fmt --manifest-path tools/prometheus-cli/Cargo.toml --all -- --check`
   after Clippy — passed.
4. `prometheus kbd --path . gate run --kind compiler-check --scope
   add-kbd-registry-prune:kbd-runtime-clippy-retry -- cargo clippy
   --manifest-path substrate/kbd-runtime/Cargo.toml --all-targets -- -D
   warnings` — passed in 43.114s; gate
   `a843cf9e289c7cc795c04d8db89a060f297f2b1f5fb6ec9d3ddecd7cfe4e7464`,
   revision 239.
5. `prometheus kbd --path . gate run --kind compiler-check --scope
   add-kbd-registry-prune:prometheus-cli-clippy-retry -- cargo clippy
   --manifest-path tools/prometheus-cli/Cargo.toml -p prometheus-cli
   --all-targets -- -D warnings` — passed in 13.984s; gate
   `c35b0b7bf0798595816f132b1815531dd5ff0b404bdda51fe9d7b02405facebe`,
   revision 243.
6. `cargo test --manifest-path substrate/kbd-runtime/Cargo.toml --test
   registry_prune` — passed 2/2 external integration scenarios in 0.31s.
7. `cargo test --manifest-path tools/prometheus-cli/Cargo.toml -p
   prometheus-cli --test kbd` — passed 7/7 external binary integration
   scenarios in 6.13s. No unit-test or bare `cargo test` command was run.
8. `prometheus kbd --path . gate run --kind compiler-check --scope
   add-kbd-registry-prune:kbd-runtime-release -- cargo build --manifest-path
   substrate/kbd-runtime/Cargo.toml --release` — passed in 16.963s; gate
   `fd245270a38e8ec80d515d268f8350cca38117e4c7a3903e5636628c16fbb23b`,
   revision 245. `control-plane-recover` SHA-256:
   `25319bead48d2732b0bec383107ce18441182a90930fe2242dd2fc0a49b06e7d`.
9. `prometheus kbd --path . gate run --kind compiler-check --scope
   add-kbd-registry-prune:prometheus-cli-release -- cargo build
   --manifest-path tools/prometheus-cli/Cargo.toml -p prometheus-cli --release`
   — passed in 210.086s; gate
   `26bf7f96b0f01d266621645d3c9e04faa2f954918dd0b441540806a791586548`,
   revision 247. Built version: `prometheus 1.7.0`; SHA-256:
   `f64fb91fd7545deab248f4226ba9b8943a6bd78d790675d997e36ddf836454d3`.
10. `npm run check:protected-tests` — passed twice with zero protected-test
    changes between base `1a3ada30aef2287fd0c962fc6c5dee692f333faa` and candidate
    `d1e48927d61370727dbde734d3da4938f235d6b8`.
11. `npx openspec validate add-kbd-registry-prune --strict` — passed.
12. `npm --prefix site run check:public-docs` — passed.
13. `git diff --check` — passed before and after all certification commands.

One broader, non-task docs probe—`npm --prefix site run
check:docs-contracts`—stopped before content evaluation because the generated
`.claude-plugin/plugin.json` is absent. Distribution regeneration and its
contract validation are explicitly owned by the next change,
`reconcile-kbd-control-plane-projections` task 2.1. This deferred parent-phase
gate is recorded without claiming a pass and must be satisfied before parent
certification, commit, or push.
