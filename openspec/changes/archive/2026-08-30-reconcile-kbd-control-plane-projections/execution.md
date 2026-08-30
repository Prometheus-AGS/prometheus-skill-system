# Execution Evidence: reconcile-kbd-control-plane-projections

## Task 1.1 — Evidence-preserving compatibility relocation

Confirmed the two previously ambiguous filesystem identities before mutation:

- Compatibility source:
  `.kbd-orchestrator/phases/openspec-mirror-drift-cleanup::sovereign-sync-service-reliability`
  contained exactly one non-canonical evidence file, `prior-context.md`.
- Canonical target:
  `.kbd-orchestrator/phases/openspec-mirror-drift-cleanup/children/sovereign-sync-service-reliability`
  contained the populated signed child-phase projection, scope, plan, execution,
  handoffs, review evidence, tasks, and progress.

The compatibility source was preserved at
`.kbd-orchestrator/backups/compatibility-projections/20260829T214439Z/openspec-mirror-drift-cleanup::sovereign-sync-service-reliability`.
Its one-file tree digest is
`d2c3cdf352169be84dd4816e7096c4dc9213246f6f525fd45139164af6e5696c`;
the source and destination `prior-context.md` SHA-256 is
`c9c0102048996caae4fd4185df5022b16b26e4dd16daad3c4c81166bcf2d9ded`.

The tracked receipt at
`.kbd-orchestrator/phases/kbd-control-plane-recovery/reconciliation/compatibility-projection-20260829T214439Z.json`
records the source, destination, canonical target, per-file digest, deterministic
tree-digest contract, file counts, byte counts, and the canonical tree hash
before and after the move. The canonical tree remained byte-identical at
`53f77f26f35925256a630d4d0b76e7f7f426e37e512283777ff38642418e2f51`
across 39 files and 540743 bytes.

No canonical child file or signed runtime event was edited. Task 1.2 owns the
separate post-move live-discovery and backup-readability verification.

## Task 1.2 — Live discovery and evidence readability

Verified the post-move state through the production KBD discovery paths and an
independent digest/readability probe:

1. Top-level filesystem discovery under `.kbd-orchestrator/phases` returns only
   the real parent `.kbd-orchestrator/phases/openspec-mirror-drift-cleanup`; the
   duplicate colon-named compatibility path no longer exists or appears.
2. `prometheus kbd --path . migrate --check` reports the canonical nested child
   ID exactly once, with zero invalid files, zero alias conflicts, no migration
   backup creation, and no journal migration required.
3. `prometheus kbd --path . status --json` at authoritative revision 270 reports
   the canonical child exactly once with parent ID
   `openspec-mirror-drift-cleanup` and slug
   `sovereign-sync-service-reliability`.
4. The preserved one-file backup remains readable and matches the move receipt
   tree SHA-256
   `d2c3cdf352169be84dd4816e7096c4dc9213246f6f525fd45139164af6e5696c`.
   The move receipt itself is readable with SHA-256
   `c609b9c31016a8a1b9d14da5e6674aa54c7ee09fc47f1bb7f3eeca2ff7efb86c`.
5. The canonical target remains readable with all 39 expected files and 540743
   bytes. Its current tree SHA-256 is
   `dc57127add203d453b86f837208ea9391093b1875697fd12f39ca880661a287c`.

The current canonical tree hash is intentionally not compared to the move-time
tree as an immutable value. After the move, the detector replayed the two
runtime-owned `progress.json` and `tasks.md` projections at signed source
revision 261. The original receipt retains equal before/after hashes
(`53f77f26f35925256a630d4d0b76e7f7f426e37e512283777ff38642418e2f51`),
which proves the relocation itself changed no canonical file. The separate
tracked verification receipt at
`.kbd-orchestrator/phases/kbd-control-plane-recovery/reconciliation/live-discovery-verification-20260829T215100Z.json`
records the current revision-bound discovery and readability evidence.

`git diff --check` passed. No hosted validation or canonical hand edit was used.

