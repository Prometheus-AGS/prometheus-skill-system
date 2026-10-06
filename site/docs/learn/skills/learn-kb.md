---
id: learn-kb
title: /learn-kb
sidebar_label: learn-kb
---

# /learn-kb

Manage named entries in `~/.prometheus/learn/kb-registry.json` for later grounding:

```text
/learn-kb add --type local --name architecture-docs --content-dir /path/to/docs
/learn-kb add --type dify --name legal-frameworks
/learn-kb add --type palace --name company-playbook --palace-id kb-company-playbook --content-dir /path/to/playbook
/learn-kb add --type url --name api-docs --url https://docs.example/api
/learn-kb list
/learn-kb query --name architecture-docs --subject "ownership boundaries" --top-k 5
/learn-kb update --name architecture-docs
/learn-kb remove --name architecture-docs
```

URL registration requires Firecrawl for a scrape and a configured Palace ingestion destination. It is not a `web:` grounding-helper prefix or a live web-search guarantee. Dify and Palace require their configured collaborators; local entries read files. There is no `status` subcommand.

Select a registered name with `/learn-goal "Understand our architecture" --kb architecture-docs`. Registry commands are skill requests interpreted by the harness, not shell executables. The grounding helper itself supports only `dify:`, `palace:` and `local:` references. See [KB Adapters](/docs/learn/kb-adapters) for destination and privacy limits.
