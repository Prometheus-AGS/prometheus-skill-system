# Reflection — mcp-2026-07-28-adoption

**Child of:** `uar-host-execution` · **Closed:** 2026-07-31 · **Changes:** 3/3

## Goal achievement

| Goal | Verdict | Evidence |
|---|---|---|
| Assess whether to adopt MCP `2026-07-28` | **MET** | Decided: defer; converge `rmcp` in a dedicated phase. 4 review rounds. |
| Unblock the `rmcp`/`sse-stream` compile break | **MET** | `cargo test --lib provenance` → **8 passed** from a cold build. |
| Decide per-consumer with a stated falsifier | **PARTIAL** | Stack-level decision with 3 falsifiers, one **run**. Not decided per-crate — deliberately deferred to convergence. |
| Return to the parent at `change-uhe-008` | **MET** | Return contract intact; parent at 7/16. |

**3 MET, 1 PARTIAL.** The PARTIAL is honest: the goal said "decide per-consumer",
and five crates did **not** each get a decision. Assess found no forcing function
(all five build; 12-month deprecation window), so the stack-level defer covers
them — but that is a narrower deliverable than the goal named.

## What this phase actually produced

The phase was opened on a premise that turned out to be wrong: that the build
break and the spec release were the same problem. They were **adjacent**. The
unblock needed two lines and no protocol adoption at all.

What it produced instead was **four corrected claims**, each caught by re-running
a command rather than re-reading a document:

1. **`from_bytes_stream` was ADDED in 0.2.4, not removed in 0.2.3.** My proposed
   "cheap fix" (pin 0.2.2) was exactly backwards and failed when tested.
2. **`surreal-memory-server` had the same latent defect UAR had** — found by
   deliberately deleting a lockfile (falsifier 2, which fired during assess).
3. **"Nothing uses MCP sessions" was false.** The grep was for a literal header
   no one writes by hand; `rmcp` writes it. Real surface: 4 crates, 3 server
   mounts, and a session id on every live handshake.
4. **20 wiki entries had never parsed.** An unquoted colon in `title:`; pk
   skipped them silently. Decisions recorded for durability were invisible.

## Delta: what I got wrong, and why it was invisible

**I dismissed finding 3 in round 1 by re-reading the file.** It looked fine. It
had never parsed. The check I ran (read the text) could not detect the failure
mode (the parser rejects it) — a category error, not carelessness.

**Root cause:** I verified with the wrong instrument. "It looks right" and "the
consumer accepts it" are different claims, and only the second one mattered.

**Corrective action, already applied:** the generator now emits quoted scalars,
all 20 entries were repaired, and `pk lint` is the check — errors 20 → 0.

## The adversarial review earned its cost

Four rounds against a 2-round cap, exceeded deliberately because **rounds 2 and 3
each produced a real fix**:

| Round | Caught | Outcome |
|---|---|---|
| 1 | Client handshakes conflated with server mounts | Verified disjoint (6 files, 0 overlap); reason withdrawn |
| 2 | Calling the claim "conditional" tested nothing | **Falsifier 1 run** — 4 live data points; did not fire |
| 3 | Record malformed in the store | **20 entries unparseable**; generator fixed |
| 4 | Compatibility still untested vs a real server | Answered from `rmcp 3.0.1` source; no new defect → **stop** |

Judge `kbd-judge` via `rest-gateway`, `cross_model_check: verified-distinct`,
producer `claude-opus-5` on every round. **A same-family judge sharing my
"I read it and it looked fine" assumption would have passed round 3.**

Round 4 producing evidence but no new defect is the signal that the cap should
bind again next time.

## Technical debt

- **Five `rmcp` versions across three majors** — 1.4.0 · 1.8.0 ×2 · 2.2.0 · 3.0.1.
- **Three hand-rolled handshake sites** two and three spec revisions behind.
- **`ContentBlock as Content` is compile-verified only** — no end-to-end MCP
  exercise; scoped explicitly as compile-only risk.
- **No deployed 2026-07-28 server was contacted.** First task of convergence.
- **`surreal-memory-server` still uses `rmcp::model::Content`** — the `rev` pin
  stops the drift; it does not port the crate.

## Carry-forwards for the parent's reflection

Tooling defects found here. None patched in installed skills or runtime data,
where edits die at the next install.

1. **`kbd-new-child.sh`** — `child_label` read at line 156, assigned at line 234.
   Deterministic; the child was created by hand.
2. **`kbd-apply begin-task` fails** with "failed to commit canonical task start"
   against the stale runtime store. `mark-done` works, so the loop survives.
3. **Stale runtime store** — `current-waypoint.json` carries
   `generatedBy: "kbd-runtime"`; that store holds a **Completed** run from
   2026-07-29 with an **expired lease**, reporting the wrong phase and `170/229`.
   `progress.json` per phase dir is the truth.
4. **`build-review-packet.sh --mode artifact` has no child-phase support** —
   `--phase <parent>` packaged the parent's assessment; round 1 judged the wrong
   document.
5. **`decision-log.sh record` keeps only `Decision` / `Assumptions` / `Falsifier`**
   and silently discards every other section — it dropped 55 of 118 lines,
   including all measurements. **Fixed in-repo** for the title bug; the
   section-dropping remains.

## Return contract

**The parent is unblocked** — verified, not asserted: `cargo test --lib
provenance` → **8 passed** from cold.

Resume at **`/kbd-apply change-uhe-008-builtin-db-registration`**
(`uar-host-execution`, 7/16).

## Recommended follow-on phase

**`rmcp-convergence`** — five crates, three majors, three handshake sites, and a
session surface across 4 crates. Its **first** task is standing up a real
2026-07-28 server, because that is the one claim this phase could not test.
