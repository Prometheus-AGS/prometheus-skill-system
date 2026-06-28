# KB Adapter Guide

Reference for `content-grounding-kb.sh` — the privacy-safe KB grounding path
for learn-* skills.

---

## When to Use KB Grounding

Use KB grounding instead of (or in addition to) the public corpus service when:

- **Professional knowledge** — legal, medical, counseling, financial, or compliance
  content that must not leave your infrastructure.
- **Proprietary frameworks** — internal methodologies, custom playbooks, or
  bespoke domain models that are not publicly documented.
- **Domain-specific corpora** — vertical datasets, curated ontologies, or
  regulatory reference material not indexed on the public web.
- **Sensitive client data** — any subject where even the query string must not
  be sent to a third-party API.

If the subject is well-covered by public documentation and privacy is not a
concern, prefer `content-grounding.sh` (change-learn-003) to benefit from live
web search and broader coverage.

---

## Privacy Guarantee

`content-grounding-kb.sh` enforces a hard boundary:

- It calls only local or self-hosted endpoints (`DIFY_BASE_URL`, `SURREAL_MEMORY_URL`,
  or local filesystem paths).
- It never reads or uses `FIRECRAWL_API_KEY`, `OPENAI_API_KEY`,
  `ANTHROPIC_API_KEY`, or any other external API credential — even if those
  variables are set in the shell.
- On startup the script emits a warning to stderr listing any external API env
  vars it detected, and confirms it will ignore them.
- The output JSON includes `"privacy_mode": true` as an auditable field.

The script cannot guarantee privacy of the calling process or the network
between the script and a self-hosted service. Ensure `DIFY_BASE_URL` and
`SURREAL_MEMORY_URL` point to endpoints you control.

---

## Adapter Setup

### Dify adapter (`dify:<kb-name>`)

Prerequisites:
1. A running Dify instance (self-hosted or cloud with a private API endpoint).
2. `DIFY_API_KEY` set to an API key with read access to knowledge bases.
3. `DIFY_BASE_URL` set to the Dify API base URL (default: `http://localhost/v1`).
4. A knowledge base named `<kb-name>` exists and has documents ingested.

```bash
export DIFY_API_KEY="your-dify-api-key"
export DIFY_BASE_URL="https://dify.internal.example.com/v1"
```

### Palace adapter (`palace:<palace-id>`)

Prerequisites:
1. A running surreal-memory server.
2. `SURREAL_MEMORY_URL` set to the server base URL (e.g. `http://localhost:3000`).
3. A palace with ID `<palace-id>` exists and has memories ingested
   (use `mcp__surreal-memory__palace_ingest` to load documents into a palace).

```bash
export SURREAL_MEMORY_URL="http://localhost:3000"
```

Verify the palace is reachable:
```bash
curl -s "${SURREAL_MEMORY_URL}/health"
```

### Local file adapter (`local:<directory-path>`)

Prerequisites:
1. A directory containing `.md`, `.txt`, or `.json` files.
2. No credentials required — this adapter never makes network calls.

Supported file formats:
- `.md` / `.txt` — first 500 characters used as `content_summary`.
- `.json` — if the file matches the grounding-corpus schema (has a `sources`
  array), its inner source entries are unpacked individually. Otherwise the
  file is treated as a single source using `content_summary` / `content` /
  `summary` fields.

---

## `--kb` Flag Usage Examples

```bash
# Query a Dify knowledge base named "legal-contracts"
content-grounding-kb.sh \
  --kb dify:legal-contracts \
  --subject "force majeure clauses" \
  --level "practitioner" \
  --budget-sources 6 \
  --output /tmp/legal-corpus.json

# Query a surreal-memory palace by ID
content-grounding-kb.sh \
  --kb palace:medical-protocols-2025 \
  --subject "post-operative pain management" \
  --level "expert" \
  --budget-sources 8 \
  --output /tmp/medical-corpus.json \
  --include-misconceptions

# Ingest from a local directory of markdown files
content-grounding-kb.sh \
  --kb local:/home/user/documents/company-handbook \
  --subject "onboarding process" \
  --level "beginner" \
  --budget-sources 5 \
  --output /tmp/handbook-corpus.json
```

---

## Integration with learn-goal

Pass `--kb` through the learning flow by setting it when invoking the grounding
stage. In a learn-goal skill invocation, the KB corpus takes the place of (or
supplements) the public corpus:

```bash
# Build a KB corpus first
content-grounding-kb.sh \
  --kb palace:my-palace \
  --subject "$SUBJECT" \
  --level "$LEVEL" \
  --budget-sources "$BUDGET_KB" \
  --output "${WORK_DIR}/kb-corpus.json"

# Then pass the corpus path to the downstream learn step
learn-goal.sh \
  --subject "$SUBJECT" \
  --level "$LEVEL" \
  --corpus "${WORK_DIR}/kb-corpus.json"
```

The `--corpus` flag (when supported by the learn skill) accepts any file that
validates against `grounding-corpus.schema.json` or `kb-corpus.schema.json`.

---

## Corpus Merging

To combine a KB corpus (private) with a public corpus (from `content-grounding.sh`),
merge the `sources` arrays with `jq`:

```bash
# Build public corpus
content-grounding.sh \
  --subject "contract law" \
  --level "practitioner" \
  --budget-sources 5 \
  --output /tmp/public-corpus.json

# Build KB corpus
content-grounding-kb.sh \
  --kb dify:legal-kb \
  --subject "contract law" \
  --level "practitioner" \
  --budget-sources 5 \
  --output /tmp/kb-corpus.json

# Merge: combine sources arrays, keep KB fields where present
jq -s '
  .[0] as $pub |
  .[1] as $kb |
  {
    corpus_id:     ($kb.corpus_id + "-merged"),
    subject:       $pub.subject,
    target_level:  $pub.target_level,
    schema_version: "1.0.0",
    built_at:      (now | todate),
    sources:       ($pub.sources + $kb.sources)
  }
' /tmp/public-corpus.json /tmp/kb-corpus.json > /tmp/merged-corpus.json
```

The merged file validates against `grounding-corpus.schema.json` (the public
schema, which does not require `kb_source` or `privacy_mode`). If you need
downstream consumers to know the merged file includes private sources, add a
`kb_source` field manually or keep the two corpora separate.

**Important:** the public corpus may contain URLs returned by external APIs.
Do not send the merged `sources` array back to an external API as input — only
the merged file's `content_summary` values (already extracted locally) should
be used as context for local model inference.
