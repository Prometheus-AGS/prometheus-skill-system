# change-drt-007-install-surface-repair

**Title:** Repair the two install-surface defects the predecessor's evidence run found
**Repository:** `prometheus-skill-pack`
**Phase:** deep-research-onyx-parity
**Goal:** G1 (unblocks process-level workers) — **not G4**: this change contains no budget, force-complete path, or dispatch ledger; G4 is delivered entirely by drt-002 (adversarial plan review, finding 1)
**Depends on:** none — independent of the thread work, but blocks demonstrating it under the installed service
**Backend:** native-kbd

## Why

Two defects carried in from `research-agent-hardening`, both found by running the
pipeline for real rather than by inspection, and both recorded in
`docs/research-agent-hardening-evidence.md`.

**D-A.** `~/Library/LaunchAgents/com.prometheus.research.plist` sets no
`EnvironmentVariables`/`PATH`, so launchd gives the daemon the bare default and
it cannot resolve `claude` or `codex` although both are installed. Its source
template `substrate/prometheus-research/com.prometheus.research.plist` has no
`PATH` key, and `scripts/install-binaries.sh:582` substitutes only `__HOME__`,
while sibling templates in `shared/launchagents/` carry `__PROMETHEUS_PATH__`.
**This blocks dispatch strategy B outright**: under the installed service no
process-level worker can ever spawn.

**D-B.** The installed plugin generation ships a **1920-byte stub driver**
against the repo's 30718-byte driver, with zero occurrences of `--resume`,
`checkpoint`, `next_stage`, or `RESEARCH_STAGE_RUNNER`. `resolve_driver()`'s
`~/.claude` candidate therefore resolves to a script with none of the stage
contract.

Neither is a defect in daemon logic: in both cases it refused to report a success
it had not achieved, which is the behaviour change-rah-004 built.

## What Changes

- Add `EnvironmentVariables` with a `__PROMETHEUS_PATH__` placeholder to the
  research plist template, matching the `shared/launchagents/*.plist` convention.
- Extend the `sed` in `scripts/install-binaries.sh` to substitute it, alongside
  the existing `__HOME__`.
- **Correction to analysis D-11b (adversarial round-1 finding 8).** The claim
  that this edit forces a services-manifest regeneration was **wrong**, and the
  judge was right to challenge it. `scripts/generate-service-manifest.mjs` reads
  only `shared/launchagents/*.plist` and `shared/systemd/*` (lines 35-36); the
  research plist lives at `substrate/prometheus-research/com.prometheus.research.plist`
  and is **not** an input. `shared/services.manifest.json` lists eleven services
  and `com.prometheus.research` is not among them. So C-01's manifest obligation
  does **not** apply, and running the generator here would change nothing.
  This raises a real question the plan should answer: the research daemon is a
  launchd service that the generated manifest does not know about. Bringing it
  under `shared/launchagents/` would make it visible to the manifest and to
  `check:services-manifest` — but that is a *migration*, with its own C-01
  obligation, and it is out of this change's scope. Recorded as an open question
  rather than silently done or silently ignored.
- Republish the plugin generation so the installed driver matches the repo, then
  prove the generator is idempotent and pass the drift validators.
- Add a daemon self-check that reports the resolved harness path and driver path
  with their sizes, so a stale install is visible before a job is started rather
  than after it fails.

## Scope

- `substrate/prometheus-research/com.prometheus.research.plist`
- `scripts/install-binaries.sh`
- `substrate/prometheus-research/src/job/daemon.rs`
- `site/docs/substrate/prometheus-research.md`
- `docs/research-agent-hardening-evidence.md` (record the repair against the defect)
- `skills/research/deep-research/tests/installed-service-smoke.sh`

## Capabilities

- `research-pipeline-execution (install-surface repair)`

## ADDED Requirements

### Requirement: The daemon can resolve a harness under launchd
WHEN the service is installed and started by launchd, THEN the daemon SHALL resolve a harness binary without the operator exporting a PATH.

#### Scenario: Installed service spawns a worker
- **WHEN** the service is bootstrapped from the generated plist and a job is started
- **THEN** the job reaches a running harness with a non-null `harness_pid`, rather than `blocked: no harness binary on PATH`

### Requirement: A stale install is visible before it fails
The daemon SHALL report the resolved driver path and its size on startup and in its health output.

#### Scenario: Stub driver detected
- **WHEN** the resolved driver lacks the stage-contract markers
- **THEN** the daemon reports it as stale with both paths and sizes, instead of failing only after a job produces no package

### Requirement: The manifest obligation is proven inapplicable, not assumed
The claim that this edit forces a services-manifest regeneration SHALL be settled by command, not by assertion, and the result recorded either way.

#### Scenario: The generator's inputs are checked
- **WHEN** `scripts/generate-service-manifest.mjs` is inspected for its input paths
- **THEN** it reads only `shared/launchagents/*.plist` and `shared/systemd/*`, the research plist is confirmed absent from `shared/services.manifest.json`, and the correction to analysis D-11b is recorded in this spec
- **AND** if that ever ceases to hold — for instance if the plist migrates under `shared/launchagents/` — the C-01 obligation returns and this requirement is rewritten

## Constraints

