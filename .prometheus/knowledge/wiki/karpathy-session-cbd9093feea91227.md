---
type: SessionRecord
id: karpathy-session-cbd9093feea91227
title: Karpathy session cbd9093feea9
tags:
- karpathy
- session-learning
sources:
- session:444a77f7-71ee-4a60-8ce1-1b19da7d22ab
timestamp: 2026-08-07T20:42:52.983203+00:00
created_at: 2026-08-07T20:42:52.983203+00:00
updated_at: 2026-08-07T20:42:52.983203+00:00
revision: 0
---

## Delta

You're correct, and my earlier framing was too narrow. Here's the corrected picture, with what's actually built versus what isn't.

## Where I was wrong

I treated "no `server` feature ⇒ no routes ⇒ no reachable API" as if listening were the only way to be reachable. That conflates **binding a TCP port** with **being addressable**. A P2P transport gives you the second without the first — the mobile device dials out, establishes an authenticated session, and then serves requests over it.

WebRTC data channels and iroh (QUIC + public-key addressing) both do exactly that. Iroh is especially apt: node identity *is* the public key, so "a trust relationship with another device the user owns" is the native addressing model, not something layered on.

## What that changes architecturally

The mobile surface stops being "an embedded library only" and becomes **the same capability set behind a different transport**:

```
cloud/desktop:  HTTP  ─┐
mobile P2P:     iroh  ─┼─→  one capability layer  →  runtime
in-process:     direct ┘
```

That's a stronger argument for the shared trait I flagged earlier, not a weaker one. With three transports and hand-maintained parity, drift is near-certain — and KnowMe has already shipped three parity bugs with only two.

## What exists today

| | Status |
|---|---|
| UAR P2P dependencies | **none** — no iroh, libp2p, str0m, webrtc, quinn |
| Fabric transport | `str0m` (WebRTC), `iggy`, `loro` — **no iroh** |
| Fabric P2P CRDT path | **`live: no`** in its own implementation plan |

So the capability is architecturally available and unbuilt on both sides. The fabric's plan does treat "Sync-Mesh P2P" as the availability fallback, which shows the intent is real — but as of today the P2P row is explicitly not live.

I should also correct something I said earlier: I cited the fabric's `frf-store-redb` as its "only genuine on-device capability" and used the gateway requirement to argue UAR couldn't delegate. That reasoning still holds for *today's* fabric, but it isn't a permanent property — if the P2P path ships, the delegation calculus changes.

## What I'd want to pin down

Two questions where the answer changes the spec materially:

**Is P2P serving a UAR capability or a partner capability?** If UAR embeds iroh directly, it owns transport — which cuts against the clean layering we established (fabric owns transport, UAR owns agent execution). If the fabric owns it, UAR needs a transport-agnostic capability layer and the fabric supplies the pipe. The second preserves the boundary; the first is faster.

**Does this create a new trust boundary?** A device reachable from outside is a different security posture than one that only dials out. It would need the same JWKS/tenant verification as the HTTP surface (GAP-02, C-21) — otherwise P2P becomes a way around the auth path rather than a peer to it.

Want me to record this as a capability in the spec — provisionally, marked as architecturally-available-and-unbuilt — or investigate the transport-ownership question first?

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 444a77f7-71ee-4a60-8ce1-1b19da7d22ab
- Captured: 2026-08-07T20:42:41.208437Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
