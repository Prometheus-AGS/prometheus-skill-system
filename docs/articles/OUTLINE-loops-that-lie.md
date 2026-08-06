# Series Outline — "Loops That Lie"

**Status:** DRAFT OUTLINE for review. Not the articles.
**Platform:** Medium.com — 5-part series
**Author:** Travis James | AI-drafted in Travis's voice
**Standard:** Authentic Digital Twin Content Standard v2, Tier 1
**Predecessor:** `docs/articles/autonomous-loops-prometheus-skill-pack.md` (2026-06-24, **7,616 words**)

---

## Why a series, and why these lengths

Researched against current Medium guidance (sources at end):

| Constraint | Finding | Consequence here |
|---|---|---|
| Optimal length | **1,200–2,000 words**; 5–8 min read, **7 min the sweet spot** | The original was **7,616 words** — roughly 4× the ceiling |
| Series length | **3–5 parts** is the explicitly recommended shape | 5 parts, each standalone |
| Subheads | H2 every **300–500 words** | 4–6 H2s per piece |
| Paragraphs | **2–4 sentences max** — long blocks kill mobile readability | Hard rule; the original violated it repeatedly |
| Algorithm | Rewards **reading time + completion rate**, not clicks | Shorter pieces finished > one long piece abandoned |
| Curation | Curated articles reach **10–100×** more readers | Optimize each part for curation independently |
| Headlines | Specific numbers and concrete outcomes beat abstractions | Every title below carries a number or a falsifiable claim |
| **AI content** | **Medium actively detects and deprioritizes purely AI-generated content in 2026** | ⚠️ See the AI-disclosure risk note below — this is the single biggest publication risk |

**Total series:** ~7,500 words — the same budget as the original, but delivered as five finishable pieces with five entry points, five curation chances, and four internal cliffhangers instead of one 30-minute wall.

Use Medium's native **Series** feature to bind the parts, plus explicit prev/next links in each.

---

## ⚠️ The AI-disclosure risk (decide before drafting)

Medium's 2026 algorithm demotes content detected as purely AI-generated. The original article openly states *"AI-drafted in Travis's voice"* and carries a full provenance manifest.

That transparency is admirable and it is a **distribution risk**. Three options:

1. **Keep the manifest, lead with human framing** *(recommended)* — the war stories are genuinely yours; the cold opens should be human-authored and read that way. Keep the provenance footer, but make the opening 200 words unmistakably first-person and specific. Detection keys on generic AI cadence, not on disclosure.
2. Move the manifest to a linked appendix — preserves the standard, reduces the surface.
3. Drop the AI-drafting note — **not recommended**; it would violate the Tier 1 standard the series is built on, and the series' whole subject is honest disclosure.

The irony is worth one line in Part 1: *a series about verification machinery that silently failed, published on a platform whose verification machinery may silently penalize it.*

---

## The series thesis

The first article argued the unit of work is the **loop**. That held.

This series argues the loop was the **easy part**. Once loops run unattended at volume, the constraint stops being autonomy and becomes **verification** — and the repo's own reflection names the defect better than I could:

> **"Asserting on a result without checking that the code path producing it executed."**

Every part below is that sentence in a different costume.

---

# Part 1 — "Eight Adversarial Reviews. Zero Adversaries."

**Target:** 1,600 words (7 min) · **The anchor piece.** Publish first; it earns the series.

**Headline options:**
- Eight Adversarial Reviews. Zero Adversaries. *(recommended — number + falsifiable claim)*
- Your Loop Is Lying to You
- The Guard That Never Fired

**Subtitle:** *I built an anti-sycophancy gate so my AI couldn't grade its own homework. It had never once run.*

### Beats

