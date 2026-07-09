---
license: MIT
name: bdd-testing
version: '2.0.0'
description: >
  DEPRECATED alias — use bdd-cucumber-js instead. This skill is retained
  as a redirect so downstream projects that reference "bdd-testing" by
  name continue to work.
metadata:
  author: prometheus-skill-pack
  category: testing
  tags: [testing, bdd, deprecated, redirect]
  supersededBy: bdd-cucumber-js
---

# bdd-testing (deprecated alias)

**This skill is superseded by [bdd-cucumber-js](../bdd-cucumber-js/SKILL.md).**

`bdd-testing` used to bundle cucumber-js + Playwright guidance with
Next.js SSR wording. It has been replaced by four project-agnostic skills:

| New skill | Purpose |
|-----------|---------|
| [bdd-cucumber-js](../bdd-cucumber-js/SKILL.md) | Author + run cucumber-js 13 + playwright-bdd + tsx |
| [bdd-cucumber-rs](../bdd-cucumber-rs/SKILL.md) | Author + run cucumber 0.23 + thirtyfour for Rust |
| [bdd-lifecycle-loop](../bdd-lifecycle-loop/SKILL.md) | create → run → triage → maintain workflow |
| [bdd-video-proof](../bdd-video-proof/SKILL.md) | Certification bundle format |

**If you invoked this skill:** switch to `bdd-cucumber-js`. All of the
guidance previously here (feature files, step definitions, CustomWorld,
video capture) now lives there in a project-agnostic form.

## Deprecation timeline

- **v2.0.0 (2026-07-09)** — redirect only, no substantive guidance in
  this file
- **v3.0.0 (planned)** — removal
