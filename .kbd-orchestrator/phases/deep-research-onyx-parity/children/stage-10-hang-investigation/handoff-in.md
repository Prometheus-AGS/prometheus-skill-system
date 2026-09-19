# Handoff in — deep-research-onyx-parity› stage-10-hang-investigation

> ## ⚠ CANONICAL STATE IS WRONG — READ THIS FIRST
>
> **`position.json` records this child phase as `status: COMPLETE`. That record is
> false. This phase is NOT complete and NOT started.**
>
> At revision 1105 on 2026-09-11 the handing-off session ran:
>
> ```
> prometheus kbd phase transition \
>   --command-id 35b0372f-73fd-4c47-8273-0fbe655e56d2 \
>   --id 'deep-research-onyx-parity::stage-10-hang-investigation' \
>   --status complete
> ```
>
> The intent was to advance the *stage pointer* from assess to analyze. But
> `transition --status` sets the **lifecycle status of the work item**, not a
> stage pointer — there is no `--status analyze`. The command therefore marked
> the whole child phase COMPLETE. It was a mistake by the previous session, not
> a real completion.
>
> **The revert failed and cannot be done with the forward-only API:**
>
> ```
> Error: local runtime rejected the command
> Caused by:
>     work item deep-research-onyx-parity::stage-10-hang-investigation
>     cannot transition from Complete to InProgress
> ```
>
> **What is actually true** (from `assessment.md`, which IS trustworthy and DID
> pass adversarial review):
>
> - Goal 1 — identify the blocking command: **PARTIAL** (chain enumerated, blocker not isolated)
> - Goal 2 — pre-existing vs introduced: **NOT MET**
> - Goal 3 — fix proven by driver-contract: **NOT MET** (no fix exists)
> - Goal 4 — adversarial review of the diagnosis: **NOT MET** (no diagnosis exists)
>
> **Do not trust this child's canonical status, its `progress.json` completion
> block, or any rollup that counts it as done.** Work from `assessment.md` and
> this document. Treat the phase as in-progress at the analyze stage.
>
> **Also still stale:** `exactNextCommand` and `position-reminder.txt` both read
> `/kbd-assess`. Assess is genuinely finished (see below) — **the real next
> command is `/kbd-analyze`.**
>
> Pre-change backups of the runtime projections, at revision 1104, are at
> `.kbd-orchestrator/current-waypoint.json.bak-20260911-181801` and
> `.kbd-orchestrator/position.json.bak-20260911-181801`. They were NOT restored:
> these are projections over an append-only event log now at 1105, so a
> filesystem restore risks desyncing the projection from the log. Reconciling
> this is an operator decision and is deliberately left open.

