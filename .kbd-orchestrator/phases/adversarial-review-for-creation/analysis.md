# Analysis — adversarial-review-for-creation

**Stage:** analyze
**Date:** 2026-07-30
**Question posed:** does exposing a Rust agent that speaks A2A + AG-UI + A2UI, and serves an
MCP server with MCP Apps for ideation (including *dynamically generated* apps mid-task),
change the calculus from assess?

**Short answer: yes, materially — but not for the reason it appears to.** It does not
rescue the persona-debate idea the assess research invalidated. It changes *delivery
economics* for the part that survived, and it converts one assess recommendation from
"probably right" to "clearly right".

---

## 1. What already exists (do not rebuild)

Verified in-repo before researching anything. Commands and counts are inlined so this
section is checkable without the tree:

- `grep -c "MCP client" skills/process/native-agent/SKILL.md` → **3**; hits for the agent
  *exposing* a server → **0**. The generated `agent-mcp` crate is described as a
  "lightweight JSON-RPC 2.0 MCP client".
- `grep -c "/mcp/" substrate/surface-bridge/src/main.rs` → **4** routes
  (`/mcp/detect-surface-tier`, `/mcp/render-ui-intent`, `/mcp/submit-response`, plus health).
- `curl :7890/health` → **HTTP 200** (live at time of writing).
- rmcp versions: `prometheus-research` and `sovereign-sync` at **1.8**, `liter-llm` at
  **2.1**; `substrate/sovereign-sync/src/mcp_server.rs` exists — we already run a Rust MCP
  server.


| Asset | State |
|---|---|
| `native-agent` generated agent | Already speaks **A2A + AG-UI + A2UI** — verified in its SKILL.md |
| Generated `agent-mcp` crate | **MCP client only.** Not a server. This is the real gap. |
| `substrate/surface-bridge` | Live on `:7890`, routes `/mcp/detect-surface-tier`, `/mcp/render-ui-intent`, `/mcp/submit-response` — an MCP-App-shaped UI server we already run |
| `skills/learn/ui-surface` | Tier-degradation contract (Tier 0 text → Tier 1 AskUserQuestion → Tier 2 MCP App iframe) |
| rmcp in Rust | Already used at **1.8** (`prometheus-research`, `sovereign-sync`) and **2.1** (`liter-llm`); `sovereign-sync/src/mcp_server.rs` is a working Rust MCP **server** |

So the proposed capability is roughly **one crate away**, not a new product line: the
generated agent needs an MCP *server* surface alongside its client, and `surface-bridge`
already proves the UI-over-MCP pattern works in our stack.

## 2. What the research says about MCP Apps

**MCP Apps went GA as the first official MCP extension** (2026-01-26). This is a
significant change from the assess-stage picture.

- **Cross-client, today:** Claude (web + desktop), Goose, VS Code Insiders, ChatGPT.
  > "For the first time, an MCP tool developer can ship an interactive experience that
  > works across a broad range of widely-adopted clients without writing a single line of
  > client-specific code."
- **The contract is two primitives:** a tool carrying `_meta.ui.resourceUri`, and a UI
  resource served under the `ui://` scheme. The host renders it in a sandboxed iframe and
  talks JSON-RPC over `postMessage`.
- **`updateModelContext`** lets the UI write back into the model's context — which is
  exactly what a decision-capture surface needs.
- **Security model is real:** iframe sandboxing, pre-declared templates, auditable
  JSON-RPC, and host-enforced consent for UI-initiated tool calls.

**The SDK is TypeScript-only** (`@modelcontextprotocol/ext-apps`). That is a genuine
friction point for a Rust agent — but *not* a blocker, because the wire contract is just
tool metadata plus a resource. Any spec-compliant server can serve it, and we already run
rmcp servers.

## 3. The finding that actually changes the calculus

Google's A2UI team, with the MCP Apps co-creators, published **three integration patterns**
(2026-06-17). One of them removes the Rust-SDK objection entirely.

### Pattern 1 — A2UI over MCP

An MCP server returns a JSON payload with MIME type **`application/a2ui+json`** under the
**`a2ui://`** URI scheme. The host renders it with *its own native components*.

> "A2UI-over-MCP bypasses the iframe, allowing the host application to natively render the
> agent's intent using its own design system."

Why this is the decisive finding for us:

- **It is backend-agnostic.** Emitting `application/a2ui+json` from Rust is ordinary
  `serde_json`. No TypeScript SDK required. The Rust-SDK gap evaporates.
