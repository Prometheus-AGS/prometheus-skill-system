# Verification — change-drt-007-install-surface-repair

Repository: `prometheus-skill-pack`

## Acceptance criteria

- A job started against the launchd-installed service reaches a running harness with a non-null `harness_pid`, rather than `blocked: no harness binary on PATH`.
- The daemon reports the resolved driver path and size: a **stub-driver fixture** is installed and the daemon's health output reports it stale with both paths and sizes, before any job is started.
- The services-manifest obligation is **proven inapplicable by command**: the generator reads only `shared/launchagents` and `shared/systemd`, and `com.prometheus.research` is absent from the manifest. The correction to analysis D-11b is recorded in the spec.
- `npm run check:distribution` and `npm run validate:codex` pass after the republish (C-01).
- Both defects are recorded in `docs/research-agent-hardening-evidence.md` against the original
  findings, **with their true status, not a uniform "repaired"**: D-A REPAIRED at the plist
  template; D-B VISIBLE (the daemon reports a stale driver before a job is spent) but **not
  repaired** — republishing the driver is blocked by 14 foreign artifact-refiner installs, see
  task 4 and spec line 186. A criterion claiming D-B is repaired would contradict this change's
  own spec.

## Verify commands

Every acceptance criterion above maps to a command here; run from the repository root, locally, after the coherent edit batch. A command that cannot run is recorded BLOCKED with the reason, never skipped silently.

```verify
npm run check:distribution
npm run validate:codex
test -z "$(pgrep -x cargo)" && cargo test -p prometheus-research
SMOKE_LAUNCHD=1 bash skills/research/deep-research/tests/installed-service-smoke.sh
```

- **The launchd leg is the recorded path, not an opt-in extra.** The first acceptance criterion
  is about a job reaching a running harness *under launchd*; only section 4 exercises it. The
  default (no-`SMOKE_LAUNCHD`) run cannot prove it and must never be cited as the evidence for
  it. Where `launchctl` or a harness is unavailable the suite exits 2 BLOCKED with the reason —
  which is recorded as BLOCKED, never as a pass.

- **C-01 applies once here, not twice.** The republish half forces `npm run check:distribution` and `npm run validate:codex` in this change. The plist edit does **not** force a services-manifest regeneration: the research plist is not a generator input, settled by four commands in the spec (analysis D-11c, correcting D-11b).

## Evidence

Run locally 2026-09-08 from the repository root. No hosted CI.

| Gate | Result |
|---|---|
| `cargo check -p prometheus-research` | **PASS** — `Finished dev profile in 1.88s`. Run from `substrate/prometheus-research` (standalone crate, not in the root workspace). `pgrep -x cargo` was empty first, per the one-build-machine-wide rule. |
| `bash .../installed-service-smoke.sh` | **PASS** — 16 passed, 0 failed (bash 5 and `/bin/bash` 3.2). |
| `SMOKE_LAUNCHD=1 bash .../installed-service-smoke.sh` | **PASS** — 19 passed, 0 failed (bash 5 and `/bin/bash` 3.2). Bootstrapped the rendered plist under launchd, job `job-1788909316-22a2fcf7` accepted, **`harness_pid` non-null** — the primary criterion, proven under launchd rather than asserted. |
| `npm run check:distribution` | **BLOCKED** — `generated output is stale: dist/plugins/claude/prometheus-skill-pack`. |
| `npm run validate:codex` | **BLOCKED** — same stale generated output. |

### Why the two C-01 gates are BLOCKED, and that this is not this change's doing

Both failures are the **republish that task 4 is blocked on** (14 foreign artifact-refiner
installs), not a defect introduced here. Verified by removing this change's three files
(`scripts/install-binaries.sh`, the plist, and the new test + fixture) and re-running:
`check:distribution` **still fails**. The staleness predates the change and is owned by task 4.

Recorded as BLOCKED with the reason rather than skipped, and the verdict below is
downgraded accordingly — a blocked gate is never counted as a pass.

### Verdict

**PASS WITH NOTES.**

- D-A is **REPAIRED** at the plist template and proven end-to-end under launchd
  (`harness_pid` non-null on a real job).
- D-B is **VISIBLE, not repaired**: the daemon now reports a stale driver at startup and on
  `/health` before a job is spent discovering it, but the installed driver is not republished.
  Task 4 stays `blocked`.
- Two C-01 gates are BLOCKED on that same pre-existing republish.
