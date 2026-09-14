# Stage contracts

The table the driver (`scripts/run-research.sh`) enforces at every stage
boundary. A stage is complete only when its required artifacts exist and pass
the validation in the third column; only then does the driver record the stage
in `checkpoint.json`, fire the after-hook, and allow the next stage. A missing
or invalid artifact stops the run with the stage and path named, and the
provenance sidecar is written with `Verification: BLOCKED`.

Paths are relative to the package directory
`${RESEARCH_OUTPUT_DIR:-~/.prometheus/research}/<package_id>/`. Stage skills
still describe their artifacts as `<job_id>/...`; the driver passes the package
directory, so the two names refer to the same place.

| Stage | Requires | Produces (required) | Validation | Hook after |
|---|---|---|---|---|
| 01 planner | query | `plan.md` | Markdown; has `## Sub-questions` with at least one `- ` bullet; the driver counts the bullets to decide scale | `post-stage.sh` |
| 02 search | `plan.md` | `sources/url-list.json` | JSON object with `source_urls` array (may be empty on a blocked search) | `post-stage.sh` |
| 03 retrieve | `sources/url-list.json` | `sources/chunk-<n>.json`, at least one | Each is a JSON object with `url`, `chunk_id`, `text` | `post-stage.sh` |
| 04 collect | chunks | `sources/registry.json` | JSON object with `sources` array, or a JSON array | `post-stage.sh` |
| 05 verify | `sources/registry.json` | `sources/credibility.json` | JSON object with `credibility_scores` object or `verified_sources` array | `post-stage.sh` |
| 06 resolve | `sources/credibility.json` (validated) | `contradictions.json` | JSON object with `contradictions` array | `post-stage.sh`, then `on-contradiction.sh` when any entry has `resolved: false` |
| 07 graph | registry, contradictions | `graph.json` | JSON object with `claims` and `relations` arrays (spec shape) or, until change-rah-010 lands, `nodes` and `edges` arrays (legacy shape) | `post-stage.sh` |
| 08 cite | credibility, graph | `citations.json` | JSON object with `citations` array, or a JSON array | `post-stage.sh` |
| 09 report | all above | `report.md` | Front matter block with `type: research-report` and `verification_status` in `verified`, `partial`, `unverified` | `post-stage.sh` |
| 10 export | everything | `manifest.json`, `index.md` | `scripts/check-research-package.sh --package <dir>` exits 0 | `post-stage.sh`, then `post-export.sh` |

## Stage sets by scale

| Scale | Stages | Chosen when |
|---|---|---|
| `direct` | 01 02 03 05 09 10 | `--scale direct`, or `--scale auto` and `plan.md` has fewer than three sub-question bullets |
| `full` | 01 to 10 for `deep` and `exhaustive`; 01 02 03 04 05 09 10 for `shallow` | `--scale full`, or `--scale auto` with three or more sub-questions |

Every depth ends with 09 report and 10 export, so a run always leaves a package
with a manifest and a sidecar. `shallow` skips resolve, graph, and cite. A
`direct` run additionally skips 04 collect and spawns no subagent stage;
`manifest.json.scale` is `direct`.

## Ordering rules the driver enforces

- Stage 06 does not start until `sources/credibility.json` validates. This is
  the verification-before-review rule at the stage level; change-rah-006 adds
  the same rule for the adversarial review of `report.md`.
- Hooks fire only after validation. `pre-research.sh` fires once before stage
  01 with `RESEARCH_PACKAGE_ID` set; `post-stage.sh` after every validated
  stage; `on-contradiction.sh` after stage 06 when an unresolved entry exists;
  `post-export.sh` after stage 10.
- A hook's non-zero exit marks the stage `blocked` in `checkpoint.json` and
  stops the run.
- `--resume` skips a stage only when `checkpoint.json` lists it in
  `stages_completed` and its artifacts still validate now. A listed stage whose
  artifact was deleted or corrupted is re-run.

## checkpoint.json

Written by the driver, read by `--resume`, `export-package.sh`, and the daemon.

```json
{
  "package_id": "vector-db-rag-20260905-a1f3",
  "job_id": "job-1788000774-ffa477f9",
  "query": "...",
  "depth": "deep",
  "scale": "full",
  "citation_style": "APA",
  "kb_ids": [],
  "model_routing": {},
  "stages_planned": ["01", "02", "03", "04", "05", "06", "07", "08", "09", "10"],
  "stages_completed": ["01", "02"],
  "current_stage": "03",
  "status": "running",
  "blocked": null,
  "created_at": "2026-09-05T10:00:00Z",
  "last_updated_at": "2026-09-05T10:04:12Z",
  "completed_at": null,
  "integrations": {
    "surreal_memory_used": false,
    "sycophancy_correction_used": false,
    "feynman_gate_used": false,
    "adversarial_review_used": false
  },
  "hook_log": [
    { "hook": "pre-research", "exit": 0, "at": "2026-09-05T10:00:00Z" }
  ]
}
```

`status` is `running`, `complete`, `blocked`, or `awaiting_stage`. `blocked`
carries `{stage, reason}` when set. `awaiting_stage` is checkpoint mode: the
driver has validated everything so far and is waiting for the harness to run
`current_stage`.

## The thread merge

