---
id: SP-012
title: 4-layer pipeline enforcement hook
status: planned
priority: P1
estimated_effort: 2-3d
agent_role: hooks-engineer
depends_on: [SP-006]
unblocks: [SP-018, XC-004]
related: [SP-013]
created_from_conversation_turn: 3-4
---

# SP-012 — 4-layer pipeline enforcement hook

## Problem

The skill-pack documents a 4-layer pipeline: **ZeeSpec → PMPO → OpenSpec → forge-rs**. Each layer has a defined contract. The pipeline is documented but not enforced — a session can skip from "user asks for change" directly to "agent writes code," bypassing ZeeSpec, PMPO planning, and OpenSpec change records.

When this happens, the work lacks the artifacts the pipeline produces (specs, planning trace, change records). Reviewing later is harder, and KBD lifecycle violations accumulate silently.

## Evidence

1. Read the skill-pack and prometheus-knowledge documentation describing the pipeline.
2. Inspect a recent set of commits across the repos. For changes that should have been "broad" (per the broad-change threshold in `05-references/architectural-patterns.md`), check whether the OpenSpec change record exists, whether PMPO planning artifacts are on disk, etc.
3. Many will be missing. The pipeline is aspirational.

## Why it matters

Without enforcement, the pipeline is documentation theater. Teams adopt the pieces they like and skip the rest. The KBD lifecycle protocol in CLAUDE.md (the strict one in SSR's CLAUDE.md is good evidence of a high-discipline implementation) only enforces phases that have on-disk artifacts. The skill-pack version of the same discipline lacks that on-disk-artifact gate.

## Proposed fix

A `UserPromptSubmit` hook that classifies the incoming request and gates progression:

1. **Classify the request.** Cheap-LLM call (or heuristic) decides: trivial / narrow / broad change. Trivial = "explain this", "fix this typo". Narrow = single-file edit with no contract change. Broad = anything meeting the broad-change threshold.

2. **For broad changes only:** require pipeline artifacts to exist before allowing tool use. Specifically, before any `Edit`/`Write`/`MultiEdit` to source files, require:
   - A ZeeSpec entry referencing the work (path: `zee/specs/<id>.md`).
   - A PMPO plan with at least an `assessment.md` (path: `.kbd-orchestrator/phases/<phase>/assessment.md`).
   - An OpenSpec change record (path: `openspec/changes/<change-id>/proposal.md`).

3. **Enforcement style.** *Not* a hard block. A `PreToolUse` hook on Edit/Write/MultiEdit checks for the artifacts and emits a warning to stdout if missing, with a strong message: "Broad change detected without pipeline artifacts. Continuing will result in a warning in `~/.prometheus/hooks.log`. To clear this, run `prometheus pipeline init <change-id>` and re-run." The user can override; the warning is logged for audit.

4. **Hard block in `prod` environment.** When `PROMETHEUS_ENV=prod`, missing artifacts block the edit (PreToolUse returns blocking exit code).

## Trade-offs and risks

- **Risk: classifier is wrong.** A narrow change misclassified as broad is annoying but recoverable (override or run pipeline init). A broad change misclassified as narrow lets the work happen without artifacts. Mitigation: the classifier prompt is conservative — "if uncertain, classify as broad."
- **Risk: pipeline-init friction makes users disable the hook.** Mitigation: `prometheus pipeline init <change-id>` is one command that scaffolds all three artifacts. Friction must be low.
- **Cost: the classifier is a cheap-LLM call on every prompt.** Already similar to SP-002/004; share the same classifier output where possible.

## Acceptance criteria

- [ ] UserPromptSubmit classifies prompts as trivial/narrow/broad.
- [ ] PreToolUse on Edit/Write/MultiEdit checks for required artifacts when classification was "broad."
- [ ] Missing artifacts → warning in dev, hard block in prod.
- [ ] `prometheus pipeline init <change-id>` scaffolds all three artifacts in <2 seconds.
- [ ] All decisions logged via SP-006's hook log shim.
- [ ] Test: a synthetic broad-change prompt without artifacts is blocked in prod and warned in dev.

## Implementation steps

1. Implement the classifier in `shared/scripts/lib/classify-change.sh`.
2. Implement the artifact-check function in `shared/scripts/lib/check-pipeline-artifacts.sh`.
3. Add UserPromptSubmit hook entry that runs the classifier.
4. Add PreToolUse hook entry that gates Edit/Write/MultiEdit on artifact check.
5. Implement `prometheus pipeline init <change-id>` as a small script (or as part of the prometheus CLI).
6. Test end-to-end with synthetic prompts.

## Dependencies

SP-006 (hook log for audit) recommended. SP-013 is independent; landing both in the same period gives the strongest reflection-quality + pipeline-artifact pair.

## Open questions

- Should the classifier also surface as a slash command (`/classify`)? Useful for users to check their own prompts. Yes; tiny addition.
- What's the escape hatch when the user genuinely needs a broad change with no time to scaffold? `--no-pipeline` flag on the editing tool's invocation — but only available in dev. Logged loudly.
