# research-agent-hardening — integration evidence

Local evidence for phase `research-agent-hardening` (change-rah-011). Every
result below is the real output of a command run on this machine on the date
given. No hosted CI. A gate that could not run is recorded BLOCKED with the
reason, never described as passed.

Repository: `prometheus-skill-pack`, branch `feat/cpc-001-002-integration-contract`,
base `cfbc262`, working tree uncommitted.

---

## 1. Driver run — the pipeline end to end in a harness session

**Date:** 2026-09-08
**Command:**

```bash
bash skills/research/deep-research/scripts/run-research.sh \
  --query "What execution modes does the deep-research driver support and how does the checkpoint mode differ from runner mode?" \
  --depth shallow
# then, once per stage boundary:
bash skills/research/deep-research/scripts/run-research.sh --resume <package_dir>
```

The query is answered from the pack's own source on purpose: the run must not
depend on a paid search tier, and the phase's open question named a
pack-internal question as the default.

**Package:** `~/.prometheus/research/execution-modes-deep-research-driver-20260908-e404`
**Job id:** `job-1788844973-0f19a4e9`
**Depth / scale:** shallow / full (the scale gate chose `full` from 3 sub-question bullets)
**Stages planned and completed:** 01 02 03 04 05 09 10

### Tool tier before the run

`run-research.sh --check-tools` reported, verbatim:

```
search:tavily      absent
search:firecrawl   absent
search             NONE (stage 02 will be blocked)
python3            present
jq                 present
gateway            http://localhost:4000/v1
surreal-memory     absent (stages 04 and 07 degrade to on-disk only)
runner             none (checkpoint mode)
output-root        /Users/gqadonis/.prometheus/research
```

No web-search adapter is configured on this machine. The run went ahead
deliberately, to prove that an absent tier is reported rather than papered over.

### What the run produced

| Observation | Result |
|---|---|
| Package id shape `<slug>-<yyyymmdd>-<4hex>` | `execution-modes-deep-research-driver-20260908-e404` |
| Provenance sidecar written on entry, before any stage | yes (`.provenance.log`, sidecar present at stage 01) |
| `plan.md` as task ledger + verification log + decision log | yes — ledger lines for every stage, `scale full: planner emitted 3 sub-question(s)` in the decision log |
| Checkpoint mode handshake | exit 3 plus one JSON line `{"next_stage":..,"skill":..,"package_dir":..}` at each of six boundaries |
| `--resume` skipping completed stages | yes — `stage NN already complete; skipping` for each |
| Stage 05 credibility scoring | `score-sources.py` exited **2**, "no sources to score". Scores were **not** synthesized; `sources/credibility.json` records `"label": "blocked"` with the reason |
| Adversarial report review (stage 09→10) | **fired for real**, 132 s, k3 judge, producer `claude-opus-5` |

### Adversarial review of the report

`review/findings.json`: **verdict PASS**, 0 CRITICAL, 4 WARNING, 1 SUGGESTION.
The four warnings were all legitimate criticisms of the draft report and were
fixed before the package was finalized:

| # | Warning | Fix |
|---|---|---|
| 1 | Sub-question 2 carried a `verified` label contradicting the report's own global labelling statement | Relabelled `inferred`; a package is not an independent source for claims about the tool that wrote it |
| 2 | Report said stages 03–05 were "blocked" while the sidecar records `Blocked: none` | Corrected: those stages *completed* on empty inputs with `"label": "blocked"` on each artifact; no stage blocked in the driver's sense |
| 3 | The evidence table asserted behaviours with no per-row labels | Every row now carries a label and names where in the package it is observed |
| 4 | The plan ledger implied `verification_status` was derived after review, before the review ran | Wording corrected in the report's verification-status section |

This is the rah-006 contract working: the producer did not review itself, and
the critic caught a real internal inconsistency the producer had introduced.

### Package validation

```bash
bash skills/research/deep-research/scripts/check-research-package.sh --package <pkg>
```

**RESULT: PASS** (exit 0), after the report corrections. Checks that passed:
manifest validates against the jsonschema; all seven required `files.*` present;
`report.md` `verification_status` agrees with the manifest; provenance verdict
agrees with the manifest; `graph.json` validates against
`research-graph.schema.json` (0 claims); and `verification_status: unverified`
agrees with the independent derivation rule (`--derive` returns `unverified`).

