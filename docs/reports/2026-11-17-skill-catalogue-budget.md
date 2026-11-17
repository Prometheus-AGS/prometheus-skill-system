# Skill Catalogue Size and Triggering

**Date:** 2026-11-17
**Follows:** [Phase 2 pilot](2026-11-17-certification-phase2-pilot.md), finding 1
**Scope:** Claude Code 2.1.326, `claude-sonnet-5-5`, the generated Claude plugin. One harness, one model.

## Result

**The pack's skill listing is larger than Claude Code's listing budget, and triggering falls as the catalogue grows.** The pilot's hypothesis is supported, with one correction: there is no single cliff. How early a skill stops triggering depends on how closely the request matches its description.

- A skill with a closely matching request (`prometheus-rust-workspace`) triggers reliably until the catalogue passes roughly 22,000 to 42,000 characters, then stops completely.
- A skill with a loosely matching request (`kbd-status`, asked "where are we on this project?") degrades much earlier: 3 of 3 alone, 3 of 8 in a 29-skill catalogue, 0 of 3 at 69 skills.

The shipped pack is past both points.

## What Claude Code documents

From the Claude Code skills documentation:

- The listing budget is 1% of the model's context window.
- When the listing overflows, descriptions are dropped, starting with the skills invoked least.
- Each entry's description is capped at 1,536 characters.
- The budget can be raised with the `skillListingBudgetFraction` setting or the `SLASH_COMMAND_TOOL_CHAR_BUDGET` environment variable, and individual skills can be listed name-only through `skillOverrides`.

At about four characters per token, 1% is roughly 8,000 characters on a 200,000-token model and 40,000 on a 1,000,000-token model. The conversion is an assumption.

## The pack's listing size

Measured with `scripts/certification/skill-listing-size.mjs` on `dist/plugins/claude/prometheus-skill-pack`:

| Measure | Skills | Listing characters |
|---|---|---|
| Top-level skills | 217 (213 model-invocable) | about 72,700 |
| Nested skills (inside a bundle) | 97 | a further 29,100 if they are listed |

The top-level listing alone is about 1.8 times the 1M-token budget and 9 times the 200k-token budget. No description exceeds the 1,536-character cap, so the cap is not what is cutting them; the total is.

`evals/skill-activation/harness-budgets.json` records an inventory of 145 skills and 44,882 characters. The pack has grown by half since that file was written.

## Experiment

Trimmed copies of the generated plugin were built at several catalogue sizes, always keeping the skills under test. Each case ran three times with the plugin loaded (five for the diagnostic). The count is how often the skill under test was invoked.

### `prometheus-rust-workspace` (request names Rust, Cargo and tokio)

| Top-level skills | Listing chars | tokio-task-command-plan | phase-end-verification-command |
|---|---|---|---|
| 2 | 1,000 | 3/3 | 3/3 |
| 42 | 12,500 | 3/3 | 3/3 |
| 70 | 21,900 | 3/3 | 2/3 |
| 128 | 42,400 | 0/3 | 0/3 |
| 217 (full pack) | 72,700 | 0/3 | 0/3 |

The change falls between 21,900 and 42,400 characters. That bracket contains the 40,000-character budget for a 1M-token model. It is consistent with the documented overflow behaviour and does not prove it.

### `kbd-status` (request does not name the skill)

| Catalogue | Listing chars | mid-phase-status |
|---|---|---|
| The skill alone | under 1,000 | 3/3 (pilot) |
| 29 skills, all KBD | 9,600 | 3/8 |
| 69 skills | 21,100 | 0/3 |
| 217 (full pack) | 72,700 | 0/3 (pilot) |

In the 29-skill run, graders for "any skill invoked" and "any `kbd-*` skill invoked" passed on exactly the runs where `kbd-status` did. When it did not fire, nothing fired: the model read the state files and answered directly. No sibling skill took its place.

## What this does and does not establish

Established:

- Trigger rate falls as the catalogue grows, for both skills tested.
- The full pack is beyond the point where either skill triggers.
- A loosely matching request fails at a catalogue size well under any plausible budget, so budget overflow is not the only cause.

Not established:

- **Whether the 22k to 42k change is the documented budget or gradual dilution.** The direct test is to raise the budget and re-run the full pack. `claude plugin eval` passes only an allowlist of environment variables to the session and loads no user settings, so neither control could be set there.
- **Behaviour on a real machine.** The eval runs in a fresh home where no skill has ever been invoked. Claude Code keeps the descriptions of the skills used most, so an established user may see better triggering for their frequent skills and worse for the rest.
- **Other models and harnesses.**

One of my own runs was invalid and is excluded: the first trimmed copies deleted the top-level `kbd-status` skill, so its 0 of 3 results there measured nothing. The table above uses the corrected copies.

## Call graph and options

Added after the first version of this report, which recommended marking the
command-style `kbd-*` skills user-invoked only. That recommendation was wrong and
is withdrawn: `disable-model-invocation` stops Claude from loading a skill at
all, so no other skill could call it.

### Skills call each other

