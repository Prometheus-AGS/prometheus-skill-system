# Verification — change-cpc-003-companion-workspace

Repository: `prometheus-companion`  
Depends on: change-cpc-002-skill-ffi-kbd-mobile-split

## Acceptance criteria

- `bash scripts/audit-all.sh` exits 0 with no FAIL (SKIPs for tokens/skill-desc/ui remain honest SKIPs); `bash scripts/audit-deps.sh` passes after the operator's pins edit.
- `cargo check -p prometheus-substrate --features headless` and `--features desktop` succeed (one at a time; one Cargo build machine-wide).
- Every pack dependency in `crates/prometheus-substrate/Cargo.toml` carries `rev = `; no path into the pack.
- Companion runtime shows the reconciled assess stage for `docs-review-for-build-assessment`.
- Change is BLOCKED until the operator confirms the `versions.toml` `[pins]` edit.

- Companion runtime query asserts the specific phase and stage (`docs-review-for-build-assessment` assess complete), not top-level completion; the jq is adjusted to the runtime schema at execution and the adjusted command is recorded.

## Verify commands

Every acceptance criterion above maps to a command here; run from the repository named above, locally, after the edit batch.

```verify
grep -c 'rev = ' crates/prometheus-substrate/Cargo.toml && ! grep -q 'path = "../../' crates/prometheus-substrate/Cargo.toml
prometheus kbd --path . status --json | jq -e '.phases["docs-review-for-build-assessment"] | tostring | test("assess") and (test("\"ready\"") | not)'  # adjust to the runtime schema at execution: assert the assess stage of that phase is complete, not ready
git ls-tree --name-only 773aa5e scripts/ | grep -c audit && bash scripts/audit-all.sh
cargo check -p prometheus-substrate --features headless
cargo check -p prometheus-substrate --features desktop
bash scripts/audit-deps.sh
```

## Evidence

Partially executed 2026-09-02 locally. **3 of 6 tasks complete; the change is BLOCKED.** No hosted CI cited.

### Completed (tasks 3, 4, 5)

- **Task 3 — pins staged.** `prometheus-companion/docs/decisions/versions-pins-proposal.md` written with the exact entries the operator must add: `iroh 1.0.3`, `iroh_gossip 0.101`, both `iroh_*_address_lookup 0.5.0`, `loro 1.13`, `redb 2`, `axum 0.8`, `rmcp 1.8`, `keyring =3.6.3` (exact, inherited from kbd-runtime), `dirs_next 2`, `tauri_plugin_stronghold 2.3.2`, plus three `[decisions]` entries.
- **Task 4 — Appendix A rewritten.** Verified in `docs/00-architecture-and-implementation-plan.md`: `iroh = { version = "1.0.3", ... }` (was 0.30), `rmcp = { version = "1.8", ... }` (was 0.1), the nine `path = "../<crate>"` workspace-internal dependencies replaced by four local members plus four git-rev pack dependencies, the feature list and presets corrected so adopted external services are no longer cargo features, and A.1's `path = "../../crates/prometheus-substrate"` corrected to `../crates/` (0 occurrences of the old path remain).
- **Task 5 — OpenSpec mirrored and runtime reconciled.** OpenSpec was already initialized. `openspec/changes/change-cpc-003-companion-workspace/{proposal.md,tasks.md}` written; `openspec list` shows the change at 3/6. Runtime reconciliation of `docs-review-for-build-assessment`: before, the Companion runtime reported `status: pending` with `stages: {}` while `assessment.md` and `handoffs/assess.handoff.json` existed on disk dated 2026-08-23. After three typed commands (`phase transition`, `stage enter`, `stage transition`), the runtime reports `status: in_progress` with `stages.assess.status: complete` at revision 5.

### BLOCKED (tasks 1, 2, 6) — two independent gates

**Gate 1 — operator `versions.toml` pins (task 1, task 6).**
`prometheus-companion/versions.toml` `[pins]` is still the empty stub (`# name = "x.y.z"`), verified after task 5. `.claude/settings.json` denies `Edit(versions.toml)` to agents; this is a deliberate human decision point, not an obstacle to route around. Creating the workspace before the pins land would either commit unpinned dependencies (violating Companion AGENTS.md §15, exact pins, which `audit-deps.sh` enforces) or require an agent edit to the denied file.

