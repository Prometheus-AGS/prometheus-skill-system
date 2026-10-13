## Why

Commit `8157450` removed the lease control API and, with it, the
`kbd-handoff` skill that called `prometheus kbd handoff`. Harness skill
directories installed before that commit (for example
`~/.kimi-code/skills/kbd-handoff`) still link to the deleted path, so
harnesses list a broken skill. Operators still move a phase between harnesses
mid-flight and now write the handoff note by hand
(`HANDOFF-KIMI.md` in the kubo fork is a real example).

## What Changes

- Restore `/kbd-handoff --to <harness>` with a lease-free design: a script
  that reads canonical KBD state and writes one handoff packet plus a JSON
  sidecar under `.kbd-orchestrator/phases/<phase>/harness-handoffs/`.
- Refuse (exit 1) on a half-closed boundary: an open task obligation, or a
  change or phase obligation the runtime marks complete or does not know,
  printing the `prometheus kbd guard evaluate ... --repair-projections`
  command for each. Ignore the active phase path and the active in-progress
  change; warn on carried work. The rule and exit codes match the mini pack's
  `kbd-handoff`.
- Warn on dirty or unpushed worktrees, broken installed `kbd-*` skills and
  missing native role definitions for the target harness.
- Add `references/schemas/harness-handoff.schema.json`, register the skill in
  the orchestrator command list, `workflows.yaml` and the generated slash
  commands, update `references/cross-tool-handoff.md`, and add the site page
  `kbd/harness-handoffs`.
- No lease, heartbeat or ownership command is reintroduced. The skill never
  writes `progress.json`, the waypoint, position files or task checkboxes.

## Capabilities

### New Capabilities

- `kbd-cross-harness-handoff`: lease-free packet that lets another harness
  continue the active KBD phase.

## Impact

Adds one skill script, one schema, one site page and a fixture test under
`shared/lib/tests/`. The critical skill-activation corpus is unchanged: the
new skill is advisory and read-only, unlike the lifecycle controls the corpus
covers. Installed links to the removed path become valid managed links on the
next `./install.sh` after this change reaches `main`.