1. **Cold open (human-authored, ~250 words).** June's article. The comparison table. The ✅ next to "anti-sycophancy gate."
2. **The discovery.** All 8 stored `findings.json`: `judge_model: "harness-subagent (claude, parent-session family)"`. Claude reviewing Claude. Every one `PASS`.
3. **Five compounding causes** — one short paragraph each:
   - Three model-config surfaces, none authoritative. `project.json model_policy` declared as `policy_source` by five skills, **read by none**.
   - Flat `[aliases]` table vs the required `[[aliases]]` array. The parser extracted nothing and passed the literal string `"frontier"` through as a model id.
   - Every packet carried `producer_model: "unknown"`, so `candidate != producer` passed **trivially**. The guard existed but never fired.
   - `isolation_mode` hardcoded regardless of endpoint — a self-grade was indistinguishable from a real review.
   - The shipped proxy config couldn't serve a request at all (401 on every route).
4. **The pattern.** Every one of those five produced a **green check**. Not one produced an error.
5. **The fix, and how it's now provable.** Fixtures: flawed→BLOCK, clean→PASS, `cross_model_check: verified-distinct`, judge names the *specific* planted defects. That's discrimination, not a coincidental block.
6. **Close + series promise.** "This was one of five. Here are the other four." → link Part 2.

**Why it leads:** it's a confession, it's specific, it's falsifiable, and it indicts the author's own prior article. Highest curation odds in the set.

### ⭐ Recommended addition — a second confession, ~150 words

The landscape research turned up a finding that belongs *here*, not buried in Part 5:

**OpenAI shipped `/goal` in Codex CLI 0.128.0 on 2026-04-30 — two months before the June article published.** Same plan→act→test→review→iterate loop, same goal-conditioned termination, state persisted across sessions. The June comparison table implied that capability was Claude Code's differentiator.

That row wasn't aged by events. **It was wrong on the day it published, and one changelog check would have caught it.**

Pairing the two confessions makes Part 1 far stronger than either alone:
- The gate that never ran = *I didn't verify my own system.*
- The `/goal` row = *I didn't verify my claim about someone else's.*

Same failure, both directions. It sets up the series thesis in the opening piece instead of deferring it, and it earns the reader's trust for the four parts that follow.

---

# Part 2 — "The Consensus Layer That Could Never Gain a Second Voter"

**Target:** 1,500 words (6–7 min)

**Subtitle:** *236 MB of storage backing a 49 KB journal, and a health check — a static constant — that took 12 seconds.*

### Beats

1. **Hook: the absurd measurement.** `GET /health` returns a hardcoded JSON constant. It took **12 seconds**.
2. **What it claimed to be.** OpenRaft over redb; distributed durability for the KBD control plane.
3. **What it was.** Initialized with exactly one voter. `grep -rn 'add_learner\|change_membership'` → nothing outside tests. **A second voter could never join.** Structurally incapable of the thing it was named for.
4. **The other symptoms.** `raft.redb` at 236 MB behind a 49 KB journal; 2-minute daemon startup.
5. **Root cause.** One unpruned JSON blob (`command_results`) deserialized on every read, re-serialized on every write, on copy-on-write storage that never shrinks.
6. **The deletion.** Removed entirely; replaced with Loro CRDT authority + per-replica fsynced WAL. CHANGELOG 1.7.0 lists it under **Removed** — "obsolete KBD voter/quorum facade."
7. **The lesson.** Shipping a consensus layer is a press release. Measuring it, finding a facade, and tearing it out is engineering.

**Verified:** no `openraft` dependency remains in any manifest or source file. ✅

---

# Part 3 — "Green Tests That Assert Nothing"

**Target:** 1,600 words (7 min) · **Most broadly applicable — likely best standalone performer.**

**Subtitle:** *A 26-test suite that finished in 0.00 seconds, an 18-of-18 audit failure, and a drift check that could never catch its own bug.*

### Beats

