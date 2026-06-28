# Tasks — change-learn-023

- [ ] Write `tests/learn/integration-kb.sh`: invoke `learn-kb add --local tests/learn/fixtures/sample-kb/` in a clean temp directory, capture exit code and stdout, assert exit 0
- [ ] Assert corpus entries: read the output `grounding-corpus.json` (or the palace store index) and verify at least one entry has `source_type: "operator_kb"` and a non-empty `content` field
- [ ] Extend `tests/learn/fixtures/sample-kb/` if needed: ensure the fixture contains at least one markdown file with a clearly identifiable concept term that can be detected in downstream grade output
- [ ] Add grade integration assertion: run `learn-grade` with the KB corpus path and a mock `feynman-artifact.json`; assert that `grade-result.json` transfer problems reference at least one concept term from the fixture KB
- [ ] Add negative test: invoke `learn-kb add --local /nonexistent/path` and assert exit code is non-zero with a human-readable error message (no stack trace leakage)
