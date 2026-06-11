# Reflection — position-and-handoff-guarantee

Gate: sycophancy-correction analyze_reflect_phase — score 0.0, S-08 not detected.

## Delta

1. The first version of the renderer's `_wr_get` shipped with a broken jq conditional: when the camelCase key was absent the filter produced an empty stream instead of falling through to snake_case, so legacy waypoints rendered almost nothing. Caught by tests 3 and 5, fixed by replacing the conditional with `//` chaining.
2. change-006's delivered file set exceeded its declared scope by one file: the ratchet baseline (scripts/progress-signals-baseline.json) was not named in the plan or scope frontmatter and was added during implementation.
3. The per-turn position guarantee delivered is inject-every-turn plus one enforced Stop retry — not an absolute guarantee, and the hooks cannot take effect in the session that authored them because hooks.json is snapshotted at session start. The guarantee is therefore UNVERIFIED in live operation as of phase close.
4. 23 of 37 process skills do not declare progress signals; the CI gate shipped as a ratchet with all 23 baselined rather than fixed. The rule is now enforced only for new and newly-edited skills.
5. kbd_position_sync is invoked manually and documented in SKILL.md, but not wired into kbd-apply end-task or the waypoint write path — position.json will go stale during execution driven by other tools until that wiring lands.

## Root Cause

1. Untested assumption about jq semantics (`// empty` inside an if-condition propagates empty, skipping else); the bug survived authoring because only the happy camelCase path was mentally simulated. Fixture tests per key-generation were what caught it.
2. Planning named the lint script but did not think through that a ratchet requires a committed baseline artifact; scope lists were written before the enforcement design was final.
3. Claude Code's hook lifecycle (session-start snapshot; no mechanism to force prose) bounds what any implementation can guarantee; this was known and documented in the plan's risk section, but it still means phase verification is incomplete until a fresh session confirms injection.
4. The signal rule predates most process skills and was never machine-checked, so drift accumulated for months; fixing 23 skills in this phase would have ballooned scope across three other skill suites.
5. Deep wiring into kbd-apply was deliberately deferred because Phase 2 rewrites kbd-apply's backend dispatch (native-kbd adapter); touching it twice risks conflict.

## Corrective Actions

1. House rule for hook/renderer bash: any jq read over a possibly-absent key uses `//` chaining, never conditionals; every new key fallback gets a fixture test in the same change. Applied already in this phase's tests; carry into Phase 2 scripts.
2. When the scope guard ships (Phase 3), kbd-plan instructions must require listing companion artifacts (baselines, generated state, lockfiles) in scope at plan time; until then, record expansions in change.md as done here.
3. First action of the next session: confirm the position block is injected on prompt and the Stop gate fires on a footer-less reply; record the result in the next phase's assessment. Add a rollout note to the phase handoff.
4. Burn down the 23-entry baseline opportunistically in Phases 2–6: every phase that edits an evolver/zeespec/creator skill must also add its Progress Signals section and shrink the baseline in the same change.
5. Phase 2's native-backend change must add kbd_position_sync to kbd-apply end-task and the waypoint write helper — tracked as an explicit task in that phase's plan, not a doc note.

## Recommended Next Phase

canonical-lifecycle — kbd-analyze (tiered research pipeline), kbd-spec, native-kbd backend (tasks.json + nk_* adapters in kbd-apply, including the position-sync wiring from corrective action 5), and pmpo-elicit in inline mode, per approved framework-evolution plan Phase 2.
