---
type: Lesson
id: use-rebase-regenerate-sh-for-generated-only-rebase-conflicts
title: Use rebase-regenerate.sh for Generated-Only Rebase Conflicts
tags:
- vis:project
- kbd:reflect
- phase:phase-team-learning-hardening
- rebase-conflicts
- generated-files
- automation
- repo-workflow
- phase-reflection
sources:
- id: lesson
  resource: learning:944c292f0d68b2f5
generated:
  by: pk/1.11.0
  at: 2026-10-05T07:07:51.793593+00:00
created_at: 2026-10-05T07:07:51.793593+00:00
updated_at: 2026-10-05T07:07:51.793593+00:00
revision: 0
content_hash: 09edda069d582a7a89a61ae53aa0c6f32350ba7ca23b845d187b09fd5647ad94
---

## Lesson

`scripts/rebase-regenerate.sh` resolved a real generated-only conflict in PR/issue `#151` in one step. Prefer this script over manual regeneration when handling generated-only conflicts during rebases in this repository.[^lesson]

## Operational Guidance

- During rebases, identify conflicts that affect only generated artifacts.
- Run:

```sh
scripts/rebase-regenerate.sh
```

- Prefer the scripted regeneration path to hand-editing or ad hoc regeneration, because it captures the repository-specific conflict recovery flow in one repeatable command.

[^lesson]: phase learning-hardening reflection note