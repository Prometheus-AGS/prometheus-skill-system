# Decision: unify the WIT worlds into `prometheus:component/*` before porting any skill

**Status:** accepted · 2026-07-31 · `change-idt-009-record-fabric-decisions`
**Phase:** ideation-and-decision-tools

## The decision

Define one WIT package family, **`prometheus:component/*`**, and settle it
**before a single skill is ported to WASM**. UAR's and KnowMe's existing worlds
become views onto it rather than independent contracts.

## Why the ordering is the whole decision

Porting first and unifying later means **every skill is ported twice** — once per
world — and "100% parity on mobile" becomes true by construction rather than by
measurement: each port satisfies whichever world it targeted. The cost of
unifying first is one design pass. The cost of unifying second is proportional
to the number of skills already ported, and it grows every week.

## The divergence is wider than first recorded

Verified on disk 2026-07-31:

| Package | Version | Location |
|---|---|---|
| `uar:skill@0.1.0` | 0.1.0 | `universal-agent-runtime/wit/uar-skill.wit:12` |
| `uar:plugin@0.1.0` | 0.1.0 | `universal-agent-runtime/wit/uar-plugin.wit:12` |
| `knowme:plugin@0.1.0` | 0.1.0 | `know-me-system/rust/crates/knowme_plugin_host/wit/knowme-plugin.wit:17` |
| `knowme:plugin@1.0.0` | **1.0.0** | `.../knowme_plugin_host/wit/v1/types.wit:14` |

Two corrections to the analyze-stage framing:

1. UAR's second world is **`uar:plugin@0.1.0`**, not `knowme:plugin@0.1.0`. The
   plan attributed KnowMe's package name to UAR.
2. `knowme:plugin` exists at **two versions simultaneously** (0.1.0 and 1.0.0),
   with additional unstable worlds in `wit/v1/worlds-unstable.wit` (`agent`,
   `provider`, `service`, `workflow`).

So the problem is not two worlds to reconcile. It is **four packages across two
repositories, one of them already versioned twice**. That strengthens the
ordering argument rather than weakening it: the divergence is compounding on its
own.

## Shape

```
prometheus:component@0.1.0
├── types      — shared value/error types
├── capabilities — host-granted capability handles
├── skill      — replaces uar:skill
└── plugin     — replaces uar:plugin and knowme:plugin
```

`skill` and `plugin` stay separate worlds. They have genuinely different
lifecycles: a skill is invoked and returns; a plugin registers and receives
events. Collapsing them would trade one migration for a worse abstraction.

## Alternatives considered

- **Keep both, write an adapter layer.** Rejected: an adapter must be maintained
  for every type in both worlds, forever, and the two worlds keep drifting —
  `knowme:plugin` is already at two versions.
- **Adopt `uar:skill` wholesale as the standard.** Rejected: it has no
  plugin-side event model, so KnowMe's plugin host would still need its own
  world, leaving the split in place under a different name.
- **Port first, unify later.** Rejected — this is the ordering the decision exists
  to prevent.

## Deferred, deliberately

Authoring the `prometheus:component/*` WIT files is **not** in this phase. It
belongs to `mobile-skill-portability`. This record fixes the decision and its
ordering constraint so that phase does not reopen it.

## What would change this

Evidence that the two worlds serve genuinely incompatible host models — such
that a shared `types`/`capabilities` base could not express both — would justify
keeping them separate and building the adapter instead.

## `knowme_sync` — evidence verifiable from this repository

The companion guide for this decision lives in another repository. A reviewer
scoped to *this* repo cannot open it, so the record carries the identifying
facts instead of an unverifiable claim that an edit happened.

```yaml
knowme_sync:
  external_path: docs/prometheus-skills-integration.md
  external_repo: know-me-system
  repo_sha: 28c0e10f854ef2b999884bb2a1b0cd06b592c30b
  repo_branch: feat/embedded-memory-crud
  guide_sha256: 8d98fc0d211583e4308e470e7d2e69a4d4a298dc20f9db7c17113a4da975de53
  guide_bytes: 12638
  recorded_at: 2026-07-31
```

Check it with:

```bash
cd <know-me-system>
git rev-parse HEAD                                   # must equal repo_sha
shasum -a 256 docs/prometheus-skills-integration.md  # must equal guide_sha256
```

A mismatch means the guide moved on and this record is stale — which is the
signal the block exists to produce. It does not mean the decision is wrong.

**No code was written to `know-me-system`, `flint-realtime-fabric`, or
`universal-agent-runtime` in this phase.** The guide at `repo_sha` was authored
earlier; nothing in `change-idt-009` modified any of those repositories.
