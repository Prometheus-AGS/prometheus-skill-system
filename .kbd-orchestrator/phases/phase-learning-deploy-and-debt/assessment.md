# Assessment: phase-learning-deploy-and-debt

Project: prometheus-skill-pack. Date: 2026-10-05 (America/Chicago).
Stage: assessment; implementation has not started. Source baseline: full pack `f35fe2a`, mini `6044ecd`, surreal-memory release `v1.10.1` peeled to `0cf8c7e19b52a0f0b6f8fad0f92448bc8f3561d1`.

## Takeover and independently observed state

The handoff, AGENTS.md, CLAUDE.md and previous reflection were read. Canonical rules take precedence over skill suggestions to run builds or reviewers during assessment. No build, integration test, hosted validation, installation, tag, merge, version-file edit or user-home configuration edit was performed.

- `prometheus kbd --path . status --json` exited 0 at revision 2129. The previous phase had 9/9 changes and 24/24 tasks complete, with all recorded stages complete, but its phase status was still `in_progress`. Run-wide implementation was 82/86; that is not the previous phase's counter. The next command still named the old assessment.
- `git -C . status -sb` exited 0: main tracks origin/main, with pre-existing modified KBD projections, knowledge files, session log and surreal-memory submodule plus untracked phase/archive/knowledge artifacts. Those were preserved.
- `git -C ../prometheus-skills-mini status -sb` exited 0: main tracks origin/main, with untracked historical evidence, review packets and other artifacts. Those were preserved.
- `git -C ../surreal-memory-server log --oneline -1` exited 0: `777cf72 chore: stop tracking SurrealQL REPL history and KBD child progress`. Further inspection found this checkout's main behind its origin/main by nine commits; it does not contain `0cf8c7e`. Do not use its local HEAD as the release pin.
- `prometheus doctor --json` exited **1**, with 16 pass, 1 fail, 3 warn, 2 skip. `hooks.lifecycle` fails because `shared/scripts/lib/__pycache__/agent_identity.cpython-311.pyc` is not manifested. `codex.memories` passes: generate_memories=false and no summary exists. Warnings cover a non-answering discovered control endpoint, missing rollout evidence and unmeasured discovery budgets. The installed hook graph otherwise reports agreement on 33 hooks. This is diagnosis, not release certification.
- Full pack's committed surreal-memory gitlink is `0af8ae1f3c7486bf7128beff8ef83aaf1a832ea6` (v1.10.0); its checked-out submodule is `4161fc4845d59fc06fc748f310410a0011353ec7`. Preserve that local state.
- The handoff's full-pack `versions.toml` reference is inaccurate: no root file exists. Full pack has the gitlink and `config/release-version-matrix.json`; mini has a root `versions.toml` and the same v1.10.0 pin. Do not create an invented full-pack version file.

Worktree: `/Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt`, branch `codex/learning-deploy-and-debt`, created from fetched origin/main. The canonical KBD ledger remains in the main checkout, as documented in the handoff. `worktrees/deploy-main` and `deploy/main` were not changed.

## Lifecycle reconciliation

The requested next-phase helper seeded goals but exited 1 because the previous phase lacked a boundary-start receipt. This was treated as BLOCKED, not a pass. The operator explicitly approved reconciliation in chat: mark the previous phase complete from its existing completion evidence, preserve the missing-receipt gap, then create/activate the new phase. No historical receipt was invented. See `takeover-reconciliation.md` for exact commands and evidence.

The new phase has a valid start receipt `56c3998ff5aa9ab39b85591b5ff78bf78ab8b610fa882a3b421279832111c4dd`, recorded at revision 2135. Canonical progress is 0/0 changes and 0/0 tasks: no implementation plan has been registered yet. This is an unplanned phase, not 100% implementation completion.

## Implementation status and goal gaps

These 14 work areas are assessment scope, not registered implementation tasks. Priority 1–3 follows the operator's order; later ordering remains for analysis/planning.

