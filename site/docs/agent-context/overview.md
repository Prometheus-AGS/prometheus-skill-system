---
title: Agent Context Architecture
description: Why agent rules moved from one large resident file to a small resident core with on-demand layers and deterministic enforcement.
---

# Agent Context Architecture

Every coding agent reads a file at session start. For three years the advice was
to put everything in it: conventions, commands, architecture, style, the rules
someone wrote after a bad afternoon. The file grew because growing it was the
only lever available.

That lever reversed. This section explains why, what replaced it, and how to move
a project across without losing the rules that were doing real work.

The mechanism is `prometheus-context-bootstrap`. It scaffolds the structure into a
new or existing project, migrates a Prometheus Base Rules v3 file into it, and
verifies the result.

## The problem, stated precisely

A rules file is loaded on every request and re-read after every compaction. It is
the only channel that reliably survives a long session. That makes it valuable and
makes it expensive, and those two facts pull in opposite directions.

Two things happen as it grows.

**Adherence falls.** Instruction-following degrades measurably as instruction
count and context length rise. Benchmarks that vary instruction density find
frontier models dropping well below full compliance at high counts, with a bias
toward instructions that appeared earlier. Multi-turn evaluations find
single-constraint accuracy declining as a conversation lengthens, and
multi-constraint accuracy declining considerably faster. A rule at position 200
does not carry the weight of a rule at position 5.

**Contradictions accumulate.** Nobody writes a contradiction on purpose. They
arrive because one section says to document as appropriate and another says not to
add comments, and both were correct when written. Anthropic named this specific
pattern when describing what it removed from Claude Code's system prompt:
instructions that conflicted within a single request.

The failure is quiet. The file looks authoritative, the agent reads it, and the
rule you care about is the one that got dropped.

## What changed on the model side

Anthropic reported removing over 80% of Claude Code's system prompt for its
Claude 5 generation models with no measurable loss on internal coding evaluations.
The accompanying guidance describes a shift from rules to judgment, from upfront
context to progressive disclosure, and from repetition to clear interfaces.

The prompting guidance for those models goes further and says to *remove* explicit
verification instructions, because they cause over-verification and waste tokens
without improving quality. Guidance for the Fable generation adds that skills
written for earlier models are often too prescriptive and can degrade output.

That is a real result and a narrow one. It is Anthropic's number, on Anthropic's
evaluations, for Anthropic's newest models. It does not generalize to every model
in a working fleet, which is why this architecture has profiles rather than a
single answer. See [Model Profiles](./model-profiles).

## The four layers

The structure replaces one resident file with four layers, each with a different
loading cost.

| Layer | Loads | Cost | Holds |
|---|---|---|---|
| Root `AGENTS.md` | every session, re-read after compaction | resident, always paid | invariants only |
| `.claude/rules/*.md` | when a matching file is read | zero until triggered | per-stack commands and constraints |
| Skills | description resident, body on match | description only | domain procedures |
| Hooks | on tool events | zero tokens | deterministic enforcement |

The judgment behind the split is simple. Ask what happens if the agent forgets
this rule mid-session. If the answer is "the work is wrong and nobody notices," it
belongs in the root file or in a hook. If the answer is "it looks it up when it
matters," it belongs in a lower layer.

## Compaction decides what is durable

This is the part most guidance skips.

Anything loaded from disk at startup — the project-root rules file — is re-read
after compaction and re-injected. Anything that arrived through the conversation —
a file the agent read, a skill body, a nested rules file — is summarized, and
summarization is lossy.

Standing policy is among the first things a summarizer discards, because it does
not look like the task. So the root file is not merely a convenient place for
invariants. It is the only place they survive.

A corollary that costs real tokens: an `@import` resolves at launch, so an imported
file is loaded in addition to the importing one. A `CLAUDE.md` containing
`@AGENTS.md` loads `AGENTS.md` twice. A symlink does not. The bootstrap prefers the
symlink.

## Prose asks; hooks enforce

Some rules are not judgment calls. "Do not run a release build during
implementation" is a predicate over a command string. Written as prose it asks the
agent to remember it on every turn, forever, competing with every other
instruction in the file.

Written as a hook it is checked by a process that does not get tired, does not
compact, and does not weigh it against 200 other rules. Anthropic's own framing is
direct: rules files are advisory, hooks are deterministic.

Four rules moved from prose to enforcement:

| Was prose | Now |
|---|---|
| Tier discipline | `tier-guard.sh`, blocks Tier 3 outside a release gate |
| Single-writer builds | `single-writer.sh` |
| Anti-sycophancy gate | `sycophancy-gate.sh` |
| Session bootstrap and compaction re-anchor | `reanchor.sh` |

Critic isolation moved too, into a subagent that receives the artifact and no
generation history — a structural guarantee rather than an instruction the
reviewing model is asked to honor about itself.

A hook installed but not referenced in settings enforces nothing, which is worse
than prose because it looks like enforcement. `verify.sh` checks for exactly that.

## What this section covers

| Page | Question it answers |
|---|---|
| [Quick Start](./quick-start) | How do I run it on my project right now? |
| [Use Cases](./use-cases) | Which of the six situations am I in? |
| [Model Profiles](./model-profiles) | My fleet is not only Claude. What changes? |
| [Harness Support](./harness-support) | How does this work in Codex, Cursor, Cline, Gemini CLI, Zed? |
| [Skill Budget](./skill-budget) | Why do my skills exist but not fire? |
| [Theory and Sources](./theory-and-sources) | Where does this come from, and what would refute it? |

## The uncomfortable part

This architecture bets that the model supplies judgment the deleted prose used to
supply. For current frontier models that bet is supported. For a small local model,
an older harness, or a quantized checkpoint it is not — and the failure presents as
carelessness rather than as a missing rule, which makes it hard to diagnose.

The default profile is set against that risk rather than for the token saving. The
saving is real but secondary, and a configuration that costs fewer tokens and
passes fewer tasks is a regression wearing an efficiency argument.
