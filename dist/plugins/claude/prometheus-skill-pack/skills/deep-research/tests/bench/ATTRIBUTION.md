# Attribution

## Adopted work

Files under `criteria/` and `queries/` are derived from
**[deep_research_bench](https://github.com/Ayanami0730/deep_research_bench)**,
pinned at upstream commit **`469cce54`** (`469cce54ea7f`, 2026-05-11).

| File here | Upstream path at `469cce54` | Relationship |
|---|---|---|
| `queries/subset-10.jsonl` | `data/prompt_data/query.jsonl` | 10 of 100 rows, content unmodified |
| `criteria/subset-10-criteria.jsonl` | `data/criteria_data/criteria.jsonl` | the 10 matching rows, content unmodified |
| `criteria/score_prompt_en.py` | `prompt/score_prompt_en.py` | verbatim copy |

## Licence

deep_research_bench is licensed **Apache-2.0**. That licence permits
redistribution of the work and of derivative works provided the recipient
receives a copy of the licence, modified files carry notices stating they were
changed, and existing attribution notices are preserved. This file is that
notice. Subsetting — selecting whole rows — is the only modification; no
upstream file's content was altered.

Full licence text:
<https://github.com/Ayanami0730/deep_research_bench/blob/469cce54/LICENSE>

## Why the criteria are adopted rather than written

A home-grown rubric produces a number comparable to nothing. Onyx's standing is
quoted against *this* benchmark, so scoring against the same published criteria
is what turns "on par with Onyx" from a slogan into a claim that can be checked
— including checked and found false.

## Why the subset is 10 tasks, and how it was chosen

A bench run costs real gateway tokens; 10 tasks is this phase's budget ceiling.

Selection is **deterministic, not random**. A random sample would be
irreproducible, which would defeat the point of pinning the rubric at all. The
rule:

1. English-language tasks only, matching `score_prompt_en.py`.
2. One task per topic, taking the lowest `id` within each topic.
3. Order the survivors by `id` and take the first ten.

Re-running that rule against `469cce54` yields the same ten tasks every time.

Selected ids **51, 58, 66, 71, 75, 79, 81, 83, 85, 87** — one each from Finance
& Business, Science & Technology, Software Development, Education & Jobs,
Health, Literature, History, Hardware, Industrial, and Art & Design.

Ten tasks is **indicative, not statistically meaningful** for RACE.
`BENCH-RESULTS.md` states that beside every number. A full 100-task run is a
later decision, not a silent omission.

## What is NOT adopted

The upstream Python scorer is not ported. The harness here is bash driving this
pack's own gateway with the `judge` role, so the judge is a different model from
the producer. A same-family judge shares the producer's blind spots; that is a
failure, not a fallback, and `score-race.sh` refuses to score when the two match.

**FACT metrics** (effective citations, citation accuracy) and the
**verified-claim ratio** are built here, not adopted. The upstream benchmark
does not compute them, and the verified-claim ratio is the metric this pack can
report because it carries per-claim labels — something Onyx structurally cannot
report at all.
