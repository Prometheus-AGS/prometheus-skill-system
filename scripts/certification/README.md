# scripts/certification

The certification runner, summariser and listing-size scripts moved into the
`skill-certification` skill, so they ship with it:

| Was | Now |
|---|---|
| `scripts/certification/run-skill-evals.sh` | `skills/testing/skill-certification/scripts/run-evals.sh` |
| `scripts/certification/summarize-evals.mjs` | `skills/testing/skill-certification/scripts/summarize-evals.mjs` |
| `scripts/certification/skill-listing-size.mjs` | `skills/testing/skill-certification/scripts/listing-size.mjs` |

`skill-call-graph.mjs` stays here: it analyses the whole pack and is not part of
certifying one skill. The entry point is
`skills/testing/skill-certification/scripts/certify.mjs`.