1. **Hook.** 26 REST tests, zero persistence assertions, whole suite green in `0.00s` — because nothing ever touched a database. `SkillService::new(None, None)`.
2. **The residual class.** Mobile execution class E1 defined as "pure text/JSON transformation" — the *residual* after other rules matched. Audit: **all 18 of 18 members touch the filesystem or clock.** Not one was pure.
   - **The killer detail:** `--check` drift detection *can never catch this*. It compares the committed file to a freshly generated one; both come from the same wrong rule, so they agree forever. The risk was written in the script's own header and shipped anyway.
3. **The probe that was wrong, not the service.** Two services reported broken for weeks; neither was. One health check sent malformed JSON-RPC (missing `jsonrpc`); a strict server correctly returned 422 while a lenient one accepted it — so one looked broken and the other didn't, from the same bad probe.
   - **The consequential half:** the *installer* used that same dead probe to decide whether the service was running. It always failed, so **every install restarted a healthy daemon.** The monitoring was the outage.
4. **Two of three providers passing was not two-thirds of the evidence.** In-memory and SurrealDB passed; Postgres rejected with `vector must have at least 1 dimension`. The two passing providers *structurally could not fail that way* — `InMemoryProvider::save_skill` takes `_embedding` and discards it.
5. **Synthesis.** One sentence, from the repo: *asserting on a result without checking that the code path producing it executed.*
6. **Rules.** If a class is "everything left over," hand-verify a sample. When only *some* servers fail, suspect the client. A green suite that runs in 0.00s ran nothing.

---

# Part 4 — "From Markdown to a Compiled Core"

**Target:** 1,500 words (6–7 min)

**Subtitle:** *The system I described in June was shell scripts. Four days later I started writing Rust, and I didn't have a choice.*

### Beats

1. **Hook.** `substrate/`, `crates/`, and `wit/` did not exist when the first article published. `git ls-tree` at that commit returns empty for all three.
2. **The delta table** (verified this session):

   | | 2026-06-24 | 2026-08-06 |
   |---|---|---|
   | Version | 1.2.0 | 1.7.0 (+443 commits) |
   | Native skills | 102 | **147** |
   | Rust | **0 crates** | 17 crates, **55,732 LOC**, **385 tests** |
   | Install targets | 7 | 14 |
   | MCP servers | 7 | **7 — unchanged** |

3. **The counter-intuitive one.** MCP servers stayed at 7 while everything else grew. Depth, not surface.
4. **Why Rust was forced.** Shell can *do* the work; it cannot *prove* it. `prometheus-exec`: Ed25519-signed receipts, content-addressed artifacts, offline `verify-bundle`, tiered isolation (macOS Seatbelt / Wasmtime 46 component model).
   - Live evidence: run `ad5e5f54-…` Succeeded, receipt `sha256:3cba728f…`
5. **The honest boundary — do not skip.** `mobile-size: blocked`, `physical-device: pending_evidence`, `remote-deployment: pending_evidence`. Linux has cross-build but no kernel runtime evidence.
6. **Lesson.** Signed receipts aren't gold-plating. They're the price of "it ran unattended and I believe it."

---

# Part 5 — "37 Phases, Two Zeros, and One 3.5"

**Target:** 1,400 words (6 min) · **The closer.**

**Subtitle:** *What six weeks of autonomous loops actually shipped — including the phases that scored nothing.*

### Beats

1. **Hook: the miscount.** My first pass at this series said the pack grew from 102 to **311** skills. Wrong — that counted 177 submodule files and installed copies. The real like-for-like figure is **147**.
   - Three of my own numbers were wrong on first pass (skills 311→147, Rust LOC 32k→55.7k, tests 234→385). All three caught by re-running the command. **The impressive number was always the unverified one.**
2. **The record.** 37 completed phases in six weeks, each with a `reflection.md` carrying MET/PARTIAL/NOT MET against goals declared *before* the work.
3. **The scores worth publishing:**

   | Phase | Score |
   |---|---|
   | `phase-external-validation` | **0/5 MET** |
   | `phase-first-user-onboarding` | **0/5 MET** |
   | Codex verify-and-publish | **3.5/4** — refused to round up |
   | `kimi-desktop-extensibility` | 2 MET, 1 PARTIAL, **1 "MET-but-unobserved"** |

