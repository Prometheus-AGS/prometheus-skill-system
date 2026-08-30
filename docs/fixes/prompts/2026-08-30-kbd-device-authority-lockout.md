# Fix prompt — KBD device-authority lockout, and the design that caused it

- **Raised:** 2026-08-30
- **Against:** `prometheus-skill-pack` @ `main` (2e89568), `kbd-runtime` v0.1.0
- **Raised by:** the graph-explorer team
- **Status:** open
- **Severity:** this package gates all engineering work company-wide. Treat
  availability as a correctness property, not a nice-to-have.

---

## The incident

Project **graph-explorer** (`39b50c1a-ddbb-4469-8b86-ffec6640e6c8`) is frozen at
revision 112. Every write is refused:

```
Error: local runtime rejected the command
Caused by: event signer ed25519:2c27c749aeba... is not enrolled
```

This blocks `/kbd-new-child`, task and change transitions, and completion
updates. **There is no CLI recovery path.** `prometheus kbd --help` lists 28
subcommands; none is `device`, `enroll`, or `trust`.
`prometheus kbd migrate --check` returns `journalMigrationRequired: false` — it
covers only progress-ledger reconciliation, not device authority.

---

## Root cause (verified, not inferred)

It is **not** a missing enrollment — genesis worked correctly. It is a **key
identity split**:

| | Key |
|---|---|
| Signed all 112 events | `ed25519:2e0e7292…` — macOS Keychain, service `prometheus-kbd-device`, account `<project-id>:unknown-device` |
| This session signs with | `ed25519:2c27c749…` — `~/.config/sovereign-sync/device-key.json` |

In `substrate/kbd-runtime/src/lib.rs`:

```
3372  device_signer() resolves by SEARCH ORDER:
        1. PROMETHEUS_DEVICE_KEY_FILE
        2. managed sovereign-sync key, when uses_managed_canonical_data_root()
        3. platform credential store (Keychain)
1705  genesis enrols the signer via the bootstrap branch  (correct)
1698  every later operator event requires operator_key_ids.contains(signer)
1761  DeviceEnrolled itself requires an already-enrolled operator
```

The managed-key branch (2) was added **after** this journal's genesis. The
runtime now signs with a key its own journal never enrolled, and enrolling
requires a key that is already enrolled. **Permanently unwritable, by
construction.**

Any journal bootstrapped before a signer-discovery change — or on a box where
the Keychain entry later becomes unavailable — hits this. It is latent across
the whole fleet.

---

## What was already tried, and why it failed

A self-healing enrollment was attempted and got it wrong **twice**. Both
attempts have been fully reverted. Recorded here so the next attempt does not
repeat them:

**Attempt 1 — hooked `append_command()`.** The CLI does not use it.
`phase create` goes `CommandKind::PhaseDefine` → `execute_command` →
`append_unchecked`. There are **five** `append_unchecked` call sites
(`lib.rs` 4245, 4283, 4351, 4421, 4496). Patching one caller fixes one caller.
`cargo check` was clean, the tests were green, the binary installed — and the
fix did nothing.

**Attempt 2 — moved it into `append_unchecked`** (the shared choke point).
It then fired at revision 0, injecting `DeviceEnrolled` *before*
`RunInitialized`, breaking **every newly-created journal**:
73 passed / 1 failed → **37 passed / 37 failed**.

Both were caught by *running* things; neither by inspection. That is the
signal about this codebase: the authority rules for a single event are spread
across four locations in a 9,867-line file —

```
1067  verify_signature
1689  operator-key gate
1705  bootstrap enrolment
1761  DeviceEnrolled handler
```

— and nothing co-locates them.

**Current tree state:** reverted to baseline, **73 passed / 1 failed**. That
one failure (`project_document::tests::divergent_phases_union_conflicts_stay_
visible_and_resolution_is_authoritative`, asserts `"candidate B"`, gets
`"candidate A"`) was **already failing beforehand** — verified by stashing.

---

## Part A — recover the locked-out project (the emergency)

Provide a supported recovery path. Options, pick one and justify it:

1. A `prometheus kbd device enroll|list|adopt` subcommand.
2. Narrow self-healing: when the signer is unenrolled **and** the replica
   passes `ensure_writable_replica()` **and** holds the exclusive journal lock
   **and** `state.revision > 0`, append a signed `DeviceEnrolled` for the
   current key.
3. Make `device_signer()` **prefer a discoverable key the journal already has
   enrolled**, before falling back to search order.

**Option 3 is the recommendation.** It is read-only, touches no authority
rules, needs no new event type, and fixes the actual root cause — identity
bound to machine configuration rather than to the log. You have better context;
choose deliberately.

Non-negotiable for whichever option wins:

- **A revoked key stays rejected.** Re-enrolling one silently reverses an
  operator decision.
- **Never at revision 0.** Genesis already enrols its signer.
- **Never on a replica** the registry classifies as observation-only, CI, or
  recovered — otherwise a machine can enrol itself into a journal it only
  mirrors.
- **Every event stays attributable.** Remove the *ceremony*, not the audit
  trail.

**Acceptance:** in graph-explorer,
`prometheus kbd phase create --parent multi-root-code-workspace --id
compass-shell-architecture-spike …` succeeds, `prometheus kbd audit` replays
every event with no integrity error, and `status` reports revision 113.

---

## Part B — stop this class of bug recurring

This matters more than Part A.

1. **One authority choke point.** Compute append-time authority in a single
   place shared by all five `append_unchecked` callers. Today each path must
   independently arrive at the same answer — which is precisely why a
   one-caller fix looked correct.

2. **Co-locate the authority rules.** Four scattered locations for one decision
   is why both attempts missed a case. Consider a single
   `authorize(event, state) -> Result<()>` carrying every rule and its
   rationale.

3. **Integration tests.** There are 54 inline `#[cfg(test)]` tests in `lib.rs`
   and **no `tests/` directory**. The bug that mattered — CLI →
   `execute_command` → append — has no end-to-end test, which is exactly why it
   shipped. Note this project's own doctrine (`CLAUDE.md`): *"Integration tests
   only. No unit tests. Test only across a real seam, against something real."*
   This crate does the opposite. Add tests driving the real CLI against a real
   journal.

4. **Fix the pre-existing red test** named above. A permanently-failing test
   trains everyone to ignore the suite.

5. **Packaging.** The 1.7.0 marketplace payload ships **no `scripts/`
   directory**, so `scripts/install-plugin-generation.js` is missing and the
   hook bootstrap fallback fails with:
   ```
   {"status":"NOT_ACTIVATED","reason":"bootstrap payload incomplete"}
   ```
   Fix packaging so the published plugin contains what its own bootstrap
   requires.

---

## Context

- Version is **already bumped to 1.8.0** in `package.json` and three
  `Cargo.toml` files (`prometheus-cli`, `prometheus-knowledge`,
  `surreal-memory-server`). A 1.8.0 binary built from the *pre-revert* source is
  installed at `~/.local/bin/prometheus`; it is harmless — it contains only the
  ineffective first attempt — but should be replaced once the real fix lands.
- **Baseline to protect:** 73 passed / 1 failed. Do not let it regress.
- `kbd-runtime` is 33 days old with 38 commits: **22 fixes, 10 features**. A
  2.2:1 fix-to-feature ratio says the design keeps discovering its invariants
  after shipping. That is the argument for Part B.
