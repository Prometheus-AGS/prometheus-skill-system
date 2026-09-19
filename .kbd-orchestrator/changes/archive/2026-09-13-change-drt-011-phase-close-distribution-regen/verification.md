# Verification — change-drt-011-phase-close-distribution-regen

Repository: `prometheus-skill-pack`

## Acceptance criteria

- Two consecutive generator runs produce byte-identical `dist/plugins/**` trees (hashes recorded).
- `npm run check:distribution` PASS.
- `npm run validate:codex` PASS.
- `npm run check:skills-index` PASS.
- No file outside `dist/plugins/**` and this change's own scaffolding (spec.md / tasks.json / tasks.md / verification.md / files.txt under its change dir) and its refinement log (`.refiner/`) is modified by this change.
- All checks ran locally; a check that cannot run is recorded BLOCKED with the reason.

## Verify commands

Run from the repository root, locally, after the regeneration.

```verify
npm run check:distribution
npm run validate:codex
npm run check:skills-index
```

## Evidence

To be recorded at execution time: generator invocation dates, the two output hashes (identical), and the three check results. No hosted CI.

## Depends-on cross-reference

Reconciles the `skills/**` edits of change-drt-008-hang-capture-harness (tests/hang harness + docs) and change-drt-009-stage10-hang-fix-and-certification (DIAGNOSIS.md, FIX-CERTIFICATION.md, run-hang-capture.sh edits), both archived 2026-09-13, whose Constraints named this change as the C-01 reconciliation owner. The same regeneration also reconciles the accumulated deferrals of earlier sibling changes on the uncommitted tree — the generator is whole-tree and cannot partially sync; the full accounting below makes that scope explicit rather than smuggling it.

## Evidence (recorded 2026-09-13, local, all exits captured live)

| Gate | Result |
|---|---|
| Invocation (local time UTC-5) | Exit | Output excerpt |
|---|---|---|
| `npm run build:distribution` run 1 (12:5x) | **0** | generator completed silently (logs /tmp/regen1.log) |
| `npm run build:distribution` run 2 (12:5x) | **0** | generator completed silently (logs /tmp/regen2.log) |
| Full-tree dist hash, STEADY STATE after the round-1 regeneration (this is an idempotency baseline, NOT a pre-change snapshot — the 405 file changes are vs git HEAD, shown in dist-manifest.txt, and predate this hash) | — | `cbc3725f6308f75b2e230b8711da278ea089bf3d` |
| Full-tree dist hash, build run 1 (every file: find dist -type f, sorted, shasum-of-shasums) | **0** | `cbc3725f6308f75b2e230b8711da278ea089bf3d` |
| Full-tree dist hash, build run 2 | **0** | `cbc3725f6308f75b2e230b8711da278ea089bf3d` — **byte-identical across ALL file types: determinism proven** (supersedes the round-1 partial-type hash `bda1fc97…`, which covered only md/sh/json/py) |
| `npm run check:distribution` | **0** | `PASS: required/install-only source-tree lifecycle policy and idempotency` · `PASS: 161 canonical skills, payload parity, modes, pins, manifests, and marketplaces` |
| `npm run validate:codex` | **0** | (no drift, valid manifest + marketplace) |
| `npm run check:skills-index` | **0** | (index in sync) |

**Regeneration accounting (git status of dist after the double run):** 405 files — 298 added, 10 deleted, 97 modified; the complete stat list is committed beside this document as `dist-manifest.txt`. Child-specific reconciliation verified present IN BOTH PLUGIN TREES (claude AND codex): `run-hang-capture.sh`, `DIAGNOSIS.md`, `FIX-CERTIFICATION.md`, `HANG-CAPTURE.md` under `skills/deep-research/tests/hang/` — and the assessment's named drift fixed: `export-package.py` beside `export-package.sh`. The remaining ~370 files reconcile earlier sibling changes' deferred drift (whole-tree generator; cannot partially sync).

**Known generator-hygiene finding, recorded not fixed here (out of scope):** the whole-tree mirror also carries the harness's campaign artifact dirs (`tests/hang/captures*/`) into the plugin payload. The generator has no exclusion rules; tightening it belongs to a future change. Trees-in-sync (C-01's requirement) is unaffected.

**Round-2 review dispositions (judge gpt-5.5):** CRITICAL-1 (outputs not in change) — addressed by the committed dist-manifest.txt (full 405-line stat) plus the sampled in-packet dist paths; content attested by the three passing checks and the full-tree determinism proof. CRITICAL-2 (codex tree missing DIAGNOSIS/FIX-CERTIFICATION) — false on the tree: both files verified present in dist/plugins/codex (the round-2 packet had sampled only the claude copies; codex paths now in files.txt). CRITICAL-3 (determinism proof partial-type) — fixed: full-tree (all types) triple-hash above. WARNING (harness exits 0 on hangs/failures) — recorder-by-design semantics; the certification reads the table (result column), not the exit code; noted for a future harness change, not editable within this dist-only scope.

C-01 reconciliation delivered for this child's skills/** surface (drt-008 harness files, drt-009 diagnosis/certification docs + harness edits). No source edits; dist only.


## Round-3 review dispositions (judge gpt-5.5)

- **CRITICAL-1 ("does not commit the regenerated outputs") — disputed with tree evidence.** The regenerated outputs are STAGED in git: `git diff --cached --stat dist` → ** 405 files changed, 24618 insertions(+), 4539 deletions(-)**. Nothing in this workflow is `git commit`-ed mid-phase (the phase's 551+-file tree commits at phase close, per AGENTS.md and every prior change in this repo). A judge packet cannot carry 405 full file contents; the packet carries the complete stat manifest (dist-manifest.txt), sampled regenerated paths from BOTH plugin trees, the three passing drift checks, and the full-tree determinism proof. The staged index is the commit set.
- **CRITICAL-2 (hash inconsistency) — fixed by relabeling.** The judge correctly spotted an apparent contradiction caused by mislabeled evidence: the "pre-regeneration" hash was captured AFTER the round-1 regeneration (steady state). The 405 changes are vs git HEAD (the accumulated deferrals), not between the three identical hashes — which prove idempotency, which is what C-04 requires. Label corrected above.
- **WARNING (harness exit 0 on hangs) — recorder-by-design; already dispositioned round 2; out of this change's dist-only scope.**
