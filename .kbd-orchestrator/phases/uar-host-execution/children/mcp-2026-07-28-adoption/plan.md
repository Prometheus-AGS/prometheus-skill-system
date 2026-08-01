# Plan — mcp-2026-07-28-adoption

**Child of:** `uar-host-execution` (paused at `change-uhe-008`, 7/16)
**Planned:** 2026-07-31 · **Backend:** OpenSpec · **Changes:** 3

## What this phase is now for

Assess answered the question it was opened on: **adopting MCP 2026-07-28 is not
required to unblock the parent**, and the upgrade belongs in its own phase. So
this plan is deliberately small — commit the unblock, record what was found,
return.

Everything here is already **measured**. The plan's job is to commit it under
change control, not to discover more.

## Premises re-verified at plan time

| Check | Result |
|---|---|
| The two unblock edits survived the rebase | ✅ both present |
| `sse-stream` still at 0.2.4 in `Cargo.lock` | ✅ |
| `cargo check --lib` | ✅ **Finished** |

## One correction to the assessment's mitigation

Assess proposed *"commit a lockfile for `surreal-memory-server`, or convert its
git dependency to a pinned rev"*. **The first half is wrong** — checked at plan
time:

```console
$ git ls-files Cargo.lock
Cargo.lock              # already tracked, and clean
```

The lockfile **is** committed, and it pins `rmcp` to
`git+…rust-sdk#a64be231…` (1.4.0). The defect is one level up: `Cargo.toml:42`
declares the git dependency with **no `rev`, `tag`, or `branch`**, so
`cargo update` floats to the default branch HEAD and re-resolves past the
lockfile — which is exactly what the falsifier reproduced.

**The mitigation is therefore a single `rev = "a64be231…"` line**, not a
lockfile commit. Smaller and more precise than assess estimated.

## Assumptions

Declared because they are load-bearing, and two are unverified:

- **The `ContentBlock` alias is behaviour-preserving.** Verified by
  `cargo check` and 8 provenance tests; **no MCP server was exercised end to
  end**. All six call sites are `Content::text(...)`, the shape most likely to be
  a pure rename — but "compiles" is not "behaves identically".
- **`a64be231` is a good rev to pin `surreal-memory-server` to.** It is what the
  committed lockfile already resolves to and the crate builds against it, so
  pinning changes nothing *today*. Unverified: whether it is the rev its
  maintainers would choose.
- **The three changes are independent as claimed.** 002 and 003 touch different
  repos from 001. If 002 is BLOCKED, 001 and 003 are unaffected — asserted from
  the file sets, not from a trial run.
- **Authorisation does not extend to `Prometheus-AGS/surreal-memory-server`.**
  The grant named `universal-agent-runtime`. Treating a sibling repo under the
  same org as covered would be inference, not permission — hence the gate on 002.

## Changes

### `change-mcp-001-uar-build-unblock`
**In-repo:** `universal-agent-runtime` (authorised) · No dependencies

Commit the two changes that make the crate compile, so
`change-uhe-008` can run its tests.

