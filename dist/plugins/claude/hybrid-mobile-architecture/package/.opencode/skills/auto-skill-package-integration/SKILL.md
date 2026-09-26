---
name: auto-skill-package-integration
description: Auto-detect newly available skill packages from a watched git remote or a watched local directory and bring them through a confirmed install, instead of installing each one by hand. Use when an organization ships skills from a shared folder or feed, when a marketplace should be polled for new releases, or when onboarding a device to a team's skill set. Triggers on auto install, watch directory, watch git, skill feed, marketplace feed, plugin marketplace add, organization skills, team skills, detected skill, new skill available.
---
<!-- TJ-ARCH-MOB-001 compliant -->

# Auto Skill Package Integration

This is the low-friction path on top of manual install, upgrade, validate, and
remove. It removes the *discovery* step, never the *approval* step. Detection is
automatic; installation is confirmed.

Invoke the `connected-skill-packages` skill first — its 4-condition install
contract applies unchanged to anything detected here.

## Two source types

| Source | Detection | Suits |
|---|---|---|
| Watched git remote | Poll the remote on an interval; compare the resolved ref against the last seen commit | A team or vendor marketplace |
| Watched local directory | Watch for filesystem changes; treat a new `SKILL.md` as a candidate | A shared drive or a monorepo checkout |

Poll git on an interval; do not hold a persistent watch on a remote. For the
local case, debounce filesystem events — an editor writing a file emits several,
and an unbounded watcher will detect the same skill repeatedly mid-write.

## The detection cycle

1. **Observe** the source and compute what is new since the last recorded state.
2. **Parse** each candidate's frontmatter for its name and description. A
   candidate whose frontmatter does not parse is surfaced as malformed, not
   silently skipped.
3. **Verify** the containing package against the 4-condition install contract.
   A package that fails the contract is never presented as installable.
4. **Present** the candidate to the operator with its source, its parsed
   description, and its verification result.
5. **Install on confirmation** through the ordinary install path.
6. **Record** the outcome — including declines, so the same candidate is not
   presented again on the next cycle.

## Confirmation is not optional

Auto-installing code from a watched source is remote code execution triggered by
someone else's commit. Detection can be automatic; the install gate stays with
the operator. This holds even for a source the operator marked trusted — trust
narrows what is shown, never whether it is asked.

Present enough to decide on: where it came from, what it claims to do, and
whether it passed verification.

## State that must persist

Without durable state the cycle re-detects the same skills forever:

- Last seen commit per git source.
- Last seen file inventory per directory source.
- Per-candidate disposition: installed, declined, or failed verification.

Keep this in the same durable store as the package records, not in memory.

## Anti-patterns

- Installing without operator confirmation.
- Treating any new `SKILL.md` as installable without running the contract check.
- Watching a remote with a persistent connection instead of polling.
- Forgetting declines, so a rejected candidate reappears every cycle.
- Watching a directory the operator does not control.
- Installing outside the package install path.

## Verification

```bash
node scripts/verify-skill-manifest.mjs <detected-package-root>
```

A detected package that cannot pass this is a finding to report, not a package
to install.
