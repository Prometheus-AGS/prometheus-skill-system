---
tags: [certification, outcome]
max_turns: 25
timeout_seconds: 420
allowed_tools: [Read, Glob, Grep, Skill]
---

Grade my explanation of binary search against the corpus at corpus/binary-search.json. Concept id is binary-search, learner id is learner-7. If you cannot update the learner model, skip that step and still give me the grade result.

My explanation: "Binary search finds a value in an array that is already sorted. You compare the target with the middle element. If they match you're done. If the target is smaller you keep only the left half, otherwise the right half, since the other half can't contain it. You repeat until you find it or the range is empty, which means it isn't there. Because the range halves each time, the worst case is O(log n) comparisons."
