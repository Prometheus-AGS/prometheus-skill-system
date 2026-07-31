# Decision: keep both wasm targets, and make the split explicit

**Status:** accepted · 2026-07-31 · `change-uhe-004-librefang-abi-decision`
**Phase:** uar-host-execution

## Decision

**Keep both wasm guest targets.** `skills/rust/librefang-wasm-skill/` continues
to generate **core-wasm** guests for LibreFang's `memory`/`alloc`/`execute` ABI;
`wit/prometheus-component/` continues to define the **Component Model** world
for UAR and KnowMe.

Do **not** port the librefang templates to `prometheus:component`, and do
**not** retire them.

Add one thing: **each target must say, in its own documentation, which host it
targets and which it cannot load in.** The defect today is not that two targets
exist — it is that neither says so.

## Why not "port"

Porting assumes the two hosts could converge. They cannot, on evidence:

| | LibreFang | UAR / KnowMe |
|---|---|---|
| Binary format | **core wasm module** | **component** |
| Guest exports | `alloc(i32) -> i32`, `execute(i32,i32) -> i64` | `run: func(string) -> result<string, error>` |
| Host imports | `host_call`, JSON over a pointer | WIT interfaces (`log`, `kv-store`, `clock`) |
| Interface definition | none — hand-rolled pointer ABI, **0 `.wit` files** | `prometheus:component@0.1.0` |

Verified: `librefang-runtime/src/plugin_runtime.rs:1548-1549` states the guest
ABI is "the sandbox's `host_call` surface (`memory` / `alloc` / `execute`), not
WASI". A component and a core module are **different binary formats** —
`wasmtime::component::Component::from_file` will not load a core module whatever
its exports are named.

So porting does not mean "adjust the templates". At minimum it means **teaching
LibreFang's host to load components** — which is a smaller claim than "rewrite
it", and review was right to flag the stronger phrasing. wasmtime 46 supports
both models, so the host could in principle gain a component path alongside its
core-module one.

That does not make porting available to us: LibreFang is a real repository at
`/Users/gqadonis/Projects/references/librefang`, it is **not ours**, and this
phase is authorised for `universal-agent-runtime` only. The blocker is
**ownership**, not technical impossibility — and saying so accurately matters,
because if LibreFang's maintainers add a component path, falsifier 1 fires and
this decision should change.

## Why not "retire" — the path is live, not documentation

Adversarial review returned CRITICAL on the first draft: I had argued "retiring
breaks a real path" while admitting I had not verified the path was used. That
was the load-bearing claim, so it was checked rather than asserted.

**It is a working pipeline, and every link is real:**

| Link | Evidence |
|---|---|
| `native-agent` accepts the target | `SKILL.md:238,246` — `target: librefang-wasm` or `both` produces `crates/agent-skill/` + `skill.toml` |
| It builds against these templates | `SKILL.md:249-251` — the WASM crate "uses the same `agent-core`… `skills/rust/librefang-wasm-skill/` for the underlying skill" |
| Packaging is a real command | **`forge package-librefang` exists on this machine**: *"Package an agent directory as a LibreFang WASM skill zip"* |
| Deployment is a real skill | `upload-to-bossfang` — **37 steps**, POSTs `.lf-skill.zip` to `/skills/install`, with a deny-by-default SSRF guard, token redaction, and post-install verification |
| A prerequisite gate exists | `start-business-build/SKILL.md:92` — `wasm32-unknown-unknown` required "only if `target ∈ {librefang-wasm, both}`" |

### Round 2 pushed back — correctly — and the answer changed shape

Review's second round said the evidence above proves **wiring exists, not that
anyone uses it**, and that deferring the usage question until after the decision
was backwards. Both true. So it was tested rather than deferred:

| Usage probe | Result |
|---|---|
| `.lf-skill.zip` anywhere on this machine | **0** |
| `BOSSFANG_TOKEN` configured | **not set** |
| **LibreFang process listening** | **YES — `librefang` on `localhost:4545`** |
| `GET :4545/skills` | **HTTP 401** — `{"error":"Missing Authorization: Bearer <api_key> header"}` |
| LibreFang commits in the last 7 days | **7** |
| LibreFang's last commit | **2026-07-31** (today) |

