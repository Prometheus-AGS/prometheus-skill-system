---
license: MIT
name: zeespec-interrogate
version: '1.0.0'
description: >
  Run a full ZeeSpec 5W1H interrogation on a named subject. Asks 10 questions
  per dimension (Why, Who, When, What, Where, How), scores coverage, and
  produces a constraint manifest with a GO/CAUTION/NO-GO recommendation.
metadata:
  tags: [process, orchestration, automation]
---

# /zeespec-interrogate

Runs the full ZeeSpec interrogation loop for a named subject.

## Usage

```
/zeespec-interrogate "<subject_name>" [--caller <caller>] [--change-id <id>] [--dimensions <dim,...>]
```

## Setup

1. Parse arguments: extract `subject_name`, optional `caller` (default: `standalone`),
   optional `change_id`, optional `dimensions` subset
2. Run `scripts/state-resolve-provider.sh`
3. Run `scripts/state-init.sh <subject_name> <caller>`
4. If resuming a prior session, show the user the current progress and ask: resume or restart?
5. Load all dimension reference files from `references/dimensions/`
6. Load `prompts/meta-controller.md`
7. Execute: Interrogate → Score → Manifest → Persist

## Default Behavior

- All six dimensions interrogated (Why → Who → When → What → Where → How)
- Coverage threshold: 0.70
- Caller: standalone
- State dir: `.zeespec/`