`scripts/certification/skill-call-graph.mjs` reads every skill's text and records
which other skills it names (as `name`, `/name` or "name skill"). It is static:
naming a skill is not proof of a hand-off, and bare-word mentions are not counted.

| Measure | Count |
|---|---|
| Skills, by name | 217 |
| References between skills | 445 |
| Named by at least one other skill | 139 |
| Named by no other skill | 78 |
| `kbd-*` skills named by another skill | 25 of 28 |

The most-called are `kbd-reflect` (15 callers), `kbd-assess`,
`sycophancy-correction`, `adversarial-review`, `kbd-new-phase`, `learn-goal` and
`learn-grade`. Only `kbd-doctor`, `kbd-loop` and `kbd-bottleneck-detector` are
named by no other skill. The existing activation corpus
(`evals/skill-activation/critical-30.json`) also expects `kbd-pause`,
`kbd-resume`, `kbd-cancel` and `kbd-audit` to start from a plain request.

### Nested skills are not listed

A trimmed pack that kept `kbd-status` only as a nested skill inside
`kbd-process-orchestrator` could not invoke it when asked by name (0 of 3). The
same request with a top-level copy present fired 3 of 3. So Claude Code registers
only the top-level skills of a plugin. The 97 nested copies cost nothing in the
listing, and the listing figure to use is the top-level one, about 72,700
characters. Removing the duplicates would not help on Claude Code. It may still
matter on Codex, which the repository documents as recursing into skill
directories.

### What each option saves

Against the top-level listing of 72,736 characters and an assumed 40,000-character
budget:

| Option | Listing after | Share of budget | Keeps every skill callable |
|---|---|---|---|
| Today | 72,736 | 1.82x | yes |
| Shorten to 120 chars the descriptions of skills named by 2 or more others (99 skills) | 56,333 | 1.41x | yes |
| Shorten to 120 chars the descriptions of skills named by any other skill (135) | 50,023 | 1.25x | yes |
| Cap every description at 200 chars | 49,643 | 1.24x | yes |
| Cap every description at 120 chars | 34,746 | 0.87x | yes, but no room for trigger and exclusion clauses |
| Names only | 9,386 | 0.23x | yes, by name only |

No description-only option gets the full pack under the budget while leaving
room for the "use when" and "do not use" clauses that the pilot showed matter.

### Domain plugins fit; the full pack does not

Listing size of each plugin the marketplace already defines:

| Plugin | Skills | Listing chars | Share of a 40k budget |
|---|---|---|---|
| `prometheus-skill-pack` (everything) | 213 | 72,736 | 1.82x |
| `prometheus-ui-ux-skills` | 37 | 12,172 | 0.30x |
| `prometheus-process-skills` | 19 | 7,742 | 0.19x |
| `learn` | 12 | 3,710 | 0.09x |
| `prometheus-testing-skills` | 8 | 3,436 | 0.09x |
| `prometheus-react-skills` | 6 | 2,606 | 0.07x |
| `prometheus-devops-skills` | 4 | 2,501 | 0.06x |
| Three single-skill plugins | 3 | 1,354 | 0.03x |

All of the domain plugins together come to about 33,500 characters for 89
skills, which fits. But they cover only 89 of the 213 skills. The Rust, Flutter,
research-stage and most process skills (including `kbd-status`) are in no domain
plugin, so today the only way to get them is the full pack.

## Recommendations

Every skill stays callable under all of these.

1. **Complete the domain plugins and make them the normal install.** This is the
   only measured option that fits the budget with full descriptions. It needs the
   124 skills that are in no domain plugin to be assigned to one, and the KBD
   lifecycle skills added to the process plugin.
2. **Shorten the descriptions of skills that are mainly called by other skills.**
   A caller names the skill, so those need one line. On its own this takes the
   full pack from 1.82x to about 1.25x: useful, not sufficient.
3. **Carry must-have defaults in hooks, not skills.** The VLBV session rule is the
   model: it does not depend on selection.
4. **Add a listing-size check to the release gate**, with a ceiling the owner
   chooses. `skill-listing-size.mjs --json` provides the numbers.
5. **Document the user-side controls** (`skillListingBudgetFraction`,
   `skillOverrides`) for people who install the full pack.
6. **Use `disable-model-invocation` only for a skill that nothing calls and that
   should never start on its own**, decided one skill at a time.

Rewriting descriptions helps loosely matched skills such as `kbd-status` and is
worth doing, but it cannot fix a catalogue that is nearly twice the budget.

## Consequences for certification

- Level C2 (trigger) must state the catalogue it was measured in. "Fires alone" and "fires in the shipped pack" are different claims, and only the second describes what users get.
- A skill that is meant to be invoked by name should be certified as such, with trigger cases that use the name, and not failed for refusing to auto-trigger.
- The certifier should report the plugin's listing size alongside every C2 result.

## Cost

$3.92 for the catalogue-size runs and about $0.40 for the nested-skill test. One further single-run attempt to read a listing warning from the debug log produced nothing useful: the parent process's debug log does not include the session's.