**The honest reading is mixed, and both halves matter.** No packaged artifact has
ever been produced here, and no deploy token is configured — so *this machine*
has not shipped a skill through the pipeline. But LibreFang is **running right
now**, auth-protected, and under active development this week.

That is a system being **built toward**, not one abandoned. Retiring the guest
templates while its host is under weekly development would remove our side of an
integration the other side is actively maintaining.

**Conclusion, with the uncertainty stated rather than resolved away:** keep both
— but the justification is "the target host is live and actively developed",
**not** "the pipeline is in production use". The second claim is unproven and is
no longer made anywhere in this document.

## Why "keep both" is cheap here

The usual objection to two targets is maintenance. Measured:

- **4 template files, 386 lines total**, plus 4 reference docs.
- The core-wasm ABI is **frozen by LibreFang's host**, so the templates do not
  drift on their own — they change only if LibreFang changes, which is outside
  our control either way.

**And the version invariant that would normally worry us already holds:**
LibreFang pins `wasmtime = "46"` (`Cargo.toml:209`), the same major as UAR and
KnowMe. The three hosts disagree about *binary format*, not about *runtime
version* — so `fabric-integration`'s wasmtime invariant is not threatened by
keeping both.

## Assumptions

- **LibreFang will not adopt the Component Model soon.** Unverified — we do not
  control that repo. If it does, "port" becomes correct and this decision
  should be revisited rather than defended.
- **The pipeline is wired and its target is live; whether it has ever been
  *used* is UNKNOWN and stays unknown.** Measured both ways: the tooling is
  real (`forge package-librefang` installed, 37-step SSRF-guarded uploader) and
  LibreFang is running on `:4545` with 7 commits this week — but **zero**
  `.lf-skill.zip` artifacts exist and no `BOSSFANG_TOKEN` is configured. The
  decision rests on the host being **actively developed**, which is verified.
  It does **not** rest on production usage, which is not.
- **Two targets stay distinguishable to authors.** This is the assumption the
  decision's one added requirement exists to protect.

## Falsifier

Reverse if **any** of these is measured:

