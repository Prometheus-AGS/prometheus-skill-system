---
type: Decision
id: decision-keep-both-wasm-targets-and-make-the-split-explicit
title: Decision: keep both wasm targets, and make the split explicit
tags:
- decision
- outcome-recorded
outcome_status: recorded
outcome_recorded_at: 2026-07-31T15:03:50Z
decided_at: 2026-07-31T14:56:13Z
links: []
sources: []
---

# Decision: keep both wasm targets, and make the split explicit

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

## Assumptions

- **LibreFang will not adopt the Component Model soon.** Unverified — we do not
  control that repo. If it does, "port" becomes correct and this decision
  should be revisited rather than defended.
- **`native-agent`'s consumers actually exercise the templates.** Partly
  unverified: the references are documented, but **no generated core-wasm guest
  exists in this repo** (`find skills -name '*.wasm'` returns only the
  Component-Model `entity-graph-optimize/skill.wasm`). If the path is
  documentation-only, "retire" gets stronger — see falsifier 2.
- **Two targets stay distinguishable to authors.** This is the assumption the
  decision's one added requirement exists to protect.

## Falsifier

Reverse if **any** of these is measured:

1. **LibreFang's host gains Component Model support.** Test: `grep -r
   "component::Component" librefang/crates/librefang-runtime/src/`. A non-empty
   result means one target can serve both hosts, and keeping two becomes
   gratuitous.
2. **No consumer actually generates a core-wasm guest.** Test: over the next
   phase, does anything invoke `librefang-wasm-skill`'s templates and produce a
   `.wasm`? If the answer is no — and today there is **no such artifact in this
   repo** — the templates are documentation for a path nobody walks, and
   **retire** becomes correct.
3. **An author ships a guest against the wrong host.** If someone builds from
   the librefang templates and tries to load it in UAR (or vice versa), the
   "make the split explicit" requirement failed and needs to be a hard gate — a
   check, not a paragraph.

Falsifier 2 is the one to watch: it is the cheapest to test and the most likely
to fire.

## Outcome

**Status: recorded** (2026-07-31T15:03:50Z)

Accepted after two adversarial rounds, both of which attacked the same soft spot: I proved the librefang pipeline was WIRED (forge package-librefang installed, 37-step SSRF-guarded uploader, native-agent target gating) but not that it was USED. Round 2 forced the usage check: 0 .lf-skill.zip artifacts anywhere, no BOSSFANG_TOKEN — but LibreFang IS running on localhost:4545, auth-protected, with 7 commits in the last 7 days. Decision stands on 'the host is actively developed' (verified), NOT on production usage (unproven, and now stated as unknown). Falsifier rewritten to something measurable: reverse to retire only if librefang has 0 commits in 90 days AND nothing listening. Follow-up carried to reflection: librefang-wasm-skill/SKILL.md must state it generates core-wasm guests that cannot load in UAR.
