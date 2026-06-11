# Reflection — canonical-lifecycle

Gate: sycophancy-correction analyze_reflect_phase — score 0.0, S-08 not detected.

## Delta

1. The native-backend migration awk shipped with an unescaped `/` inside a bracket expression on the gate-pattern line (`[ xX/]`), which aborted awk on the legacy-change.md test. Only the lazy-migration test case exercised that path, so it was caught at test time, not authoring time — the same class of "only the happy path was mentally simulated" bug as Phase 1's jq conditional.
2. The hooks.sh enum edit listed in change-004's scope was a no-op: the hook system routes by kind:edge match and never validated against a closed enum, so analyze/spec fired without any code change. The scope frontmatter named a file that did not need editing.
3. kbd-analyze, kbd-spec, and pmpo-elicit are documentation skills (SKILL.md prose) with NO executable test coverage. Their research pipeline, zeespec gate, and elicitation option-classes are described, not verified. The native backend and position-sync (the code changes) have tests; the three new stage skills do not.
4. pmpo-elicit ships in inline-fallback mode only; the child-isolated research path it documents depends on the child-loop primitive from a later phase and does not exist yet.
5. zeespec and kbd-capability are named as consumers of pmpo-elicit but neither is wired to call it this phase; pmpo-elicit has exactly one real caller (kbd-analyze, itself untested).
6. The orchestrator SKILL.md grew to 620 lines, over the 500-line recommendation; the validator warns.

## Root Cause

1. awk bracket-expression escaping is non-obvious; authoring did not dry-run the migration branch. The test surfaced it.
2. Scope lists were written before checking hooks.sh's actual validation behavior; the "new kind needs enum registration" assumption came from how most systems work, not from reading the file first.
3. The three skills are inherently procedural (the "test" is whether a model following the prose produces good output) and the repo has no harness for asserting on prose-skill behavior. Deferring tests was a real coverage gap.
4. The child-loop primitive is sequenced into a later phase by the approved plan; pmpo-elicit was built now because kbd-analyze needs it, accepting documented degradation until isolation lands.
5. Wiring zeespec/kbd-capability would have pulled two other skill suites into scope; the plan deliberately deferred them for bounded phase size.
6. No line budget was tracked for SKILL.md during edits.

## Corrective Actions

1. Carry the Phase 1 house rule to awk: any regex over user-formatted text gets a fixture exercising the exact input shape in the same change. Applied here; make it standard.
2. Before listing a file in change scope, verify the edit is actually needed by reading the target's current behavior — especially "add to enum/registry" assumptions. Record verified no-ops rather than making vacuous edits.
3. Establish a prose-skill smoke pattern in a later phase: fixture check that each stage skill's documented artifacts exist and schema references resolve. Until then these skills are validated only by validate:strict (structure) — stated honestly.
4. When the child-loops phase lands, pmpo-elicit option-3 switches to the child-isolated path and the "inline-fallback (current)" note is removed in the same change — explicit task there.
5. zeespec interrogate.md edit + kbd-capability are explicit deliverables of later phases; the integration-contract is the handoff. Keep it authoritative.
6. Next phase touching orchestrator SKILL.md must extract a section to references/ to get back under 500 lines.

## Recommended Next Phase

safeguards — protect-tests.sh (close the false CLAUDE.md claim), scope guard in warn mode, sycophancy-gate generalization to all reflection/assessment artifacts, per approved plan Phase 3. The scope guard retroactively enforces the `scope:` declarations every change in Phases 1–2 has carried.