- **"Write-once, render-natively anywhere"** — React, Flutter, or Angular hosts, no custom
  wiring per surface. That maps precisely onto our existing `ui-surface` tier contract.
- **Capability-based security:** the client renders only components from a predefined
  catalog, versus shipping raw HTML.
- It also sidesteps the iframe's documented costs — "aesthetic inconsistencies like
  clashing design systems or redundant scrollbars… hurdles in both computational
  performance and security encapsulation."

Two delivery modes matter for ideation:

| Mode | Mechanism | Fit |
|---|---|---|
| **Static** | `resources/read` on `a2ui://<name>` | decision-record forms, commitment capture, review-date prompts — cacheable, zero LLM synthesis |
| **Dynamic** | `tools/call` returning an embedded resource | a persona panel assembled from *this* idea's actual critique findings |

### Patterns 2 and 3 (noted, not recommended for v1)

- **MCP Apps inside A2UI components** — for state-heavy widgets needing a real sandbox.
- **A2UI inside MCP Apps** — a modernization bridge for hosts with no native A2UI.

Both are escape hatches. Neither is needed for an ideation surface.

## 4. Does this rescue the persona-debate idea?

**No, and this is the important discipline.**

The assess stage found a NeurIPS 2025 spotlight *proving* debate alone induces a martingale
over belief trajectories and does not improve expected correctness. **A better UI does not
change a mathematical result.** A prettier round-table is still a round-table.

What the UI capability *does* change is the surface for the parts that survived:

| Survived assess | How MCP Apps / A2UI helps |
|---|---|
| Asymmetric producer → critic → judge | Render findings as a structured, inspectable panel instead of a wall of text |
| Longitudinal decision memory | A **static** `a2ui://decision-record` form — the highest-value, lowest-risk surface |
| Anti-automation-bias design | Forms that make the user *state their own reasoning first*, then reveal critique — structurally resisting "False Confirmation" |
| Feynman/Karpathy loops | Progress and calibration views over the wiki |

> **Design consequence.** The automation-bias literature says visible AI reasoning can
> *increase* over-reliance. Therefore the ideation UI should **capture the user's position
> before showing the critique**, and record both. An interface that leads with a confident
> verdict makes the documented failure mode worse, no matter how good it looks.

## 5. Dynamically generated apps — the sharpest part of the question

You asked specifically about generating MCP Apps *dynamically as part of other agent work*.
This is supported (Pattern 1 dynamic delivery, and Pattern 2's Pong demo literally ships app
code in the payload), and it is genuinely powerful: an agent mid-task can raise a bespoke
form rather than a paragraph of questions.

**It is also the highest-risk item in this analysis**, and should be gated:

1. **It is remote code execution by construction.** The MCP Apps security model assumes
   *pre-declared* templates the host can review before rendering. Dynamically synthesised
   HTML defeats that review step. Prefer **dynamic A2UI JSON** (constrained to a component
   catalog) over dynamic HTML — capability-based rather than arbitrary.
2. **ChatGPT's 5000-token limit applies to all tool schemas combined** (verified at assess).
   A dynamic app generator must be *few, terse tools*, not one tool per persona.
3. **Non-determinism.** A generated form is unreviewable and unversioned. For anything that
   records a commitment, use a static `a2ui://` resource so the artifact is stable and
   auditable.

**Recommendation:** static A2UI resources for decision capture; dynamic A2UI for
presentation of findings; **no** dynamically-generated raw HTML in v1.

## 6. Revised recommendation vs assess

| Assess said | Analysis says |
|---|---|
| Ideation product worth building in narrowed form | **Unchanged** — narrowing still holds |
| Persona debate club: not supported | **Unchanged** — a UI does not repeal the martingale proof |
| MCP Apps for Claude Desktop delivery | **Strengthened** — GA, cross-client, and A2UI-over-MCP removes the Rust friction |
| Refusal boundary for relationship/health | **Strengthened** — a polished UI raises perceived authority, so the boundary matters *more* |
| — | **New:** generated agent needs an MCP **server** surface; it is currently client-only |
| — | **New:** prefer `application/a2ui+json` over `ui://` HTML for anything we generate |

**Does this change Part 1 of the phase (creation gating)?** Only mildly and positively: if
the judge's findings can render as a structured A2UI panel, the "is this review theatre?"
question becomes visually answerable — `cross_model_check` and `checked_classes` as fields a
human actually reads, rather than JSON nobody opens. Worth noting for plan; **not** worth
expanding this phase's scope to include.

