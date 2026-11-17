---
tags: [certification, outcome]
max_turns: 25
timeout_seconds: 420
allowed_tools: [Read, Glob, Grep, Skill]
---

Grade my explanation of binary search against the corpus at corpus/binary-search.json. Concept id is binary-search, learner id is learner-7. If you cannot update the learner model, skip that step and still give me the grade result.

My explanation: "Binary search is a fast way to find a value in an array. You look at the middle element first; if it's the one you want you're done, otherwise you throw away half and repeat on the other half. The nice thing is it works on any array, sorted or not, because checking the middle first always narrows things down. It takes O(log n) steps in the worst case."