## Task 2.1 — Deterministic shared distribution generation

Ran the repository-owned shared generator twice with
`npm run build:distribution`. Each pass generated 164 skills for both Claude
and Codex and exited 0. After each pass, every Git-tracked entry across the two
package trees and two marketplace manifests was hashed using its relative path,
POSIX mode, and SHA-256 of file bytes (or symlink-target bytes), then the sorted
manifest was hashed again.

Both passes produced the same aggregate SHA-256:
`2a1e7eed67212a4af4661a4a2a928a75f5b896107bf42971e9c07c152240ec17`
across 2339 tracked files. The per-output counts and byte totals also matched:

- Claude package: 1227 tracked files, 5145503 bytes.
- Codex package: 1110 tracked files, 4659107 bytes.
- Claude marketplace: 1 tracked file, 2690 bytes.
- Codex marketplace: 1 tracked file, 4818 bytes.

Local validation results:

- `npm run validate:codex` — exit 0; checked-in outputs match a fresh isolated
  materialization.
- `npm run validate:strict` — exit 0; strict source validation passed. Existing
  advisory description warnings were non-fatal.
- `git diff --check -- dist/plugins/claude/prometheus-skill-pack dist/plugins/codex/prometheus-skill-pack .claude-plugin/marketplace.json .agents/plugins/marketplace.json`
  — exit 0.

The generated refresh modifies 44 tracked output files. The complete
machine-readable receipt is
`.kbd-orchestrator/phases/kbd-control-plane-recovery/reconciliation/distribution-generation-20260829T215552Z.json`.
No hosted validation was used.

## Task 2.2 — Owned user-install refresh and collision preservation

Preflight verified the active immutable generation before mutation:

- `node scripts/verify-skill-install.js --json` reported 2282/2282 current
  placements (163 skills across 14 targets).
- The refreshed source contains 164 skills. The only new canonical name is
  `kbd-bottleneck-detector`, and that path was absent at every target before
  installation, so no pre-existing user object occupied the new name.
- Every non-pack top-level entry across all 14 skill roots was fingerprinted
  recursively. The pre-install foreign-content digest covered 1548 top-level
  entries, 12201 filesystem objects, and 113490605 bytes with SHA-256
  `24084bd95449ac78fe0dea0278f4705dae00a558d5f749299c644ce3e910ca35`.

Ran only the repository-owned user entrypoint, `npm run install:user`. It
created and activated signed generation
`4158951e90c675b037a01b401a8a6c299e9b684b0e13f21c47bd8cc8a0eaef8d`
with bundle ID
`a76e4233f51ef3236b36248a6a9ba3338b7a1321bef13979c3d72d285ef6e96`,
stamped source commit `d1e48927d61370727dbde734d3da4938f235d6b8`, and exited 0.
The installer's own exhaustive gate then reported 2296/2296 current placements
(164 skills across 14 targets) with zero failures.

Post-install evidence proves both parity and collision safety:

- The same foreign-content inventory retained exactly 1548 entries, 12201
  objects, 113490605 bytes, and SHA-256
  `24084bd95449ac78fe0dea0278f4705dae00a558d5f749299c644ce3e910ca35`.
  Thus unrelated user-owned skill content was not modified, removed, or
  replaced. There were zero live canonical collisions.
- Deterministic full-tree hashes match across source, the active signed
  generation, and both copy-based targets for `adversarial-review`,
  `kbd-apply`, `kbd-bottleneck-detector`, `kbd-inject-agent-rules`,
  `kbd-memory-recall`, and `kbd-process-orchestrator`.
- The exhaustive placement verifier covers the remaining 12 symlink targets
  and reports zero stale, absent, foreign, or dangling managed placements.
- `git diff --check` passed. No hosted validation or manual skill copying was
  used.

The machine-readable evidence is recorded at
`.kbd-orchestrator/phases/kbd-control-plane-recovery/reconciliation/user-install-refresh-20260829T220120Z.json`.

