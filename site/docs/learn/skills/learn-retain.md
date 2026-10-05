---
id: learn-retain
title: /learn-retain
sidebar_label: learn-retain
---

# /learn-retain

Review due concepts for a named goal using the learner-model's FSRS scheduling:

```text
/learn-retain rust-basics
/learn-retain rust-basics --concept-id borrow-checker --max-cards 5
```

The default batch is five cards. The procedure selects due concepts, asks retrieval questions, grades responses and updates review state. Scheduling depends on stored observations; it does not guarantee an optimal interval for every learner.

Mastery requires a successful retention check at least 24 hours after the initial claim. The skill's retention pass threshold is `>= 0.6`; record the timestamp and evidence in the concept artifact. An immediate repeat cannot satisfy the delayed gate.