| ID | Area | Status and evidence | Required next outcome |
|---|---|---|---|
| G1 | Hook bytecode prevention | NOT MET. SessionStart/SubagentStart/SubagentStop wrappers invoke Python without `-B`; `learning_write.py` also spawns a Python feeder without it. The live doctor confirms payload contamination. | Suppress bytecode at production hook dispatch and cover direct hook entry paths. Preserve strict payload integrity verification. Exercise a packaged generation under scratch homes, then verify no `.pyc` or `__pycache__` appears. |
| G2 | Codex memory regrowth | PARTIAL. CLI is 0.158.0; read-only inspection finds `[memories].generate_memories=false`, `[features].memories=true`, no explicit use_memories setting and no summary now. The prior regrowth is recorded but not reproduced. | Identify the actual writer and effective process/agent configuration; distinguish new extraction, retained thread eligibility, queued/in-flight consolidation and stale process settings. Prove the behavior using scratch CODEX_HOME; propose any machine edit separately. |
| G3 | v1.10.1 pins, deployment, recall measurement | PARTIAL. Release tag exists and peels to the expected commit; committed pack pins remain v1.10.0. | Prepare exact full/mini pin diffs, obtain explicit approval for protected edits, then user merge/deployment approvals. Measure recall with the cache deployed, using isolated load services rather than tests against live :23001. |
| G4 | Cache identity | NOT MET. Released `crates/surreal-memory/src/storage/surreal.rs:embed_query` keys on dimensions and query. | Bind cache identity to embedding model/configuration; characterize whether a model can change within the owning cache lifetime before claiming an observed collision. |
| G5 | Cache counters/errors | NOT MET. Disabled mode increments misses before embedding; enabled mode increments after a fallible await. `src/operations.rs` converts `.surreal()` errors to None with `.ok()`. | Define consistent failure/coalescing/disabled semantics and expose storage failures appropriately; retain the write-path-does-not-touch-cache invariant. |
| G6 | Generated-path assertion | NOT MET. `assertSharedGeneratedPaths` checks outputPaths against `generated-paths.mjs`, which asks the same generator for those paths. | Check an independently meaningful invariant or remove the unsupported assertion claim; use production generator/consumer boundaries and a negative control. |
| G7 | Installed doctor repair hint | NOT MET. `tools/prometheus-cli/crates/prometheus-cli/src/commands/doctor.rs:1299` suggests a repo-relative helper path. | Resolve an installed, valid helper from arbitrary working directories and verify through the CLI. |
| G8 | Custom CODEX_HOME | NOT MET. `scripts/install-system.js:82,106` overwrites inherited CODEX_HOME with home/.codex for plugin inspection and memory configuration. | Define precedence with explicit --home, preserve the operator's intended Codex home consistently, and exercise installer dispatch. |
| G9 | Cortex feeder capacity | NOT MET. Every `mirror_cortex` spawns a detached feeder; each server may run for 180 seconds. No cap is present. | Bound concurrency without silent loss of promised durable work, using a real scratch Cortex instance. |
| G10 | Cortex test isolation | NOT MET. `shared/scripts/tests/test-cortex-mirror.sh:190,217` reads/asserts mtime and size of the real home database. | Assert scratch ownership/isolation without depending on live database activity. Check protected-test policy before any edits. |
| G11 | Partition footer boundaries | UNVERIFIED debt. `scripts/memory-index-partition.py:198-225` reserves a conservative footer size and later emits a differently sized count. | Inspect/reproduce 9/10 and 99/100 boundaries through the CLI after coherent implementation; preserve byte budgets and idempotence. Do not report the historical claim as reproduced. |
| G12 | Mini failure baseline | UNVERIFIED current health. The existing baseline contains 19 test-name lines; reflection reports 11 failures on mini main. No suite was run during assessment. | At the final implementation boundary, derive a version-bound baseline from permitted local integration entry points. Legacy isolated tests are not acceptance evidence. |
| G13 | Mini executable modes | NOT MET. `lib/distribution/package-builder.mjs:copyTree` writes bytes without copying modes. Source refresh script is 755; packaged Claude copy is 644. | Preserve executable modes and verify packaged script execution plus generator drift detection. |
| G14 | Production-entry-point acceptance rule | NOT MET in the requested template scope; canonical repository policy already requires integration boundaries. | Locate the operative native/OpenSpec spec-authoring surfaces and make entry-point coverage explicit, without allowing a structural-only assertion to count as full integration evidence. |

## Codex research limits

