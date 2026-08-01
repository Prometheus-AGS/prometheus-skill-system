---
type: Decision
id: decision-mobile-update-transport
title: "Decision: mobile skill updates ride sovereign-sync, not a signed HTTPS bundle"
tags:
- decision
- outcome-pending
outcome_status: pending
decided_at: 2026-08-01T13:22:32Z
links: []
sources: []
---

# Decision: mobile skill updates ride sovereign-sync, not a signed HTTPS bundle

## Decision

**Use the existing `sovereign-sync` P2P substrate.** Do not build a signed HTTPS
bundle fetcher.

## Assumptions

- **`skill-index` carries enough to update a device.** It syncs the index; it is
  **unverified** whether a device can reconstruct a usable pack from the synced
  entries alone, or whether it also needs skill *bodies*.
- **A mobile client can hold a bearer token.** Consistent with the REST design,
  not exercised from an actual device here.
- **CRDT merge is the right semantic for a skill catalogue.** Two devices
  editing the same skill merge rather than conflict — desirable for an index,
  and untested for a pack upgrade that removes skills.

## Falsifier

1. **Kills "sovereign-sync is sufficient."** A device with no paired desktop and
   no peer cannot update at all. Test: run a mobile host with the daemon
   unreachable and attempt an update. If it cannot, the HTTPS bundle stops being
   redundant and becomes the offline path — this decision then covers only the
   paired case.
2. **Kills "the index is enough."** Test: sync `skill-index` to a device with an
   empty pack and check whether the skills are *usable*, not merely *listed*. If
   bodies are missing, a content transport is still required.
3. **Kills "no new work is needed."** — **RUN 2026-08-01. IT FIRED.**

   ```console
   $ lsof -nP -iTCP:7892 -sTCP:LISTEN
   sovereign  127.0.0.1:7892          # loopback ONLY

   $ curl http://10.0.0.17:7892/health   # the host's own LAN address
   -> 000                                 # connection refused
   ```

   **A phone on the same network cannot reach the daemon today.** The substrate
   exists and the domain is declared, but the transport is not reachable from a
   second device.

   Consequence applied: the claim "no new work is needed" is **withdrawn**. What
   survives is narrower and still correct — *sovereign-sync is the right
   transport, and it needs a binding/exposure decision before a mobile host can
   use it.* That is a smaller, better-specified piece of work than building a
   parallel HTTPS bundle path, so the decision stands while its scope shrinks.

## Outcome

**Status: pending.** Nothing has been recorded yet.

A decision without a recorded outcome cannot be checked against what actually
happened — and idea rankings are known to flip after execution, so the judgement
made here is exactly the thing that needs checking later.

Record it with:

```
decision-log.sh outcome --id decision-mobile-update-transport --result -
```


## Round 2 — falsifier 2 fired, and it REVERSES the decision

The judge's second CRITICAL was that the decision rested on an explicitly
unverified assumption: that the synced index is enough to update a device. It
was right to press, and running the check settles it against me.

`SkillIndexAdapter::export_json` (`domains.rs:162-174`) exports exactly:

```rust
skills.insert(entry.name.clone(), json!({
    "description": entry.description,
    "keywords":    entry.keywords,
}));
```

**Metadata only. No skill bodies, no `SKILL.md`, no scripts.**

A device receiving this learns *that* a skill exists and roughly what it does.
It cannot **run** it. So `skill-index` is a discovery channel, not an update
transport — and the sentence "use the existing sovereign-sync P2P substrate"
claimed something the substrate does not currently do.

### The decision as originally written is WITHDRAWN

Not footnoted — withdrawn. Two of three falsifiers fired:

| Falsifier | Result |
|---|---|
| 3 — "no new work is needed" | **FIRED**: daemon binds `127.0.0.1` only; a LAN device gets connection refused |
| 2 — "the index is enough" | **FIRED**: index carries metadata only; a device cannot reconstruct a usable pack |
| 1 — "sovereign-sync is sufficient" | un-run; no mobile host to test against |

Two independent gaps, either of which alone blocks a mobile update.

### What actually survives

A narrower claim, and it is the honest one:

> **sovereign-sync is the right *direction*** — it already has a declared
> `skill-index` domain, a Public privacy class, a live daemon, bearer auth, and
> a REST surface. **It is not yet an update transport**, and making it one
> requires two things it does not have: content sync (bodies, not just an index)
> and reachability from a second device.

Against that, the HTTPS-bundle alternative is **no longer redundant** — it is
simply a different way to solve the same two missing pieces. Choosing between
them is a real decision that this change cannot make on the evidence available,
because the deciding factor (whether sovereign-sync gains content sync) belongs
to that component's roadmap.

### Consequence for the change

**`change-uhe-014` is BLOCKED, and R5 is PARTIAL — not MET.** Named
prerequisites, both measured rather than asserted:

1. `skill-index` syncs metadata only — a content-bearing domain is required.
2. The daemon binds loopback only — exposure is a security decision (bind
   address, TLS, token distribution) owned by sovereign-sync.

Recording this as BLOCKED is the correct outcome. Implementing "the chosen
transport" on a substrate that carries no content would have produced a mobile
update path that reports success and updates nothing — the same failure class as
a check that reports `up-to-date` while offline, which `change-uhe-013` exists to
prevent.

### Review record

One round, judge `kbd-judge` via `rest-gateway`, `cross_model_check:
verified-distinct`, producer `claude-opus-5`. **BLOCK** — 2 CRITICAL, 4 WARNING.

| # | Severity | Response |
|---|---|---|
| 1 | CRITICAL | **Accepted.** I kept the conclusion after falsifier 3 fired, narrowing scope instead of re-testing. Running falsifier 2 then reversed the decision outright. |
| 2 | CRITICAL | **Accepted — and it was the decisive one.** The "index is enough" assumption was unverified; verifying it killed the decision. |
| 3 | WARNING | **Accepted.** With both gaps open, HTTPS is no longer redundant; the comparison is reopened rather than closed. |
| 4 | WARNING | The prior entry with this id is this document, read back mid-write. |
| 5 | WARNING | **Accepted.** Token exposure on a LAN is part of prerequisite 2, not a separate afterthought. |
| 6 | WARNING | **Accepted, unresolved.** CRDT merge for a pack update that *removes* skills is untested; a merge that resurrects deleted skills would be a real defect. |

**Worth keeping:** the judge pressed on the one assumption I had labelled
unverified and moved on from. Labelling an assumption is not the same as testing
it — and here the untested one was load-bearing.
