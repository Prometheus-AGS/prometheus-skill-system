# Plan — adversarial-review-for-creation

**Stage:** plan · **Backend:** OpenSpec · **Date:** 2026-07-30
**Inputs:** `assessment.md`, `analysis.md` (5 review rounds, 2 findings unresolved)

## Ordering rationale

**change-arc-001 comes first and is not optional.** Analyze deadlocked across five
adversarial rounds on exactly one thing: goals 4 and 6 are worded so they admit readings the
implementation cannot satisfy literally. Writing code before ratifying that wording buys
another five rounds. It is a documentation change that unblocks everything else.

After that: shared plumbing (002–003) → wire the two creators (004–005) → prove it
(006) → the additions you asked for (007–009). The falsifiability fixture (006) lands
*before* the new features, so the new features inherit a working gate rather than a claimed
one.

## Changes

| # | Change | Goal | Depends on | Agent |
|---|---|---|---|---|
| 001 | `ratify-goal-wording` | 4, 6 | — | — (human decision) |
| 002 | `producer-model-fail-closed` | 4 | 001 | `rust-auditor` n/a — bash |
| 003 | `creation-review-packets` | 3 | — | `code-architect` |
| 004 | `wire-skill-creator-review` | 1, 5, 6 | 001, 002, 003 | `code-reviewer` |
| 005 | `wire-agent-creator-review` | 2, 5 | 001, 002, 003 | `code-reviewer` |
| 006 | `flawed-input-fixtures` | 7 | 004, 005 | `tdd-guide` |
| 007 | `rejection-cap-override` | — (extends 5) | 004, 006 | — |
| 008 | `cowork-skill-discovery` | — (new) | — | — |
| 009 | `openai-proxy-vendoring-decision` | — (new) | — | — (decision doc) |

---

### change-arc-001 · ratify-goal-wording

**Why first.** Five adversarial rounds could not converge because the artifact and the goals
genuinely disagree. This is a decision, not an implementation.

Two ratifications needed, both recommended positions already argued in `analysis.md`:

1. **Goal 4** — "so `cross_model_check` can never record `unverified-producer-unknown`"
   → ratify as: *the creator **fails closed** (exit 2) before dispatching a review when
   `KBD_PRODUCER_MODEL` is unset.* The value is never synthesized. `unverified-producer-unknown`
   becomes unreachable because the review does not run, not because the symptom is masked.
2. **Goal 6** — "an enforced gate in `validate-skill.sh`"
   → ratify as: *`validate-skill.sh` is the single enforced gate and **shells out to**
   `check-findings-sycophancy.sh`.* Creators invoke `validate-skill.sh`; they never call the
   helper directly.

**Acceptance:** `goals.md` amended in place with both clarifications; `analysis.md`'s
"Unresolved review findings" section updated to record the ratification.

---

### change-arc-002 · producer-model-fail-closed

Add a shared guard, sourced by both creators:

```bash
kbd_require_producer_model() {
  if [ -z "${KBD_PRODUCER_MODEL:-}" ]; then
    echo "[creator] REFUSING to dispatch review: KBD_PRODUCER_MODEL is unset." >&2
    echo "[creator]   Without it the review cannot prove judge != producer, and a" >&2
    echo "[creator]   synthesized default would fabricate a verified-distinct result." >&2
    return 2
  fi
}
```

Lives in `shared/scripts/lib/kbd-model-resolve.sh` beside the existing resolver.

**Acceptance:** unset → exit 2, no findings file written, refusal message on stderr. Set →
review proceeds and records `cross_model_check: verified-distinct`. Asserted concretely by
**006 Group B**, which runs both creators with the variable unset and then set.

---

### change-arc-003 · creation-review-packets

Extend `build-review-packet.sh` with two modes. **Manifest-level, never full source** — a
Cargo workspace exceeds any judge's context.

| Mode | Fields |
|---|---|
| `--mode skill` | `SKILL.md` verbatim; parsed frontmatter; script inventory (path, exec bit, shebang); `references/` cross-ref map with resolve/dangle status; `validate-skill.sh` output; original `intent` |
| `--mode agent` | `agent.toml`; `system_prompt.md`; workspace members; per-crate purpose; declared `[[mcp_servers]]`; Specify answers; `cargo check` result |

