# Make universal-agent-runtime compile again

**Change:** `change-mcp-001-uar-build-unblock`
**Phase:** uar-host-execution / mcp-2026-07-28-adoption (child)

## Why

See `.kbd-orchestrator/phases/uar-host-execution/children/mcp-2026-07-28-adoption/plan.md`
for full rationale, acceptance criteria, and the adversarial review record.

## Outcome

### The floor is proven, not asserted

The acceptance criterion required cargo to **refuse** a downgrade. It does:

```console
$ cargo update -p sse-stream --precise 0.2.2
error: failed to select a version for the requirement `sse-stream = "^0.2.4"`
candidate versions found which didn't match: 0.2.2
required by package `universal-agent-runtime v1.0.0`
                                                      # exit 101
```

Before this change that command **succeeded silently** and left the crate
uncompilable. That is the whole defect, now closed at the constraint level.

**And it is a floor, not a pin** — verified in the other direction too:

```console
$ cargo update -p sse-stream --precise 0.2.5
    Updating sse-stream v0.2.4 -> v0.2.5        # allowed, as intended
```

A pin would have refused 0.2.5. Review flagged that the first draft of the plan
called this a pin; it is `^0.2.4` (`>=0.2.4, <0.3.0`), and 0.2.5 was built
against and compiles.

### Why a lockfile was not enough

`Cargo.lock` records a resolution; it does not constrain the next one.
`tools/surreal-memory-server` is the proof: it has a **committed** lockfile and
still fails `rm Cargo.lock && cargo check`. Committing only the lock here would
have left UAR one `cargo update` away from the same break.

### What was committed

| File | Change |
|---|---|
| `Cargo.toml` | `sse-stream = "0.2.4"` floor, with the rationale inline |
| `Cargo.lock` | resolution updated; **8 removals, all upgrades — no downgrades** (`jsonschema` 0.46→0.49, `liter-llm` 1.9.3→1.12.0, `sse-stream` 0.2.3→0.2.4) |
| `src/uar/mcp_server.rs:33` | `Content` → `ContentBlock as Content` |
| `src/uar/memory/mcp_server.rs:24` | same |

**Exactly 2 source lines changed.** `rmcp` still resolves to the pinned `2.2.0`.

### Verified from cold

`cargo clean -p universal-agent-runtime` removed **1,005 files / 4.2 GiB**, then
`cargo check --lib` finished clean — so the result is not an artifact of warm
build state.

### Stated limit

The `ContentBlock` alias is verified by compilation and the provenance tests.
**No MCP server was exercised end to end.** All six call sites are
`Content::text(...)` — the shape most likely to be a pure rename — but
"compiles" is not "behaves identically", and that gap is recorded rather than
glossed.
