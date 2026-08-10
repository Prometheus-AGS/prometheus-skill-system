# Citation Formats

Stage 08 generates formatted citations for all verified sources.

## Supported Styles

| Style | Key | Example |
|-------|-----|---------|
| APA 7 | `APA` | Author, A. A. (Year, Month Day). *Title*. Publisher. URL |
| MLA 9 | `MLA` | Author Last, First. "Title." *Publication*, Date, URL. |
| Chicago 17 | `Chicago` | Author Last, First. "Title." *Publication*, Month Day, Year. URL. |
| IEEE | `IEEE` | A. Author, "Title," *Journal*, vol. X, pp. YY-ZZ, Year. |
| Vancouver | `Vancouver` | Author AA. Title. Publication. Year Mon Day;Vol(Issue):Pages. URL |

## Default Style

**APA 7** is the default. Override via:

```bash
export RESEARCH_CITATION_STYLE=MLA
/deep-research "my query"
```

Or at invocation time:
```
/deep-research --citation-style Chicago "my query"
```

## Citation Object Schema

```json
{
  "id": "cite-001",
  "style": "APA",
  "formatted": "Author, A. (2025). Title. Publisher. https://...",
  "url": "https://...",
  "title": "...",
  "authors": ["Author Name"],
  "publication_date": "2025-03-01",
  "publisher": "...",
  "access_date": "2026-07-08",
  "credibility_score": 77,
  "confidence": 0.82,
  "used_in_claims": ["claim-001", "claim-003"]
}
```

## Credibility → Confidence Mapping

| Credibility score | Confidence range |
|------------------|-----------------|
| 80–100 | 0.85–1.00 (high) |
| 60–79 | 0.65–0.84 (medium) |
| 40–59 | 0.40–0.64 (low) |

## Metadata Extraction Priority

For each source URL, metadata is extracted in this priority order:

1. **Structured metadata** — Open Graph tags, `<meta>` tags, JSON-LD schema.org
2. **Retrieved content** — title from `<h1>`, author from byline patterns, date from visible dateline
3. **URL-derived** — domain as publisher, URL path as title approximation
4. **Minimal fallback** — `{url}` as the citation with "Retrieved [date]"

## Unknown Authors

When author cannot be determined:
- APA: Use organization name or `[Anonymous]`
- MLA: Start with title
- IEEE: Use organization name

Never fabricate an author. If unknown, use the fallback form.
