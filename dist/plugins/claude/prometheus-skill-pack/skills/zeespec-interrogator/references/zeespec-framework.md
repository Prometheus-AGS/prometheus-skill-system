# ZeeSpec Framework — Theory and Background

## What ZeeSpec Is

ZeeSpec is a structured constraint discovery system built on the Zachman Framework's
five interrogatives (5W1H). It applies these interrogatives not as a documentation
exercise but as a decision gate: **if a question cannot be answered, the system will
make an implicit assumption in its place**. Making those assumptions visible — before
planning, before implementation — is the entire purpose.

ZeeSpec is not a specification format. It is an interrogation method. The output
is not a spec document — it is a constraint manifest that feeds into a specification
or planning process.

## The Zachman Framework Connection

The Zachman Framework is a formal ontology for enterprise architecture, organized
as a matrix of interrogatives (What, How, Where, Who, When, Why) versus abstraction
levels (Conceptual, Logical, Physical, etc.). ZeeSpec does not implement the full
matrix. It takes the interrogative row and applies it at the system or change level —
one interrogation, six dimensions, ten questions each.

The Zachman Framework's insight is that every complex system can be fully described
by answering these six questions from multiple perspectives. ZeeSpec's insight is that
most systems are built while answering only some of them, and the unanswered questions
become hidden assumptions that are discovered through failures.

## The 60-Question Constraint System

ZeeSpec defines exactly 10 questions per dimension, for 60 total. The count is not
arbitrary — 10 questions per dimension is enough to achieve coverage without
becoming a documentation burden. The questions are calibrated to:

1. Surface the constraints that are most commonly left implicit
2. Be answerable in a single focused session for most systems
3. Produce actionable gaps rather than academic completeness

Questions are answered in one of three states:
- **Defined** — clear, specific, unambiguous answer
- **Partial** — direction known, some gaps remain
- **Implicit** — unanswered; the system or AI will decide

The key product of ZeeSpec is the `implicit` inventory. This is the list of things
that will be decided without your input if you proceed.

## Dimension Ordering

ZeeSpec interrogates dimensions in a specific order:

1. **Why** (first, always) — highest criticality; all other dimensions serve Why
2. **Who** — access and ownership constrain Where and How
3. **When** — event model constrains How
4. **What** — data model constrain How
5. **Where** — topology constrains How
6. **How** (last) — implementation dimension; informed by all prior answers

This ordering is deliberate. If Why has coverage below threshold, the interrogation
will likely produce NO-GO regardless of other scores. Interrogating Why first surfaces
this early and avoids working through 50 more questions toward an inevitable conclusion.

## GO/NO-GO Philosophy

ZeeSpec recommends but does not govern. A NO-GO recommendation means:
"Proceeding with this level of constraint definition is high risk." It does not
mean "you cannot proceed." The caller — whether a human user, `kbd`, or
`iterative-evolver` — makes the final decision.

A CAUTION recommendation with an empty `blocked_until` list means the caller
should note the gaps and proceed with awareness. A CAUTION with a non-empty
`blocked_until` list means specific gaps should be resolved before key decisions.

## Relationship to Other Skills

| Skill | ZeeSpec's Role |
|---|---|
| `iterative-evolver` | ZeeSpec is called during Assess when a domain is under-constrained. Manifest feeds analysis and planning. |
| `kbd-process-orchestrator` | ZeeSpec is called during Assess/Plan when a change's coverage is below threshold. Manifest feeds the OpenSpec proposal. |
| `forge-rs` (planned) | ZeeSpec manifests will be consumed by forge's constitution layer — defined constraints become enforced coding standards. |
| `artifact-refiner` | ZeeSpec constraints on content/UI artifacts feed the artifact's constraint definition. |

## Why Not SpecKit

SpecKit applies a similar philosophy but from a different angle. SpecKit enforces
a phase-gated workflow with heavy Markdown artifacts. ZeeSpec is a questioning
method that produces a constraint manifest. They are complementary:

- ZeeSpec: "Have we defined what we're building before we commit to building it?"
- SpecKit/OpenSpec: "Are we tracking changes with the right level of specification?"

ZeeSpec fires before SpecKit/OpenSpec. Its output enriches OpenSpec proposals.