Added by change-drt-004. `scripts/merge-threads.sh --package <dir>` folds
`threads/*/` back into the **existing** stage 02/03/04 artifacts. Stage numbers
and their validators do not change; only how those files are produced.

It is a pure function — no model call, no network, no clock, no randomness — so
two runs over the same threads emit byte-identical output. That is the property
Onyx's `citation_utils.py::collapse_citations` has and the reason it cannot
drift, and it is asserted by hash in `tests/merge-threads.sh`.

| Input | Output | Keyed by |
|---|---|---|
| `threads/*/sources.json` | `sources/url-list.json` (02), `sources/registry.json` (04) | canonical URL |
| `threads/*/chunks/*.json` | `sources/chunk-<n>.json` (03) | canonical URL + chunk id |
| `threads/*/claims.json` | `claims.json` | the pack's content-addressed claim id (rah-010) |
| per-thread markers | `citation-map.json` | canonical URL → one global number |

**One document, one citation number.** Three threads citing the same page under
three spellings (`/Guide/`, `?utm_source=…`, `:443/Guide#intro`) resolve to a
single number. The canonical rule lives in `shared/scripts/lib/canonical-url.sh`
so stage 02's dedupe and the merge cannot drift — two copies of a normalisation
rule is exactly how they diverge. It folds case in scheme and host, drops a
default port, a trailing slash, the fragment and tracking parameters, and sorts
the query. It deliberately does **not** strip `www.` or collapse `http` to
`https`: either can change which document you get, and a merge that silently
unions two different pages is worse than one that keeps them apart.

**Duplicate claims collapse; provenance does not.** The same claim id asserted
by three threads becomes one claim carrying three `(thread_id, source_id, quote)`
tuples. Three independent threads reaching the same sentence is evidence, not
duplication.

### The merge is where the no-search rule is enforced

A `tools:` allowlist is intent — a harness may ignore frontmatter. A dossier
citing a source absent from its own thread's `sources.json` means something
fetched outside a worker: a director that searched, or a worker that
sub-dispatched. The merge exits **2** naming the thread, the source and the
rule, and writes no partial artifacts. That single check enforces both the
director's no-search rule and the two-levels-never-three depth ceiling.

## Budgets and bounded concurrency

Added by change-drt-002. A job carries its budgets from the moment it is
created, so a reader never has to guess what bounded a run:

```json
"budgets": {
  "thread_cycles": 8,
  "thread_force_report_minutes": 12,
  "thread_timeout_minutes": 30,
  "director_cycles": 4,
  "job_force_report_minutes": 30,
  "budget_hit": false
}
```

Starting values are adopted from Onyx and verified at source (analysis D-07):
thread hard timeout 30 min and force-report 12 min (`research_agent.py:88,91`),
job force-report 30 min (`dr_loop.py:85`). Only `director_cycles` varies with
depth — shallow 2, deep 4, exhaustive 8 — because thread length is bounded by
how much a worker can hold accurately, not by how deep the job is. These are
defaults, not findings: the per-depth cycle counts are not yet cost-validated.

`budgets` is optional and defaulted, so checkpoints written before drt-002 still
deserialize; a job from an older binary simply has none.

### The concurrency cap is enforced in code, not in a prompt

```bash
prometheus-research threads run --package <dir> --max-parallel 3
```

Onyx states its "never more than 3 in parallel" rule four times in prompt prose
(`orchestration_layer.py:88,116,181,197`) and enforces it nowhere in code. A
prompt is a request, not a bound. The scheduler holds the cap in a semaphore
whose permit is acquired *before* a child process is spawned, so no prompt can
exceed it. `tests/thread-scheduler.sh` over-subscribes six tasks against a cap
of two and reconstructs the true peak from the workers' own timestamps, on the
outside — with the semaphore removed the observed peak is 6 and the assertion
fails.

This binds **process workers only**. In-session subagent dispatch runs inside
the harness's own process, where this semaphore has no reach; that path is
bounded by the director's prompt and, after the fact, by the merge gate.

A worker exceeding its thread budget is **killed**, not merely abandoned: the
child is spawned with `kill_on_drop(true)` and killed explicitly before its row
is written, so the ledger never claims a terminal state while the process is
still alive.

### Every dispatch gets a ledger row

`threads/index.json` carries one row per planned thread — `complete`, `partial`,
`failed`, or `timeout` — because a ledger with silent holes cannot tell a
director what to re-dispatch. Threads the job budget cut before dispatch are
recorded `partial` with the budget named as the reason; silence would read as
"never needed". Every non-complete row carries a `reason`.

## Execution modes

- **Runner mode.** `RESEARCH_STAGE_RUNNER=<command>` names a command the
  driver invokes as `<command> <stage> <package_dir>` for each stage. The
  fixture runner in `tests/fixtures/stage-runner.sh` is one; a headless harness
  wrapper (change-rah-004) is another.
- **Checkpoint mode.** No runner is set. The driver validates what exists,
  then stops at the first incomplete stage: it prints the exact stage skill to
  run, emits one JSON line `{"next_stage": "NN", "skill": "stage-NN-<name>", "package_dir": "..."}`
  on stdout, writes `status: awaiting_stage`, and exits 3. The harness runs the
  stage skill against the package directory and re-invokes the driver with
  `--resume`; the loop repeats until stage 10 validates.
