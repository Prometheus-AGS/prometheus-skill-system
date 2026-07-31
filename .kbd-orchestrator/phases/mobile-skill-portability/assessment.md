# Assessment — mobile-skill-portability

**Phase:** `mobile-skill-portability` · **Assessed:** 2026-07-31
**Preflight:** `status: ok`, 2 distinct models, gateway `http://localhost:8181/v1` (HTTP 200)
**Adversarial review:** round 1 `BLOCK` (2 CRITICAL, 3 WARNING), round 2 `BLOCK`
(1 CRITICAL, 3 WARNING). All nine findings addressed; see [Review record](#review-record).
Round 2's residue is a **structural limit, not an unfixed defect** — see below.

## How cross-repo claims are evidenced

Three of this assessment's findings concern repositories **outside** this one.
A reviewer scoped to this repo cannot open them, and round 1 correctly refused
claims it could not check. Every external claim below is backed by a committed
excerpt under [`evidence/`](evidence/) carrying the source repo's commit SHA and
the file's SHA-256 — which makes the claim **reproducible**, not repo-verifiable
(see the limit stated below):

| Evidence file | Backs |
|---|---|
| [`evidence/uar-wasm-stub.md`](evidence/uar-wasm-stub.md) | Gap 1 — UAR's Wasm tier |
| [`evidence/knowme-host-real.md`](evidence/knowme-host-real.md) | Gap 1 — KnowMe's host |
| [`evidence/liter-llm-ffi-surface.md`](evidence/liter-llm-ffi-surface.md) | Gap 2 — FFI/JNI/WASM crates |
| [`evidence/skill-inventory.md`](evidence/skill-inventory.md) | the 319/60/259 counts, all 60 enumerated |

Paths are given in full from each repo root. Round 1 flagged that bare
`src/host.rs` collides with an unrelated `skills/rust/librefang-wasm-skill/references/example-echo/src/host.rs`
in *this* repo — a real ambiguity, now removed.

**A limit this evidence does not overcome, stated plainly.** A hash proves a file
had given contents when I read it; it does not let a reviewer *scoped to this
repo* confirm the file exists or that my excerpt is faithful. Gaps 1 and 2 are
therefore **cross-repo findings carrying reproduction commands**, not
packet-verifiable claims. Round 2 was right to keep flagging them, and the
honest resolution is to label them rather than to keep adding hashes:

> **Verification status of every claim below**
> - **Repo-verifiable** — gaps 3, 4, the 319/60/259 counts, out-of-scope tests
> - **Cross-repo, reproduction-command only** — gaps 1 and 2
>
> Anything in the second class must be **re-verified at plan time** by running
> the stated command, not accepted from this document.

## Headline

Two of the four seeded goals rest on assumptions that do not survive contact
with the code. Both corrections make the phase **more** tractable — but planning
against the seeded framing would build the wrong things first.

| Correction | Seeded assumption | Verified |
|---|---|---|
| **Goal 1's real blocker** | authoring `prometheus:component/*` blocks porting | UAR's Wasm tier is an **explicit stub** — loads components, never invokes them. A perfect WIT world changes nothing until that is fixed. |
| **Goal 3's pattern** | copy `frf-ffi` (uniffi); this pack has no cdylib | `tools/liter-llm` (submodule, **in-tree**) already ships `liter-llm-ffi` (cdylib+staticlib, **767-fn C ABI**), `liter-llm-jni` (**46** Android entry points + 150 Java files), `liter-llm-wasm`. |

## Gap 1 — UAR's Wasm execution tier is a stub (blocks goal 1)

> **Verification: cross-repo.** Not confirmable from this repo's review packet.
> Reproduce: `sed -n '92,111p' /Users/gqadonis/Projects/prometheus/universal-agent-runtime/src/uar/runtime/skills/wasm_runtime.rs`

`universal-agent-runtime` @ `563ecc2`, file
`src/uar/runtime/skills/wasm_runtime.rs:92-111` (excerpt + SHA-256 in
[`evidence/uar-wasm-stub.md`](evidence/uar-wasm-stub.md)):

```rust
let _store = Store::new(&self.engine, WasmHostState {});
let _linker: Linker<WasmHostState> = Linker::new(&self.engine);
// Concrete component bindings will be added once wit-bindgen
// integration lands …
let _ = input;
Ok(format!("<wasm skill '{skill_id}' loaded but binding not yet generated; …>"))
```

Every binding is `_`-prefixed and discarded. `run()` returns a placeholder
**string**, not skill output; the component is loaded and never instantiated.

**Why this reorders the phase.** The seeded goal says the WIT authoring "blocks
porting, by design". It does — but it is not the *first* blocker. A skill ported
to a perfectly-authored world and handed to UAR today yields a placeholder
string. **A ported skill cannot be verified to work until the host invokes it.**

The contract is small — `universal-agent-runtime/wit/uar-skill.wit:14`:

```wit
world skill {
  export run: func(input: string) -> result<string, string>;
}
```

One function. This is a gap in implementation, not in design.

### The other host is already real

`know-me-system` @ `28c0e10`, crate `rust/crates/knowme_plugin_host`
([`evidence/knowme-host-real.md`](evidence/knowme-host-real.md)):

- `src/host.rs:116` — `instantiate()` with capability enforcement; a component
  importing a host interface outside its declared set fails with
  `SandboxError::MissingCapability` **before any guest code runs**
- `src/sandbox/bindings.rs` — generated `bindgen!` bindings (present)
- `src/sandbox/e2e.rs` — **10 tests**, **3** `instantiate()` call sites

**This inverts the plan's implicit direction:** the reference implementation is
KnowMe's host, not something to build fresh in UAR.

## Gap 2 — the FFI pattern is already in-tree (reframes goal 3)

> **Verification: partly repo-verifiable.** `tools/liter-llm` IS a submodule of
> this repo, so the crates are in-tree — but the review packet's file tree does
> not descend into submodules, so a reviewer sees only `./tools/liter-llm`.
> Reproduce: `grep -c 'literllm_[a-z_]*(' tools/liter-llm/crates/liter-llm-ffi/include/liter_llm.h`

The reflection recorded "this pack has no cdylib/staticlib and no uniffi;
`frf-ffi` (uniffi 0.31.2) is the pattern to copy." True of `skills/` and
`substrate/` — but `tools/liter-llm` is a submodule **inside this repository**
(@ `3545cf6`):

| Crate | `crate-type` | Surface |
|---|---|---|
| `liter-llm-ffi` | `cdylib`, `staticlib`, `rlib` | cbindgen → `include/liter_llm.h`, **767 declared fns / 800 unique symbols / 6767 lines** |
| `liter-llm-jni` | `cdylib` | **46** `Java_..._LiterLlmBridge_native*` fns + **150** files in `packages/java/` |
| `liter-llm-wasm` | `cdylib` | wasm-bindgen |

> **Correction to my own first count.** I initially reported "4 exported
> functions" and concluded the ABI was "error-inspection only". That grep pattern
> was too narrow and the conclusion was wrong. The recount and its reproduction
> command are in the evidence file. This is a **complete client ABI**.

**Two competing patterns; the plan must pick one.** liter-llm uses **cbindgen +
hand-written JNI**; `frf-ffi` uses **uniffi 0.31.2**, generating Kotlin/Swift
from one definition. uniffi is less code across two platforms; cbindgen is
already working, in-tree, and at scale here.

## Gap 3 — `fabric-integration` does not exist; three of four invariants unchecked

> **Verification: repo-verifiable.** Search output committed to
> [`evidence/gap3-negative-search.md`](evidence/gap3-negative-search.md),
> including the grep exit code (1 = no matches) — a negative claim needs the
> search, not a summary of it.

Verified in this repo: no `skills/*/fabric-integration`, and no file under
`.github/workflows/` or `scripts/*.sh` references `loro`, `wasmtime`, or `iroh`
for version checking.

Exactly one invariant is enforced (`iroh >= 1.0.2`, by `change-idt-008`, via
Cargo). The other three are prose. The WIT-version invariant is **already
violated**: `knowme:plugin` resolves ambiguously between 0.1.0 and 1.0.0.

## Gap 4 — `zed` falls to Tier 0 (one line)

`skills/learn/ui-surface/scripts/render.sh:174` routes `opencode|codex|kimi` to
`_render_tier1_file_pair`. `zed` is detected at line 153 but hits the `*)`
fallback. `opencode` and `kimi` share codex's verified path but have never run.

## The portability surface is smaller than it looks

All 60 script-bearing skills are enumerated in
[`evidence/skill-inventory.md`](evidence/skill-inventory.md) with the generating
command, so these numbers are reproducible rather than asserted:

| Category | Count | Mobile story |
|---|---|---|
| No `scripts/` dir | **249** | E0 manifest-only — **portable today** |
| With `scripts/` | **60** | need E1 (Wasm), E2 (native), or remote execution |
| **Total** | **309** | |

> **Corrected by `change-msp-001`.** The first count said 319/259 because it
> included `SKILL.md` files under `node_modules/`, which are vendored package
> content, not skills. Excluding them gives 309/249. The script-bearing count of
> 60 was unaffected.

**81% need no porting at all.** The real scope is the 60 — and many of those
scripts are build/validation tooling a phone would never invoke.

**Recommendation:** classify the 60 before porting any. Remote execution (the
P2P story from the previous phase) may cover most at far lower cost than porting.

## Out of scope — carried forward, not absorbed

Two `sovereign-sync` integration tests fail:
`one_projects_token_is_rejected_by_another_project` and
`two_projects_mint_distinct_identities_and_tokens`. Confirmed last phase via
`git stash` that they fail identically without those changes. Control-token
derivation — unrelated to WASM, FFI, or iroh.

**They are not this phase's work unless explicitly selected.** Recorded here so
they cannot quietly become this phase's failure, per the goals-file constraint.

## Open questions for plan

1. **Does UAR adopt KnowMe's `knowme_plugin_host` sandbox, or reimplement?**
   One host is real with 10 e2e tests; the other is a stub. Reimplementing
   duplicates working capability-enforcement code.
2. **cbindgen (liter-llm, in-tree, 767 fns) or uniffi (frf-ffi)?** Cannot have
   both as "the pattern".
3. **How many of the 60 script-bearing skills must actually run on a phone?**
   Unknown until classified; this gates the phase's size.
4. **Is cross-repo code authorised?** Closing gap 1 means writing into
   `universal-agent-runtime`. The previous phase touched **zero** files in the
   three external repos. This needs the user's explicit agreement — it cannot be
   done inside this repository, and recording a decision did not authorise it.

## Suggested reordering

The seeded order (WIT → fabric-integration → FFI → harness) puts the least
verifiable work first. Evidence supports:

1. **Classify the 60 script-bearing skills** — cheap, in-repo, sizes everything else
2. **`zed` routing + verify `opencode`/`kimi`** — one line plus two runs; closes goal 4
3. **`fabric-integration`** — in-repo; makes 3 unenforced invariants enforced
4. **UAR host de-stubbing** — needs cross-repo authorisation; blocks all porting
5. **WIT authoring** — correct as designed, unverifiable until 4 lands

## Review record

Round 1 verdict **BLOCK**, judge `kbd-judge` via
`rest-gateway:http://localhost:8181/v1`, `cross_model_check: verified-distinct`,
producer `claude-opus-5`.

| # | Severity | Finding | Response |
|---|---|---|---|
| 1 | CRITICAL | Gap 1 cites UAR paths absent from the packet | **Accepted.** Excerpts + repo SHA + file SHA-256 committed to `evidence/uar-wasm-stub.md`. Files verified present on disk. |
| 2 | CRITICAL | KnowMe citations resolve to missing/unrelated files | **Accepted, and the collision was real** — bare `src/host.rs` matched an unrelated example in this repo. Full paths now given; excerpt committed. |
| 3 | WARNING | liter-llm crate claims unsupported by the file tree | **Accepted.** Crate types, symbol counts, and reproduction commands committed. Recount also **corrected my own error**: 767 fns, not 4. |
| 4 | WARNING | Omits the sovereign-sync do-not-absorb boundary | **Accepted.** Added as an explicit out-of-scope section. |
| 5 | WARNING | Skill counts unverifiable from the packet | **Accepted.** All 60 enumerated with the generating command in `evidence/skill-inventory.md`. |

No finding was rejected. Finding 3 surfaced a factual error of mine that would
otherwise have understated the in-tree FFI surface by two orders of magnitude.

### Round 2 — `BLOCK` (1 CRITICAL, 3 WARNING)

| # | Severity | Finding | Response |
|---|---|---|---|
| 1 | CRITICAL | Gap 3's repo-wide negative search has no packet evidence | **Accepted.** Search commands **and their exit codes** committed to `evidence/gap3-negative-search.md`. A negative claim needs the search, not a summary. |
| 2 | WARNING | iroh enforcement claimed without a citation | **Accepted.** `grep -n "^iroh"` output for both Cargo.toml files, plus the archived change path `openspec/changes/archive/2026-07-31-change-idt-008-feature-gate-iroh-docs`, added to the same file. |
| 3 | WARNING | The WIT contract cites a MISSING external path | **Accepted as a limit, not fixed by more evidence.** Gap 1 is now labelled cross-repo with a reproduction command. |
| 4 | WARNING | KnowMe bare paths still collide or are missing | **Accepted.** Absolute paths + SHA-256 per file added; the section states these are *not* in this repo. |

**Why round 3 was not run.** Rounds 1 and 2 differ in kind: round 1 found a
factual error (the 4-vs-767 ABI count) and genuine path ambiguity; round 2 found
only that cross-repo claims cannot be checked from a single-repo packet. That is
true and unfixable by adding evidence — the packet builder does not descend into
other repositories or into submodules. Continuing to iterate would produce more
hashes without changing what the reviewer can verify.

The honest resolution is the **verification-status labelling** at the top: gaps 1
and 2 are marked cross-repo with reproduction commands and **must be re-verified
at plan time**. Per the skill's 2-round cap, this section is the required
"Unresolved review findings" disclosure.