Both carry `producer_model`, phase goals, and mandate — matching the existing artifact packet
shape so `dispatch-judge.sh` needs no change.

**Size guard:** cap each packet and **record the cap inside the packet**, so a truncated
review is never mistaken for a complete one.

**Acceptance:** both modes emit schema-valid packets under the cap for a real generated
skill and a real generated agent.

---

### change-arc-004 · wire-skill-creator-review

Insertion point: `pmpo-skill-creator/prompts/reflect.md`, after `validate-skill.sh`, before
the loop-decision table.

```
Execute → validate-skill.sh (now incl. sycophancy check group)
        → /adversarial-review --mode skill <dist/skill-name>
        → loop decision
```

Also implements **goal 6**: `validate-skill.sh` gains an 8th check group shelling out to
`check-findings-sycophancy.sh`, propagating its exit into the `FAIL` counter and surfacing
feedback in the existing `=== RESULT ===` block.

**Review retry loop, explicit:** review once on the *final* artifact → CRITICAL → fix →
re-review → **cap at 2 rounds**, then accept with `## Unresolved review findings` appended.
**Cost: 1–2 judge calls per creation**, not one per Execute iteration.

**Acceptance:** a generated skill with a dangling reference is BLOCKed; a clean one PASSes.

---

### change-arc-005 · wire-agent-creator-review

Insertion point: end of `native-agent/prompts/generate.md`, after `cargo check` and
`npm install` succeed, **before** the ready banner.

**Blocking semantics (ratified in 001):** the workspace persists regardless; only the
*ready declaration* is withheld until findings are written and no CRITICAL remains. A failed
review never deletes generated output.

**Review retry loop — identical to 004, stated explicitly.** Goal 5 covers *both* creators;
an earlier draft specified the loop only for skills. Review once on the final workspace →
CRITICAL → fix → re-review → **cap at 2 rounds**, then withhold the ready banner and append
`## Unresolved review findings` to the generation report. **Cost: 1–2 judge calls.**

**Acceptance:**
- an agent whose `system_prompt.md` contradicts its stated intent is BLOCKed, the ready
  banner is withheld, and the workspace still exists on disk
- repeated CRITICALs stop at 2 rounds (asserted in 006 Group C)

---

### change-arc-006 · flawed-input-fixtures

**This is the change that makes 001–005 falsifiable.** Without it, every other change can be
"done" while the gate silently passes everything — precisely what happened to the eight
historical reviews.

| Fixture | Planted defect | Must produce |
|---|---|---|
| `fixtures/flawed-skill/` | dangling `references/` link, non-executable script, no failure modes | `BLOCK` + sycophancy rejection |
| `fixtures/flawed-agent/` | provider with no key; `system_prompt.md` contradicting intent | `BLOCK` |
| `fixtures/clean-skill/` | none | `PASS` |
| `fixtures/clean-agent/` | none | `PASS` |

**The clean pair is the control.** A gate that fails everything is as useless as one that
passes everything; only the pair proves discrimination.

**Acceptance — three assertion groups**, not one. An earlier draft claimed 002 and 007 were
"asserted in 006" without 006 actually testing them; the judge caught it.

*Group A — discrimination (4 judge calls):*
- flawed pair → `verdict == BLOCK` and `cross_model_check == verified-distinct`
- clean pair → `verdict == PASS`
- inversion of either → non-zero exit

*Group B — fail-closed, proving change-arc-002 (0 judge calls):*
- for **each** creator, run with `KBD_PRODUCER_MODEL` unset → assert **exit 2**, assert **no
  findings file written**, assert the refusal message reached stderr
- re-run with it set → assert the review dispatches and records `verified-distinct`

*Group C — cap enforcement, proving goal 5 (≤2 judge calls):*
- force repeated CRITICALs against a fixture → assert the loop stops at **2 rounds** for
  **both** creators
- assert `## Unresolved review findings` is appended on the capped run

Group C tests only the **existing hardcoded cap**, which is what goal 5 requires. The
override case belongs to 007 and is asserted there — an earlier draft put it here, which
would have tested a feature before it was built.

**Judge-call budget:** Group A = 4, Group B = 0 (fail-closed exits before dispatch),
Group C ≤ 2 → **≤ 6 total**. On demand and in the release gate, never per-commit.

