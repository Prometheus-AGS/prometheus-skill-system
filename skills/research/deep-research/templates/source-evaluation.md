# Source Credibility Evaluation

**URL:** {{url}}
**Evaluated at:** {{evaluated_at}}

## Scoring Rubric

Score each dimension. Total = credibility score (0–100).

### 1. Domain Authority (0–25 points)

| Score | Criteria |
|-------|----------|
| 25 | `.edu`, `.gov`, peer-reviewed journal, major academic publisher |
| 20 | Established trade publication, well-known vendor primary docs |
| 15 | Known industry blog with editorial standards |
| 10 | General reference site with identifiable authorship |
| 5  | Aggregator, secondary summary, unclear origin |
| 0  | Content farm, SEO-spam, social media post |

**Score:** {{domain_authority_score}} / 25
**Notes:** {{domain_authority_notes}}

### 2. Publication Recency (0–20 points)

| Score | Criteria |
|-------|----------|
| 20 | Published within past 6 months |
| 15 | Published within past 12 months |
| 10 | Published 1–2 years ago |
| 5  | Published 2–3 years ago |
| 0  | Published >3 years ago or undated |

**Score:** {{recency_score}} / 20
**Publication date:** {{publication_date}}

### 3. Author Credentials (0–20 points)

| Score | Criteria |
|-------|----------|
| 20 | Named author(s) with verifiable expertise in the subject domain |
| 15 | Named author with general professional background |
| 10 | Organizational authorship (team, department) with clear accountability |
| 5  | Anonymous but from credible institution |
| 0  | Anonymous, unattributed, or AI-generated without review |

**Score:** {{author_score}} / 20
**Author(s):** {{authors}}

### 4. Cross-Reference Count (0–20 points)

| Score | Criteria |
|-------|----------|
| 20 | Cited by 5+ other retrieved sources |
| 15 | Cited by 3–4 other retrieved sources |
| 10 | Cited by 1–2 other retrieved sources |
| 5  | Not cited but corroborates claims from other sources |
| 0  | Contradicts all other sources without supporting evidence |

**Score:** {{cross_reference_score}} / 20
**Referenced by:** {{referenced_by}}

### 5. Methodology Transparency (0–15 points)

| Score | Criteria |
|-------|----------|
| 15 | Shows raw data, methodology, primary sources, or reproducible benchmarks |
| 10 | Cites primary sources; methodology partially explained |
| 5  | Some sourcing visible; methodology unclear |
| 0  | No sourcing, no methodology, claims stated without basis |

**Score:** {{methodology_score}} / 15

## Sycophancy Penalty

Applied when `detect_sycophancy` flags the source:

| Penalty | Severity |
|---------|----------|
| -20 | critical: systematic suppression of contradictory evidence |
| -15 | high: strong over-confidence in claims without basis |
| -10 | medium: selective framing that misleads |
| -5  | low: minor over-confidence |
| 0   | no sycophancy detected |

**Penalty:** {{sycophancy_penalty}}
**Flags:** {{sycophancy_flags}}

## Final Score

```
Domain authority:        {{domain_authority_score}} / 25
Publication recency:     {{recency_score}} / 20
Author credentials:      {{author_score}} / 20
Cross-reference count:   {{cross_reference_score}} / 20
Methodology transparency:{{methodology_score}} / 15
Sycophancy penalty:      {{sycophancy_penalty}}
─────────────────────────────
TOTAL:                   {{total_score}} / 100
STATUS:                  {{status}}  (threshold: 40)
```
