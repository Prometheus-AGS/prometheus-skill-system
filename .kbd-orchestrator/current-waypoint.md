# Current Waypoint

> **Phase**: `phase-compliance-and-power-multiplier`
> **Stage**: executing (5/8 changes done — **4/4 P0 COMPLETE** ✨)
>
> **⚠️ Important correction noted on 2026-04-29**: WASM target is
> `wasm32-unknown-unknown`, NOT `wasm32-wasip2`. LibreFang's
> `WasmSandbox` uses `wasmtime::Module` + `Linker` (core wasmtime), not
> the Component Model. All downstream changes already updated.
> **Backend**: native KBD (no openspec/, no .evolver/)
> **Last updated**: 2026-04-28

## Where We Are

- ✅ **Assessment**: written to
  [`.kbd-orchestrator/phases/phase-compliance-and-power-multiplier/assessment.md`](phases/phase-compliance-and-power-multiplier/assessment.md)
  (437 lines, 27 gaps in 4 priority bands)
- ✅ **Plan**: written to
  [`.kbd-orchestrator/phases/phase-compliance-and-power-multiplier/plan.md`](phases/phase-compliance-and-power-multiplier/plan.md)
  (8 ordered changes)
- ✅ **Change proposals**: 8 files under
  [`.kbd-orchestrator/changes/`](changes/) — one per ordered change
- ✅ **Verification scheduled**: remote agent fires 2026-05-05T14:00Z
  ([routine](https://claude.ai/code/routines/trig_01MK1jtQZj3z1mQ7joETevuJ))
- ⏭ **Execute**: not started

## Next Action

All P0 changes complete. Remaining work is P1/P2 (parallel-safe):

```
/kbd-execute change-006-karpathy-loop-hooks     # P1 — close the Karpathy loop
/kbd-execute change-007-opencode-real-plugin    # P1 — real opencode Plugin function
/kbd-execute change-008-rustbpe-skill           # P2 — karpathy-tokenizer skill
```

Scheduled remote verification fires **2026-05-05T14:00Z** to check P0
against assessment §9: [routine](https://claude.ai/code/routines/trig_01MK1jtQZj3z1mQ7joETevuJ).

## Ordered Change List

| # | Change | Priority | Effort | Status |
|---|--------|----------|--------|--------|
| 1 | [change-001-compliance-quickfixes](changes/archive/2026-04-28-change-001-compliance-quickfixes/change.md) | P1 | XS | ✅ DONE |
| 2 | [change-002-toolchain-bootstrap](changes/archive/2026-04-28-change-002-toolchain-bootstrap/change.md) | **P0** | S | ✅ DONE |
| 3 | [change-003-librefang-wasm-skill](changes/archive/2026-04-29-change-003-librefang-wasm-skill/change.md) | **P0** | M | ✅ DONE |
| 4 | [change-004-native-agent-wasm-target](changes/archive/2026-04-29-change-004-native-agent-wasm-target/change.md) | **P0** | M | ✅ DONE |
| 5 | [change-005-package-and-upload](changes/archive/2026-04-29-change-005-package-and-upload/change.md) | **P0** | M | ✅ DONE |
| 6 | [change-006-karpathy-loop-hooks](changes/change-006-karpathy-loop-hooks/change.md) | P1 | S | proposed |
| 7 | [change-007-opencode-real-plugin](changes/change-007-opencode-real-plugin/change.md) | P1 | S | proposed |
| 8 | [change-008-rustbpe-skill](changes/change-008-rustbpe-skill/change.md) | P2 | S | proposed |

## Phase Exit Criteria

- All P0 gaps closed (verified by scheduled remote agent on 2026-05-05).
- `npm run validate` green.
- End-to-end smoke test produces a valid `<name>.lf-skill.zip` containing
  `skill.toml` with `runtime.type = "wasm"`.
- Remote agent's GitHub issue recommends "ready for `phase-librefang-wasm-onramp`".
