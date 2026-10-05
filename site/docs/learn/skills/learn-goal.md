---
id: learn-goal
title: /learn-goal
sidebar_label: learn-goal
---

# /learn-goal

Start by describing a subject and why it matters:

```text
/learn-goal "I want to understand Rust's borrow checker"
```

The skill elicits target level (`novice`, `practitioner` or `expert`), weekly time and horizon. It records a goal before routing to survey and curriculum planning. These are planning estimates, not a guarantee of learning outcomes.

For a custom corpus, first register it with [learn-kb](/docs/learn/skills/learn-kb), then select the registry name:

```text
/learn-goal "Understand our architecture" --kb architecture-docs
```

The skill does not expose `--depth deep` or `--depth overview`. Give those preferences during elicitation. Subsequent skills use the resulting goal ID and stored corpus, rather than an arbitrary subject string.
