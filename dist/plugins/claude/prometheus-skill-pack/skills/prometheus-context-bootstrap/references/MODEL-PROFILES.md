# Model profiles

`AGENTS.md` is per repository, not per model. When several models read one
file, the file cannot be tuned to any of them individually — the weakest
reader in the fleet governs its content.

The costs are asymmetric, which decides the default:

| Choice | Cost on a frontier model | Cost on a smaller model |
|---|---|---|
| Scaffold present | wasted tokens; over-verification; some redundancy | none |
| Scaffold absent | none | fabricated APIs, elided code, skipped checks, silent scope drift |

Wasted tokens are recoverable. A fabricated identifier that reaches a commit is
not. So `mixed` is the default and `lean` is opt-in.

## Profiles

| Profile | Region contents | Use when |
|---|---|---|
| `mixed` (default) | invariants + execution scaffold | more than one model family works this repo, or you are unsure |
| `lean` | invariants only | every model reading this repo is a current frontier model, and you have measured that removing the scaffold did not lower pass rate |
| `strict` | same as `mixed` today | reserved; kept distinct so a future stricter tier does not silently change `mixed` |

Switch by re-running the bootstrap. The region is spliced, so a profile change
rewrites only the managed block.

```bash
bash scripts/bootstrap.sh --path . --profile lean
```

## What the scaffold actually buys

Each rule in `AGENTS.scaffold.md` targets a specific observed failure mode
rather than general good practice:

| Rule | Failure it prevents |
|---|---|
| Do not fabricate | Invented APIs, flags, and package names presented with full confidence |
| Explicit verification | A summary that reports a test result nobody ran |
| No elision | `// rest unchanged` written into a real file, silently truncating it |
| One thing at a time | A batched pass that fails without indicating which change broke it |
| Format contracts | Preamble prose emitted where a parser expects JSON |
| Restate before executing | Work done against the wrong reading of an ambiguous task |

## Frontier guidance points the other way

Anthropic's guidance for its newest models says to remove explicit verification
instructions, because they cause over-verification and waste tokens with no
quality gain. That guidance is correct **for those models** and is the reason
`lean` exists at all.

It does not generalize to a mixed fleet, and applying it there is how a rules
file gets optimized for the model that happened to write it.

## Measure before going lean

Do not adopt `lean` because a benchmark or a blog post says the model is
capable. Adopt it when this repo's task set says so.

1. Fix a task set of ~10 representative tasks for this repo.
2. Run it under `mixed`. Record pass rate and token cost per model in the fleet.
3. Run it under `lean`. Record the same.
4. Adopt `lean` only for repos where no model in the fleet regressed.

Pass rate is the gate. Token cost is the tiebreaker. A configuration that costs
less and passes less is a regression wearing an efficiency argument.

## Per-model notes

These are starting assumptions to be replaced by measurement from step 2 above,
not established results. Record what you actually observe in
`.prometheus/decisions.md` and supersede this table.

| Family | Starting assumption |
|---|---|
| Claude Opus 5 / Fable 5 | Scaffold is largely redundant. Explicit verification prose measurably hurts. Best `lean` candidate. |
| GPT-5.x | Strong instruction following; format contracts and no-fabrication rules still earn their place. |
| Kimi K-series, MiniMax M-series | Keep the scaffold. Treat no-elision and explicit verification as load-bearing until measured otherwise. |
| Local and quantized models | Keep the scaffold. Expect stop conditions and one-thing-at-a-time to matter most. |
| Older harnesses without hooks | Keep the scaffold. Nothing deterministic is enforcing the rules, so the prose is the only gate. |

## Where per-model conditioning is actually possible

The file layer cannot branch. Two layers can:

- **Runtime routing.** A runtime that selects the model — for example one
  routing through liter-llm — knows the target before the prompt is built and
  can attach the scaffold only when the target needs it. That is the correct
  place for per-model conditioning.
- **SessionStart hooks.** In a harness that supports them, a hook can inspect
  the active model and print the scaffold pointer conditionally.

Until one of those is wired, the file is the only layer available, and it must
serve the weakest reader.