**Sidecar verdict:** `Verification: PASS`, `Blocked: none`,
`Adversarial review: PASS (0 CRITICAL, 4 WARNING)`, sources consulted 0.

**Verification: the package is valid and the report is honestly labelled
`unverified`.** A run without a search tier cannot produce a verified research
finding, and this pipeline says so instead of inventing one. That is the
intended G1/G3 behaviour, not a failure of the run.

---

## 2. Daemon run — `research_start` and `research_export`

**Date:** 2026-09-08. The same query, started through the daemon's REST job API
(`POST /api/v1/jobs`, the same path `research_start` uses), polled to completion,
then exported with the `research_export` MCP tool.

### Attempt 1 — the installed launchd service: BLOCKED (real defect D-A)

```
{"status":"blocked","error":"no harness binary on PATH (looked for claude, then codex);
 install one or set PATH for the daemon"}
```

Both harnesses are installed on this machine (`~/.local/bin/claude`,
`/opt/homebrew/bin/codex`). The daemon could not see them because
`~/Library/LaunchAgents/com.prometheus.research.plist` sets **no
`EnvironmentVariables`/`PATH`**, so launchd gives it the bare default. Its
source template `substrate/prometheus-research/com.prometheus.research.plist`
has no `PATH` key, and `scripts/install-binaries.sh:582` substitutes only
`__HOME__`. Sibling services do it correctly — `shared/launchagents/*.plist`
carry a `__PROMETHEUS_PATH__` placeholder.

The daemon's behaviour here is **correct**: it refused with an actionable
message instead of pretending to run. The defect is the deployment template.

### Attempt 2 — correct PATH, stale installed driver: FAILED (real defect D-B)

With a correct `PATH`, the daemon resolved `claude`, spawned a real headless
child (pid 57785), and the child exited 0 without producing a package. The
daemon refused to call that success:

```
{"status":"failed","exit_code":0,
 "error":"harness exit code 0 without producing a package under /Users/gqadonis/.prometheus/research"}
```

The child's own log (`~/.prometheus/research/<job>/harness.log`) diagnosed it
precisely: `resolve_driver()` fell through to
`~/.claude/skills/deep-research/scripts/run-research.sh`, which is a **1920-byte
stub from the installed plugin generation dated 2026-08-30** — before this phase
began. The repo driver is 30718 bytes. The stub contains zero occurrences of
`--resume`, `checkpoint`, `next_stage`, or `RESEARCH_STAGE_RUNNER` (repo: 19,
15, 2, 5). Its stage loop body is a comment; it emits fake `started`/`completed`
lines and exits 0.

The headless child explicitly **refused to work around it**: "Running it
positionally would have printed twelve lines of fake stage-completion JSON and
exited 0 — a green result with an empty package. That is worse than reporting
the block, so I stopped." It also ruled out a stale-copy fluke by comparing all
five other copies on the machine. That is the anti-sycophancy discipline holding
inside an autonomous child, which is worth more than the run it declined to fake.

### Attempt 3 — `RESEARCH_DRIVER` pointed at the hardened driver: COMPLETE

`resolve_driver()` honours `RESEARCH_DRIVER` first (daemon.rs:96), which is the
documented override.

**Job:** `job-1788849795-776d92a3`
**Package:** `~/.prometheus/research/execution-modes-deep-research-driver-20260908-62ee`

The headless child drove all seven stages autonomously with no human turn.
Observed stage progression from the job endpoint: `starting → planner → search →
retrieve → collect → verify → report → export → complete` with `exit_code: 0`.

| Check | Result |
|---|---|
| `check-research-package.sh --package` | **PASS WITH NOTES** (exit 0) |
| `--derive` (independent derivation) | `partial` — agrees with the manifest |
| Sources consulted | 8 (found without a paid search adapter) |
| Labelled claims | 30, in `sources/credibility.json` |
| Feynman grade in the report | 0.965, `misconceptions_absent: 1.0` — the learn integration gate ran |
| Sidecar verdict | `Verification: PASS WITH NOTES`, `Blocked: none` |
| Adversarial review | **blocked**: `judge unavailable (Invalid API key)` — the child's environment had no gateway credential |

