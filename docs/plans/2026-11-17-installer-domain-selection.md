# Installer Domain Selection — Design

**Date:** 2026-11-17
**Status:** implemented on `feat/installer-domain-selection`; see "As built" at the end
**Depends on:** PR #231 (complete domain plugins), which supplies the domain definitions
**Follows:** [Skill catalogue budget](../reports/2026-11-17-skill-catalogue-budget.md)

## Problem

The pack installer projects every skill in the generation into `~/.agents/skills`
and links each one into `~/.claude/skills` and `~/.qwen/skills`. The full listing is
about 72,700 characters, 1.8 times Claude Code's listing budget on a 1M-token model,
and skills stop triggering. Hooks only run from an installer-activated generation, so
anyone who wants hooks gets the full listing. Domain plugins do not help them.

Two further facts from reading `scripts/install-plugin-generation.js`:

- The installer never removes a projected skill. A skill dropped from a later
  generation stays in the target root until uninstall.
- `verifyTargets`, the receipts and the activation journal all assume that every
  skill in the generation is projected into every selected target.

## Design

### 1. The generation stays complete; selection filters the projection

The signed generation keeps every skill, the hook runtime, the shared scripts and
the skill index. Its identity does not change with a machine's selection, so one
generation serves any selection, rollback keeps working, and `/skill-name` prompt
files and by-path references can still reach a skill that is not projected.

Selection decides only which skills `installTargets` copies or links into the
target roots.

### 2. Domains are defined once, in the generation

At staging the installer writes `indexes/domains.json` into the generation, so it is
part of the signed file set:

```json
{ "schemaVersion": "prometheus-skill-domains-v1",
  "domains": [
    { "id": "process", "plugin": "prometheus-process-skills",
      "requires": ["iterative-evolver", "testing"], "skills": ["kbd-status", "..."] }
  ] }
```

Source: `skill-system.json` `marketplace.plugins` and each plugin manifest, which
`check:domain-plugins` already proves cover every native skill exactly once. Each
entry gains a short `domain` id and an optional `requires` list in
`skill-system.json`. The 21 imported skills (artifact-refiner and
sycophancy-correction) get domain entries too; staging fails if any skill in the
generation has no domain.

### 3. Requirements are declared, not derived

Measured on the PR #231 tree: 326 skill references stay inside a domain and 84 cross
one. The process domain names skills in five other domains, because the KBD
orchestrator routes to them. Taking the closure of the call graph would pull almost
everything back in, so it is not used.

Instead a domain lists the few domains it cannot work without. Proposed start:
`process` requires `iterative-evolver` (19 references) and `testing` (VLBV). Every
other reference is soft: the calling skill already has to cope with a skill that is
not installed. `--verify` reports soft references that point at an unselected
domain; it does not fail on them.

### 4. Command line and persistence

```
install-plugin-generation.js --domains process,rust     # replace the selection
install-plugin-generation.js --domains all              # everything (today's behaviour)
install-plugin-generation.js --add-domains learn
install-plugin-generation.js --remove-domains flutter
install-plugin-generation.js --list-domains             # id, skills, listing chars, selected
```

The selection is stored signed at `<pluginRoot>/selection.json` (requested domains,
resolved domains after `requires`, per target). An install with no domain flag
reuses the stored selection, so `cowork pack update` keeps it. With no stored
selection the installer behaves as today and installs everything: no existing
machine changes until its operator chooses.

Selection is per target, so `.claude/skills` can be narrow while `.agents/skills`
stays wide for Codex, whose catalogue is already curated by
`config/codex-catalog.txt`.

### 5. What changes in the installer

