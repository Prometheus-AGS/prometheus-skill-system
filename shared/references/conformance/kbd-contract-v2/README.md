# KBD adopted-contract v2 conformance (VLBV)

Fixtures for the contract v2 schema and the adoption lint
(`shared/scripts/kbd-contract-lint.mjs`, run by `kbd-contract-adopt.sh`).
Drive them through the production entry point:

```bash
bash shared/scripts/kbd-contract-adopt.sh --lint-only shared/references/conformance/kbd-contract-v2/golden/contract.json
for f in shared/references/conformance/kbd-contract-v2/defects/*.json; do
  bash shared/scripts/kbd-contract-adopt.sh --lint-only "$f"   # exit 2; stderr names _expect.code
done
```

| Path | Expectation |
|---|---|
| `golden/contract.json` | Passes structural validation and adoption lint (exit 0). |
| `defects/NN-name.json` | Exactly the golden contract plus one seeded defect; adoption is refused (exit 2) and stderr carries `_expect.code`. |

The `--lint-only` flag performs no canonical read or write, so these run outside a
KBD project. Full adoption (identity, binding, snapshot) is covered by the v1
phase-closure corpus, which v2 contracts also satisfy.