4. **The negative close.** An operator asked whether Kimi Desktop UI customization was achievable. Answer: **no.** Recorded as **MET (the answer is no)** so it's never re-investigated. A loop that can close a question negatively is doing real work.
5. **The stale-warning twist.** CLAUDE.md still warns UAR's Wasm tier "is still a stub… nothing has executed it." That is now **out of date** — `uar-host-execution` S1 is MET, a component executed and returned its own JSON. Verification debt cuts both ways: it makes you overclaim *and* underclaim.
6. **The landscape re-scored — and the row the first article got wrong.** ✅*research complete; key claims independently re-verified*

   *(The `/goal` confession moves to Part 1 — see the starred addition there. Reference it here in one sentence, don't re-tell it.)*

   **Commoditized, not validated.** The loop-centric thesis held — and then the platforms absorbed it:
   - Loop primitives converged across vendors *before* the discourse named them (Codex `/goal`, April).
   - **Agent Skills won by adoption with no standards body at all.** OpenAI, Google, Cursor, and GitHub *adopted* rather than competing; Cursor ships `/migrate-to-skills` to convert its own proprietary rules. GitHub Copilot code review skills + MCP hit **GA 2026-07-29**.
   - MCP downloads: **400–500M/month** (Anthropic: "400M monthly SDK downloads, a 4x increase this year"; MCP maintainers: "close to half a billion").

   **Anthropic put guardrails on its own fleets** — a four-day retune (2026-07-21 → 07-24): v2.1.217 capped concurrent subagents at 20 and disabled nested spawns entirely; v2.1.219 reinstated nesting at depth 3. Changelog rationale: *"so one message can't fan out unbounded background agents."* The vendor that popularized fleets spent a week limiting them.

   **The honest counter-evidence** — include it, do not bury it. METR **superseded** (did **not** retract) its 2025 finding of a **19% slowdown**, now estimating ~18% speedup in early 2026. Their own caveat is the quotable part:

   > "because of the selection effects in our experiment, our data is only very weak evidence for the size of this increase"

   Frame it exactly that way. Calling it a "retraction" would be the same overclaim this series is about.
7. **Close.** Cherny's quote still stands; the job is still writing loops. Six weeks at volume adds a second sentence: *writing the loop is the first half of the job. Proving it did what it said is the second — and it decides whether the loop compounds or merely accumulates.*
8. Land on: **the measure of an autonomous system isn't what it does when it works. It's what it tells you when it doesn't.**

---

## ⚠️ DO NOT CLAIM — binding constraints for every part

Verified against repo evidence. Violating any of these makes the series self-refuting, since its subject is unverified verification.

| # | Do not claim | Reality |
|---|---|---|
| 1 | OpenRaft as a shipped feature | Added **and removed**; no dependency remains ✅*verified* |
| 2 | exec certified on mobile/Linux/Windows/remote | `blocked` / `pending_evidence` per evidence file ✅*verified* |
| 3 | KBD control plane production-certified | "**Not certified; launch agent intentionally unloaded**" |
| 4 | The two Wasm formats interoperate | core-wasm vs Component Model; **no adapter** |
| 5 | "Blocking" gates are unbypassable | The creators are **prompt files, not executables** |
| 6 | UI skill administration works end-to-end | PARTIAL — e2e unrunnable; package-name mismatch breaks 90+ imports |
| 7 | Mobile skill updates have a transport | BLOCKED — metadata only; daemon binds loopback |
| 8 | Clean OpenSpec validation | 102 legacy changes invalid under `--strict` |
| 9 | Skill count 310/311/324 | **147 native** / 145 canonical index ✅*verified* |
| 10 | CI verifies fabric invariants | Verifies **1 of 4**; "SKIP is honest but not coverage" |
| 11 | `sovereign-sync` tests fully green | Two control-token tests fail; **deliberately unfixed** so they stay visible |
| 12 | Kimi Desktop integrations function | Goal covers *durability only* |

**Exception:** #4's cousin — UAR's Wasm tier — **now executes** (`uar-host-execution` S1 MET). Cite the reflection, not CLAUDE.md.

### External claims — do not use, or hedge

Research surfaced several plausible-looking claims that do not survive checking. A series about false green checks cannot ship these.

| Claim | Status |
|---|---|
| "METR **retracted** its slowdown study" | **False.** Superseded, not retracted; they call the new data "only very weak evidence." Use their words. ✅*verified* |
| "Anthropic donated **Agent Skills** to the AAIF" | **False.** AAIF hosts MCP, goose, AGENTS.md, agentgateway — not Skills. The spec repo has no `GOVERNANCE.md`. Likely conflation with the real MCP donation. |
| MCP governance moved to Linux Foundation "recently" | **Out of window** — donation was **2025-12-09**. Only roster churn happened Jun–Aug. Attach the date or drop it. |
| MCP "250M downloads/week" | **Discard** — implies ~1.08B/month, 2.5× the official figure. Likely a unit slip. Use **400–500M/month**. |
| MCP "97M downloads / 10,000 servers" | **Stale** December 2025 data still circulating in 2026 coverage. |
| MCP 2026-07-28 "clarifies relationship to Agent Skills" | **False.** The official changelog contains zero mentions of Agent Skills or SKILL.md. |
| "101 plugins" in the marketplace | **Stale** March data. Measured: official 206 → 278 (+35%), community 2,201 → 2,298 (+4.4%). |
| AAIF membership counts, registry 20,304 count, skills.sh sizing, OWASP AST10 dates | **Single-sourced or internally contradictory.** Hedge or omit — OWASP dates especially should not be cited without re-verification. |

**Also worth carrying:** MCP's **Roots, Sampling, and Logging are deprecated** (12-month minimum window), and the Tasks redesign moved from blocking `tasks/result` to polling `tasks/get`. Server-initiated sampling — how a server could drive the model — now has a clock on it. That is loop-relevant and under-covered elsewhere; it could seed a Part 6 if the series extends.

---

## Fact-check protocol

1. **Every number re-runs at draft time.** Not copied from this outline — re-executed. Three numbers here were wrong on first pass; all three were caught this way. That rate is the argument for the rule.
2. Every DO-NOT-CLAIM row re-verified — some are already stale in the *conservative* direction.
3. Every external claim carries a primary-source URL and a date.
4. Distinguish three states in prose: *implemented* / *verified locally* / *certified across platforms*. The first article's ✅ column collapsed all three; that's how a broken judge survived eight reviews.
5. Run sycophancy-correction on each draft before publishing.

---

## Publishing plan

| Part | Title | Words | Cadence |
|---|---|---|---|
| 1 | Eight Adversarial Reviews. Zero Adversaries. | 1,600 | Week 1 |
| 2 | The Consensus Layer That Could Never Gain a Second Voter | 1,500 | Week 1 (+3 days) |
| 3 | Green Tests That Assert Nothing | 1,600 | Week 2 |
| 4 | From Markdown to a Compiled Core | 1,500 | Week 2 (+3 days) |
| 5 | 37 Phases, Two Zeros, and One 3.5 | 1,400 | Week 3 |

**Total ~7,600 words** — same budget as the original, five entry points instead of one.

- Twice weekly matches the "2–3 long-form per week" pattern of fastest-growing accounts without straining quality.
- Each part opens with one line of series context and ends with a specific next-part hook.
- Each must stand alone: a reader landing on Part 3 from search needs no prior part.
- Bind with Medium's native **Series** feature plus explicit prev/next links.
- Retrofit a link into the June article pointing at Part 1.

---

## Assets

**Reuse:** existing diagrams in `docs/articles/` (lifecycle, KBD pipeline, evolver cycle, MCP map, distribution) — still accurate.

**New (SVG + PNG + light PNG, matching convention):**
1. `guard-that-never-fired.svg` — 5 causes, each terminating in a false green *(Part 1)*
2. `raft-facade.svg` — claimed vs actual topology, with the 236 MB / 49 KB figure *(Part 2)*
3. `verification-stack.svg` — request → signed receipt → offline verify → 14-target attestation *(Part 4)*
4. Re-baselined capability table graphic *(Part 5)*

**Cover:** variant of `article-cover.png` per part for visual series identity.

---

## Open questions for Travis

1. **AI-disclosure handling** — option 1 (keep manifest, human-authored cold opens) is my recommendation. Confirm?
2. **Order** — Part 1 is the confession. Alternative: lead with Part 4 (the build) and hold the confession for Part 2. I recommend confession-first; it's the differentiated hook.
3. **Name the judge model** in Part 1? More credible, but dates the piece and invites vendor noise.
4. **Publication** — your own profile, or pitch to a publication (curation reach is 10–100×)?
5. **Part 5's landscape section** — still pending research. If it comes back thin, Part 5 works without it; say the word and I'll cut it rather than pad.

---

## Sources — landscape research (verified this session)

- [OpenAI Codex CLI 0.128.0 integrates `/goal`, 2026-04-30 — AgentUpdate](https://www.agentupdate.ai/news/openai-codex-cli-goal-feature-0-128-0/)
- [Codex `/goal`: the autonomous coding loop explained — Habr](https://habr.com/en/articles/1037362/)
- [We Are Changing Our Developer Productivity Experiment Design — METR, 2026-02-24](https://metr.org/blog/2026-02-24-uplift-update/) ← the "very weak evidence" quote
- [Measuring the Impact of Early-2025 AI on Experienced OS Developer Productivity — METR](https://metr.org/blog/2025-07-10-early-2025-ai-experienced-os-dev-study/) ← the original 19% slowdown
- [Self-Reported Impact of Early-2026 AI on Technical Worker Productivity — METR, 2026-05-11](https://metr.org/blog/2026-05-11-ai-usage-survey/)
- [Claude Code put guardrails on its own agent fleets (v2.1.217 → v2.1.219, Jul 21–24) — Digital Applied](https://www.digitalapplied.com/blog/claude-code-subagent-depth-limits-budget-caps-2026)
- [Measuring AI agent autonomy in practice — Anthropic](https://www.anthropic.com/research/measuring-agent-autonomy)
- [How we contain Claude across products — Anthropic](https://www.anthropic.com/engineering/how-we-contain-claude)

⚠️ Prefer primary sources at draft time: pull the Codex CLI GitHub release notes for 0.128.0 and the Claude Code changelog entries for v2.1.217/219 directly, rather than citing the secondary write-ups above.

---

## Sources — Medium format research

- [How to Grow on Medium in 2026: Complete Writing Strategy Guide — Teract](https://www.teract.ai/resources/grow-medium-audience-2026)
- [Grow on Medium in 2026 — Proven & New Strategies That Actually Work](https://medium.com/@Saifullah-Ghanghro/grow-on-medium-in-2026-proven-new-strategies-that-actually-work-6196dd3a3a6d)
- [The Optimal Structure and Length for Medium Articles](https://medium.com/@florian-schroeder/the-optimal-structure-and-length-for-medium-articles-in-2025-0bd49fdddd7c)
- [How Long is The Ideal Medium Article? — Medium Course](https://mediumcourse.com/how-long-is-the-ideal-medium-article/)
- [How to Use Medium's "Series" Feature — Medium Course](https://mediumcourse.com/how-to-use-mediums-series-feature-to-publish-articles-on-a-single-topic/)
- [Medium Titles, Subtitles, and Kickers — Blogging Guide](https://medium.com/blogging-guide/medium-titles-subtitles-and-kickers-ce28a5700487)