`research_export` over MCP stdio returned:

```json
{"format":"json","job_id":"job-1788849795-776d92a3",
 "package_id":"execution-modes-deep-research-driver-20260908-62ee",
 "output_path":".../manifest.json","report_path":".../report.md",
 "verification_status":"partial","verification_verdict":"PASS WITH NOTES"}
```

**Verification: the daemon path works and produces a package that validates
identically to the driver path.** Both requirements of this change are met on
the code. Two deployment defects were found and are recorded below rather than
repaired inside this change, because both are install-surface concerns outside
its declared scope.

### Defects found by this run

| Id | Defect | Evidence | Disposition |
|---|---|---|---|
| D-A | `com.prometheus.research.plist` sets no `PATH`, so the launchd daemon cannot find any harness binary | attempt 1 error; plist has no `EnvironmentVariables`; `install-binaries.sh:582` substitutes only `__HOME__` | **Open.** Fix is a `__PROMETHEUS_PATH__` placeholder matching `shared/launchagents/*.plist` and the matching `sed` in the installer. Outside this change's file scope |
| D-B | The installed plugin generation ships a pre-phase stub driver, so `resolve_driver()`'s `~/.claude` candidate resolves to a script with none of the stage contract | 1920 vs 30718 bytes; zero matches for four contract markers; generation dated 2026-08-30 | **Open.** Fix is a reinstall so the current generation is published; until then set `RESEARCH_DRIVER`. Outside this change's file scope |

Neither defect is in the daemon's own logic: in both cases it refused to report
a success it had not achieved, which is the behaviour change-rah-004 built.

---

## 3. Final local certification

**Date:** 2026-09-08. All commands run locally from the repository root. No hosted CI.

### Integration suites

| Suite | Result | Environment note |
|---|---|---|
| `skills/research/deep-research/tests/driver-contract.sh` | **128 passed, 0 failed** | Requires `KBD_PRODUCER_MODEL` **unset**: its `review-real-dispatch` negative control asserts that a missing producer identity is a refusal the driver cannot hide |
| `skills/research/deep-research/tests/scoring-graph.sh` | **45 passed, 0 failed** | — |
| `skills/learn/feynman-loop/tests/learn-coherence.sh` | **58 passed, 0 failed** | Passes under `/bin/bash` 3.2 and bash 5 |
| `skills/process/adversarial-review/tests/run-fixture-suite.sh` | **28 passed, 0 failed** | Requires `KBD_PRODUCER_MODEL` **set**: Group A cannot prove judge ≠ producer without it |

The two producer-model requirements are **opposite and both correct**. A single
shell exporting the variable fails the driver's negative control; a shell without
it fails the review gate's discrimination test. Each suite is run in the
environment it specifies. An earlier combined run that exported the variable
globally showed one failure in each suite for exactly this reason — recorded here
because the misreading ("flaky suites") was available and wrong.

### Cargo suites (changes rah-004, rah-009)

Run with the Cargo slot free (`pgrep -x cargo` polled first, per the one-build rule).

| Suite | Result |
|---|---|
| `cargo test -p learner-model --test rpc_roundtrip` | 1 passed, 0 failed |
| `cargo test -p prometheus-research` — `job_execution` | 1 passed, 0 failed |
| `cargo test -p prometheus-research` — `job_lifecycle` | 4 passed, 0 failed |
| `cargo test -p prometheus-research` — `mcp_tools` | 3 passed, 0 failed |
| `cargo test -p prometheus-research` — `sse_stream` | 2 passed, 0 failed |
| `cargo test -p prometheus-research` — lib unit tests | 3 passed, 0 failed |

### C-01 generator idempotence

The constraint requires the reconciliation change to run each generator twice
and prove byte-identical tracked output.

| Generator | Hash before | After run 1 | After run 2 |
|---|---|---|---|
| `generate:skills-index` (SKILLS.md, sha256) | `8fc4cb02…f2653` | `8fc4cb02…f2653` | `8fc4cb02…f2653` |
| `build:distribution` (dist tree, sha256 of sorted per-file hashes) | `ca7cba5b…8a859` | `ca7cba5b…8a859` | `ca7cba5b…8a859` |

