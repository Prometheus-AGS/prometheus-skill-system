# Skill authoring: writing a description the model will actually act on

> Applies to every `SKILL.md` in this repo. Enforced (as a warning) by
> `npm run validate` and (as an error) by `npm run validate:strict`.

## Why the description is the whole game

A skill is a prompt template, not executable code. Nothing routes to it
algorithmically: the model picks a skill by **reading its `description`** and
nothing else. The body is not consulted at selection time — only after the skill
is already chosen. A perfect 500-line `SKILL.md` behind a vague description is
never read.

Two facts about this repo make that sharper than it sounds.

**1. Descriptions compete for a fixed budget.** Codex renders every
discoverable skill into one `## Skills` section with a fixed size budget. Names
and paths are mandatory; descriptions get what is left. Each additional skill
shortens the description of every other skill. From
[`config/codex-catalog.txt`](../config/codex-catalog.txt):

| Catalog entries | Avg description | Effect |
|---|---|---|
| ~130 | ~166 chars | full — the model triggers reliably |
| ~200 | ~66 chars | usable |
| ~360 | ~10 chars | **broken** — the model cannot tell skills apart |

This pack currently exposes **321 catalog entries**. We are near the bottom of
that curve, which is why trigger wording — not prose quality — is the thing that
matters.

**2. Similar skills actively compete.** `iterative-evolver`, `pmpo-outer-loop`,
and `pmpo-elicit` all describe "a loop that improves things". Without an
exclusion clause, every one of them is a plausible match for every prompt about
improving something, and the model picks close to arbitrarily.

## The shape

```
[What it does, one clause.] Use when [situation], [situation], or when the user
mentions "[keyword]", "[keyword]". Do NOT use for [the neighbouring skill's job].
```

Three required parts:

1. **What it does** — one clause, third person, concrete.
2. **`Use when` …** — the *situations and words that should fire it*. This is
   the part almost every description in this repo was missing.
3. **`Do NOT use for` …** — what to route elsewhere. Required whenever a
   neighbouring skill could plausibly claim the same prompt.

### Good

```yaml
description: >
  Tokio-native actor model for Rust services: actors as tokio tasks
  communicating over mpsc with a typed message enum. Use when a component must
  own state exclusively while serving concurrent callers, when replacing
  Arc<Mutex<T>> contention, or when the user mentions "actor", "mpsc",
  "message passing", or "shared mutable state". Do NOT use for
  request/response HTTP handlers (see axum-patterns) or for cross-process
  work (see mcp-server).
```

### Weak — and why

```yaml
description: >
  Isolated, cross-model adversarial review of KBD artifacts and change diffs.
  Dispatches a fresh-context LLM judge over an OpenAI-compatible REST gateway…
```

Accurate, and it tells the model **nothing about when to reach for it**. It
describes the implementation to a reader who already decided to use it. A user
who types *"check this plan for problems before I commit"* does not match a word
of it.

## Rules

- **Third person.** "Generates a report", not "You should generate".
- **Under 1024 characters.** Hard cap; the validator enforces it. Aim for
  **150–400** — remember every character is taken from a sibling skill.
- **Lead with the trigger, not the architecture.** Gateway names, crate names,
  and protocol versions belong in the body.
- **Quote the user's words, not yours.** Write the keywords a user would
  actually type. If they say "flaky test", the description must contain
  *flaky test*, not *nondeterministic assertion behaviour*.
- **Name the neighbour in the exclusion.** "Do NOT use for X (see `other-skill`)"
  is far more useful than a bare "Do NOT use for X".
- **No marketing.** "Comprehensive", "powerful", "enterprise-grade", and
  "production-ready" match no user prompt and consume budget.

## Deciding between `Use when` and the catalog

Not every skill should be in the auto-trigger catalog. A skill excluded in
`config/codex-catalog.txt` is **still reachable** as `/<skill-name>` — it just
stops taxing every other skill's description budget. Prefer exclusion for
skills that are always invoked deliberately by name (sub-skills, stage-* steps,
one-off migrations) over writing trigger words nobody will ever type.

## Checking your work

```bash
npm run validate                       # warns on missing triggers/exclusions
npm run validate:strict                # fails the build
bash scripts/codex-sync-skills.sh --report   # catalog entry count
```