## Task 2.3 — Tested release CLI installation

Before the release gate, a machine-wide process inventory found zero active
Cargo or rustc processes. `sccache` was available at
`/opt/homebrew/bin/sccache` (version 0.16.0; 23.43% Rust cache-hit rate).

Ran the serialized signed gate:

`prometheus kbd --path . gate run --kind compiler-check --scope reconcile-kbd-control-plane-projections:prometheus-cli-release -- cargo build --manifest-path tools/prometheus-cli/Cargo.toml -p prometheus-cli --release`

The gate passed in 951 ms with ID
`e06e6559e77b8b99f7198a0aa14ccce2d932b047cc636eb9507d9d1446b2fb0f`,
source revision 287, and finish revision 289. The workspace-local target was
`tools/prometheus-cli/target`, so no shared worktree target lock was introduced.

The resulting artifact reports `prometheus 1.7.0`, is 15533984 bytes, and has
SHA-256
`f64fb91fd7545deab248f4226ba9b8943a6bd78d790675d997e36ddf836454d3`.
That is the same release hash recorded after the seven external `kbd` CLI
integration scenarios in `add-kbd-registry-prune`, proving this is the tested
release artifact rather than an unverified rebuild variant.

Installed through a single-use staging file in `/Users/gqadonis/.local/bin`,
verified the staged version, hash, and macOS signature, then atomically renamed
it over `/Users/gqadonis/.local/bin/prometheus`. Post-install readback proves:

- `command -v prometheus` resolves to `/Users/gqadonis/.local/bin/prometheus`.
- Installed version is `prometheus 1.7.0`.
- Installed SHA-256 equals the built SHA-256 byte-for-byte:
  `f64fb91fd7545deab248f4226ba9b8943a6bd78d790675d997e36ddf836454d3`.
- `codesign --verify --strict` passes for the installed arm64 executable.
- `prometheus kbd projects --help` exposes both `--prune-missing` and
  `--apply` from the tested change.
- `git diff --check` passes. No hosted validation was used.

The complete receipt is
`.kbd-orchestrator/phases/kbd-control-plane-recovery/reconciliation/prometheus-cli-install-20260829T220621Z.json`.

## Operator course correction — daemon-free KBD and optional sharing

The operator revoked the always-on KBD control plane after repeated service
instability blocked work without providing the intended sharing value. The
service `ai.prometheus.sovereign-sync` was disabled and booted out; launchd no
longer has it loaded and no sovereign-sync process remains. SurrealDB and the
surreal-memory service were not included in this stop action.

The correction was recorded as signed plan revision 7. Canonical task 8 (the
obsolete two-restart certification) was cancelled at runtime revision 296 and
replacement task `8-local` was registered at revision 297. No generated KBD
projection was hand-edited.

Production behavior now has these boundaries:

- ordinary KBD status, audit, event reads, and typed mutations use the signed
  local `kbd-runtime` journal without probing sovereign-sync;
- `prometheus doctor --check control.kbd-runtime` reports an optional skip when
  sharing is disabled;
- ordinary full setup stops/disables an existing sovereign-sync service;
- `--sharing` is required to build, render, install, or start sovereign-sync;
- health/toolchain output identifies the stopped daemon as optional/disabled
  instead of a system failure.

Local full-integration evidence:

- `cargo test --manifest-path tools/prometheus-cli/Cargo.toml -p prometheus-cli --test kbd`
  passed all 7 external CLI scenarios. The first pass exposed an uninitialized
  revision-0 status regression; it was fixed in the production path and the
  same integration target then passed 7/7.
- A real default `install-mcp-services.sh --render-only` omitted both
  sovereign-sync service definitions; the same entrypoint with `--sharing`
  emitted both launchd and systemd definitions.
- The built CLI's `kbd status` emitted no control-plane/unreachable diagnostic,
  and its explicit doctor check returned `optional: true`, `status: skip`.
