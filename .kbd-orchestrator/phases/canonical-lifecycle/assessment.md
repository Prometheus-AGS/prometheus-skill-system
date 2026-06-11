# Assessment — canonical-lifecycle

Phase 2 of the approved framework-evolution plan. Builds on Phase 1
(position-and-handoff-guarantee, complete). Source plan:
`~/.claude/plans/i-want-to-do-staged-pinwheel.md`.

## Carry-forwards consumed from Phase 1 reflection

- CF-3 (hooks unverified live): user chose "proceed now, verify later" — not
  blocking. First task of a future fresh session confirms injection.
- CF-5 (position-sync not wired into kbd-apply): **addressed this phase** as an
  explicit change, not a doc note.
- CF-2 (scope companions): scope guard ships Phase 3; this phase records any
  scope expansions in change.md as Phase 1 did.

## Ground truth (verified by reading kbd-apply.sh)

| Fact | Location | Implication |
|------|----------|-------------|
| Backend dispatch is a clean `b_*` case over `BACKEND` | `kbd-apply.sh:166-172` | native-kbd is a third case arm — no architecture change |
| `backend_detect()` checks openspec then speckit | `kbd-apply.sh:49-57` | add native-kbd as the final fallback (always available) |
| `end-task` recomputes progress from backend then `sync_progress` | `kbd-apply.sh:241-249` | the exact insertion point for `kbd_position_sync` (CF-5) |
| `_phase_dir` is already child-aware | `kbd-apply.sh:176-190` | child loops already sync correctly; reuse for native |
| Native `change.md` uses `[ ]/[/]/[x]` checkboxes inline | SKILL.md §309 (kbd-plan) | tasks.json becomes source of truth; lazy-migrate checkboxes |
| OpenSpec task ids are positional ordinals | `kbd-apply.sh:80-94` | native tasks.json uses explicit ids — no ordinal fragility |

## Gaps this phase closes

| ID | Gap | From plan |
|----|-----|-----------|
| G1 | KBD lifecycle names Analyze but ships no kbd-analyze skill; no engineering-landscape research stage. | Phase 2.1 |
| G2 | No PMPO-native spec backend — only openspec/speckit; native change.md has no kbd-apply adapter (dies "no backend detected"). | Phase 2.3 |
| G3 | No Spec stage skill; changes are created ad hoc inside planning. | Phase 2.2 |
| G4 | position.json goes stale during execution — kbd_position_sync is manual only. | Phase 2 (CF-5) |
| G5 | No ask-or-research elicitation primitive; zeespec silently marks unanswered questions implicit. | Phase 2.4 |

## Verdict

GO. G2/G4 are mechanical extensions of a clean dispatcher. G1/G3/G5 are new
skills composing existing infrastructure (research tools already in-session;
zeespec/evolver already exist). No rewrites.
