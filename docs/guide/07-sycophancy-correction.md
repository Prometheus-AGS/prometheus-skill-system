# 07 · Sycophancy correction

Sycophancy screening looks for patterns such as ungrounded agreement, collapsed caveats and reflections that substitute praise for concrete deltas. It is a content diagnostic. It cannot prove that a reviewer used a distinct model, that a conclusion is correct or that a release meets its acceptance criteria.

## Pack integration

`shared/scripts/lib/sycophancy.sh` locates the installed `sycophancy-correction` binary, constructs a stdio MCP call to `detect_sycophancy` and extracts score and high/critical classifications. Its strictness mapping sends `loose` to `permissive` and `adversarial` to `strict`; other selected values pass through. Binary or interpreter absence produces an explicit degraded path in the caller.

The reflection and artifact hooks are declared in `shared/harnesses/hook-contract.json`. `sycophancy-check-reflection.sh` handles reflector output; `sycophancy-check-artifact.sh` handles already-written reflection/assessment files. A PostToolUse hook cannot undo a write. The artifact helper can emit actionable rejection and record a canonical blocker when runtime authority is available, or retain its legacy projection behavior otherwise. Neither is a shell mutation fence.

The default screening threshold and bounded rejection counter belong to these helpers. A soft-cap acceptance or missing-binary skip must be recorded as such; it does not manufacture independent QA. Complete all planned phase production before the applicable review and integration boundary.

## Local CLI

The pack CLI implements these diagnostic forms:

```bash
prometheus sycophancy detect reflection.md --strictness strict
prometheus sycophancy score reflection.md
prometheus sycophancy correct reflection.md --strictness standard
```

The `correct` command in `tools/prometheus-cli/crates/prometheus-cli/src/commands/sycophancy.rs` currently prints detection and manual correction guidance; full LLM correction needs a separately configured executor. Read the selected imported source and its actual MCP capabilities before relying on provider-backed rewriting. The imported component has its own gitlink and release identity; no live inference or installed acceptance is asserted by this documentation.

For independent model evidence, use [adversarial review](09a-adversarial-review.md) and [team model assignment](24-agent-teams.md). For hook packaging and trust, use [lifecycle hooks](15-hooks-and-lifecycle.md).

*Previous: [06 · Memory and learning](06-memory-and-learning.md) · Next: [08 · Skills overview](08-skills-overview.md)*
