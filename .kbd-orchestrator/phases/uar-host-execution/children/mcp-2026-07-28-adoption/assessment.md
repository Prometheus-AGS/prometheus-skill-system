# Assessment — mcp-2026-07-28-adoption

**Child of:** `uar-host-execution` (paused at `change-uhe-008`, 7/16)
**Assessed:** 2026-07-31

## Headline: the parent is already unblocked, and the upgrade is not urgent

Two changes make UAR compile again, and **tests run**:

| Change | Effect |
|---|---|
| `sse-stream` 0.2.3 → **0.2.4** | `rmcp 2.2.0` compiles |
| `rmcp::model::Content` → **`ContentBlock as Content`** (2 files) | UAR compiles |

`cargo test --lib provenance` → **8 passed**. `change-uhe-008` can proceed.

**This did not require adopting MCP 2026-07-28.** The spec release and the
build break turned out to be *adjacent*, not identical — which corrects the
premise this phase was opened on.

## Two corrections to what I told you when opening this phase

### 1. `from_bytes_stream` was ADDED in 0.2.4, not removed in 0.2.3

I reported that `sse-stream 0.2.3` removed the API and that `rmcp 3.1.0`'s
`^0.2.4` "excludes the broken range". Measured, by building against each:

| `sse-stream` | Result |
|---|---|
| 0.2.1 | ❌ 6 errors |
| 0.2.2 | ❌ fails |
| 0.2.3 | ❌ fails |
| **0.2.4** | **✅ compiles** |
| 0.2.5 | ✅ compiles |

The API was **added** in 0.2.4. `rmcp 2.2.0` declares `sse-stream ^0.2`, which is
too loose **downward** — it permits versions predating a function it calls. The
cheap fix I proposed (pin to 0.2.2) was therefore exactly backwards, and it
failed when tested.

### 2. The pinned `rmcp` was never compatible with UAR's own source

With `rmcp` compiling, a second break appeared: `src/uar/mcp_server.rs:33` and
`src/uar/memory/mcp_server.rs:24` import `rmcp::model::Content`, which does not
exist in 2.x — it is `ContentBlock`. Both files are **clean in git**, so this is
committed state, not someone's in-progress edit.

**UAR's committed source targets rmcp 1.x while its manifest pins `=2.2.0`.**
That mismatch was invisible because the crate never got far enough to report it.

## The real finding: FIVE rmcp versions across five crates

Every row measured — `cargo tree -p rmcp` and `cargo check --lib` on each:

