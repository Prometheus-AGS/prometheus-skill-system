---
title: Model Profiles
description: Why the rules file is tuned to the weakest model in the fleet, and what to measure before making it leaner.
---

# Model Profiles

`AGENTS.md` is per repository, not per model.

When Claude Opus 5, Kimi K3, MiniMax M3, and GPT-5.6 all read the same file, it
cannot be tuned to any one of them. Every model gets the same bytes, so the
weakest reader in the fleet governs the content.

That single fact decides the default, and it is the fact most easily lost when the
model writing the rules file is also the strongest model that will read it.

## The costs are asymmetric

| | Frontier model | Smaller model |
|---|---|---|
| Scaffold present | wasted tokens, some over-verification | none |
| Scaffold absent | none | fabricated APIs, elided code, skipped checks, silent scope drift |

Wasted tokens are recoverable. A fabricated identifier that reaches a commit is
not. So `mixed` ships the execution scaffold by default and `lean` is opt-in.

## The three profiles

| Profile | Region contains | Use when |
|---|---|---|
| `mixed` (default) | invariants + execution scaffold | more than one model family works this repo, or you are unsure |
| `lean` | invariants only | every model is a current frontier model **and** you measured that removing the scaffold did not lower pass rate |
| `strict` | same as `mixed` today | reserved, so a future stricter tier cannot silently change `mixed` |

Switching is a re-run. The profile lives inside one marker pair, so changing it
re-splices the region rather than adding a second one.

```bash
bash "$SK/scripts/bootstrap.sh" --path . --profile lean
```

Measured sizes: **847 words** lean, **1,396 words** mixed. `verify.sh` enforces a
900-word ceiling for lean and 1,500 for mixed, and fails a repo whose declared
profile does not match its contents.

## What the scaffold contains

Each rule targets a specific observed failure mode, not general good practice.

| Rule | Failure it prevents |
|---|---|
| Restate before executing | Work done against the wrong reading of an ambiguous task |
| Do not fabricate | Invented APIs, flags, and package names stated with full confidence |
| Verification is explicit | A summary reporting a test result nobody ran |
| No elision | `// rest unchanged` written into a real file, silently truncating it |
| One thing at a time | A batched pass that fails without indicating which change broke it |
| Stop conditions | Continuing past the goal into adjacent changes |
| Format contracts | Preamble prose emitted where a parser expects JSON |
| Self-check before completion | Unrequested code shipped alongside the requested change |

## The frontier guidance points the other way

Anthropic's guidance for its newest models says to remove explicit verification
instructions because they cause over-verification and waste tokens with no quality
gain. That guidance is correct **for those models** and is the entire reason
`lean` exists.

It does not generalize to a mixed fleet. Applying it there is how a rules file
ends up optimized for whichever model happened to write it.

## Measure before going lean

Do not adopt `lean` because a benchmark says a model is capable. Adopt it when
this repository's task set says so.

1. Fix ~10 representative tasks for this repo.
2. Run them under `mixed`, per model. Record pass rate and token cost.
3. Run them under `lean`, per model. Record the same.
4. Adopt `lean` only if **no** model regressed.

Pass rate is the gate. Token cost is the tiebreaker.

`verify.sh` **fails** a repo declaring `lean` without a measured entry in
`.prometheus/model-fleet.md`. Going lean is a measurement result, not a
preference, and the gate keeps it one.

```
FAIL  lean is measured    lean profile with no measured fleet entry
```

## Record the fleet

`.prometheus/model-fleet.md` is created with an empty table. Fill it in. A profile
choice nobody wrote down gets re-argued every time someone new arrives.

```markdown
| Model | Harness | Scaffold needed | Measured |
|---|---|---|---|
| Claude Opus 5 | Claude Code | unknown | no |
| Kimi K3 | liter-llm | unknown | no |
| MiniMax M3 | liter-llm | unknown | no |
| GPT-5.6 | Codex | unknown | no |
```

## Per-model starting assumptions

Starting points to be replaced by your own measurement, not established results.
Record what you observe in `.prometheus/decisions.md` and supersede this table.

| Family | Starting assumption |
|---|---|
| Claude Opus 5 / Fable 5 | Scaffold largely redundant; explicit verification prose measurably hurts. Best `lean` candidate |
| GPT-5.x | Strong instruction following; format contracts and no-fabrication rules still earn their place |
| Kimi K-series, MiniMax M-series | Keep the scaffold. Treat no-elision and explicit verification as load-bearing until measured otherwise |
| Local and quantized models | Keep the scaffold. Stop conditions and one-thing-at-a-time matter most |
| Older harnesses without hooks | Keep the scaffold. Nothing deterministic is enforcing anything, so prose is the only gate |

## Where per-model conditioning is possible

The file layer cannot branch. Two layers can.

**Runtime routing.** A runtime that selects the model — one routing through
liter-llm, for instance — knows the target before the prompt is built and can
attach the scaffold only when the target needs it. That is the correct place for
per-model conditioning, and it lets one repo serve `lean` bytes to a frontier
model and scaffolded bytes to a smaller one.

**SessionStart hooks.** In a harness supporting them, a hook can inspect the
active model and print the scaffold pointer conditionally.

Until one of those is wired, the file is the only layer available, and it must
serve the weakest reader.

## The uncomfortable part

`mixed` has a real cost: every frontier-model session pays for scaffolding it did
not need, and the over-verification that scaffolding induces is a documented,
measurable quality loss on those models.

If your fleet consolidates onto frontier models, `mixed` becomes the wrong default
and nothing here will notice. `verify.sh` checks that the profile is *consistent*
with the file's contents. It never checks that the profile is *correct* for your
fleet. Only your measurement does that.
