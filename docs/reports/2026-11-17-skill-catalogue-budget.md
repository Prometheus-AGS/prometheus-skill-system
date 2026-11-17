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
- **Which measure the harness uses.** Whether nested skills count towards the listing was not verified.
- **Behaviour on a real machine.** The eval runs in a fresh home where no skill has ever been invoked. Claude Code keeps the descriptions of the skills used most, so an established user may see better triggering for their frequent skills and worse for the rest.
- **Other models and harnesses.**

One of my own runs was invalid and is excluded: the first trimmed copies deleted the top-level `kbd-status` skill, so its 0 of 3 results there measured nothing. The table above uses the corrected copies.

## Recommendations

In order of leverage:

1. **Make command-style skills user-invoked only.** The 28 `kbd-*` lifecycle skills are slash commands that operators call by name. Setting `disable-model-invocation: true` on those that should never auto-trigger removes them from the listing. This needs an owner decision per skill, since it also stops them preloading into subagents.
2. **Ship the domain plugins as the normal install, and the monolith as the exception.** The marketplace already defines them. A user who needs the learn domain gets a dozen skills, not 217.
3. **Remove the nested duplicates from the listing.** 97 skills exist both nested in a bundle and at the top level.
4. **Carry must-have defaults in hooks, not skills.** The VLBV session rule is the model for this: it does not depend on selection.
5. **Add a listing-size check to the release gate**, with a ceiling the owner chooses, so the catalogue cannot grow past it unnoticed. `skill-listing-size.mjs --json` provides the numbers.
6. **Document the user-side controls** (`skillListingBudgetFraction`, `skillOverrides`) in the install guide for people who want the full pack.

Rewriting descriptions helps loosely matched skills such as `kbd-status` and is worth doing, but it cannot fix a catalogue that is twice the budget.

## Consequences for certification

- Level C2 (trigger) must state the catalogue it was measured in. "Fires alone" and "fires in the shipped pack" are different claims, and only the second describes what users get.
- A skill that is meant to be invoked by name should be certified as such, with trigger cases that use the name, and not failed for refusing to auto-trigger.
- The certifier should report the plugin's listing size alongside every C2 result.

## Cost

$3.92 for the catalogue-size runs. One further single-run attempt to read a listing warning from the debug log produced nothing useful: the parent process's debug log does not include the session's.
