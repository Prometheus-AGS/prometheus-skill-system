## Why

The `sycophancy-correction` skill exists in the pack but is invoked **manually only**. The PMPO Reflect phase produces a reflection artifact that is supposed to honestly evaluate completed work — but the reflector subagent has full access to the generation-pass conversation history, biasing it toward agreement with conclusions already reached.

Assessment confirmed: the `reflector` SubagentStop hook invokes only `log-reflection.sh`, `state-checkpoint.sh`, and `workflow-dispatch.sh`. Zero invocations of `sycophancy-correction`. `forge-reflect-on-stop.sh` similarly has no sycophancy check.

This is the **highest-leverage fix in the entire pack**. The work is 1–2 days; the structural effect is that the Reflect phase becomes resistant to agreement bias without any change to the user's workflow. The fix has been validated manually in the session that produced the assessment: running sycophancy-correction at adversarial strictness surfaced a critical S-03 pattern that improved the session quality.

**Critical structural property**: the critic must NEVER see the generation-pass conversation history. Only the artifact. This is why the fix belongs at the SubagentStop boundary — it is the only place where the input can be controlled to artifact-only.

## What Changes

- Write `shared/scripts/sycophancy-check-reflection.sh`:
  1. Reads the reflection artifact path from the hook event (not the conversation history).
  2. Invokes `sycophancy-correction` at `strict` strictness (configurable via `PROMETHEUS_REFLECT_STRICTNESS=loose|standard|strict|adversarial`).
  3. Parses the result: if score below threshold OR any pattern in `[high, critical]` severity → reject with actionable feedback including suggested rewrites.
  4. Implements 2-rejection soft cap: after two consecutive rejections, the third allows through with a logged warning.
  5. Logs all decisions to `~/.prometheus/hooks.log` via the SP-006 shim.
  6. Returns exit code 0 (accept) or non-zero (reject with stderr message).
- Wire `sycophancy-check-reflection.sh` as an additional command in the `reflector` SubagentStop matcher in `hooks/hooks.json`.
- Add `.prometheus/reflect-rejections.txt` as the per-session consecutive-rejection counter (state file).
- Document the gate in skill-pack `CLAUDE.md` so users know it exists and how to configure strictness.

## Capabilities

### New Capabilities
- `reflector-sycophancy-gate`: Automatic sycophancy-correction check at the SubagentStop boundary for the reflector subagent, using artifact-only input and configurable strictness.
- `reflect-rejection-counter`: Per-session consecutive rejection tracking with 2-rejection soft cap.

### Modified Capabilities
- `reflector-subagent-stop`: Extended with sycophancy gate as an additional hook command.

## Impact

- `shared/scripts/sycophancy-check-reflection.sh` — new script
- `hooks/hooks.json` — add command to `SubagentStop` reflector matcher
- `.prometheus/reflect-rejections.txt` — new runtime state file (not committed)
- `prometheus-skill-pack/CLAUDE.md` — document the gate and `PROMETHEUS_REFLECT_STRICTNESS` env var
- Depends on SP-006 (`hook-log.sh` shim) being available — change-005 should land first
- No changes to the sycophancy-correction skill itself
