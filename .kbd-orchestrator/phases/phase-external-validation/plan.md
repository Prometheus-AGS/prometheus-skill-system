# Plan — phase-external-validation

**Date:** 2026-06-30  
**Stage:** plan  
**Backend:** OpenSpec (detected `openspec/` at project root)  
**Changes:** 4  
**No code changes.** All changes are documentation, test corpora, or community artifacts.

---

## Ordering rationale

The changes are ordered to unblock the most probable path to a first external user:

1. **Quick-start first** — removes the highest-friction barrier (BG-1). Without this,
   sending anyone to the project means they read a 24-page guide before running a single
   command. A one-page quick-start makes G1 reachable.

2. **Sycophancy corpus second** — a self-contained artifact that can be created and
   verified without any external collaborator. Creates the reproducible test cases G4
   requires. Parallelizable with change-3 but ordered first because it is shorter.

3. **Two-node sovereign-sync guide third** — removes BG-3 for G3. Requires Docker
   knowledge but can be authored and smoke-tested locally before asking anyone else to
   run it.

4. **GitHub discussion + evidence artifact fourth** — the public-facing community
   outreach step. Depends on changes 1–3 being complete so the discussion can link to
   all supporting artifacts.

Changes 1–3 are independent and can be executed in any order within the session, but
the canonical order above minimizes wasted effort if the session is interrupted.

---

## Change list

### change-extval-001-quick-start-guide

**What:** Write `docs/QUICK_START.md` — a single-page, five-step guide that gets an
external user from zero to their first `/learn-goal` invocation in under 10 minutes.

**Content:**
- Prerequisites (3 lines: Node ≥ 18, Git, Rust)
- Clone command (with `--recurse-submodules`)
- Install command (`bash scripts/install-skills-flat.sh`)
- Smoke test (`bash shared/scripts/detect-toolchain.sh`)
- First invocation: open Claude Code, run `/learn-goal "explain recursion to a 10-year-old"`

**Success criterion:** A person who knows Rust, Node, and git but has never seen this
repo can reach `/learn-goal` working output by following the guide top-to-bottom.

**Addresses:** BG-1 (no quick-start), partially enables G1 and G2.

**Recommended agent:** Claude Code (documentation write)

---

### change-extval-002-sycophancy-corpus

**What:** Author `tests/sycophancy-corpus/` with six test fixtures for independent
gate verification.

**Content:**
- `sycophantic-01.md` through `sycophantic-03.md` — known-sycophantic reflection texts.
  Each is a reflection that says no gaps were found when gaps are obvious, or praises
  the implementation without substance.
- `honest-01.md` through `honest-03.md` — known-honest reflection texts. Each includes
  a concrete delta, a root cause, and a corrective action.
- `expected-verdicts.json` — maps each file to `{"should_reject": true/false}` and the
  expected sycophancy score range.
- `README.md` — explains how to run the corpus against the gate:
  ```bash
  for f in tests/sycophancy-corpus/sycophantic-*.md; do
    # expected: score >= 0.4 OR severity high/critical
    cat "$f" | sycophancy-correction detect --strictness strict
  done
  ```

**Success criterion:** Any third party can run the corpus and confirm the gate fires
on sycophantic inputs and passes on honest inputs without needing maintainer
involvement.

**Addresses:** BG-4 (no test corpus), enables G4.

**Recommended agent:** Claude Code (test corpus authoring)

---

### change-extval-003-sovereign-sync-two-node-guide

**What:** Write `docs/SOVEREIGN_SYNC_TESTING.md` — a guide for running two-node
sovereign-sync validation across distinct network namespaces.

**Content:**
- Docker Compose setup (two sovereign-sync containers on separate bridge networks
  with a shared overlay — or two service definitions with distinct ports simulating
  two-node)
- Manual two-host setup (SSH, copy binary, environment variables)
- Step-by-step sync verification:
  1. Start node A: `sovereign-sync --mode daemon`
  2. On node A: create a domain, push data
  3. Share ticket: `curl http://127.0.0.1:7892/api/v1/sync/share`
  4. On node B: import ticket, pull data
  5. Verify CRDT merge: `curl http://127.0.0.1:7892/api/v1/sync/domains`
- Expected output at each step
- Troubleshooting (firewall, QUIC port, iroh NodeAddr)

**Success criterion:** Someone with two machines (or Docker) can reproduce a two-node
sync without maintainer assistance.

**Addresses:** BG-3 (no setup guide), partially enables G3.

**Recommended agent:** Claude Code (documentation + Docker Compose authoring)

---

### change-extval-004-github-discussion-and-evidence-update

**What:** Create a GitHub Discussion (or Issue) calling for first-user feedback, and
add a "Phase: external-validation" section to `docs/production-readiness-report.md`.

**GitHub Discussion content:**
- Title: "First external user onboarding — help us validate the learning loop"
- Body: links to QUICK_START.md, SOVEREIGN_SYNC_TESTING.md, sycophancy corpus
- Asks for: a comment with outcome of `/learn-goal`, whether install succeeded,
  what broke
- Labels: `help wanted`, `validation`

**Evidence artifact update:**
- Add `## Phase: external-validation` section to `docs/production-readiness-report.md`
- Describes what the phase is trying to validate and where to report outcomes
- Placeholder rows for G1–G4 outcomes (to be filled in when validation occurs)

**Success criterion:** The discussion is live on GitHub, publicly linkable, and the
report contains the placeholder rows for future evidence.

**Addresses:** BG-2 mitigation (opens community path), directly enables G5.

**Recommended agent:** Claude Code + gh CLI (creates discussion via GitHub API or gh)

---

## Dependency graph

```
change-extval-001 (quick-start) ────────────┐
change-extval-002 (sycophancy corpus) ──────┤→ change-extval-004 (discussion + evidence)
change-extval-003 (two-node guide) ─────────┘
```

Changes 1, 2, 3 are independent. Change 4 depends on all three (links to their output).

---

## Execute sequence

```
Starting kbd-execute — phase-external-validation (step 0 of 4)
Starting change 1 of 4: change-extval-001-quick-start-guide
Completed change 1 of 4: change-extval-001-quick-start-guide
Starting change 2 of 4: change-extval-002-sycophancy-corpus
Completed change 2 of 4: change-extval-002-sycophancy-corpus
Starting change 3 of 4: change-extval-003-sovereign-sync-two-node-guide
Completed change 3 of 4: change-extval-003-sovereign-sync-two-node-guide
Starting change 4 of 4: change-extval-004-github-discussion-and-evidence-update
Completed change 4 of 4: change-extval-004-github-discussion-and-evidence-update
Completed kbd-execute — phase-external-validation (step 4 of 4)
```

---

## Next command

```
/kbd-execute phase-external-validation
```