Context7 resolved the primary `/openai/codex` documentation. Current upstream `codex-rs/state/src/runtime/memories.rs` filters extraction candidates by thread memory_mode; the memories README describes a separately leased global consolidation stage and an agent writing filesystem artifacts. These support investigation of retained/queued work; they do **not** prove the installed 0.158.0 behavior or the cause of this machine's historical write. `use_memories=false` must not be presented as a proven write-side fix.

Sources: https://github.com/openai/codex/blob/main/codex-rs/state/src/runtime/memories.rs and https://github.com/openai/codex/blob/main/codex-rs/memories/README.md. Pin an installed-version source/schema during analysis before recommending settings.

## Spec alignment and recalled lessons

`openspec/specs/kbd-memory-integration/spec.md` requires bounded, relevant prior-context recall and reliable lifecycle logging. `openspec/specs/installed-surface-verification/spec.md` requires checking the installed artifact under the original failure conditions. The implementation gates must install a scratch packaged generation and unset temporary bytecode workarounds; source-only validation is insufficient.

Applicable recalled lines from `prior-context.md`:

- “A component-level test passing is not evidence that the production installer entry point uses that component.” Apply this to G1, G7, G8 and G13 through real dispatch. The lesson's structural-test fallback does not override the current integration-only policy.
- “A `BLOCKED` result from this guard is not evidence that the gate passed.” Apply to every Cargo contention, missing service and lifecycle failure.
- “Tests should discover their execution environment … instead of encoding assumptions from one developer checkout.” Apply to Cortex paths, submodule initialization and mini mode evidence.
- “delivery-cadence freezes every source named in ready; do not point those sources at the actively edited main checkout.” Preserve the protected deployment worktree and use a separately approved refresh.
- Older recalled claims that mark-done never syncs the ledger are superseded by #153. Continue begin-task/end-task and use current reconciliation behavior.

## Knowledge gaps

The recall file contains one knowledge-gap entry: “Goal PARTIAL: G2b less repeated query embedding — sm #46 + v1.10.1 shipped and measured; not yet deployed, because the skill-pack and mini tools/surreal-memory-server pins (versions.toml, operator-authored) still point at 1.10.0.” Carry the deployment/measurement gap forward, with the full-pack version-file correction above. Optional learning suggestion: `/learn-goal query-embedding cache deployment and recall measurement`. No learning session was started.

## Build health, constraints and handoff

- Build/integration health: UNKNOWN for this new phase; intentionally not run before implementation. The requested local doctor is FAIL as reported above, not an acceptance test. An initial `pgrep -x cargo || pgrep -x rustc` found neither; recheck immediately before any eventual build.
- Test coverage: PARTIAL historically; no fresh coverage percentage claimed. Final gates must use scratch HOME, CODEX_HOME, CORTEX_DATA_DIR and PROMETHEUS_PLUGIN_ROOT; no real home databases/configuration, credentials or live :23001 test traffic.
- Generated outputs must be regenerated from source after shared/skills edits and checked at the completed-phase boundary. All validation remains local. Exit 2 is BLOCKED.
- No `.agent-team/project-routing.json` or project team manifest was found. Assessment used sequential lead inspection. Choose the applicable installed team and disjoint implementation ownership before dispatch; reviewers remain dormant until the completed production boundary.
- Model preflight reports ok and two configured identities, but its judge is OpenAI-family. It is not independent review evidence for a Codex producer; choose a verified different-family judge at the final boundary. Assessment adversarial review is deferred under the canonical completed-phase policy, not reported passed.
- User retains all merges and explicit approval over tags, versions.toml and changes under ~/.codex or ~/.claude. Reconciliation approval applies only to the named lifecycle repair, not those actions.

Assessment complete. Next: `/kbd-analyze phase-learning-deploy-and-debt`, then specifications and a registered plan. No implementation task is marked done by this assessment.

## Owner-directed Plan expansion (2026-10-05)

Superseding scope: final full/mini product closure, all relevant worktree/dependency convergence, complete final pin graph (owner-selected full skill-system ba5c4516fd72e60c21ab3bb1d09a63105aeb3c61; other release decisions pending), published Companion, both README families and ALL owned content, in-depth team/model/cross-project documentation in both sites, and maintenance readiness. See scope-amendment.md and plan.md. Earlier v1.10.1-only requirements are historical and no longer the final target.
