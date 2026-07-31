# Goals: mcp-2026-07-28-adoption

**Parent:** `uar-host-execution` (paused at `change-uhe-008`)
**Created:** 2026-07-31

## Why this is a child phase, not a detour

`change-uhe-008` needs to **run** tests across three persistence providers. It
cannot: the UAR crate does not compile.

```
error[E0599]: no associated function `from_bytes_stream` for SseStream<B>
error: could not compile `rmcp`
```

Confirmed pre-existing at `563ecc2` (before any of this session's work). The
root cause is a dependency-resolution defect, and investigating it surfaced
something larger than a version pin — see below. That is scope worth its own
assess/plan/reflect rather than a fix smuggled into an unrelated change.

## Goals

- **Assess whether to adopt MCP `2026-07-28` across the stack.**
- **Unblock the `rmcp`/`sse-stream` compile break** that halts every UAR test.
- **Decide per-consumer** — upgrade now, pin, or defer — each with a stated
  falsifier.
- **Return to the parent phase at `change-uhe-008`** either way.

## What research already established (2026-07-31)

Verified via firecrawl and the crates.io API, not assumed:

### The spec release is real and large

`2026-07-28` shipped on schedule. Per the official changelog it is **the most
substantial revision since authorization was added**, and it is **breaking**:

- **Protocol-level sessions removed**, along with the `Mcp-Session-Id` header
  (SEP-2567). List endpoints no longer vary per connection.
- **The `initialize`/`initialized` handshake is gone** (SEP-2575). Protocol
  version moves to a per-request `MCP-Protocol-Version` header.
- `ping`, `logging/setLevel`, `notifications/roots/list_changed` **removed**.
- All results gain a required `resultType` field; clients **MUST** treat results
  from earlier-protocol servers that omit it as `"complete"` (SEP-2322).
- Roots, Sampling, and Logging are **deprecated** (not removed) under a new
  12-month deprecation policy.

### The Rust SDK is ready, and it is our actual blocker

| Fact | Value |
|---|---|
| `rmcp` newest | **3.1.0**, published **2026-07-31** (today) |
| `rmcp` 3.0.0 | 2026-07-28 — same day as the spec |
| Our pin | **`=2.2.0`** (`Cargo.toml:235`) |
| Official SDK claim | *"implements the stable MCP 2026-07-28 specification while remaining fully compatible with the 2025-11-25 release and earlier"* |

**The dependency defect and the upgrade are the same problem.** `rmcp 2.2.0`
declares `sse-stream ^0.2` — too loose. `sse-stream 0.2.3` (2026-04-28) removed
`from_bytes_stream`, so any fresh resolution of our pinned `rmcp` picks a
version that cannot compile. **`rmcp 3.1.0` requires `sse-stream ^0.2.4`**,
which excludes the broken range.

So the choice is not "upgrade *or* fix the build". It is:

1. **Pin `sse-stream` to 0.2.2** — smallest change, keeps `rmcp 2.2.0`, leaves us
   on a protocol version whose successor is already shipping. Untested; it is
   the first thing to try.
2. **Upgrade to `rmcp 3.1.0`** — fixes the resolution properly and adopts the new
   spec, but it is a **breaking** SDK change across every MCP server we ship.

## Scope constraint

**Authorised repos: this pack and `universal-agent-runtime` only.** Not
`flint-realtime-fabric`, not `know-me-system`. If adoption implies changes
there, that is a finding to record, not work to do.

## The blast radius to assess

This stack ships **7 MCP servers** and configures MCP across **14 platforms**.
Assess must enumerate them rather than assume, and state which are affected by a
stateless protocol core.

## Return contract

On reflect, the parent resumes at **`/kbd-apply change-uhe-008-builtin-db-registration`**
with the parent waypoint restored (`phase: uar-host-execution`, 7/16).

**Whatever is decided, `change-uhe-008` still needs a compiling crate** — so a
"defer the upgrade" outcome must still deliver a working build, or the parent
phase stays blocked and that must be reported as such.

---

## Tooling defect found while creating this phase

`kbd-new-child.sh` **failed with `child_label: unbound variable`**. The
runtime-authority branch uses `${child_label}` at **line 156**, but the variable
is not assigned until **line 234**. Deterministic, not environmental.

The phase directory and waypoint were therefore created by hand. **Not patched
here** — it is an installed skill under `~/.claude/skills/`, where edits are
destroyed by the next install. Carried to reflection alongside the two
`kbd-reflect` / `kbd-next-phase` defects already recorded, which are the same
class of problem.
