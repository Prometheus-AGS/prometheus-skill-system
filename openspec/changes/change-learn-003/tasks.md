# Tasks — change-learn-003

- [ ] Write `shared/scripts/content-grounding.sh` with `--subject`, `--level`, `--budget-sources`, and `--budget-minutes` flags
- [ ] Implement source priority chain: primary literature > textbooks > reference implementations > surveys > secondary > LLM fill
- [ ] Add `--include-misconceptions` flag that retrieves known-wrong-model sources alongside correct ones
- [ ] Define `grounding-corpus.schema.json` specifying output envelope (sources[], misconceptions[], metadata)
- [ ] Test script with a sample subject and verify priority chain ordering and schema conformance