- A single serialized release build completed in 2m41s. The corrected
  `prometheus 1.7.0` binary was installed atomically with SHA-256
  `be620476158488051809d7bfddba2746327cce773ae46373c57682a674ba78a8`;
  the prior binary remains recoverable at
  `/Users/gqadonis/.local/bin/prometheus.pre-daemon-free-20260829T223037Z`.
- Installed-binary status and doctor probes reproduced the daemon-free results,
  and live launchd/process checks still showed the service disabled and stopped.
- `git diff --check` passed. No hosted validation was used.

Machine-readable evidence is recorded at
`.kbd-orchestrator/phases/kbd-control-plane-recovery/reconciliation/control-plane-disabled-20260829T223037Z.json`.

## Task 3.1 — Live memory write, readback, recall, and retention

The live memory stack was initially unavailable: both SurrealDB on `:28000`
and surreal-memory on `:23001` timed out. launchd still considered both
services running. Service logs showed repeated transaction-conflict/resource-
busy responses and bounded query timeouts. A supervised restart was applied
only to `ai.prometheus.surrealdb-native`; its previous PID 2833 was replaced by
PID 84253. No database directory, RocksDB file, registry, or memory record was
deleted or renamed.

After bounded recovery, the dependency chain was healthy independently of the
now-disabled sharing plane:

- SurrealDB `/health` returned HTTP 200.
- surreal-memory `/health` returned HTTP 200 with service version 1.7.0.
- surreal-memory `/ready` returned HTTP 200 with coordinator, ledger, model
  executor, search index, storage, and tokenizer all ready.
- `ai.prometheus.sovereign-sync` remained stopped and unloaded.

The installed lifecycle writer then posted one uniquely named
`kbd_lifecycle_event` through the production entity route. It exited 0 with no
diagnostic bytes. Entity search returned HTTP 200 and exactly one entity named:

`6ac090a4-3656-4d83-8eb6-2891508196d5/kbd-control-plane-recovery/task/after/6/2026-08-30T03:47:44Z`

The stored observation decoded to the expected project, phase, task/after
edge, index 6, revision-bound total 10, Codex source, and timestamp.

The installed recall skill then replaced the one-line unreachable stub with a
15-line live digest at
`.kbd-orchestrator/phases/kbd-control-plane-recovery/prior-context.md`. Its
SHA-256 changed from
`c9c0102048996caae4fd4185df5022b16b26e4dd16daad3c4c81166bcf2d9ded`
to
`ab6704af3afbaa8161f1a16fa21df7f85e27f6ccdd6f0c61b0295f76e1144230`.
The digest contains five same-project/same-phase task events and none of the
unreachable, MCP-only, HTTP-error, invalid-response, allocation-failure, or
empty-match stubs.

The certification entity is intentionally retained under the documented
365-day server policy. No `.prior-context-search.*` or digest temporary file
remains, no secret-bearing hook output was captured, and the probes did not
mutate signed KBD authority. The machine-readable receipt is
`.kbd-orchestrator/phases/kbd-control-plane-recovery/reconciliation/memory-live-certification-20260830T034744Z.json`.

## Task 3.2 — Recoverable registry prune and idempotency

The live dry run evaluated
`/Users/gqadonis/Library/Application Support/prometheus/kbd/registry.json`
without changing its 31,730 bytes or SHA-256
`fc83b06c3f9f36f5dcfefe445b22675c7e03fd6094f112132cc4a482559096ca`.
It found 28 absent checkout registrations: 14 worktrees, 8 standalone
scratch paths, 5 retired local CI checkouts, and 1 recovered plugin-cache
checkout. Every candidate path was rechecked and remained absent before apply.
The other registrations referenced 27 unique projects, and every corresponding
runtime directory existed.

One explicit `--apply` removed exactly those 28 registrations and retained 28.
The CLI created a permission-restricted rollback bundle under:

`/Users/gqadonis/Library/Application Support/prometheus/kbd/registry-maintenance-backups/20260830T035259.528471Z-20076fd3-17c0-4229-b8e3-fb4ec44c13af`