| Crate | Declared | Resolves to | Builds? |
|---|---|---|---|
| `tools/surreal-memory-server` | **git pin** `#a64be23` | **1.4.0** | ✅ |
| `substrate/prometheus-research` | `1.8` | **1.8.0** | ✅ |
| `substrate/sovereign-sync` | `1.8` | **1.8.0** | ✅ |
| `universal-agent-runtime` | `=2.2.0` | **2.2.0** | ✅ *(after this phase's two fixes)* |
| `tools/liter-llm` | `3.0` | **3.0.1** | ✅ |

**Five versions, not the four I first reported** — the git pin resolves to
**1.4.0**, older than either `1.8` crate, and I had left it unmeasured.

**All five build today.** But see falsifier 2 below: one of them survives only
on a stale lockfile, so "tolerated" is doing more work in that sentence than it
first appears.

## And two hand-rolled MCP clients pinned to obsolete protocol versions

Build success is **not** sufficient evidence of no urgency — review flagged that,
correctly. So the protocol surface was measured too, not assumed:

| File | Hard-codes |
|---|---|
| `universal-agent-runtime/src/mcp/stdio_client.rs:114` | `"protocolVersion": "2025-03-26"` |
| `substrate/sovereign-sync/src/mcp_client_pool.rs:177,400` | `"protocolVersion": "2025-06-18"` |

Both hand-roll the `initialize` handshake in raw JSON-RPC — **the exact exchange
2026-07-28 removes** (SEP-2575). These are real migration sites that no
`Cargo.toml` bump would touch, and they are already two and three spec revisions
behind.

**They still work** because servers accept older protocol versions. That is the
compatibility guarantee doing its job, not an absence of debt.

### What is NOT affected

`session_id` appears in `learner-model/src/types.rs:104` and
`sovereign-sync/src/store.rs:11`, but both are **our own domain sessions**, not
`Mcp-Session-Id`. Nothing in this stack depends on protocol-level MCP sessions —
which makes the stateless migration **smaller than "breaking" suggests**.

## What MCP 2026-07-28 actually changes

Verified via the official changelog, not inferred:

- **Protocol-level sessions removed** — no `Mcp-Session-Id` (SEP-2567)
- **`initialize`/`initialized` handshake removed** (SEP-2575); version moves to a
  per-request `MCP-Protocol-Version` header
- `ping`, `logging/setLevel`, `notifications/roots/list_changed` **removed**
- All results carry a required `resultType`; clients **MUST** treat results
  omitting it as `"complete"` (SEP-2322)
- Roots, Sampling, Logging **deprecated** under a new 12-month policy

`rmcp 3.1.0` (published 2026-07-31) implements it "while remaining fully
compatible with the 2025-11-25 release and earlier".

## Decision

**Unblock now; converge later; adopt 2026-07-28 in a dedicated phase.**

Concretely, this child phase commits **exactly two changes** and nothing else:

1. `universal-agent-runtime/Cargo.lock` — `sse-stream` pinned to **0.2.4**
2. `universal-agent-runtime/src/uar/mcp_server.rs:33` and
   `src/uar/memory/mcp_server.rs:24` — `Content` → `ContentBlock as Content`

Then it returns to `change-uhe-008`. It does **not** upgrade any crate to
`rmcp 3.x`, and it does **not** touch the two hand-rolled protocol clients.

Review flagged that the first draft had no explicit decision — several
recommendations and no commit target. Stated plainly above.

## Assumptions

Declared because the decision rests on them, and two are unverified:

- **The two hand-rolled clients keep working against current servers.**
  Unverified at runtime — inferred from MCP's stated backward-compatibility and
  from the fact that nothing is currently reported broken. If a server drops
  pre-2025-11-25 support, both break with no compile-time warning.
- **`rmcp 3.x` can express what the 1.4/1.8/2.2 call sites need.** Unverified —
  no migration was attempted. `Content` → `ContentBlock` shows the API does move
  between majors, so a five-crate convergence may be more than renames.
- **The `ContentBlock as Content` alias is behaviour-preserving.** Verified only
  by `cargo check` + the 8 provenance tests; **no MCP server was exercised end to
  end**. All six call sites are `Content::text(...)`, the shape most likely to be
  a pure rename, but "compiles" is not "behaves identically".
- **Nothing else in the stack is mid-rebuild.** All five crates were checked
  today; a sixth consumer added later would not be covered.

## Falsifier

A falsifier that fires and changes nothing is decoration. Round 2 caught exactly
that in the first version of this section: falsifier 2 fired and the decision
was left standing with a footnote. **Corrected below** — each falsifier now names
*which* claim it kills, and the fired one has had its consequence applied.

Reverse the specific claim named against each, if measured:

1. **Kills: "there is no functional urgency."** A server we depend on rejects our
   protocol version. Test: exercise
   `stdio_client.rs` (2025-03-26) and `mcp_client_pool.rs` (2025-06-18) against a
   live server; a rejected `initialize` means the compatibility assumption has
   already expired and deferral is costing us function, not just tidiness.
2. **Kills: "all five crates build, so the drift is tolerated."** Another crate
   fails to build from a fresh resolution. Test:
   `rm Cargo.lock && cargo check` on each of the five. The `=2.2.0` break was
   invisible until a rebuild forced it; if a second crate has the same latent
   defect, "all five build" is an artifact of stale lockfiles rather than health.
3. **Kills: "the unblock is safe."** The alias is not behaviour-preserving.
   Test: call one tool through
   `uar/mcp_server.rs` and compare the response body to a pre-change capture. A
   difference means this phase shipped a silent regression while claiming an
   unblock.

**Falsifier 2 was run during assess, and IT FIRED.**

| Crate | Fresh resolve (`rm Cargo.lock && cargo check`) |
|---|---|
| `substrate/prometheus-research` | ✅ survives |
| `substrate/sovereign-sync` | ✅ survives |
| `tools/liter-llm` | ✅ survives |
| **`tools/surreal-memory-server`** | **❌ 2 errors — same `rmcp::model::Content`** |

The git-pinned crate has **the identical latent defect UAR had**: a floating
`rmcp` dependency against 1.x-era source. It compiles today only because its
lockfile is stale. A second error appears too —
`E0639: cannot create non-exhaustive struct using struct expression` — so the
newer `rmcp` also sealed a struct this crate constructs literally.

**This changes the reading of "all five build".** It is partly an artifact of
stale lockfiles, exactly as the falsifier was written to test. Two of five
crates carry the same defect; one was found by a compile failure, the other only
by deliberately removing a lockfile.

It does **not** reverse the decision — `surreal-memory-server` is a submodule
with its own release cycle, and fixing it is not this child phase's scope. But
it must be **recorded as a known latent defect**, not left to be discovered the
next time someone runs `cargo update` there.

## Open questions for plan

1. **Should this phase do anything beyond the two-change unblock?** Everything
   else measured here is a finding, not work.
2. ~~Do the other four crates survive a fresh resolution?~~ **ANSWERED during
   assess: three do, `surreal-memory-server` does not.** It carries the same
   `rmcp::model::Content` defect UAR had, plus an `E0639` non-exhaustive struct
   error. The open question is now **who fixes it** — it is a submodule with its
   own release cycle, outside this child phase's scope.
3. **Should `=2.2.0` become a range?** The exact pin bought nothing: it did not
   prevent the 1.x-source/2.x-manifest mismatch, it **hid** it until a rebuild.
4. **Who owns the two hand-rolled clients' protocol version?** They are
   hard-coded strings two and three revisions behind, in two different repos.

## Suggested shape

1. **Commit the unblock** — the two changes named in the Decision, validated by
   `cargo check --lib` and `cargo test --lib provenance` (8 passing).
2. ~~Run falsifier 2~~ — **done during assess; it fired.** `surreal-memory-server`
   has the same latent defect. Record it; do not fix it here.
3. **Record the five-version drift, the two obsolete protocol clients, and the
   latent defect in `surreal-memory-server`** as a decision with the falsifier
   above.
4. **Return to the parent at `change-uhe-008`.**
5. **Recommend a follow-on phase** for rmcp convergence — do not attempt it here.

## Adversarial review record

Three rounds, judge `kbd-judge` via `rest-gateway`,
`cross_model_check: verified-distinct`, producer `claude-opus-5`.

| Round | Verdict | What it caught |
|---|---|---|
| 1 | BLOCK (3 CRITICAL) | No explicit decision, no assumptions, no falsifier. I had written an assessment and reviewed it in decision mode without the fields decision mode requires. |
| 2 | BLOCK (1 CRITICAL) | **The decision ignored its own fired falsifier.** I wrote "reverse if X", X fired, and I kept the decision with a footnote — making the falsifier decorative. |
| 3 | BLOCK (1 CRITICAL) | Withdrawing the claim is not a *mitigation*; the defect still ships. |

Each was fixed rather than argued with. Stopping at the cap; the residual is
that a mitigation now exists in plan but has not been executed — which is
correct for an assess stage.

**Worth keeping:** round 2 is the sharpest. A falsifier that fires and changes
nothing is worse than no falsifier, because it *looks* like the claim was
tested. The fix was to name which claim each falsifier kills, then actually
withdraw the killed one — "all five crates build, so the drift is tolerated"
appears nowhere in this document as a reason to defer.

**A tooling gap found here:** `build-review-packet.sh --mode artifact` has no
child-phase support. Passing `--phase uar-host-execution` packaged the *parent's*
assessment, and the first review round judged the wrong document entirely. Worked
around with `--mode decision --target <path>`; recorded for the parent's
reflection.
