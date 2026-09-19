# Verification — change-drt-003-director-worker-agents

Repository: `prometheus-skill-pack`

## Acceptance criteria

- The director's `tools:` allowlist contains no search or fetch tool.
- A worker receives only its brief: a fixture assertion confirms no sibling thread content is present in its context.
- A reflection entry exists between a worker's first and second search.
- `tests/driver-contract.sh` still passes its full assertion count with stage numbers unchanged.
- `npm run check:skills-index
npm run check:distribution
npm run validate:codex` passes after the SKILL.md edit (C-01).

## Verify commands

Every acceptance criterion above maps to a command here; run from the repository root, locally, after the coherent edit batch. A command that cannot run is recorded BLOCKED with the reason, never skipped silently.

```verify
bash skills/research/deep-research/tests/driver-contract.sh
/bin/bash skills/research/deep-research/tests/driver-contract.sh
npm run validate:strict skills/research/deep-research
npm run check:skills-index
```

**The allowlist is intent, the merge is enforcement.** On harnesses that ignore `tools:`, drt-004's merge is what actually stops director-sourced content entering the package. This change must not claim enforcement it does not have.

## Evidence

Run locally 2026-09-09 from the repository root. No hosted CI.

| Gate | Result |
|---|---|
| `tests/driver-contract.sh` (`KBD_PRODUCER_MODEL` unset) | **PASS** — 128 passed, 0 failed, **unchanged** after the `run-research.sh` edit |
| `npm run validate:strict skills/research/deep-research` | **PASS** (exit 0) |
| `npm run check:skills-index` | **PASS** |
| `npm run check:distribution` | **PASS** after regenerating |
| `npm run validate:codex` | **PASS** |
| `/bin/bash -n run-research.sh` (C-05) | **PASS** — parses under bash 3.2 |

### The scheduler and merge were proven together, end to end

Not asserted from the wiring — actually run. Three fixture threads with briefs:

```
3/3 threads complete (peak concurrency 3, cap 3) -> threads/index.json
merge-threads: 3 threads -> 2 sources, 2 claims, 2 chunks
stage 02 validator on merged output: passes
ledger rows: 3
```

That is the pair's whole point: the scheduler alone leaves `threads/` populated
and `sources/url-list.json` absent, so stage 02 could never validate. The driver
runs them in one function (`run_threaded_stage_02`) which returns non-zero if
either half is missing, so one cannot be wired without the other.

### The threaded path is opt-in, which is why the driver suite is unchanged

Stage 02 takes the threaded branch only under `RESEARCH_THREADED=1`. Unset — the
default, and what `driver-contract.sh` runs — the pipeline behaves exactly as
before. 128/128 with no validator edited is the evidence for that claim.

### Licence (Onyx is NOASSERTION)

Both agents were written from the mechanism, not from Onyx prose. Verified by
command: `"Acknowledged, please continue"`, `think_tool`, `never more than 3`,
`research_agent`, and `generate_report` are all **absent** from
`research-director.md` and `research-worker.md`.

### Notes

- **`validate:strict` emits advisory warnings** for `stage-09-report` and
  `stage-10-export` (missing trigger/exclusion clauses in their descriptions).
  It exits 0, and neither file is in this change's scope — the `stage-09-report`
  diff in the working tree is earlier uncommitted work, not this change.
- **The installed `~/.local/bin/prometheus-research` predates drt-002** and has
  no `threads` subcommand (Sep 6 build). The end-to-end proof above used the
  freshly built binary. A threaded run on this machine needs a reinstall; the
  driver's helper reports a clear error rather than proceeding, and the default
  unthreaded path is unaffected.

### Verdict

**PASS.** The director has no search or fetch tool and the merge enforces that
mechanically; workers receive their brief and nothing else; reflection is
durable in `reflections.md`; stage 02 invokes the scheduler and merge together
or not at all; the two-level rule and its refusal of a third level are
documented in `SKILL.md` with the reason; and all C-01 gates pass.