The backup is 31,730 bytes and has the exact pre-apply SHA-256. Its checksum
file records the same digest, and the structured receipt lists all 28 removed
registrations plus 28 retained registrations. The receipt SHA-256 is
`601ca3c03a9cb713a97c1e46c2bc4805dfeda4136c4bf7846bc339ad577b7ad1`.

No runtime data was removed: the project-runtime directory count remained 42,
and the sorted directory-name digest remained
`63adbf264ef7627a74da46fa89777dc1939a75eda39d508a3f02aef96f26c79e`.
All 27 unique retained project runtime directories still exist. The current
project registration and runtime remained readable at canonical revision 303.

The second `--apply` found zero candidates, removed nothing, returned
`applied: false`, did not create another backup/receipt, and left the registry
hash unchanged. A normal current-project status read refreshed that retained
registration's metadata between the first and second apply; the second apply
itself was byte-idempotent at SHA-256
`ee2b3741ce2f680ad374c210560dd46920aaf8549c351d15fe2917b8c29c5bdf`.

Complete machine-readable evidence is in
`.kbd-orchestrator/phases/kbd-control-plane-recovery/reconciliation/registry-prune-live-certification-20260830T035259Z.json`.

## Task 3.3 — Daemon-free KBD and explicit sharing certification

The optional control plane remained stopped throughout live certification.
`ai.prometheus.sovereign-sync` was unloaded, launchd-disabled, and absent from
the process table before and after every probe. It was never started or enabled.

The real service renderer was exercised in both supported modes. Its default
render produced zero sovereign-sync definitions. The explicit `--sharing`
render produced exactly the launchd plist and systemd user service. The
sharing option is present in the MCP installer, system installer, installed
`prometheus setup`, and binary installer help. A default binary-install dry
run contained no sovereign-sync build step.

The installed CLI then exercised the production local-runtime boundary:

- `prometheus kbd status` exited 0 with empty stderr and no control-plane,
  unreachable, or sovereign-sync warning.
- `prometheus doctor --json --check control.kbd-runtime` exited 0 and returned
  `status: skip`, `optional: true`, with the summary `disabled by default;
  local KBD runtime is authoritative`.
- `prometheus kbd decision record` wrote decision
  `daemon-free-kbd-certification-20260830` through the typed signed runtime,
  advanced canonical revision 309 to 310, emitted no stderr, and folded the
  exact decision back into subsequent status output.

Shell syntax checks, the Node installer syntax check, and `git diff --check`
passed locally. No hosted validation was used. Machine-readable evidence is in
`.kbd-orchestrator/phases/kbd-control-plane-recovery/reconciliation/daemon-free-kbd-live-certification-20260830T040134Z.json`.

## Task 3.4 — Protected-test and diff certification

The repository-owned local protection gate ran through
`npm run check:protected-tests` and exited 0. It compared candidate commit
`d1e48927d61370727dbde734d3da4938f235d6b8` with protected baseline
`1a3ada30aef2287fd0c962fc6c5dee692f333faa` and reported `status: ok` with
zero protected changes. Stderr was empty.

`git diff --check` also exited 0 with empty stdout and stderr. No hosted CI,
workflow, or remote runner was started, watched, or used as evidence. Per the
repository's committed-state integrity rule, the parent-phase certification
will repeat the protection gate after the final commit exists and before push.

Exact argv, exit statuses, output hashes, commits, and the deferred final-commit
gate are recorded in
`.kbd-orchestrator/phases/kbd-control-plane-recovery/reconciliation/final-protected-test-certification-20260830T040433Z.json`.

## Closure remediation — optional sharing and projection-contract replay

The first bounded adversarial review blocked closure with three critical
findings and one warning. The remediation kept implementation complete while
leaving blocker `adversarial-review-reconcile-projections` unresolved until a
fresh second review can verify the finished result.

Production corrections now enforce all of the following:

- canonical and legacy sovereign-sync service identities are stopped and
  disabled by ordinary full setup on macOS and Linux;