1. `Cargo.lock` — `sse-stream` resolved at **0.2.4** (`from_bytes_stream` was
   *added* there; `rmcp 2.2.0`'s `^0.2` is too loose **downward**). A lockfile
   records a resolution; it does not constrain a future one — see the floor
   below, which is what actually prevents recurrence.
2. `src/uar/mcp_server.rs:33`, `src/uar/memory/mcp_server.rs:24` —
   `Content` → `ContentBlock as Content`

**Acceptance:** `cargo check --lib` finishes clean **and**
`cargo test --lib provenance` passes 8/8 — both after a `cargo clean` of the
crate, so the result is not an artifact of warm state.

**Also raise the floor — and be precise about what that buys.** `rmcp`'s
`sse-stream = "^0.2"` is the root defect and it is upstream, but
`universal-agent-runtime` can protect itself by adding `sse-stream = "0.2.4"` to
its own `[dependencies]`.

Review flagged that this is **a floor, not a pin**: `"0.2.4"` means `^0.2.4` —
`>=0.2.4, <0.3.0` — so 0.2.5 and later are still permitted. That is the correct
constraint here, and it is **tested rather than assumed**: 0.2.5 was built
against and **compiles**. What the floor excludes is the range that actually
breaks (0.2.0–0.2.3), which is all it needs to do. It does **not** make the
defect "impossible to violate" — a future 0.2.x that removes the API again would
slip through, and that risk is accepted rather than hidden.

**Acceptance for the floor, tested not asserted:** with the constraint added,
`cargo update -p sse-stream` must **not** move below 0.2.4. Verified by running
`cargo update -p sse-stream --precise 0.2.2` and confirming Cargo **rejects** it
as violating the requirement. Without that step this change would assert a
protection it never exercised — the exact gap review named.

### `change-mcp-002-pin-floating-git-dep`
**In-repo:** `tools/surreal-memory-server` **(different repo — see gate)** ·
No dependencies

`Cargo.toml:42` declares `rmcp` as a git dependency with no `rev`. The committed
lockfile holds `#a64be231`, but nothing *requires* it, so any `cargo update`
floats to HEAD and fails with the same `rmcp::model::Content` error UAR had,
plus `E0639`.

**Acceptance:** with `rev = "a64be231527f923e9f84d4dd7bf3c3bd695ee53e"` added,
`rm Cargo.lock && cargo check --lib` **succeeds** — the exact command that fails
today. That is the falsifier, re-run as the acceptance test.

> **Authorisation gate — first task of this change.**
> `tools/surreal-memory-server` is a **submodule pointing at
> `Prometheus-AGS/surreal-memory-server`**, a repo this phase was not authorised
> for. The grant covers this pack and `universal-agent-runtime` only.
>
> **Unauthorised or unanswered → archive BLOCKED**, touch nothing, and the
> latent defect stays recorded rather than fixed. Silence is not consent; that
> default is what made `change-msp-008` honest.

### `change-mcp-003-record-mcp-findings`
**In-repo:** this pack · Decision only · No dependencies

Record what assess measured, so it is not rediscovered:

- **Five `rmcp` versions** across five crates — 1.4.0, 1.8.0 ×2, 2.2.0, 3.0.1
- **Two hand-rolled MCP clients** on obsolete protocol versions
  (`stdio_client.rs:114` → 2025-03-26; `mcp_client_pool.rs:177,400` →
  2025-06-18), both hand-rolling the `initialize` handshake **2026-07-28
  removes**
- **Nothing uses `Mcp-Session-Id`** — the `session_id` hits are our own domain
  sessions, so the stateless migration is smaller than "breaking" implies
- **`=2.2.0` hid a 1.x-source mismatch** rather than preventing it

**Acceptance — the findings must be RE-VERIFIED, not just recorded.** Review
noted that "a decision record exists" tests the wrong thing: a record of false
findings passes that criterion. So each claim carries a command, and each
command is re-run at write time:

| Finding | Re-run |
|---|---|
| five `rmcp` versions | `cargo tree -p rmcp --depth 0` in each of the five crates |
| two obsolete protocol clients | `grep -n protocolVersion` on both files |
| nothing uses `Mcp-Session-Id` | `grep -rn "Mcp-Session-Id"` across the pack + UAR |

Plus: a decision record via `decision-log.sh` with alternatives, a stated
falsifier, and `outcome_status: pending`, passing `--mode decision` review with
`verified-distinct`. **No code.**

## Deliberately NOT in this phase

- **Upgrading anything to `rmcp 3.x`.** Five crates on five versions is a
  migration with its own assess. Assess found no forcing function: all five
  build, and the deprecation policy gives 12 months.
- **Touching the two hand-rolled protocol clients.** One lives in
  `sovereign-sync` (this pack), one in UAR — both authorised — but changing a
  protocol version without an end-to-end test is how a working integration
  breaks silently. Recorded in 003; migrated in the convergence phase.
- **Fixing `surreal-memory-server`'s `Content` usage.** That is upstream source
  in another repo. 002 stops the *floating resolution*; it does not port the
  crate.

## Ordering

001 first — it unblocks the parent and is the only change with a downstream
dependent. 002 and 003 are independent and may run in either order. 002 may end
BLOCKED, which does not affect 001 or 003.

## Return contract

On reflect, the parent resumes at
**`/kbd-apply change-uhe-008-builtin-db-registration`** with
`phase: uar-host-execution`, 7/16.

**The parent stays blocked unless 001 lands.** If 001 fails, that must be
reported as the parent still being blocked — not as this phase succeeding.

## Carry-forwards for the parent's reflection

Tooling defects found in this child, none patched (all in installed skills or
runtime data, where edits die at the next install):

1. **`kbd-new-child.sh`** — `child_label` read at line 156, assigned at line 234.
   Deterministic; the child was created by hand.
2. **Stale runtime store** — `current-waypoint.json` carries
   `generatedBy: "kbd-runtime"`, so a `prometheus kbd` store owns its `phase` and
   counters. That store holds a **Completed** run from 2026-07-29 with an
   **expired lease**, reporting `adversarial-review-for-creation` and `170/229`.
3. **`build-review-packet.sh --mode artifact` has no child-phase support** —
   `--phase <parent>` packaged the *parent's* assessment, and the first review
   round judged the wrong document entirely.

## Review record

Round 1 **BLOCK** (1 CRITICAL, 5 WARNING), judge `kbd-judge` via
`rest-gateway`, `cross_model_check: verified-distinct`, producer `claude-opus-5`.

| # | Severity | Finding | Response |
|---|---|---|---|
| 1 | CRITICAL | No stated falsifier | **Artifact of the tooling, not the plan.** `build-review-packet.sh --mode artifact` has no child-phase support, so this plan was reviewed through **decision mode**, which requires `decision`/`assumptions`/`falsifier` fields a *plan* does not have. Each change carries its own acceptance criteria instead. Recorded as a tooling gap; not papered over by bolting a falsifier onto a plan. |
| 2 | WARNING | No stated assumptions | **Accepted.** Four added, two marked unverified. |
| 3 | WARNING | The `sse-stream` floor is never tested | **Accepted.** 001 now requires running `cargo update -p sse-stream --precise 0.2.2` and confirming Cargo **rejects** it. Without that the change asserts a protection it never exercised. |
| 4 | WARNING | `"0.2.4"` is a floor, not a pin — the plan overclaimed | **Accepted, and measured.** `"0.2.4"` means `^0.2.4`, so 0.2.5+ is allowed. Built against 0.2.5: **compiles**. The floor excludes the range that actually breaks (0.2.0–0.2.3), which is all it needs to do — and the plan now says so instead of claiming a pin. |
| 5 | WARNING | 003's acceptance tests that a record *exists*, not that it is *true* | **Accepted.** Each finding now carries a re-run command, executed at write time. A record of false findings would have passed the old criterion. |
| 6 | WARNING | Prior-decision check unreliable (malformed wiki entries skipped) | **Acknowledged, not fixed.** The malformed entries are the pk wiki's, not this plan's. |

Stopping at one round: the sole CRITICAL is a review-harness limitation rather
than a defect in the plan, and every WARNING was fixed by measuring rather than
arguing. Finding 4 is the one worth keeping — **I described a caret range as a
pin**, which would have overstated the guarantee in exactly the way this phase
exists to correct.