---

> **Changes 007–009 are user-requested additions, not phase goals.** They are recorded here
> because they were asked for during plan and are cheap alongside this work. 007 *modifies*
> goal 5's cap rather than merely using it — that is a deliberate scope extension, flagged
> rather than smuggled. If the phase should stay strictly at its seven goals, defer all three
> to the next phase; nothing in 001–006 depends on them.

### change-arc-007 · rejection-cap-override *(your request — extends goal 5)*

**Which cap this changes — stated precisely.** There are two distinct caps and the judge
flagged an earlier draft for conflating them:

1. The **sycophancy-screen cap** in `check-findings-sycophancy.sh:48` — hardcoded `2`
   consecutive rejections of the *judge's report*.
2. The **review retry loop cap** in 004/005 — 2 rounds of review→fix→re-review of the
   *artifact*.

**007 changes only #1**, the sycophancy-screen cap, which is the one that is hardcoded and
not env-overridable (unlike `STRICTNESS`, which already reads
`${PROMETHEUS_REFLECT_STRICTNESS:-strict}`). The retry loop cap in #2 stays fixed at 2 per
goal 5 and is out of scope here.

**Your framing is right** — the current code decides *for* the user. But the cap exists to
prevent infinite loops, so the override must be deliberate, bounded, and visible:

```bash
CAP="${PROMETHEUS_ADV_REJECT_CAP:-2}"       # env override, same pattern as STRICTNESS
```

Three rules:

1. **Bounded.** Values above a hard ceiling (recommend 5) are rejected with an error, not
   silently honoured. An unbounded cap is the infinite loop the cap prevents.
2. **Recorded, not just applied.** When the cap is exceeded via override, the findings
   artifact records `cap_overridden: true` with the value used — so a review that ran to 5
   rounds is never mistaken for one that passed in 1.
3. **Prompt in interactive contexts only.** In a TTY, ask once when the cap is first hit,
   defaulting to *accept*. Non-interactive (CI, hooks) uses the env var or the default —
   never blocks waiting for input.

**Note the irony, deliberately:** this session hit that cap at round 2 and continued to
round 5 by my own judgement. This change makes that a *user* decision with an audit trail,
which is the correct owner.

**Acceptance (self-contained — does not rely on 006):** default unchanged at 2;
`PROMETHEUS_ADV_REJECT_CAP=4` permits 4 sycophancy-screen rejections; `=99` errors against
the hard ceiling; every override records `cap_overridden: true` with the value used. 007
ships its own test; 006 Group C deliberately does **not** cover it.

---

### change-arc-008 · cowork-skill-discovery *(your request)*

**Verified:** `cowork search <query>` already searches GitHub for skill repositories,
`cowork generate` builds skills from a repo or directory, and `cowork install` installs
globally or from a project. **Discovery already exists — this is wiring, not building.**

Two parts:

**(a) Ideation-time discovery.** During ideation, before proposing a build, query for
existing skills:

```bash
cowork search "<capability>" --limit 5
```

Surface results as a *build-vs-adopt* input, mirroring how `kbd-analyze` already consumes
`library-candidates.json`. This directly serves the "should we build this?" question the
ideation phase exists to answer.

> **Guardrail.** Installing a third-party skill from GitHub is executing someone else's
> code. `cowork audit` and `cowork verify` (checksums) already exist — discovery must route
> through them, and **never auto-install**. Propose; the user installs.

**(b) Documentation.** `cowork` has one passing mention in `docs/guide/08-skills-overview.md`
despite a full `skills/process/cowork-management` skill existing. Add a guide page covering
`init`/`list`/`status`/`doctor`/`audit`/`verify`/`install`/`generate`/`search`/`plugins`/
`pack`/`toolchain`/`disk`, with the security posture stated up front.

**Acceptance:** a documented discovery step usable from ideation; a `16a-cowork.md` guide
page; site builds with zero broken links.

---

### change-arc-009 · openai-proxy-vendoring-decision *(your request)*

**Current state:** `openai-proxy` is a *referenced sibling* at
`~/Projects/references/baseline/openai-proxy`, not a submodule. It is live on `:8181` and is
what `kbd-judge` resolves to today.