Both are idempotent: re-running changes nothing. The distribution row initially
recorded only one post-run hash while the round-1 disposition claimed two; the
round-2 judge caught the discrepancy and the second run was performed rather
than the claim softened.

### Why the two skill counts differ

`validate:strict` reports **144 skills validated** while the regenerated
SKILLS.md declares **161 distributed skills**. They enumerate different sets and
both are correct: `validate:strict` walks the source tree
(`skills/<category>/<skill>/`, recursing into sub-skills) and validates only
what carries a `SKILL.md`, while the distribution index counts every skill
published to the Claude and Codex plugin surfaces, including imported submodule
skills and bundled sub-skills that the strict validator does not enumerate as
top-level entries. The numbers are not expected to match.

### The relocated test suite has no deletion hunk, and should not

`skills/learn/tests/` was created by change-rah-008 in this same uncommitted
working tree and never reached a commit (`git ls-tree HEAD -- skills/learn/tests/`
is empty). The relocation therefore appears in the diff as new files with no
deletion counterpart. Stale `git add -N` intent-to-add entries for the old path
were cleared with `git rm --cached`, so the index now matches the disk, where
`skills/learn/tests/` does not exist.

### `research_start` over MCP

Finding 6 of round 1 was right that section 2 started the job over REST and
only asserted equivalence. `research_start` invoked directly over MCP stdio:

```json
{"depth":"shallow","job_id":"job-1788869507-25a6efde",
 "query":"What execution modes does the deep-research driver support…",
 "started_at":"2026-09-08T12:11:47Z"}
```

The equivalence is also structural, not just observed: `research_start`
(`mcp_server/mod.rs:65`) and `POST /api/v1/jobs` (`http_server/rest.rs:60`) both
call `spawn_job(&query, &depth, max_sources, &citation_style)` with the same
arguments, from the same import.

### Validators and C-01 reconciliation

| Gate | Result |
|---|---|
| `npm run validate:strict` | PASS — 144 source-tree skills validated (see "Why the two skill counts differ") |
| `npm run check:skills-index` | PASS |
| `openspec validate --specs` | PASS — 31 passed, 0 failed |
| `npm run check:distribution` | PASS |
| `node scripts/check-harness-adapters.js` | PASS |
| `npm run check:services-manifest` | PASS |
| `npm run validate:codex` | PASS |

### Two failures found and repaired in this change

| Gate | Failure | Repair |
|---|---|---|
| `validate:strict` | `tests: SKILL.md is required but not found` | change-rah-008 placed `learn-coherence.sh` at `skills/learn/tests/`, the category level where the validator enumerates skills. Relocated to `skills/learn/feynman-loop/tests/`, matching where the deep-research and adversarial-review suites live; the suite's `ROOT` was repointed and it passes unchanged (58 assertions) |
| `check:distribution`, `validate:codex` | `generated output is stale: dist/plugins/claude/prometheus-skill-pack` | Regenerated with `npm run build:distribution` (161 skills for Claude and Codex). This is the C-01 reconciliation this change exists to perform: the `SKILL.md` edits made across this phase are inputs to that generator. The regeneration also cleared two pre-existing deletions left by the companion migration (`ai.prometheus.sovereign-sync` plist and systemd unit) |

Both were real; neither was deferred.

### Package validation

| Package | Source | Result |
|---|---|---|
| `execution-modes-deep-research-driver-20260908-e404` | driver, harness session | **PASS** |
| `execution-modes-deep-research-driver-20260908-62ee` | daemon, headless child | **PASS WITH NOTES** |

### The task-4 verify string was itself defective

`kbd-apply verify` reported FAIL. The cause was the verify string, not the work:
it chained all four suites with `&&` in one shell, and two of them require
**opposite** `KBD_PRODUCER_MODEL` states (driver-contract needs it unset for its
no-producer refusal control; the review fixture suite needs it set for Group A).
No single shell can satisfy both. Every gate passes when run in the environment
it specifies, verified individually and recorded above. The archived
`tasks.json` carries the corrected string and a note explaining the change.

**Verification: certification passes.** Four integration suites, seven validators
including all three C-01 reconciliation checks, and both evidence packages.
