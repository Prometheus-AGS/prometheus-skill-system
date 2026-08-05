# Plan — kimi-desktop-extensibility

_2026-08-05. Backend: **native-kbd** (see "Backend divergence"). 5 changes._

## Backend divergence — deliberate

The skill's detection rule says OpenSpec, because `openspec/` exists at the repo
root. That rule is too coarse here: **both backends are in active use** —
`openspec/changes/` holds 107 entries (the prometheus-exec engine),
`.kbd-orchestrator/changes/` holds 150 (everything else, including this phase).

The spec stage already emitted these four changes as native-kbd. Switching
backends now would orphan them and split one phase across two systems. Plan
proceeds native-kbd.

## Ordering

Sequential. Two constraints force it, and neither is stylistic:

**File contention.** `kde-001` and `kde-002` both edit the same manifest dict in
`scripts/install-kimi-desktop-plugin.sh`. Parallel application conflicts inside
one Python heredoc, and the failure is silent — the generator still runs and
emits a *valid* manifest that is simply missing a field. That is the exact shape
of every defect this phase has already hit (Codex `[hooks]`, `{{file:}}`
commands, dangling symlinks): parses fine, does nothing.

**Package contention.** `kde-001 t1` and `kde-003` both install and remove
throwaway packages under `plugin-packages/`. Two probes present at once make an
observed result unattributable — if a tool appears, which package supplied it?

| # | Change | Depends on | Rationale |
|---|---|---|---|
| 1 | `kde-000-skillinstructions-decision` | — | Zero risk, no code. Closes a CRITICAL carried across three handoffs. Do it first so the gate stops re-raising it. |
| 2 | `kde-003-hooks-probe` | **kde-000 closed** (both edit `assessment.md`) | **Before** 001/002. It is pure measurement, and its verdict may add or remove later work. Running it first means 001/002 are planned against measured reality. Also clears `plugin-packages/` before 001 t1 needs it. |
| 3 | `kde-001-mcp-servers` | 003 done (package slot free) | Highest value: Kimi Desktop has skills but no tools. Its own t1 may end it with no manifest change — a successful negative. |
| 4 | `kde-002-session-start` | 001 applied+verified | Same file as 001. Sequential only. |
| 5 | `kde-005-catalog-budget` | no probe package installed (see constraint) | New; see below. |

`kde-004` is intentionally **not** created. It is conditional on `kde-003`'s
verdict, and specifying it now would presume the answer.

## New change: `kde-005-catalog-budget`

Adversarial review flagged that OQ-3 — "is there a catalog/description budget at
145 skills?" — was carried through assess, analyze, **and** spec handoffs with no
change owning it. That is the same failure that produced `kde-000`: a question
recorded in three handoffs is not tracked, it is unowned.

Created at this stage and appended to the spec handoff, which is why it appears
in that handoff's change list — the handoff was updated, not pre-existing.

## Goal 4 — reinstall durability, as a testable gate