## 6a. Goals 4, 6 and 7 — implementation guidance (added after adversarial review)

The judge correctly BLOCKED an earlier draft: it answered the MCP/A2UI question thoroughly
but under-covered three of the phase's own goals, leaving a downstream plan without
guidance. Corrected here.

### Goals 1 & 2 — concrete wiring points

**Goal 1 — skill creator.** The insertion point is the **Reflect** phase controller,
`skills/process/pmpo-skill-creator/prompts/reflect.md`, after `validate-skill.sh` runs and
before the loop-decision table is evaluated. Sequence:

```
Execute → validate-skill.sh → /adversarial-review --mode artifact <dist/skill-name>
        → check-findings-sycophancy.sh → loop decision
```

A CRITICAL finding routes to `loop_execute` with the finding list as the fix list — the
creator's existing table already handles that branch.

**The review retry loop, stated explicitly** (the judge flagged an earlier draft as
inconsistent on this):

1. Creator loop converges → **one** judge call on the final artifact.
2. CRITICAL findings → fix → **re-review**. This is a *second* judge call.
3. Cap at **2 review rounds**, matching the existing 2-rejection cap in
   `check-findings-sycophancy.sh`. On a third would-be round, accept with an
   `## Unresolved review findings` section appended and the verdict recorded as-is.

**Corrected cost model:** **1–2 judge calls per creation**, not one. The saving versus a
naive wiring is that the creator's own 3-iteration Execute loop is *not* reviewed each
pass — only its output is. Worst case is 2 calls, not 3+.

**Goal 2 — agent creator.** The insertion point is the end of
`skills/process/native-agent/prompts/generate.md`, after `cargo check` and `npm install`
succeed and **before** the post-generation "ready" banner is emitted. Blocking semantics per
§7 #3: the workspace persists regardless; the *ready declaration* is withheld until findings
are written and no CRITICAL remains.

### Goal 3 — artifact-mode packet definitions

`build-review-packet.sh` currently accepts `--mode diff|artifact` only, and artifact mode
assumes a single markdown file. Two new packet builders are needed. Both are
**manifest-level, never full source** — a Cargo workspace exceeds any judge's context.

| Packet | Fields |
|---|---|
| `skill` | `SKILL.md` verbatim; parsed frontmatter (`name`, `description`, `license`, `version`, `metadata.tags`); script inventory (path, executable bit, shebang present); `references/` cross-reference map with resolve/dangle status; `validate-skill.sh` output; the original `intent` from `specify` |
| `agent` | `agent.toml`; `system_prompt.md` verbatim; workspace member list from `Cargo.toml`; per-crate name + one-line purpose; declared `[[mcp_servers]]`; the Specify answers; `cargo check` result |

Both additionally carry `producer_model`, the phase goals, and the mandate — matching the
existing artifact-mode packet shape so `dispatch-judge.sh` needs no change.

**Size guard:** cap each packet and record the cap in the packet itself, so a truncated
review is never mistaken for a complete one.

### Goal 4 — enforce `KBD_PRODUCER_MODEL` at both creator entry points

**Current state.** `dispatch-judge.sh` honours the variable and warns `PRODUCER_UNKNOWN`
when absent (fixed 2026-07-30), and `build-review-packet.sh` now tries
`progress.json → KBD_PRODUCER_MODEL → ANTHROPIC_MODEL → CLAUDE_MODEL/CLAUDE_CODE_MODEL/
CLAUDECODE_MODEL` before giving up. **No harness in this environment sets any of them** —
verified: all five are unset. Neither creator exports one.

**Approach — require it; never synthesize it.** An earlier draft proposed
`export KBD_PRODUCER_MODEL="${KBD_PRODUCER_MODEL:-claude-opus-5}"`. The judge correctly
flagged this as **worse than the bug it fixes**: if the session is not actually running
`claude-opus-5`, the default manufactures a *false* `verified-distinct` — the gate reports a
verified cross-model review that never happened. That is the exact failure class this phase
exists to eliminate, reintroduced through the fix.

Correct behaviour:

```bash
set -a; . ~/.prometheus/kbd/secrets.env 2>/dev/null || true; set +a
# No default. An unknown producer must stay unknown and be RECORDED as unknown.
if [ -z "${KBD_PRODUCER_MODEL:-}" ]; then
  echo "[creator] PRODUCER_UNKNOWN — set KBD_PRODUCER_MODEL to the model running this" >&2
  echo "[creator]   session. The review will record cross_model_check as" >&2
  echo "[creator]   unverified-producer-unknown rather than claim a verified result." >&2
fi
export KBD_PRODUCER_MODEL   # exported even when empty; downstream records the truth
```

