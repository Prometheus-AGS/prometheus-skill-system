# kde-001 — emit `mcpServers` in the Kimi Desktop plugin manifest

**Phase:** kimi-desktop-extensibility
**Scope:** `scripts/install-kimi-desktop-plugin.sh` (generator only)
**Backend:** native-kbd

## Problem

Kimi Desktop has the pack's 145 skills but **no tools**. The skills describe
workflows they cannot execute. `mcpServers` is the manifest field that closes
this, used by 4 of 12 vendor packages.

## Scope boundary — the generator, never a manifest file

The pack's `kimi.plugin.json` is **generated** at install time by the Python
heredoc inside `scripts/install-kimi-desktop-plugin.sh`. The only
`kimi.plugin.json` tracked in this repo belongs to `tools/liter-llm` and is
unrelated.

**Editing the installed manifest under `plugin-packages/` is forbidden** — it is
overwritten on the next install and invisible to git, the same constraint C-01
places on `.codex-plugin/plugin.json`.

## Evidence gathered before this spec

Adversarial review (judge k3) correctly warned that "HTTP 200/405 proves a
listener exists, not that the endpoint speaks MCP." That was verified, and the
result **changed this spec**:

| Server | Probe | Result |
|---|---|---|
| `prometheus-knowledge` | `POST /mcp` `initialize` | ✅ real MCP result: `{"serverInfo":{"name":"prometheus-knowledge","version":"1.7.0"}}` |
| `surreal-memory` | `POST /mcp/sse` `initialize` | ❌ empty — wrong verb/transport |
| `surreal-memory` | `GET /mcp/sse` | ⚠️ legacy SSE: emits `event: endpoint` → `/mcp/messages?sessionId=…` |
| `forge-rs` | `GET /mcp` | ⚠️ HTTP 401 — requires a credential |

So the three servers use **three different transports**, and only one is
confirmed compatible with the vendor `url` form.

## Vendor precedent, and where it runs out

All three vendor `url` servers (`cloudflare`, `github`, `supabase`) are **remote
HTTPS endpoints**, and their value objects carry only `url` (+ optional
`enabledTools`). There is:

- no `headers` / auth field in any installed example → **no way to pass a bearer
  token**, which is why `forge-rs` is excluded;
- no local-`http://` example → whether the daimon permits a loopback URL is
  **unverified**;
- no legacy-SSE example → whether the `url` form drives a two-channel SSE
  session is **unverified**.

## Acceptance criteria

The generated manifest is **strict JSON and cannot carry comments**. Every
"record why" below therefore means a comment in the GENERATOR
(`install-kimi-desktop-plugin.sh`), never in the emitted artifact.

Criteria are conditional on t1, which may legitimately end the change with no
manifest edit at all:

1. **If t1 shows the daimon accepts a loopback `http://` URL:**
   `install-kimi-desktop-plugin.sh` emits an `mcpServers` object containing
   `prometheus-knowledge` with `"url": "http://localhost:8942/mcp"`.
2. **If t1 shows loopback URLs are refused:** no `mcpServers` field is emitted,
   the negative result is recorded in the phase directory, and the change closes
   as a recorded finding. This is a SUCCESSFUL outcome, not a failure.
3. `surreal-memory` is emitted **only if** t2 proves the daimon drives its SSE
   transport; otherwise omitted, with the reason in a generator comment.
4. `forge-rs` is **not** emitted. A generator comment records the 401 and the
   absent auth-header field.
5. The generated manifest still validates against the existing structural check
   (`name`, `version`, `skills`, `interface` present; `skills == "./skills/"`)
   and parses as strict JSON.
6. Re-running the installer is idempotent and the package still reports 145
   skills.
7. No binary is copied into the plugin package (payload-bloat rule; the
   generation payload was already cut 188M → 98M for this reason).

## Ordering

`kde-001` and `kde-002` both edit the manifest dict in
`scripts/install-kimi-desktop-plugin.sh`. They MUST be applied sequentially,
kde-001 first, and each verified before the next starts. Applying them in
parallel would conflict in the same heredoc, and a merge there is silent —
the generator would still run and emit a manifest missing one field.

## Out of scope

- stdio shims (deferred in analyze; both stdio binaries are already on PATH)
- `npx`-based third-party servers (tavily, sequential-thinking) — not ours to
  redistribute
- any change to the servers themselves

## Open questions carried in

- **OQ-A:** does the daimon accept a `http://localhost` URL at all? If it
  enforces HTTPS or a remote host, this whole change is inert and must be
  recorded as such rather than shipped.
- **OQ-B:** hardcoded ports 23001/8942 have no reinstall-durability argument
  (goal 4). If a port is user-configurable, the generator must read it rather
  than embed it.
