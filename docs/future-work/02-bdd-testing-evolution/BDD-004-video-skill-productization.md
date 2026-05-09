---
id: BDD-004
title: BDD video skill productization
status: planned
priority: P1
estimated_effort: 3-5d
agent_role: skill-pack-maintainer
depends_on: [BDD-001, BDD-002]
unblocks: []
related: [BDD-005]
created_from_conversation_turn: 5-6
---

# BDD-004 — BDD video skill productization

## Problem

The video-evidence pipeline in SSR (`run-video-proof.ts`, `validate-video-coverage.ts`, `generate-video-run-report.ts`, `upload-videos-to-ipfs.ts`, `generate-bdd-docs.ts`, `record-and-publish-videos.sh`) is excellent but locked inside one project. It's worth ~3-5 days of work to lift the project-agnostic parts into a reusable skill.

## Evidence

The local skill at `ssr-frontend/skills/bdd-testing/SKILL.md` is a 50-line stub that doesn't capture any of the wealth in scripts/. The actual implementation is project-specific.

## Why it matters

Other projects (Brius, HotSeaters, etc.) will want the same pipeline. Without productization, each project re-builds it from scratch with subtle drift. Productization captures the pattern once.

## Proposed fix

A new pack-level skill `prometheus-skill-pack/skills/bdd-video-evidence/`:

```
skills/bdd-video-evidence/
├── SKILL.md
├── scripts/
│   ├── run-video-proof.ts             (parameterized, project-agnostic)
│   ├── validate-video-coverage.ts
│   ├── generate-video-run-report.ts
│   ├── upload-videos-to-ipfs.ts       (with pluggable IPFS provider)
│   ├── generate-bdd-docs.ts           (with templated layouts)
│   └── record-and-publish-videos.sh
├── templates/
│   ├── docs-layout.html               (shadcn-stylable)
│   ├── area-card.html
│   └── feature-page.html
├── config/
│   └── default.json                   (defaults: timeout, retry counts, etc.)
└── examples/
    └── ssr-frontend.md                (how SSR uses the skill)
```

The skill exposes a `bdd:video-record` slash command that reads a project-local `bdd-video.config.json` and runs the full pipeline. Project-specific bits (which scenarios are "guide", which auth bypass to use, which IPFS endpoint, brand colors for the docs site) are config, not code.

## Trade-offs and risks

- **Risk: extracting kills SSR's working pipeline.** Mitigation: SSR continues to use its scripts as-is until the skill is verified against a second project. The skill is *new*; SSR migrates to it later as a separate task.
- **Risk: the abstraction is wrong and doesn't generalize.** Mitigation: design the config schema by projecting forward to *two* concrete uses (SSR + one other project, even hypothetical). If the abstraction can't fit both without bending, the abstraction is wrong.
- **Cost: maintenance of two copies (SSR's + the skill's) until SSR migrates.** Bounded; flag SSR migration as a follow-up.

## Acceptance criteria

- [ ] New skill at `skills/bdd-video-evidence/` exists in the pack with full file set.
- [ ] `bdd:video-record` slash command works from a project that has a valid `bdd-video.config.json`.
- [ ] Config schema documented with example.
- [ ] At least one "external" project (hypothetical or sample) can adopt the skill without modifying script code.
- [ ] SSR's pipeline continues to work unchanged (no breaking changes).
- [ ] Skill includes its own `tests/` directory with smoke tests.
- [ ] BDD-001's clean key format is the skill's contract from day one.

## Implementation steps

1. Identify project-agnostic vs project-specific pieces in SSR's scripts.
2. Design the config schema.
3. Lift the agnostic pieces into the skill, parameterized by config.
4. Add the slash command.
5. Test in a sample project.
6. Document migration path for SSR (separate task; track).

## Dependencies

BDD-001 (clean manifest format) and BDD-002 (quarantine handling) so the skill embeds the right contracts.

## Open questions

- Should the skill ship with a "how to run this in CI" template (`.github/workflows/bdd-video.yml`)? Yes — most adopters will want a CI variant.
- Should the docs-site templates (HTML) live in this skill or in a separate `bdd-docs-site` skill? Together at first; split if either evolves much faster than the other.
