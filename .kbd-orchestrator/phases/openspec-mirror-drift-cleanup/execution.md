# EXECUTION: openspec-mirror-drift-cleanup

Repository: `prometheus-skill-system` (local dir `prometheus-skill-pack`)
Date: 2026-08-23
Branch: `kbd/openspec-mirror-drift-cleanup` @ `64aa64d`
Plan: `plan.md` (cross-model vetted — BLOCK → resolved, 6 findings applied)
Changes: **6** (c400–c405), all passing `openspec validate`

## Backend selection

**Selected: `openspec`**

| Criterion | Finding |
|---|---|
| OpenSpec present | Yes — `openspec/` with `config.yaml`, `specs/`, an archive |
| CLI available | `openspec 1.10.0` — the same version whose upgrade c400 adopts |
| Spec traceability | Yes — each change introduces an enforceable capability |
| Local precedent | **Not followed.** `change-push-001` and `change-push-002` ship no deltas and both **fail** `openspec validate`; all six changes here carry real deltas and pass |

New capabilities: `harness-mirror-currency`, `harness-declaration-integrity`,
`submodule-pin-resolvability`, `session-artifact-hygiene`,
`harness-completeness-gate`, `installed-surface-verification`.

## Runtime mode — why the ledger is file-based here

Unlike the HMA repo (which reports `KBD mode: legacy`), this repository has a
**live runtime**. Typed registration still fails, for a different reason:

```
$ prometheus kbd change register --phase openspec-mirror-drift-cleanup …
Caused by:
    event signer ed25519:e7016c63… is not enrolled
```

`prometheus kbd` exposes no enrollment subcommand, and enrolling a signing
identity is an **authorization boundary**, not a workaround to route around. The
file-based ledger (`progress.json`) is therefore canonical for this phase, and
`prometheus kbd status` still points at a stale completed run
(`docusaurus-github-pages-site-…`, Lifecycle: Completed) rather than this phase.

**Recorded, not silently skipped.** If the signer is enrolled later, the typed
path becomes available and this note explains why the ledger was written directly.

## Dispatch contract

**Driver: `/kbd-apply`** — never bare `/opsx:apply`, which has no KBD awareness:
no hooks, no `progress.json`, no waypoint refresh.

```
/kbd-apply openspec-mirror-drift-cleanup change-drift-400-openspec-110-upgrade
```

### Order

```
c400 ──┐
c401 ──┼── c404 ── c405
c402 ──┤
c403 ──┘
```

c400–c403 touch disjoint paths and may run in any order. c404 must follow them
because it asserts against the **settled** harness set, which c401 may change.
c405 is the terminal barrier: it requires a fully clean tree and is the only
outward-facing change.

### Per-change gate chain

1. `/refine-validate "<change-id>"` against `.kbd-orchestrator/constraints.md`
2. On ALL PASS → `/adversarial-review --mode diff "<change-id>"`
3. On PASS → `openspec validate` → archive
4. On FAIL or BLOCK → certification `BLOCKED`; fix and re-run **both**

Cross-model review is available and verified: judge `k3`, producer
`claude-opus-5`, `verified-distinct`, gateway `http://localhost:4000/v1`.
**Requires `CLAUDE_PLUGIN_ROOT` exported until c405 lands** — the fix that
removes that requirement is what this phase is shipping.

## Execution preconditions

| Fact | Consequence |
|---|---|
| 98 dirty files, md5 `71445c7…` unchanged since the phase opened | The tree is the *input*; deciding it is the work |
| `require_clean_source` runs **5×** in `update-skill-pack.sh` | A step that regenerates files mid-run re-dirties the tree and aborts a later check — commit between stages |
| Script runs `git pull --ff-only` | Must execute on `main`; local `main` is `6de8181`, `origin/main` is `c0d2de1` |
| The submodule's facts moved **twice** during planning | c402 task 1.0 re-measures rather than trusting the plan |

## Blocking decisions inside changes

Each opens by answering its own question — these are first tasks, not
preconditions to starting:

| Change | Task | Question |
|---|---|---|
| c401 | 1.1 | Regenerate `.windsurf/skills`, or retire Windsurf and remove `skill-system.json:144`? |
| c402 | 1.1 | Publish-and-pin (needs the submodule owner) or restore the pin? |
| c403 | 1.3/1.4 | `.devin/` and `.openspec-target`: tracked or ignored? |
| c404 | 1.0 | Does this repo need a normalizer at all, having no `internal: true` invariant? |

## Constraints in force

- **C-01** generated artifacts in sync — c401 (option B regenerates distribution
  output), c403 (`.openspec-target` may be generated), c404. **Verified c400 does
  not touch a C-01 source.**
- **C-02** no committed secrets — c403 must **scan**, not inherit HMA's blanket
  session-log authorization, which does not apply in this repository.
- **C-03** docs updated with surface changes — c401, c404.
- **C-04** generators idempotent — c404 runs twice and asserts clean.
- **C-05** bash 3.2 under launchd — c404 if it ships shell.
- Every change editing a checker ships a **negative fixture before** the passing run.
- One commit per change. The 98 files are never reverted wholesale.
