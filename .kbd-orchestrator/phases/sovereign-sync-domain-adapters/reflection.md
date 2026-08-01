# Reflection — sovereign-sync-domain-adapters

**Closed:** 2026-08-01 · **Implementation:** 2/3 changes complete, 1 BLOCKED

| Change | Verdict |
|---|---|
| `change-kbd-presence-peer-auth` | **COMPLETE** |
| `change-learner-model-e2e-test` | **COMPLETE** |
| `change-verify-p2p-transport` | **BLOCKED** — prerequisites measured below |

## Why the third change is BLOCKED, not skipped

Its own plan entry says it is *"manual/operator verification — start the daemon
on both the Mac Pro and the laptop already paired earlier this session, push a
domain from one, confirm it lands on the other."*

That needs **two physical machines**. Only one is available here, and two
further blockers were found by measurement rather than assumption:

**1. The daemon binds loopback only.**

```console
$ lsof -nP -iTCP:7892 -sTCP:LISTEN
sovereign  127.0.0.1:7892
```

No second machine can reach it — the same finding that blocked
`change-uhe-014`'s mobile transport, arrived at independently from the other
direction.

**2. The daemon is in a lock-error loop.**

```
Error: Database already open. Cannot acquire lock.   (repeating)
```

`/health` answers `ok` while the store cannot be opened. **A health endpoint
that reports healthy on a daemon that cannot serve its own database is the same
failure class as an update check reporting `up-to-date` while offline** — the
green signal is measuring the wrong thing.

**3. No bearer token exists locally.** `~/.prometheus/kbd/secrets.env` holds five
LLM keys and no sovereign credential, so even the local REST surface is
unreachable: `{"error":"missing or invalid bearer token"}`.

## Why BLOCKED is the honest verdict

The change exists to answer *"does gossip genuinely connect between two real
machines?"*. Every substitute available here — a second process, a loopback
peer, a mocked transport — would answer a **different** question and produce a
verdict that reads as evidence for the real one. Its plan entry anticipates
this: the result *"determines urgency of the other two"*, which only holds if the
result is real.

## Delta — the health check is lying

The substantive finding is not the missing hardware; it is that `/health`
returns `{"status":"ok"}` while stderr repeats `Database already open`.

**Root cause:** the health handler reports process liveness, not store
reachability. Nothing consulted the thing that is actually broken.

**Corrective action (carry-forward):** `/health` must fail when the store cannot
be opened. A liveness probe that cannot observe the daemon's core dependency
gives false assurance precisely when something is wrong.

## Carry-forwards

1. **Fix the lock loop.** I hypothesised two contending daemons; **measurement
   says otherwise** — there is exactly ONE (`pid 89055`, launchd-managed
   `ai.prometheus.sovereign-sync`). So the lock is held by a **stale lockfile
   from a previous crash**, not by a live competitor.

   That changes the fix: clearing a stale lock on startup, not resolving
   contention. Recorded because the wrong hypothesis would have sent the next
   person hunting a second process that does not exist.
2. **Make `/health` reflect store reachability**, not just process liveness.
3. **Decide the bind address.** Loopback-only blocks P2P verification *and*
   mobile transport. Exposing it is a security decision — bind address, TLS,
   token distribution — owned by sovereign-sync.
4. **Provision a sovereign bearer token** in `secrets.env`.

Items 2-4 gate `change-verify-p2p-transport` even once a second machine exists.
