# Local edits found in UAR's skill-system checkout (2026-07-31)

Before fast-forwarding the submodule 8ddac9a -> origin/main (359 commits),
these uncommitted edits existed in the checkout. They are ADDITIONS not
present upstream, so a bare checkout would have destroyed them.

Saved as: uar-submodule-local-edits.patch

```
 AGENTS.md | 13 +++++++++++++
 CLAUDE.md | 11 +++++++++++
 2 files changed, 24 insertions(+)
```

## What they say
A 'Phase-Gated Testing (MANDATORY)' policy: tests are a certification gate
run once at phase end, not continuously after each task.

## Disposition
Stashed, not discarded. The policy belongs in the PACK (where AGENTS.md and
CLAUDE.md are authored), not in a consumer's submodule checkout — an edit
there is invisible to the pack's git history and is destroyed by the next
submodule update. Same class of mistake as editing a plugin cache.