| Function | Change |
|---|---|
| `parseArgs` | the four flags above; mutually exclusive with maintenance operations as appropriate |
| `install` (staging) | build and write `indexes/domains.json`; fail on an unassigned skill |
| new `resolveSelection` | flags + stored selection + `requires` → skill list per target |
| `installTargets` | project only selected skills |
| new `removeUnselected` | remove pack-owned entries (managed link, or managed copy whose marker verifies) that are not selected; never touches anything else. Also fixes the skills-never-removed gap |
| `verifyTargets` | selected skills present and correct; no pack-owned unselected entry left |
| `targetReceipt` | schema 2 adds `selectionSha256` and `selectedSkillCount`; a schema 1 receipt is read as "all" |
| `activationRecord`, `completeActivation` | journal carries the resolved selection so an interrupted install recovers to the same set |
| `verifyActive`, `rollback` | read the stored selection; a rollback re-projects the old generation under the current selection, dropping domains it does not know |
| `uninstall` | unchanged (already removes every pack-owned entry) |

The generation manifest's `targetPayloads` matrix is untouched, so every existing
generation still verifies.

### 6. Callers

`update-skill-pack.sh`, `install-skills-flat.sh`, `install.js`, `install-system.js`,
`install-platforms.ts` and the `cowork` CLI pass the flags through.
`cowork doctor` and `check-model-config.sh` gain two reports:

- the listing size of the current selection against the budget;
- a warning when Claude Code has both a pack plugin enabled and user-level pack
  skills, since the same skill is then listed more than once.

`register-slash-commands.sh` keeps writing a prompt file for every skill.

### 7. Verification

One integration batch at the phase boundary, in scratch `HOME`, `CODEX_HOME` and
`PROMETHEUS_PLUGIN_ROOT`, through the real installer entry point:

1. install with no flag → every skill projected (unchanged behaviour);
2. `--domains rust` → only Rust skills in each target, hooks still dispatch;
3. `--add-domains process` → process plus its required domains appear;
4. `--remove-domains rust` → Rust entries gone, a foreign skill of the same name untouched;
5. update with no flag → selection preserved;
6. interrupted activation → recovery lands on the journaled selection;
7. rollback to a pre-selection generation → verifies under schema 1 receipts.

Then `claude plugin eval` on the Rust and KBD cases against a home that holds a
narrow selection, to confirm triggering returns.

## Decisions for the owner

1. **Default with no stored selection.** Recommended: install everything, as today,
   and let doctor report the listing size. Alternative: make a fresh interactive
   install ask.
2. **Hard requirements.** Recommended: the declared list in section 3. Alternative:
   none at all, every cross-domain reference soft.
3. **This machine.** Its target roots hold more pack-owned links than the active
   generation has skills (244 links in `~/.claude/skills`, 219 skills). I have not
   inspected them. `removeUnselected` would clear provable leftovers on the next
   install; nothing is touched until then.

## Not in scope

Shortening descriptions, consolidating the one-skill plugins, and Codex catalogue
curation, which already has its own mechanism.

## As built

Differences from the design above:

- **The domain index is a generated file, `shared/skill-domains.json`**, not
  `indexes/domains.json` written at staging. The installer usually runs from the
  packaged payload, where skills are flat and the domain plugin manifests no longer
  exist, so the map has to be built in the repository
  (`npm run generate:skill-domains`) and ship in the payload. It is still part of
  every generation's signed file set.
- **Domains are declared in a top-level `domains` list in `skill-system.json`**,
  each naming the marketplace plugins and inventory roots it groups. One-skill
  plugins are folded into a neighbour (for example `disk-space-guardian` into
  `devops`), giving 21 domains.
- **A target that takes every skill keeps its schema 1 receipt.** Only a filtered
  target gets a schema 2 receipt, so nothing changes for an install that never
  selects.
- **The selection is carried in the signed activation journal** and stored at
  `<pluginRoot>/selection.json` only once the activation completes.
- **`PROMETHEUS_SKILL_DOMAINS`** sets the selection for wrappers that do not know
  the flags. `update-skill-pack.sh` and `install-skills-flat.sh` accept
  `--domains=…`, `--add-domains=…` and `--remove-domains=…`.
- **Not built:** the `cowork` CLI flag (use the environment variable), the
  soft-reference report in `--verify`, and a doctor entry. The duplicate-listing
  warning is printed by the installer and returned by `--list-domains`.
