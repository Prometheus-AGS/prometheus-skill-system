# Phase-specific operator authorization

Date: 2026-09-24. Applies only to phase `agent-team-creator`.

The operator explicitly requested: “In this phase, you only alter the `prometheus-skill-pack` and `prometheus-skills-mini` directories” and “When done, commit and push those directories, ensure documentation in each is fully updated as well as the docusarus sites in each one.”

Subsequent operator instructions: “Follow the KBD process and do not skip steps” and “clear up all these conflicts and inconsistencies”.

Consequences:
- Both isolated repository worktrees are authorized write targets for this phase. Mini's default reference-only rule for the full pack continues to govern other tasks; it does not override this explicit two-repository request.
- Local commits and branch pushes are authorized after the applicable KBD and local verification gates. No merge, host installation, runtime service deployment or modification of other product repositories is implied.
- Correct observed conflicting KBD guidance and phase-state attribution as a prerequisite, using the same stage and review discipline.
- Existing draft spec commits do not authorize skipping assessment, analysis, spec review or plan review.
- The rule forbidding agents from pushing personal/team knowledge-log repositories remains in force. Pushing the two code repositories is a different operation, explicitly requested here.
- Historic statements that mini has no remote/no commits are stale baseline observations, not present facts or prohibitions. Current Git evidence is in assessment.md and sources-receipt.json.
- Do not erase run-wide evidence belonging to earlier work merely because a phase-local projection misattributes it.
