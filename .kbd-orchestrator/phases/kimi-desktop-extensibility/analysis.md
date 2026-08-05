# Analysis — kimi-desktop-extensibility

_2026-08-05. Mode: **stack specified** (Kimi Desktop plugin API is the stack; no
alternative exists). Tiers used: local-artifact inspection + Tier 1 vendor-package
study. Tiers 2–4 not needed — see "Why the tiered pipeline stopped early"._

## Why the tiered pipeline stopped early

The assessment's open questions were **not** build-vs-adopt questions about
third-party libraries. They were questions about one vendor's undocumented local
runtime contract. Registry health (Tier 3) and stack comparison (Tier 4) have no
bearing on "does this daimon honour this manifest field."

The authoritative evidence was on disk: 12 vendor plugin packages that Moonshot
itself installed. Those are the specification-by-example. Budget used: well under
the 8-query/20-minute cap.

## Finding A1 — the MCP PATH prerequisite is RESOLVED, and it was the wrong worry

The assessment blocked `change-kde-001` on "is `~/.local/bin` on the daimon's
PATH?". That question is moot.

`kimi-cu/bin/kimi-cu-mcp` is a POSIX shell shim whose own header comment states
the rule verbatim (translated from the Chinese):

> kimi-code only allows a stdio plugin's `command` to be a PATH command or a
> `./` relative path, so this wrapper forwards to the system-installed KimiCU.app.

So the constraint is real and confirmed **by the vendor**, and the vendor also
ships the sanctioned workaround: a shim inside the plugin root that execs an
absolute path. PATH membership is therefore irrelevant — a plugin-relative shim
works regardless.

**But the better finding is that most of the pack does not need a shim at all.**

## Finding A2 — the pack's MCP servers are mostly HTTP, and HTTP is the vendor-preferred form

The assessment asserted "7 MCP servers" from session knowledge. The judge
correctly flagged this as unevidenced. Verified against `.mcp.json`:

| Server | Transport | Endpoint / command | Live check |
|---|---|---|---|
| surreal-memory | `url` | `http://localhost:23001/mcp/sse` | HTTP 200 |
| forge-rs | `url` | `http://localhost:8943/mcp` | HTTP 401 (listener up, auth enforced) |
| prometheus-knowledge | `url` | `http://localhost:8942/mcp` | HTTP 405 (listener up, wrong verb for GET) |
| liter-llm | stdio | `liter-llm` | on PATH |
| sycophancy-correction | stdio | `sycophancy-correction` | on PATH |
| tavily | stdio | `npx` | third-party |
| sequential-thinking | stdio | `npx` | third-party |

Three of the four vendor packages that declare `mcpServers` use the **`url`**
form (`cloudflare`, `github`, `supabase`); only `kimi-cu` uses stdio, and only
because it must drive a local macOS app.

**Adopt verdict: use the `url` form for the three local HTTP servers.** No shim,
no PATH dependency, no binary shipped inside the plugin package. This is both the
simplest path and the one with the most vendor precedent.

## Finding A3 — binary `--help` contradicts the assumed server list

Probing each binary for stdio/MCP support:

- **Speak MCP:** `pk-cherry`, `forge`, `sovereign-sync`, `prometheus-research`,
  `liter-llm`, `template-forge-mcp`
- **Do NOT advertise MCP:** `surreal-memory-server`, `surface-bridge`

`surreal-memory` reaches MCP over HTTP SSE via the server process, not via a
stdio subcommand — consistent with A2. `surface-bridge` is a UI bridge, not an
MCP server, and should never have been in the assessment's list.

This closes review WARNING #2: the "7 servers" figure conflated `.mcp.json`
entries with pack binaries. They are different sets.

## Build-vs-adopt decisions

| # | Decision | Verdict | Rationale |
|---|---|---|---|
| D1 | MCP transport for Kimi Desktop | **ADOPT `url` form** | 3/4 vendor packages use it; no shim, no PATH coupling, no binary in the package |
| D2 | stdio shims for `liter-llm` / `sycophancy-correction` | **DEFER** | Both are on PATH so they may work directly; a shim is the fallback if not. Not needed for first value. |
| D3 | `npx`-based third-party servers (tavily, sequential-thinking) | **REJECT** | Not the pack's to redistribute; the user can add them. Shipping them would make our package responsible for third-party auth. |
| D4 | Ship binaries inside the plugin package | **REJECT** | Would duplicate ~200MB of Rust binaries into app-managed state, and the payload-bloat lesson (188M→98M) applies directly. |
| D5 | `sessionStart` target | **ADOPT `kbd-status`** | Verified present among the 145 installed skills. |

## Open questions carried forward

1. **Do the three HTTP servers require auth headers Kimi cannot supply?**
   `forge-rs` returned 401. The vendor `url` schema shows no header field in the
   installed examples, so a bearer-token server may be unusable from Kimi
   Desktop. **This blocks forge-rs specifically, not the whole change.**
2. **`hooks` / `systemPrompt` remain unproven** — unchanged from assess. Still
   needs the throwaway probe (`change-kde-003`).
3. **Catalog/description budget at 145 skills** — unmeasured.

## Revised change order

1. `change-kde-001` — emit `mcpServers` (url form) for `surreal-memory` and
   `prometheus-knowledge`. **Unblocked** — A1/A2 resolved the prerequisite.
   `forge-rs` held back pending OQ-1.
2. `change-kde-002` — emit `sessionStart: {"skill": "kbd-status"}`.
3. `change-kde-003` — probe package for `hooks` + `systemPrompt`.
4. `change-kde-004` — conditional on 003.
5. `change-kde-005` — measure catalog budget.

Changes 1–2 are now evidence-backed and unblocked; the assessment listed 001 as
blocked, and that block is lifted.