- learning recovery never initializes, renders, stops, disables, or starts the
  optional sharing service;
- health output distinguishes an intentionally disabled service from a failed
  or enabled-but-unavailable sharing service;
- setup detects the real pre/post service state and accepts daemon-free success
  only when every installed service identity is inactive and disabled;
- cancelled signed tasks count as terminal evidence without being presented as
  incomplete work;
- derived projections carry `projectionContractVersion: 2`, forcing one safe
  replay whenever projection semantics change without a canonical revision
  change.

The live service was explicitly booted out and both
`ai.prometheus.sovereign-sync` and
`com.prometheusags.sovereign-sync` were launchd-disabled. Neither label is
loaded and no `sovereign-sync` process exists. The installed CLI reports the
service as `DISABLED (optional)`, and
`prometheus doctor --check control.kbd-runtime` passes with an optional skip
while confirming that the signed local runtime is authoritative. Ordinary
`prometheus setup --full --dry-run --non-interactive` plans to disable both
identities and never plans a sovereign-sync build or start.

Two signed external integration gates passed all seven production-entry-point
scenarios:

- gate `68e5834dd8ee9fd14bb7805dcfd47c86b8580c8f6842ca57279666b1f1e3549e`
  at revision 332 covered the review remediation;
- gate `db30c26357d7e5d6340026b1ca4f2a63a435a45348f21dfd15f2ca89be314f8f`
  at revision 336 covered the projection-contract upgrade.

Serialized release gates
`f7544db7dbd5fe856ada07b468b95b0725dd68060bd4242ed3ddbbd3ed2d5c76`
and
`8d1e905f182ee96e0e0fbb0aff95f167a23dd3fd3453fed87b453b2168af03e1`
also passed. The final `prometheus 1.7.0` artifact was installed atomically at
SHA-256
`9261de2b753a8db613ac9d57c2ee42d267ef97ff9d1c33933f16f799d89cf5b4`;
the prior artifact remains recoverable at
`/Users/gqadonis/.local/bin/prometheus.pre-projection-contract-20260830T044658Z`.

Projection repair then replayed seven runtime-owned files from signed revision
338 without changing that revision. The active change now reports 10 terminal
tasks out of 10 and `next_task_pending: null`; the cancelled historical restart
task remains visible in the journal rather than being deleted. The manually
owned `openspec-mirror-drift-cleanup/progress.json` was correctly refused and
left untouched.

Harness generation and distribution generation were each repeated twice with
stable aggregate hashes
`9fa1414cc7d200d59709e9fbec169f13f91b96e1f004891a408e599263199644`
and
`039c35893ac2bf808ba90b8bfdcfed41137907807d83efcc95afc1cd1f9ff3d1`.
Harness, Codex, strict-skill, documentation-sync, and diff validation passed;
the repository-owned installer then verified 2,296/2,296 user placements.
No hosted validation was used.

Machine-readable evidence is recorded at
`.kbd-orchestrator/phases/kbd-control-plane-recovery/reconciliation/adversarial-remediation-20260830T044658Z.json`.

## Round-2 remediation — fold replay, explicit sharing, and bounded closure

The second bounded adversarial review returned `BLOCK` with two critical and
two warning findings; the sycophancy screen passed with score `0.0`. All four
production defects were corrected without enabling the optional service:

- Linux doctor now distinguishes disabled intent from failed or enabled but
  inactive service state across canonical and legacy identities;
- ordinary installation verifies that both service identities finish inactive
  and disabled;
- explicit sharing installs the `sovereign-sync` binary before enabling its
  service, while all non-sharing paths exclude it;
- derived conflict counts now include only unresolved conflicts.

The final conflict-count defect exposed two deeper replay errors. Signed
`ConflictResolved` events were applied before fold-error conflicts existed, and
an old folded checkpoint could survive a replay-algorithm upgrade. Resolution
application now occurs after fold-error detection and folded checkpoints carry
schema version 2, forcing one authoritative replay after the semantic change.
Live status at revision 357 retains both historical conflict records, folds
their operator-signed resolution event IDs, and reports zero unresolved
conflicts. Evidence and certification remain blocked; implementation state was
not falsely promoted.