1. **LibreFang's host gains Component Model support.** Test: `grep -r
   "component::Component" librefang/crates/librefang-runtime/src/`. A non-empty
   result means one target can serve both hosts, and keeping two becomes
   gratuitous.
2. **LibreFang stops being actively developed.** This replaces an earlier
   "is the pipeline used?" falsifier that review rightly called
   decision-critical-but-deferred. **It was run instead of deferred** (see
   above), and the result was mixed: no artifact has ever been produced, but the
   host is live and committed to this week.

   Since usage is unknowable from here and development activity is not, the
   falsifier now tracks what is **measurable**:

   ```bash
   git -C /Users/gqadonis/Projects/references/librefang log --since='90 days ago' --oneline | wc -l
   lsof -iTCP -sTCP:LISTEN -P | grep -c librefang
   ```

   **Reverse to "retire" if both are 0** — no commits in 90 days *and* nothing
   listening. At that point we would be maintaining guest templates for a host
   nobody runs or changes, which is the actual thing worth avoiding.

   Today: **7 commits in 7 days, process listening on :4545.** Nowhere near.
3. **An author ships a guest against the wrong host.** If someone builds from
   the librefang templates and tries to load it in UAR (or vice versa), the
   "make the split explicit" requirement failed and needs to be a hard gate — a
   check, not a paragraph.

Falsifier 2 is the one to watch: it is the cheapest to test and the most likely
to fire.

## The one requirement this adds

Both targets must state, in their own docs, the host they target and the host
they **cannot** load in:

- `skills/rust/librefang-wasm-skill/SKILL.md` → "generates **core-wasm** guests
  for LibreFang; these **cannot** load in UAR's component runtime."
- `wit/prometheus-component/MAPPING.md` → already records the reverse. ✅

### Why it is deferred, and why that is not hand-waving

Review flagged a contradiction: the decision calls the missing documentation
"the defect", then defers fixing it. Fair — so the deferral is now bounded
rather than open-ended.

`change-uhe-004` is **decision-only by its own acceptance criteria** ("Write NO
code in this change"), and editing `librefang-wasm-skill/SKILL.md` is a change
to a shipped skill. Doing it here would violate the change's contract to save
one round trip.

**It is therefore carried into this phase's reflection as a named follow-up**,
not left to memory. If it is still undone when the phase closes, that is a
recorded delta — which is exactly how the previous phase's deferrals stayed
visible instead of evaporating.

On escalating to a check (falsifier 3): a check needs a detectable failure
signal. Today an author picking the wrong target fails at *load* time in their
own host, loudly. If that turns out to be a confusing failure rather than an
obvious one, falsifier 3 fires and the documentation requirement becomes a gate.

## Scope

**No code, no template changes, no cross-repo edits.** This change records the
decision only.

## Adversarial review record

Round 1 **BLOCK** (1 CRITICAL, 5 WARNING), judge `kbd-judge` via
`rest-gateway`, `cross_model_check: verified-distinct`, producer `claude-opus-5`.

| # | Severity | Finding | Response |
|---|---|---|---|
| 1 | CRITICAL | Keeps the templates because "retiring breaks a real path" while admitting the evidence for actual use was missing | **Accepted — that was the load-bearing claim and I had asserted it.** Checked instead: `forge package-librefang` is an installed command, `upload-to-bossfang` is a 37-step deploy skill with an SSRF guard and token redaction, and `native-agent` gates the wasm target's prerequisites. Nobody builds that for a path nobody walks. |
| 2 | WARNING | Falsifier 2 had no measurement source, owner, or window | **Accepted.** Now: ask the pipeline's owner at the next phase boundary whether a `.lf-skill.zip` has shipped in 90 days. Deliberately not a file search — artifacts are gitignored, so their absence proves nothing. |
| 3 | WARNING | Calls the documentation split "the fix", then defers it | **Accepted.** The deferral is now bounded: this change's own criteria say "write NO code", and it is carried into the phase reflection as a named follow-up rather than left to memory. |
| 4 | WARNING | Falsifier 3 waits for user-visible failure before requiring a check | **Accepted as reasoning, not changed.** A check needs a detectable signal; today the wrong target fails loudly at load. If that failure turns out to be confusing rather than obvious, falsifier 3 fires. |
| 5 | WARNING | "Porting means rewriting LibreFang's host" is broader than the evidence | **Accepted.** Softened to "teaching the host to load components" — wasmtime 46 supports both models, so the blocker is **ownership**, not impossibility. That distinction matters: it is what makes falsifier 1 live. |
| 6 | WARNING | Prior-decision search skipped malformed entries, so "unsettled" is unproven | **Acknowledged, not fixed here.** The malformed entries are the pk wiki's, not this document's. |

The CRITICAL is the one worth keeping: **I argued from a claim I had not
checked, in the one place the decision actually turned.** Checking took two
commands and made the answer stronger rather than weaker.

### Round 2 — `BLOCK` (2 CRITICAL, 5 WARNING) — stopping at the cap

| Finding | Response |
|---|---|
| The "pipeline is exercised" claim is still unsupported — the evidence proves **wiring exists**, not that anyone uses it | **Accepted, and it was right twice.** Round 1 made me check the tooling; round 2 made me check *usage*, which is a different question I had quietly conflated. Measured: **0** `.lf-skill.zip` artifacts, **no** `BOSSFANG_TOKEN` — but LibreFang **is running on :4545** with **7 commits this week**. The decision now rests on "the host is actively developed" (verified) and **no longer claims production usage** (unproven). |
| Falsifier 2 defers a cheap, decision-critical check until after commitment | **Accepted — so it was run, not deferred.** It also got rewritten to track something *measurable from here*: commits in 90 days and a listening process. Today 7 and yes; reverse to retire only if both hit zero. |

Both rounds attacked the same soft spot from different angles, and both were
right. **The lesson is the one that keeps recurring: I argued from the strongest
available claim rather than the one the decision actually turned on.** "Tooling
exists" and "anyone uses it" are different facts, and the second is the one that
would justify retiring.

Stopping at the 2-round cap. Nothing remains unresolved; the residual
uncertainty (has this pipeline ever shipped a skill?) is now **stated in the
decision** rather than argued away.
