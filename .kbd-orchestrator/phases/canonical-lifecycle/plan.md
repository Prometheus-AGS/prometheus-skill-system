# Plan — canonical-lifecycle

Backend: native-kbd (this phase builds it, then dogfoods it for its own later
changes once change-001 lands). Ordered so the backend + wiring (carry-forward)
land first, then the new stages compose on top. Each change carries `scope:`.

| # | Change | Gaps | Summary |
|---|--------|------|---------|
| 1 | change-001-native-kbd-backend | G2 | tasks.json schema + `nk_*` adapter arms in kbd-apply.sh (detect/list/progress/mark_done/verify/archive); lazy migrate legacy change.md checkboxes; `backend_detect` adds native-kbd fallback; `references/native-backend.md` + spec-backend-interface.md table; tests. |
| 2 | change-002-position-sync-wiring | G4 | Wire `kbd_position_sync` into kbd-apply.sh `end-task` (after sync_progress) and document the waypoint write-path call; Phase 1 carry-forward CF-5. Test that position.json advances as tasks complete. |
| 3 | change-003-kbd-spec-stage | G3 | New `KBD/skills/kbd-spec/SKILL.md` — Spec stage: creates native changes (spec.md + tasks.json + verification.md) or dispatches `/opsx:new`; reads zeespec coverage when `.zeespec/` present (NO-GO blocks spec→plan gate). Stage gate + handoff. |
| 4 | change-004-kbd-analyze-stage | G1 | New `KBD/skills/kbd-analyze/SKILL.md` + `library-candidates.schema.json` + `references/research-pipeline.md` — tiered research (gh→Context7→registries→firecrawl), two modes (stack-specified / stack-discovery), writes analysis.md + library-candidates.json; kbd-plan reads candidates; hook events analyze:*/spec:* added; SKILL.md lifecycle/enum updates. |
| 5 | change-005-pmpo-elicit | G5 | New `skills/process/pmpo-elicit/` — ask-or-research primitive (4 option classes incl. "research it for me"), inline-fallback mode; elicitation.schema.json; integration-contract.md. zeespec wiring deferred to a later phase (interrogate.md edit), noted in handoff. |

Completion per change: change.md tasks checked, tests green, commit. Phase end:
`npm run validate:strict`, `npm run build`, `validate:signals`, full shell-test
sweep. New skills (kbd-spec, kbd-analyze, pmpo-elicit) must declare Progress
Signals (don't add to baseline) and pass strict validation.