**Resolution (round 3): fail closed before dispatch.** Two earlier drafts were wrong in
opposite directions — one synthesized a default (fabricating identity), the other tolerated
an unknown producer (contradicting goal 4's "can never record"). The judge's third-round
fix resolves both: **the creator refuses to dispatch a review at all when
`KBD_PRODUCER_MODEL` is absent.**

```bash
if [ -z "${KBD_PRODUCER_MODEL:-}" ]; then
  echo "[creator] REFUSING to dispatch review: KBD_PRODUCER_MODEL is unset." >&2
  echo "[creator]   Set it to the model running this session. Without it the review" >&2
  echo "[creator]   cannot prove judge != producer, and a synthesized default would" >&2
  echo "[creator]   fabricate a verified-distinct result that never happened." >&2
  exit 2
fi
```

No default, no unknown-producer review. `unverified-producer-unknown` becomes unreachable
*because the review does not run*, which is what goal 4 asks for. The creator still emits
its artifact; only the *ready/validated* declaration is withheld.

**Acceptance:** every review dispatched by either creator records
`cross_model_check: verified-distinct`; a missing producer yields exit 2 and no findings
file, never a passing review.

### Goal 6 — promote the sycophancy pass to an enforced gate

**Current state.** `execute.md:229` and `reflect.md:118` both mandate a sycophancy pass
(S-01/S-03/S-07, `strictness: standard`). `validate-skill.sh` implements **none** of it —
it is model-honoured instruction, not a gate.

**Approach — enforce *from* `validate-skill.sh`, but do not reimplement inside it.** Goal 6
names `validate-skill.sh` as the enforcement point, and an earlier draft of this analysis
argued for wiring the creator directly instead — which contradicted the goal. The judge's
correction is better than either position: **`validate-skill.sh` invokes the existing
`check-findings-sycophancy.sh`** as an 8th check group. The gate is then enforced where the
goal requires, the 2-rejection cap and screen logic stay in one place, and nothing drifts.

Concretely, and stated single-source to remove the ambiguity the judge flagged twice:

- **`validate-skill.sh` is the enforced gate.** It gains an 8th check group that shells out
  to `check-findings-sycophancy.sh`, propagates its non-zero exit into the `FAIL` counter,
  and surfaces its feedback in the existing `=== RESULT ===` block.
- **Creators invoke `validate-skill.sh`.** They do **not** call
  `check-findings-sycophancy.sh` directly. One caller, one enforcement point, no drift.

Wiring: after the creator's Reflect writes findings, run the existing screen with a
`--counter-key` scoped to the skill being created. On rejection, feed the actionable
feedback back into Execute (the creator's loop already routes FAILs there). Cap at 2, then
accept with a logged warning — matching the reflector-gate behaviour documented in
`CLAUDE.md`.

**Acceptance:** a generated skill whose SKILL.md contains no edge cases or failure modes is
rejected by the screen, and the rejection is visible in the creator's output — not merely
noted in a prompt the model may ignore.

### Goal 7 — the flawed-input fixture (this is what makes goals 1–6 falsifiable)

Without this, every other goal can be "done" while the gate silently passes everything —
which is precisely the failure the whole phase exists to correct, and precisely what
happened to the eight historical reviews.

**Two fixtures, committed to the repo:**

| Fixture | Planted defect | Must be caught as |
|---|---|---|
| `fixtures/flawed-skill/` | a `SKILL.md` with a dangling `references/` link, a non-executable script, and no failure modes | CRITICAL from the judge **and** rejection by the sycophancy screen |
| `fixtures/flawed-agent/` | an `agent.toml` naming a provider with no key and a `system_prompt.md` contradicting the stated intent | CRITICAL from the judge |

**Control condition (the part that is usually skipped):** a matching *clean* fixture pair
that must PASS. A gate that fails everything is as useless as one that passes everything;
only the pair proves discrimination.

**Acceptance:** a CI-runnable script that generates from each fixture, asserts
`verdict == BLOCK` and `cross_model_check == verified-distinct` for the flawed pair and
`verdict == PASS` for the clean pair, and exits non-zero if either expectation inverts.

**Cost note.** Four judge calls per run. Run on demand and in the release gate, not on
every commit.

## 7. Answers to the assess open questions

| # | Question | Resolution |
|---|---|---|
| 1 | Packet shape for a generated skill | `SKILL.md` + frontmatter + script inventory + cross-reference map + original intent. Not the whole tree. |
| 2 | Packet shape for a generated agent | `agent.toml` + `system_prompt.md` + crate manifest + generation spec. **Not** source — a workspace exceeds any judge's context. |
| 3 | Blocking semantics | **Blocking on the readiness declaration.** Generation completes and the workspace persists; it is not *reported ready* until findings are written and no CRITICAL remains. Never deletes output. |
| 4 | Cost ceiling | Review the **final** artifact only, never each of up to 3 creator loop iterations. |
| 5 | sycophancy gate placement | **`validate-skill.sh` is the single enforced gate** and shells out to `check-findings-sycophancy.sh` (which already has the 2-rejection cap). Creators invoke `validate-skill.sh`; they do **not** call the helper directly. See §6a Goal 6. |
| 6 | Reuse `kbd-idea-critic`? | Generalise it out of KBD in the *next* phase, not this one. It already implements the 4-dimension rubric and separate-critic principle. |

## 8. Risks introduced by this direction

| Risk | Severity | Mitigation |
|---|---|---|
| Dynamic HTML generation becomes an RCE surface | **High** | A2UI JSON only; component catalog; no raw HTML in v1 |
| A polished UI increases automation bias | **High** | Capture user's position *before* revealing critique; record both |
| ChatGPT 5000-token schema ceiling | Medium | Few terse tools; do not add one tool per persona |
| A2UI extension not yet formalised in MCP | Medium | Google is "considering making an MCP extension to support A2UI" — treat as pre-standard; isolate behind our `ui-surface` tier contract |
| Scope creep into this phase | Medium | Everything in §4–6 belongs to `ideation-and-decision-tools`, not here |

## 9. Verdict

**The Rust/A2A/AG-UI/A2UI/MCP-Apps direction is worth developing, and the calculus improves
— for delivery, not for the debate premise.**

Three concrete consequences:

1. **Add an MCP server surface to `native-agent`.** It is the missing half of a capability
   we already ship, and it is what makes a generated agent addressable by Claude Desktop,
   ChatGPT, VS Code, and Goose rather than only by other agents.
2. **Prototype `application/a2ui+json` behind the `ui-surface` tier contract** — do not
   standardise on it yet. It is backend-agnostic, renders natively across hosts, and is
   capability-secured, but Google states they are only *"considering making an MCP
   extension to support A2UI"*. Adopting it as *the* wire format would contradict this
   analysis's own pre-standard risk finding. Isolate it behind the tier contract so a
   spec change costs one adapter, not a rewrite. Revisit when the extension formalises.
3. **Keep the ideation product narrow.** Better UI raises the stakes on the automation-bias
   finding rather than lowering them.

None of this changes the seven goals of *this* phase. It sharpens the next one.

## Recommended Next Phase (revised)

**`ideation-and-decision-tools`**, with two goals added from this analysis:

8. Add an MCP **server** surface to generated agents (`agent-mcp` currently client-only),
   exposing tools with `_meta.ui.resourceUri` and A2UI resources.
9. **Prototype** `application/a2ui+json` over `a2ui://` as a *candidate* generative-UI wire
   format behind the existing `ui-surface` tier contract, with **no dynamically generated
   raw HTML**. Ship a Tier-0/Tier-1 fallback that works if A2UI never formalises.

---

## Sources

- `blog.modelcontextprotocol.io/posts/2026-01-26-mcp-apps/` — MCP Apps GA, client support,
  `_meta.ui.resourceUri` / `ui://`, security model, `updateModelContext`
- `developers.googleblog.com/a2ui-and-mcp-apps/` — three integration patterns,
  `application/a2ui+json`, `a2ui://`, static vs dynamic delivery (2026-06-17)
- `github.com/modelcontextprotocol/ext-apps` — SDK packages (TypeScript)
- `community.openai.com/t/.../1371022` — ChatGPT 5000-token tool-schema limit
- `openreview.net/forum?id=iUjGNJzrF1` — Choi, Zhu & Li, NeurIPS 2025 spotlight (carried
  from assess; the martingale result this analysis declines to overturn)
- In-repo: `skills/process/native-agent/SKILL.md`, `substrate/surface-bridge/src/main.rs`,
  `skills/learn/ui-surface/`, `substrate/sovereign-sync/src/mcp_server.rs`

---

## Adversarial review record

Vetted across **five rounds** by a cross-model judge (`kbd-judge` / gpt-5.6-sol vs producer
`claude-opus-5`), `isolation_mode: rest-gateway:http://localhost:8181/v1`,
`cross_model_check: verified-distinct` in every round.

This is the most demanding review any artifact in this repo has received, and the judge
earned it — it caught four substantive defects that a same-model self-review would have
passed:

| Round | Verdict | What it caught | Resolution |
|---|---|---|---|
| 1 | BLOCK | Goals 4, 6, 7 had **zero** coverage — the analysis answered the user's MCP/A2UI question but under-served the phase it belongs to | §6a added |
| 2 | BLOCK | The proposed `KBD_PRODUCER_MODEL` default could **manufacture a false `verified-distinct`** — reintroducing the exact failure class this phase exists to eliminate | Default removed |
| 3 | BLOCK | Goal 6 names `validate-skill.sh`; an earlier draft argued for wiring the creator instead. Retry loop and cost model inconsistent | Fail-closed before dispatch; single-source gate; loop defined as 1–2 rounds |
| 4 | BLOCK | §7 table row 5 still carried the **superseded** position, contradicting §6a | Reconciled |
| 5 | BLOCK | Objects to the 2-rejection cap being non-blocking | **Not accepted — see below** |

### Unresolved review findings

Round 5's two CRITICALs are **declined**, with reasons:

1. **"The sycophancy gate is non-blocking after two rejections."** This is correct as a
   description and wrong as a defect. The 2-rejection soft cap is deliberate, documented
   architecture: `CLAUDE.md:977` — *"after two consecutive rejections the third attempt is
   accepted with a logged warning"* — and `kbd-assess/SKILL.md:75` mandates *"max 2 rounds,
   then accept with an 'Unresolved review findings' section appended."* An unbounded gate
   is an infinite loop. Changing it is a **pack-wide architectural decision**, not something
   this phase's analysis should smuggle in.

2. **"The CRITICAL review gate must be blocking."** It is — on the *readiness declaration*,
   per §7 #3. The disagreement is whether the cap's escape hatch makes it "non-blocking".
   Under the documented protocol, a capped gate that records unresolved findings **is** the
   blocking behaviour this repo uses everywhere else.

Both are ratification questions for plan, flagged rather than silently resolved.

Two WARNINGs are also carried forward: (a) A2UI-over-MCP implementation cost is not
estimated — plan should size it before committing; (b) verification criteria for "the judge
is actually a different model" could be stated more concretely than
`cross_model_check` alone.

**Process note.** Five rounds against a documented 2-round cap is itself a finding: the
judge will keep producing CRITICALs indefinitely if the artifact and the goals genuinely
disagree. Goals 4 and 6 are worded in ways that admit a reading the implementation cannot
satisfy literally. **Plan should ratify the goal wording before writing changes** — that is
cheaper than another five rounds.

---

## Ratification (change-arc-001, 2026-07-30)

The two round-5 CRITICALs recorded above as *declined* are now **resolved by ratification**
rather than left open. Both were symptoms of goal wording that admitted readings the
implementation cannot satisfy — which is why five adversarial rounds could not converge.

| Goal | Ratified reading | Status |
|---|---|---|
| **4** — "so `cross_model_check` can never record `unverified-producer-unknown`" | Satisfied by **failing closed**: unset `KBD_PRODUCER_MODEL` → creator refuses to dispatch (exit 2, no findings file). The value is never synthesized. | **RESOLVED** |
| **6** — "an enforced gate in `validate-skill.sh`" | `validate-skill.sh` is the single enforced gate and **shells out to** `check-findings-sycophancy.sh`. Creators invoke `validate-skill.sh` only. | **RESOLVED** |

Both readings are now recorded inline in `goals.md` with their rejected alternatives, so a
future reviewer sees the decision rather than re-deriving the ambiguity.

**What remains genuinely open** — the round-5 objection to the 2-rejection cap being
non-blocking. This is *not* resolved by ratification, because it is a real disagreement
about pack-wide architecture (`CLAUDE.md:977`), not about this phase's wording.
`change-arc-007` addresses it by making the cap value user-overridable with an audit trail,
so the decision to exceed it becomes the operator's rather than the agent's. That is a
scope extension, flagged as such, and not one of the seven phase goals.

**Process lesson, recorded for reflect.** The deadlock was diagnosable after round 3: when a
judge keeps producing CRITICALs that restate the same conflict in different words, the
artifact and the goals disagree — and no amount of artifact revision will fix a goal. The
cheap move is to stop and ratify the wording. This cost five judge rounds to learn.