**Handed off:** 2026-09-11, from a Claude Code session to opencode
**Model:** `kimi-for-coding/k3` (configured as opencode's default)
**Judge:** `MiniMax-M3` (already set in `~/.prometheus/kbd/models.toml`)

---

## State: assess is DONE and has PASSED adversarial review

Do **not** re-run `/kbd-assess`. It ran on 2026-09-10 and its artifacts are here:

| File | What it is |
|---|---|
| `assessment.md` | The assessment. Read it first — it is the current source of truth. |
| `review/assess/findings.json` | Adversarial review: **verdict PASS**, judge `MiniMax-M3`, producer `k3`, `cross_model_check: verified-distinct` |
| `sycophancy/assess-2026-09-10T01-45-35Z.json` | Sycophancy gate: score 0.018, well under the 0.3 threshold — proceed |

The review was a genuine cross-model check over the REST gateway
(`rest-gateway:http://localhost:4000/v1`), not a self-review.

**Your next command is `/kbd-analyze`, not `/kbd-assess`.**

---

## What the assessment established (read `assessment.md` for the full record)

**The defect is intermittent.** This is the single most important finding and it
invalidates the previous session's method. Three observations on 2026-09-09:

- `driver-contract.sh` full suite: **PASS 128/128**, ~2 min
- `driver-contract.sh --scenario full-run`: **HUNG** (timeout 90 → exit 124, zero assertions printed)
- identical second attempt of the same scenario: **PASS 12/12**
- a standalone full deep run through the same fixture environment: **PASS**, all ten stages, under 60 s

One hang, multiple clean passes, same tree. Serial one-shot hypothesis testing
cannot converge on a defect that reproduces roughly one run in three — that is
how six theories got disproved without finding the cause.

**A handoff claim was disproved by reading.** My earlier handoff said
`driver-contract.sh:309` and `:326` "call the live gateway". **That is false.**
Both set `RESEARCH_ADV_DIR` to a stub directory whose `dispatch-judge.sh` is a
heredoc stub written at `driver-contract.sh:288`; `run-research.sh:313` prefers
`RESEARCH_ADV_DIR`, and `RESEARCH_JUDGE_CMD` outranks even that (`:372`). No
scenario in the suite can reach the live gateway. The real defect there is
smaller: those two scenarios exercise the real `build-review-packet.sh` against a
stub dispatch.

**A structural suspect, not a confirmed cause.** `build-review-packet.sh` (919
lines, call 3 in the stage-10 chain) still embeds ~10 `python3` heredoc programs —
the same construct that hung indefinitely in six other scripts on this host and
was fixed by extraction. It was never extracted. But a full deep run completed
through it in under 60 s, so this is a lead, not a diagnosis.

---

## The four review findings to address (none are CRITICAL)

From `review/assess/findings.json`:

1. **WARNING** — the assessment cites handoff claims but the packet declared
   `prior_handoffs: null`. Cite the source path for each inherited claim, or
   populate the packet metadata.
2. **WARNING** — every `file:line` citation refers to files absent from the
   packet's `file_tree` because they exist only on the uncommitted branch. **This
   is the one with teeth** — see "Commit first" below.
3. **SUGGESTION** — packet metadata says `cited_paths: (no file paths cited)` while
   the artifact cites dozens. Fix the metadata.
4. **SUGGESTION** — the CONSTRAINT CHECK covers only C-01/C-02; note C-03/C-04/C-05
   as not-applicable or evaluate them briefly.

---

## Commit first — this blocks verifiable review

**599 files are uncommitted** on `feat/cpc-001-002-integration-contract`. That is
why review finding #2 fired: the reviewer could not verify a single `file:line`
claim, because none of those files were in the packet.

Every future review in this child inherits that problem until the branch is
committed. Commit before producing the next reviewable artifact.

**C-01 drift must be reconciled in the same pass.** Confirmed concrete, not
theoretical:

- source `skills/research/deep-research/scripts/` has **9** `.py` files
  (`assemble-report`, `build-graph`, `check-graph-claims`, `check-manifest-files`,
  `check-manifest-schema`, `detect-contradictions`, `export-package`,
  `merge-threads`, `score-sources`)
- `dist/plugins/claude/.../deep-research/scripts/export-package.py` **does not exist**

So the installed plugins still run the old heredoc form of `export-package.sh` —
the very construct that hangs. `npm run check:distribution` compares generated
output and will fail certification until this is regenerated.

---

## Method for the next session

**Capture, don't theorise.** The assessment's first open question is the method
correction: when the stall next occurs, capture the live process state —
`sample` / `spindump` of the driver process tree, or `ps` of the pipeline —
instead of testing another theory serially. A stall with zero assertions printed
means the driver invocation never returned; which child process holds it is
knowable only from a live capture.

**Redefine the acceptance gate.** Goal 3 says "proven by `driver-contract.sh`
passing 128". The suite **currently passes 128 unfixed**, so that gate certifies
nothing against an intermittent defect. Define a repeat count (e.g. 5 consecutive
clean full suites) during `/kbd-analyze`, before any fix is planned.

**Goal 4 still gates the fix.** "Pass adversarial review on the diagnosis before
any fix is written." Assess passed review; the *diagnosis* does not exist yet.
Get the diagnosis reviewed before writing code.

---

## Environment (verified 2026-09-11)

- `prometheus` CLI: `~/.local/bin/prometheus` — **use this path**, the bare
  `prometheus` on PATH is the unrelated firecrawl CLI
- kbd driver: `~/.claude/skills/kbd-apply/kbd-apply.sh`
- judge routing: `judge = "MiniMax-M3"`, `critic = "MiniMax-M3"`,
  `generator = "kbd-frontier"` — since you run as k3, judge≠producer holds and
  review will not self-block
- opencode config validated: `opencode models` exits 0 and lists
  `kimi-for-coding/k3`; 144/144 `{file:...}` references resolve

---

## Also open, outside this child

`change-drt-006-bench-and-metrics` is 3/4 and unarchived. Task 4 is `blocked`:
three real FACT metrics were delivered and reproduce, but RACE has no package
answering a benchmark task. It was held back only because `driver-contract.sh`
could not be shown passing — which, per the assessment, it now does.

**Projection staleness to be aware of, not to hand-edit.** The assessment records
several: drt-004 shows `merge-threads.sh` pending though it exists with a green
17/17 suite; drt-003 is PENDING yet shows 5/6 tasks; drt-006 reads IN_PROGRESS 0/1
against 3/4 actual. This child's own `progress.json` inherited the parent's
COMPLETE evidence/certification/publication summaries (referencing a change
archived 2026-08-30 on an unrelated branch) — wrong for a fresh child whose
implementation is PENDING with zero registered changes. Move position with
`prometheus kbd phase activate|transition`, never by editing the JSON.