Goal 4 ("keep every adopted integration reinstall-durable and free of
app-managed-state traps") was argued in prose at assess and analyze but never
converted into something execute can fail on. It is a gate now:

**Every change that alters the generated manifest MUST pass, as its final step:**

```
rm -rf "<plugin-packages>/prometheus-skill-pack"
bash scripts/install-skills-flat.sh --skills-only
# then assert: package restored, skill count == the count the generator
# reports for the current tree (NOT a hardcoded 145), and THIS change's
# field present
```

**This gate was executed, not assumed.** Adversarial review challenged whether
the command reaches the Kimi installer at all. Running it proved the reviewer
right for the wrong reason: the package was NOT restored — but not because
`--skills-only` skips the installer (it does not; the call sits before the
early-exit). It aborted earlier, at
`install-plugin-generation: release payload verification failed for
shared/scripts/lib/kbd-model-resolve.sh`.

That was a genuine latent defect introduced earlier in this phase: the
temperature fix edited a runtime file whose SHA-256 is pinned in the release
manifest, and the manifest was never regenerated. **Every install on every
machine would have failed this way.** Fixed by regenerating (bundle
`037a68d1`), after which the gate restores 145 skills.

Two lessons the gate now encodes: editing any file under `shared/scripts/` or
`hooks/` requires `node scripts/generate-harness-adapters.js`, and a gate that
has never been run is not a gate.

Deleting the package and rebuilding it is the only test that distinguishes a
durable change from one that survives merely because nobody reinstalled yet.
It applies to `kde-001` and `kde-002`. It does not apply to `kde-000`
(documentation), `kde-003` (probe, removed by design), or `kde-005`
(measurement) — none of them touch the shipped manifest.

A change whose field vanishes after this sequence has **not** met goal 4, even
with every other gate green.

## Ordering is sequential for the contended set only

Precisely: `kde-003 → kde-001 → kde-002` is strictly sequential, because those
three contend on `install-kimi-desktop-plugin.sh` or on the `plugin-packages/`
probe slot. `kde-000` contends on neither — it edits `assessment.md` only. It is
placed first for sequencing convenience (it closes a thrice-carried CRITICAL at
zero risk), not because ordering requires it.

`kde-000` DOES contend with `kde-003 t4`, which also edits `assessment.md`.
Since `kde-000` runs first and `kde-003` edits a different section (E4/E5 vs
E0), this is safe as ordered — but they must not run concurrently. `kde-005` edits no file the others touch, so it is file-independent. Its
"any time" placement carries one caveat the reviewer correctly identified: it
MEASURES the catalog at 145 skills, so it must not run while a probe package
from `kde-001 t1` or `kde-003` is installed — an extra package could change what
the catalog contains. Run it either before `kde-003` or after `kde-002`.

## When kde-001 succeeds by changing nothing

`kde-001 t1` may end that change with no manifest edit — a recorded negative,
explicitly a success. `kde-002`'s dependency is therefore on **kde-001 being
closed**, not on it having produced a diff.

Two consequences execute must honour:

- If `kde-001` closed negative (the daimon refuses loopback URLs), `kde-002`
  still runs: `sessionStart` is a manifest field with no URL, so a loopback
  refusal says nothing about it.
- The file-contention constraint disappears in that branch, since `kde-001` wrote
  nothing — but keep the sequence anyway. The cost is one ordering; the risk of
  a silent heredoc merge is a field that vanishes without failing anything.

## Carried spec contradictions — resolved before execute

The spec handoff carried nine WARNINGs. Five were AC/task contradictions that
would have misfired at execute; all five are fixed in the change files, not
merely noted:

| Was | Now |
|---|---|
| `kde-001` AC4 excluded `forge-rs` unconditionally while `t2b` existed to decide it | AC4 is conditional on `t2b`'s finding |
| `kde-002` verification gate 2 required `sessionStart` present, contradicting criterion 3 | Gate passes in the negative branch too |
| `kde-001` AC1 mandated a literal URL while `t4` could make ports config-read | AC1 defers to `t4`; `t4` reordered before `t3` |
| `kde-003` Approach described the sentinel test its own `t3` forbids relying on | Approach now points at the positive-control rule |
| `kde-000` AC3 depended on a future review's verdict | Replaced with a criterion inside the change's control |

## Agent per change

| Change | Agent | Why |
|---|---|---|
| kde-000 | general-purpose | Documentation edit |
| kde-003 | general-purpose | Probe + verdict; no code |
| kde-001 | general-purpose | Shell/Python generator edit |
| kde-002 | general-purpose | Same |
| kde-005 | general-purpose | Measurement |

No specialist agent applies — this phase edits one shell script and writes
findings. Naming a Rust or React reviewer here would be cargo-culting.

## Library candidates

`library-candidates.json` records 5 candidates: 2 adopt (`mcp-url-transport`,
`session-start-kbd-status`), 1 defer (`mcp-stdio-shim`), 2 reject
(`npx-third-party-servers`, `ship-binaries-in-package`).

Both adopts are **vendor manifest fields, not libraries** — there is nothing to
install and no `library:` annotation to carry. `mcp-url-transport` maps to
`kde-001`; `session-start-kbd-status` maps to `kde-002`.

## Risk carried into execute

The single largest risk is **inertness**: a manifest field that parses and does
nothing. Every change's `verification.md` therefore requires observed execution,
not presence. `kde-001 t1` and `kde-002 t1` are blocking for this reason, and
either may correctly end its change with no code written.

An execute run that produces five green changes and no observed behaviour in
Kimi Desktop has failed, regardless of what the gates say.

## Unresolved review findings

Round 3 reached the skill's max of 2 revision rounds. The CRITICAL above is
fixed (ordering table now encodes the `kde-000` → `kde-003` dependency the prose
described). These remain, and execute should treat them as known:

- **Backend divergence counts.** The 107/150 figures were read from directory
  listings at plan time and match neither figure in the spec handoff. The
  conclusion (both backends active, stay native-kbd) does not depend on the
  exact numbers, but the numbers are unreconciled.
- **`kde-005` table vs prose.** The table cell is terser than the prose
  constraint. An executor reading only the table may miss that the constraint is
  "no probe package installed", not "any time".
- **Ad-hoc manifest regeneration.** Regenerating the release manifest (bundle
  `037a68d1`) fixed a defect that would have failed every install, but it was
  done at plan time outside any change, so it has no spec, no verification, and
  no archive record. It is captured in this plan and in the commit message only.
- **`kde-004` has no owner.** It is conditional on `kde-003`'s verdict; nothing
  states who creates it if that verdict is positive.
