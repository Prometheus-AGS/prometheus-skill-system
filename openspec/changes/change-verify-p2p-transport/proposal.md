# Verify real P2P transport between paired machines

## Why

`sovereign-sync-domain-adapters` wired the skill-index/learner-model/
kbd-control domain adapters into real push/receive logic and proved the
pipeline end-to-end with two in-process `AppState`s (bypassing real
iroh/iroh-gossip networking, by design, to keep the test suite reliable).
Real network transport (`P2PNode::start()`/`broadcast()`/`recv()` between
two separate daemon processes on different machines) has never been
confirmed live — a sandboxed smoke test showed `start()` not completing
within several seconds, with no error logged, and it's unknown whether
that reproduces outside the sandbox.

## What Changes

No code change. Operator verification: with the Mac Pro and laptop already
paired (shared `operator_id`, bootstrap endpoint exchanged earlier this
session), start both daemons, push the `skill-index` domain from one via
`POST /api/v1/sync/push`, and confirm the other's `GET /api/v1/sync/status`
shows a non-empty `peers` list and its `SkillIndex` search reflects the
pushed content.

## Impact

- If it works: no further action, close this change with the observed
  evidence attached.
- If it doesn't: file a follow-up change to debug `P2PNode::start()` /
  `subscribe_and_join` specifically, informed by whatever the two real
  machines' logs show (which the sandboxed test couldn't produce, since it
  never got as far as a real network handshake).