- Implementation-first, integration-only evidence: no unit tests, mocks, or snapshots count as delivery evidence.
- One Cargo build machine-wide at a time. Check `pgrep -x cargo` before any `cargo` command.
- Local-only validation: no GitHub Actions run is evidence.
- Verification labels are `verified | unverified | blocked | inferred`; provenance is `PASS | PASS WITH NOTES | BLOCKED`. A gate that could not run is recorded BLOCKED with the reason.
- Scripts that launchd may invoke stay bash 3.2 compatible (C-05); `install-binaries.sh` runs under `set -euo pipefail` and a submodule build must never abort the installer (CLAUDE.md).
- The pack never depends on the Companion or any extension.
- **C-01 applies once here, not twice.** The republish forces `npm run check:distribution` and `npm run validate:codex` in this change. The plist edit does **not** force a services-manifest regeneration, because the research plist is not a generator input (see What Changes; this corrects analysis D-11b).
- **C-03 does not apply** provided the fix is regeneration only. A version bump or a plugin-manifest edit would trigger it, and then `docs/codex-plugin.md` and the CLAUDE.md Codex section must be updated in the same change (analysis D-12).

## Task 2 finding: the C-01 manifest obligation does not apply here (settled by command)

Analysis **D-11b** asserted that editing the research plist forces a
`shared/services.manifest.json` regeneration under C-01. **That was wrong.**
Four commands settle it:

| # | Check | Result |
|---|---|---|
| 1 | `grep -nE "^const (LAUNCHAGENTS\|SYSTEMD)" scripts/generate-service-manifest.mjs` | reads only `shared/launchagents` (:35) and `shared/systemd` (:36) |
| 2 | Is the research plist in either? | No — it lives at `substrate/prometheus-research/com.prometheus.research.plist` |
| 3 | `grep -c "prometheus.research" shared/services.manifest.json` | **0** — the service is not tracked at all |
| 4 | `npm run check:services-manifest` **after** the plist edit | **PASSES** — the decisive test: the edit produced no drift |

Check 4 is the one that matters. If the plist were an input, editing it without
regenerating would fail the drift gate. It passes, so C-01's manifest obligation
is not triggered by this change. Recorded upstream as **analysis D-11c**.

C-01 still binds the *other* half of this change: the plugin republish in task 4
is a generated-artifact resync and must pass `check:distribution` and
`validate:codex` in this change.

### The real gap this exposed

`com.prometheus.research` is a launchd service the generated service manifest
**does not know about**, while eleven sibling services are tracked. That is a
genuine inconsistency, and it is why defect D-A was invisible: nothing validated
this plist against the convention its siblings follow.

Moving it under `shared/launchagents/` would bring it into the manifest and
under `check:services-manifest`. **Deliberately not done here.** That is a
migration with its own C-01 obligation and its own regression surface (the
installer resolves this path, and `install-mcp-services.sh` would then also
render it — two installers writing one plist). Doing it silently inside a `PATH`
fix would be exactly the kind of scope creep that makes a repair unreviewable.
Raised as an open question below instead.

## Task 4 outcome: C-01 satisfied, the republish itself BLOCKED (pre-existing)

**Done and verified:**

| Obligation | Result |
|---|---|
| `shared/harnesses/generated/release-manifest.json` regenerated | bundle `49783d8c…`, 31 hooks |
| `node scripts/check-harness-adapters.js` | PASS |
| `npm run build:distribution` regenerated (161 skills) | PASS |
| `npm run check:distribution` | PASS |
| `npm run validate:codex` | PASS |
| `npm run check:services-manifest` | PASS |
| `npm run check:skills-index` | PASS |
| Harness generator idempotent | 3 runs, identical `dfd42aa8…` |
| Distribution generator idempotent | 2 runs, identical `cc2786f0…` |

The release manifest had to be regenerated first: the publisher **refused to
stage** because `shared/scripts/content-grounding-kb.sh` no longer matched its
pinned hash — change-rah-008 rewrote that script. That refusal is the payload
gate working, and it is why the publish attempt was useful even though it did not
complete.

**BLOCKED, and not by this change.** `install-plugin-generation.js --targets
claude` refuses with `15 skill(s) not installed at canonical name`. All 14 named
blockers are `artifact-refiner` skills (`convert-htmx-*`, `refine-*`,
`scaffold-react-vite*`, `design-svg-logo`, `rebrand-artifact`) — a git submodule
(`skills/imported/artifact-refiner`) whose skills are installed under foreign
names from an earlier install. **`deep-research` is not among them.**

Clearing them means `mv`-ing 14 skill directories the operator installed and that
this phase does not own. Doing that silently inside a `PATH` repair would be
exactly the scope creep the plan warns against, so it is **not done here** and is
raised as an open question instead.

**Consequence for D-B, stated plainly:** the installed driver at
`~/.claude/skills/deep-research/scripts/run-research.sh` is **still the 1920-byte
stub**; the repo driver is 30718 bytes. `RESEARCH_DRIVER` remains the working
override, and task 3's self-check now makes the stale state visible at `/health`
and in the startup log rather than only after a job fails. Verified live: with
`RESEARCH_DRIVER` set, the daemon reports `harness: ok claude` and
`driver: ok 30718 bytes`.

So D-B is **mitigated and made visible, not yet repaired.** The repair needs one
operator decision (what to do with 14 foreign artifact-refiner installs), which
is a different change.

## Open Questions

- Whether the daemon's stale-driver check should refuse to start or merely warn (default: warn and report, since refusing to start would make a stale install worse, not better).
- **Should `com.prometheus.research.plist` move under `shared/launchagents/`?** It is a launchd service the generated service manifest does not track, unlike the eleven that are. Moving it would bring it under `check:services-manifest` and the integration contract's service surface. Out of scope here; it is a migration with its own C-01 obligation, and it should be a decision, not a side effect of a `PATH` fix.
