# A mobile-reachable update path

**Change:** `change-uhe-014-mobile-update-transport`
**Phase:** uar-host-execution
**Goal:** R5

## Why

See `.kbd-orchestrator/phases/uar-host-execution/plan.md` for full rationale,
acceptance criteria, and the two-round adversarial review record.

## Outcome: BLOCKED — R5 PARTIAL, both prerequisites measured

Tasks 1-2 (decide, record, review) are **done**. Tasks 3-4 (implement, verify)
are **BLOCKED**, and task 5 is the branch that applies.

### The decision reversed itself under testing

Chose sovereign-sync over an HTTPS bundle on strong-looking evidence:
`skill-index` is a declared `Public` domain, `SkillIndexAdapter` exists, the
daemon is live, REST is bearer-authed. **Two of three falsifiers then fired.**

**Falsifier 3 — the daemon is not reachable from a second device:**

```console
$ lsof -nP -iTCP:7892 -sTCP:LISTEN
sovereign  127.0.0.1:7892          # loopback ONLY
$ curl http://10.0.0.17:7892/health   # the host's own LAN address
-> 000                                 # connection refused
```

I narrowed the decision's scope and kept it. The cross-model judge (`BLOCK`,
`verified-distinct`) pressed on the assumption I had *labelled* unverified and
moved past — that the index is enough to update a device.

**Falsifier 2 — the index carries no content** (`domains.rs:162-174`):

```rust
skills.insert(entry.name.clone(), json!({
    "description": entry.description,
    "keywords":    entry.keywords,
}));
```

**Metadata only. No bodies, no `SKILL.md`, no scripts.** A device learns *that* a
skill exists and cannot run it. `skill-index` is a discovery channel, not an
update transport.

### Why BLOCKED is the right outcome, not a failure to deliver

Implementing "the chosen transport" on a substrate that carries no content would
have produced a mobile update path that **reports success and updates nothing** —
the same failure class as a check reporting `up-to-date` while offline, which
`change-uhe-013` exists to prevent. Shipping it would have been worse than
shipping nothing.

### Named prerequisites

1. **Content sync.** `skill-index` syncs metadata only; a content-bearing domain
   is required before any device can be updated from it.
2. **Reachability.** The daemon binds loopback only. Exposing it is a security
   decision — bind address, TLS, token distribution — owned by sovereign-sync,
   not by a skills change.

Both belong to sovereign-sync's roadmap. With both gaps open, the HTTPS-bundle
alternative is **no longer redundant**; the comparison is reopened rather than
closed, and the deciding factor is whether sovereign-sync gains content sync.

**R5 is PARTIAL: the update *check* ships (uhe-013, 5/5 tests); the mobile
*transport* does not.**

### The lesson

**Labelling an assumption is not testing it.** I wrote "unverified" next to the
claim that carried the whole decision, then reasoned as though it were settled.
The judge's value here was refusing to accept a label as evidence.
