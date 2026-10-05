---
id: learn-plan
title: /learn-plan
sidebar_label: learn-plan
---

# /learn-plan

Build a concept DAG and curriculum from a saved goal, survey and corpus:

```text
/learn-plan rust-basics
/learn-plan rust-basics --replan
```

The curriculum records concepts, prerequisite ordering, estimated time and target evidence. It uses the configured learner-model and knowledge stores where the procedure requires them. Missing collaborators or input artifacts must be reported rather than replaced with an invented successful plan. A subject string is not a substitute for the goal ID.

The sequence is goal → survey → plan → explanation and practice → delayed retention. Estimates and a generated graph remain plans until the learner produces the required evidence.
