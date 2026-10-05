---
id: learn-grade
title: /learn-grade
sidebar_label: learn-grade
---

# /learn-grade

Grade an explanation against a named concept and its normalized corpus. The skill asks for concrete gaps, misconceptions and transfer evidence; an LLM score alone does not establish mastery.

```text
/learn-grade --concept-id borrow-checker --learner-id learner-1 --corpus-path /path/to/corpus.json --explanation "My explanation..." --goal-id rust-basics
```

The corpus supplies key points and known misconceptions. The resulting grade includes `overall_score`, `misconceptions_absent`, `gap_list` and `transfer_score`. The explanation gate requires `overall_score >= 0.7` and `misconceptions_absent == 1.0`; transfer and delayed retention remain separate gates.

The skill instructs the agent to screen its draft with sycophancy-correction S-02 when available, or use the documented manual fallback. Its `write-grade.sh` helper persists grade JSON; it does not enforce an independent grader or automatically rewrite incorrect feedback. Record the actual grader and screening evidence. See [Anti-Sycophancy](/docs/learn/anti-sycophancy).