**Gate 2 — the pack commit to pin does not exist (task 2).**
Task 2 requires git-rev dependencies "at the pack commit that includes `change-cpc-002`". `git log -1` in the pack is `1dc5a67` (the 1.8.0 release), and the `change-cpc-001` and `change-cpc-002` work is **uncommitted** in the working tree; the branch is additionally 5 commits behind `origin/main`. There is no commit to pin. Writing a placeholder rev, or pinning `1dc5a67`, would produce a Companion build that consumes a `skill-ffi` still depending on `kbd-mobile` — precisely the breakage `change-cpc-002` exists to prevent. The Appendix A rewrite therefore carries the literal placeholder `rev = "<pack commit containing change-cpc-002>"` rather than a fabricated hash.

### Second run, 2026-09-02 — gates 1 and 2 cleared, tasks 1 and 2 complete

**Gate 1 cleared.** All 11 pins are in `prometheus-companion/versions.toml`,
with `keyring = "=3.6.3"` exact as required.

**Gate 2 cleared.** Pack commit `cfbc262b47ce794cb2ea845c64dc33e70b10e890`
(branch `feat/cpc-001-002-integration-contract`) carries change-cpc-001 and
change-cpc-002.

**Task 1 complete.** Root `Cargo.toml` workspace with members
`crates/prometheus-substrate` and `src-tauri`, a `[workspace.package]` and
`[workspace.dependencies]` table sourcing every version from `versions.toml`,
and `crates/prometheus-substrate` with `desktop` / `headless` presets plus a
`lib.rs` carrying `REQUIRED_CONTRACT_VERSION` and a `Surface` resolver with two
value-asserting tests. Three corrections made while writing it:

- `src-tauri/Cargo.toml` kept its own `[profile.release]`, which Cargo **ignores
  in a workspace** and warns about. Moved to the workspace root; the warning is
  gone (`cargo metadata` reports 0). `panic = "abort"` was deliberately **not**
  carried up: HMA forbids it on FFI release profiles and the substrate will grow
  an FFI surface.
- The workspace moves build output to a root `target/` that
  `src-tauri/.gitignore` does not cover. Added `/target/` to the root
  `.gitignore` before any build ran.
- The relocated node crates (sovereign-sync, sovereign-client, kbd-mobile,
  iroh-docs-adapter) are **not** declared as dependencies yet. They arrive in
  change-cpc-004; declaring them now would make the manifest unresolvable.

**Task 2 complete.** All four pack crates are declared as
`git = "https://github.com/Prometheus-AGS/prometheus-skill-system", rev = "cfbc262..."`.
No `path` dependency into the pack and no committed `[patch]`. `cargo metadata
--no-deps` parses the workspace cleanly.

**Task 6 BLOCKED — the rev is not pushed.**

`cargo check -p prometheus-substrate --no-default-features --features headless`
fails at dependency resolution:

```
fatal: remote error: upload-pack: not our ref cfbc262b47ce794cb2ea845c64dc33e70b10e890
error: failed to get `kbd-runtime` as a dependency of package `prometheus-substrate`
  revision cfbc262b47ce794cb2ea845c64dc33e70b10e890 not found
```

Confirmed diagnosis, not a guess: `git cat-file -t cfbc262` returns `commit`
locally, while `git branch -r --contains cfbc262` returns **nothing** — the
commit is on no remote branch. Cargo resolves a `git` dependency from the
remote, so a local-only commit is unreachable no matter how the manifest is
written. This is the consequence flagged when the commit was created.

The manifests are correct and complete; only the fetch fails. Pinning a
different, already-pushed rev would resolve — and would give the Companion a
`skill-ffi` that still depends on `kbd-mobile`, the exact breakage
change-cpc-002 removed. That is not an acceptable workaround.

**Build discipline note.** A `cargo test` was running in
`universal-agent-runtime` (PID 50613) when the check was due. Per the
one-Cargo-build-at-a-time rule this session waited ~360s for it to exit before
starting, rather than running concurrently. An initial `pgrep` pattern failed to
match it; `ps -p <pid> -o command=` confirmed it was a real build.

### Third run, 2026-09-02 — PR #78 merged, task 6 COMPLETE

PR #78 merged at 2026-09-02T16:23:10Z (merge commit `f60eb81`), making
`cfbc262` reachable from `origin/main`. `git branch -r --contains cfbc262` now
lists `origin/main`.

**One real defect found and fixed by the gate.** The first `cargo check` failed:

```
error: no matching package named `dirs_next` found
help: packages with similar names: dirs-next
```