Two final local integration gates each passed all seven production CLI
scenarios: `5b1f6c49…` at revision 351 and `0c126679…` at revision 355. Serialized
release gates `8c2c1387…` and `88acb6de…` passed at revisions 353 and 357. The
installed, ad-hoc-signed `prometheus 1.7.0` binary has SHA-256
`bc2950ded5691ba938cfdcafd0943baa79b5acb393a0398567f7324a6ef0fca9`;
the previous binary remains recoverable at
`/Users/gqadonis/.local/bin/prometheus.pre-fold-checkpoint-v2-20260830T0012Z`.

Harness generation was byte-stable across two runs with bundle
`d5da8b01…`; distribution generation was byte-stable at `039c3589…`.
Documentation sync, strict skill validation, harness validation, Codex
distribution validation, shell syntax, and diff validation passed locally.
The user installer verified 2,296/2,296 placements. Both launchd identities
remain disabled and unloaded, no sovereign-sync executable is running, and
`prometheus doctor --check control.kbd-runtime` reports the intended optional
skip with the signed local runtime authoritative.

The bounded two-round review limit is exhausted. The round-2 production
findings are remediated, but blocker
`adversarial-review-round2-reconcile-projections` remains open pending an
independent review disposition. Strict OpenSpec verification, archive, parent
certification, commit, and push must not proceed while that blocker remains.

Machine-readable evidence is recorded at
`.kbd-orchestrator/phases/kbd-control-plane-recovery/reconciliation/adversarial-round2-remediation-20260830T051620Z.json`.

## Operator disposition and strict verification

The repository owner explicitly stated: `Accept the round-2 remediation and
clear the blocker`. The signed runtime cleared
`adversarial-review-round2-reconcile-projections` at revision 361, reopened the
evidence dimension for verification at revision 362, and recorded plan revision
13 at canonical revision 363. The two adversarial rounds and their original
findings remain immutable history; the operator disposition does not rewrite
or suppress them.

OpenSpec reported all four planning artifacts complete and 9/9 tasks complete.
The verification workflow mapped all three requirements and seven scenarios to
production code, live receipts, generated/install evidence, and the external
CLI integration target. The KBD-owned strict backend verifier returned
`verify: PASS`; the durable verification report is `verification.md`.

Machine-readable disposition evidence is recorded at
`.kbd-orchestrator/phases/kbd-control-plane-recovery/reconciliation/operator-round2-disposition-20260830T115258Z.json`.

## Final parent certification

The first certification attempt failed closed before starting Cargo because
three valid title-keyed KBD boundary receipts were looked up as map keys derived
from their task IDs. The signed receipts themselves contained the correct typed
`taskId`, phase, edge, source revision, and outcome. Certification now matches
those typed fields instead of assuming the receipt's subject key equals the
task ID. The external CLI integration scenario was changed to use a canonical
title subject with a different task ID, reproducing the real KBD adapter path.

Integration gate `c944880d…` passed all seven external CLI scenarios. Serialized
release gate `ff5b5aeb…` passed, and the exact ad-hoc-signed CLI was installed at
SHA-256 `c2932f7e2a0ec2b4b4e446aeb5477e14b73846e0502915eb88ec2f5b8f48cfb1`.
Certification gate `7475f530…` then passed all seven scenarios at canonical
revision 374.

Harness/Codex freshness, strict skill validation, documentation sync, all 33
main OpenSpec specifications, protected-test integrity, shell syntax, and diff
validation passed locally. Both sovereign-sync labels remain disabled and
unloaded, no process is running, and doctor passes with local signed KBD
authority. No hosted validation was used.

Machine-readable evidence is recorded at
`.kbd-orchestrator/phases/kbd-control-plane-recovery/reconciliation/final-parent-certification-20260830T120117Z.json`.
