---
id: kb-adapters
title: KB Adapters
sidebar_label: KB Adapters
---

# KB Adapters

The Learn domain supports custom knowledge bases via the `/learn-kb` skill. KB adapters
allow you to ground learning in your own proprietary content.

## Adding a KB

```bash
/learn-kb add dify:my-legal-kb
/learn-kb add local:/path/to/clinical-protocols
/learn-kb add palace:my-collection
/learn-kb add web:https://my-docs.example.com
```

## Adapter types

| Prefix | Backend | Privacy |
|--------|---------|---------|
| `dify:<kb-name>` | Dify knowledge base MCP | Dify server, requires `DIFY_API_KEY` |
| `palace:<collection>` | surreal-memory palace RAG | Fully local, no external calls |
| `local:<path>` | Filesystem markdown | Stays on machine |
| `web:<url>` | Firecrawl live fetch | Internet required |

## Privacy guarantee

`content-grounding-kb.sh` **NEVER** forwards KB content to external APIs.

- If external API env vars (FIRECRAWL_API_KEY, etc.) are set and a `local:` or
  `palace:` KB is being queried, those sources skip external calls entirely.
- The privacy guarantee is enforced in code, not convention.

## Using a KB in a learning session

```bash
/learn-goal "I want to understand clinical trial protocols" --kb local:/protocols
```

The KB adapter is loaded alongside the standard learning arc. Concept gaps are
grounded in the KB content rather than the model's training data.

## Managing KBs

```bash
/learn-kb list                    # show registered KBs
/learn-kb remove dify:my-kb       # deregister a KB
/learn-kb status                  # check adapter health
```