`versions.toml` names the pin `dirs_next` because a TOML bare key cannot contain
a hyphen; the crates.io package is `dirs-next`. The manifests carried the TOML
key name as the package name. Corrected in both the workspace and substrate
manifests, and cross-checked against `kbd-runtime/Cargo.toml`, which uses
`dirs-next = "2"`. The pin value is unchanged, and `versions.toml` was not
edited. This is exactly the class of error the gate exists to catch, and it
would have shipped had the check been skipped.

**Passing gates**

- `cargo check -p prometheus-substrate --no-default-features --features headless`
  — **Finished in 1m 13s**, resolving all four pack crates from the pinned rev:
  `prometheus-skill-index v1.7.0`, `storage-provider v0.1.0`, `kbd-runtime
  v0.1.0`, `learner-model v0.1.0`, each shown as
  `(https://github.com/Prometheus-AGS/prometheus-skill-system?rev=cfbc262b...)`.
  `keyring v3.6.3` and `loro v1.13.9` resolved at their pinned versions.
- `cargo check -p prometheus-substrate --no-default-features --features desktop`
  — Finished, no errors.
- **Headless links no tauri** (the requirement's scenario): `cargo tree -p
  prometheus-substrate --features headless -i tauri` returns `package ID
  specification 'tauri' did not match any packages`.
- `cargo test -p prometheus-substrate` under **both** presets — **2 passed** each
  (`surface_resolves_from_build_features`, `contract_version_is_the_one_the_pack_publishes`).
  The surface test asserts a different value per preset, so it proves the presets
  actually differ rather than merely compiling.
- `bash scripts/audit-all.sh` — **PASS 7 / FAIL 0 / SKIP 3**, identical to the
  pre-change baseline recorded in this phase's assessment. The three SKIPs
  (tokens, skill-desc, ui) are honest stubs, unchanged by this work.
- Scaffold audit-gate evidence: `git ls-tree --name-only 773aa5e scripts/ | grep
  -c audit` = **11**.

**Build discipline.** No competing Cargo process was running (checked with
`ps -eo pid,command | grep -E '/(cargo|rustc) '`); one build at a time
throughout.

### Superseded — original unblock list

### To unblock

~~1. Operator adds the staged pins~~ — **done**, all 11 present.
~~2. The pack work is committed~~ — **done**, `cfbc262`.

3. **Push `feat/cpc-001-002-integration-contract`** (or merge it) so
   `cfbc262` is reachable from the remote. This is not optional for a `git`
   dependency: Cargo fetches from the remote even when the commit exists in a
   local clone on the same machine.

Then re-run `/kbd-apply change-cpc-003-companion-workspace` for task 6 only.

### Scaffold audit gate (evidence for task 6's precondition)

`git -C prometheus-companion ls-tree --name-only 773aa5e scripts/ | grep -c audit` = **11**: `audit-a11y.sh audit-all.sh audit-deps.sh audit-generated.sh audit-layer.sh audit-naming.sh audit-progress.sh audit-secret-leak.sh audit-skill-desc.sh audit-tokens.sh audit-ui.sh`. The gate every Companion change depends on exists in the scaffold, which resolves the spec-review WARNING that flagged it as unverifiable from the packet.

### Runtime record correction

The driver's `end-task` passes the positional index `i` to the canonical
runtime as the task sequence. Because tasks were executed **out of order**
(3, 4, 5 first, since 1, 2, and 6 are gated), the runtime saw its third
completion arrive as index 5 of 5 and fired the change-complete sentinel:
`change-cpc-003` was briefly recorded `DONE` and counted toward phase progress
at 3 of 13, which was false.

Corrected in the same session with typed commands: tasks 1, 2, and 6 were
registered (`prometheus kbd task register`) and transitioned to `blocked`, and
the change was transitioned to `blocked`. The runtime now reports
`status: blocked` with `tasks: {1: blocked, 2: blocked, 3: complete,
4: complete, 5: complete, 6: blocked}` and phase implementation back at
**2 of 13**.

Carry-forward for the phase: `/kbd-apply` infers change completion from the
`i` argument rather than from the backend's remaining-task count, so any
out-of-order execution over-reports. Later changes in this phase should either
run tasks in order or register every task up front.

### Notes

- No Cargo build was run in the Companion: there is no workspace yet to check, and creating one is the blocked task.
- Cross-repo rule honoured: Companion edits were made in the Companion checkout and mirrored to its OpenSpec; this evidence lives in the pack, which owns the phase run.