**Your reasoning is sound** — it is the thing that lets a Max plan drive a frontier judge, so
its absence silently degrades every review to `harness-native`. That is a real availability
risk for the gate this whole phase is building.

**This change is a decision document, not an implementation.** Three options, with the
recommendation stated:

| Option | Pro | Con |
|---|---|---|
| **A. Vendor as `tools/openai-proxy`** | judge availability guaranteed by clone; matches the 8 existing `tools/` submodules | one more submodule to keep buildable — and this session already lost 7 binaries to a submodule that would not resolve |
| **B. Keep sibling, add a doctor check** | zero coupling | absence is only detected at review time, when it degrades silently |
| **C. Vendor + make it optional** *(recommended)* | guaranteed availability, no hard build dependency | slightly more install logic |

**Recommendation: C.** Vendor it, but do not make the pack's build depend on it. Add a
`prometheus doctor` check that reports judge-gateway availability explicitly, so a missing
proxy is a *reported* condition rather than a silent downgrade to same-model review.

> **Evidence for caution.** The `liter-llm` submodule was pinned back this session because an
> advanced commit was unbuildable (`cargo metadata` exit 101) and aborted `install-binaries.sh`
> mid-run. Vendoring adds that failure mode. Option C's optionality is the mitigation.

**Acceptance:** a written decision with the option chosen and rationale; if C, the submodule
added and a doctor check reporting gateway availability.

---

## Deferred to `ideation-and-decision-tools`

Not in this phase, recorded so they are not lost:

- MCP **server** surface for generated agents (`agent-mcp` is client-only)
- `application/a2ui+json` prototype behind the `ui-surface` tier contract
- Persona panel, coach/reflector personas, decision records
- librefang `ExternalHookEvent` emission

## Risks

| Risk | Severity | Mitigation |
|---|---|---|
| 001 not ratified → planning deadlock recurs | **High** | 001 blocks 002/004; it is a decision, not code |
| Gate becomes theatre | **High** | 006's clean/flawed pair is the control |
| Cap override reintroduces infinite loops | Medium | hard ceiling + recorded in artifact |
| Third-party skill install is RCE | **High** | route through `cowork audit`/`verify`; never auto-install |
| Vendoring openai-proxy breaks the build | Medium | option C: optional, not a build dependency |

## First change to apply

```
/kbd-apply change-arc-001-ratify-goal-wording
```

---

## Adversarial review record

Vetted over **2 rounds** — the documented cap in `kbd-plan/SKILL.md:9` — by a cross-model
judge (`kbd-judge` / gpt-5.6-sol vs producer `claude-opus-5`),
`cross_model_check: verified-distinct` in both rounds.

| Round | Caught | Resolution |
|---|---|---|
| 1 | 006 claimed to assert 002's fail-closed behaviour but tested only verdicts | Group B added — runs both creators with `KBD_PRODUCER_MODEL` unset, asserts exit 2 and no findings file |
| 1 | 005 omitted the 2-round retry loop that 004 specified, though goal 5 covers both creators | Loop stated explicitly for agents; cost model matched |
| 1 | 005's dependency on 001 missing from the table | Added |
| 2 | **Ordering bug:** 006 tested 007's override, but 006 runs before 007 | Group C now tests only the existing hardcoded cap; the override case moved into 007's own acceptance |
| 2 | 007 conflated two distinct caps | Named both; 007 changes only the sycophancy-screen cap |

### Unresolved review findings

Two WARNINGs carried forward rather than resolved:

1. **Changes 007–009 are not tied to any phase goal.** Correct — they are user-requested
   additions made during plan, and the plan flags them as scope extensions rather than
   smuggling them in. 007 in particular *modifies* goal 5's cap rather than merely using it.
   **Decision for execute:** either accept the extension explicitly, or defer all three to
   `ideation-and-decision-tools`. Nothing in 001–006 depends on them.
2. **Judge-call budget wording.** Tightened in round 2 to ≤6, but the exact count depends on
   how many retry rounds the fixtures actually trigger. Treat ≤6 as a ceiling, not a
   measurement.

**Process note.** This vet stopped at the documented 2-round cap, unlike the analyze stage
which ran to 5. That is the intended behaviour, and change-arc-007 exists so the decision to
exceed it becomes the user's with an audit trail — rather than the agent's by judgement, as
happened during analyze.
