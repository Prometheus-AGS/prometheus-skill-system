# KBD phase-closure conformance corpus, v1

Shared, implementation-independent acceptance cases for canonical phase
closure and scope authority (prometheus-skill-system#192,
prometheus-skills-mini#66). Both packs drive the same `prometheus kbd`
runtime, so this corpus pins what each pack's own entry points must do around
it: adopt, evaluate, gate dispatch, continue, and amend.

Each pack runs every case through its **production entry points** with a real
`prometheus` binary, a real signed journal and real `ssh-keygen` signatures,
inside an isolated `HOME`/`XDG_*`/`PROMETHEUS_DATA_DIR`. Nothing here is a
mock; a pack that cannot run a step reports the case as BLOCKED, never PASS.

## Layout

| Path | Content |
|---|---|
| `golden/digests.json` | Contract and per-scenario digests of `golden/contract.json`. `golden/contract.snapshot.json` is the byte-exact snapshot; its sha256 is the contract digest (jq's trailing newline included). |
| `golden/vectors.json` | Closure-key and scope-manifest vectors with the exact byte rules. |
| `schemas/*.schema.json` | Shapes of the disposition view, the scope-authority manifest, and a case file. |
| `cases/NN-name.json` | One acceptance case: ordered abstract steps plus expectations. |

Golden vectors are fixed bytes; a pack passes them when its own digest code
reproduces the listed hex values exactly.

## Fixture every case starts from

One project, one phase `{{phase}}` created, activated, moved to
`in-progress`, with a `phase before` guard receipt (as `kbd-new-phase`
records), one change `C1` and task `T1`, a source file `src.txt`, and a
working contract `adopted-contract.json` whose scenario matrix is `CS-1` and
`CS-2` (both owned by `T1`), budgets `timeMinutes 10`, `toolCalls 100`,
`repairAttempts 2`. Two operator keys exist: `operator` (listed in the
allowed-signers file, pinned to `namespaces="prometheus-kbd-scope-amendment"`)
and `intruder` (not listed).

## Steps

| `op` | Meaning |
|---|---|
| `adopt` | Run the pack's contract-adopt entry point on the working contract. `args.amendment` (optional) is a manifest spec, below; adding it passes `--supersedes <latest> --amendment --signature`. |
| `edit-contract` | Change the working contract: `args.addScenario` appends a scenario (an imported or "tempting" requirement); `args.modifyScenario` changes one scenario's oracle; `args.setField` sets a top-level field (e.g. `"approved": true`). |
| `evidence` | Write `boundary-results.json`. `args.statuses` maps scenario id to status; `args.extra` adds entries for ids outside the contract; `args.run` is `canonical` (default) or `prefix` (canonical run id plus `x`); `args.budget` is `ok`, `at-ceiling`, `exceeded`, `unmeasured`, or an object of measured values; `args.carry` lists scenario ids recorded as carried from the previously bound digest; `args.marker` changes observed text so the evidence digest differs. |
| `close-eval` | Run the terminal evaluator. `args.writer` = `unenrolled` runs it with a device key the journal does not trust. `args.parallel` = 2 runs two evaluators concurrently. |
| `cli-close` | `prometheus kbd phase close --id {{phase}} --evidence-file <evidence> --disposition <args.disposition>`. |
| `stage-gate-execute` | Run the Plan→Execute stage gate with a plan handoff bound to the canonical binding. |
| `next-phase` | Run the next-phase entry point. `args.authorization` is a manifest spec or `none`; `args.successor` names the successor id. |
| `cli-successor` | `prometheus kbd phase successor start` directly with `args.authorization` for `args.successor`. |
| `forge-decision` | `prometheus kbd decision record` with `args.summary` (placeholders expanded). |
| `phase-transition` | `prometheus kbd phase transition --id {{phase}} --status <args.status>`. |
| `phase-create`, `phase-activate` | Direct CLI phase create / activate of `args.id`. |
| `restart` | Delete the project's checkpoint directory, then read status (forces replay from the signed journal). |
| `assert` | Check canonical state: `disposition` token, `phaseStatus`, `bindingRevision`, `successor`, and `events` (count of journal events of each kind). |

A manifest spec is `{ "action": "amend-contract" | "start-successor",
"signer": "operator" | "intruder", "namespace": "scope" | "other",
"tamper": null | "phase" | "diff" | "garbage-signature", "reuse": <label> }`.
Manifests are produced by the pack's template entry point from canonical
state and signed at test time; `tamper` edits a field after generation and
re-canonicalizes before signing; `reuse` replays a manifest+signature
recorded earlier under that label. Private keys are generated per run and
never committed.

Placeholders: `{{phase}}`, `{{runId}}`, `{{projectId}}`, `{{boundDigest}}`.

## Expectations

`expect.exit` is the exact exit code. `expect.stdout`/`expect.stderr` are
regular expressions that must match; `expect.notStdout` must not match.
`assert.events` counts journal events by `kind.type` across the project's
replica journals.

## Results

Each pack writes a receipt per surface it ran (source tree, generated
payloads, installed generation) with the exact commit, `prometheus --version`,
tool versions, argv and exit codes, and lists the surfaces it did not run.
