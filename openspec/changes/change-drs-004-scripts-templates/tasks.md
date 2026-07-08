# Tasks: change-drs-004-scripts-templates

- [x] Write scripts/run-research.sh (QUERY, DEPTH, KB_IDS → stage-by-stage progress, exit 0)
- [x] Write scripts/export-package.sh (JOB_ID, OUTPUT_DIR → .research package structure)
- [x] Write scripts/verify-sources.sh (SOURCE_URLS → JSON credibility scores)
- [x] Write scripts/build-graph.sh (SOURCES_JSON → graph JSON for surreal-memory)
- [x] Write scripts/detect-contradictions.sh (SOURCES_JSON → JSON contradiction list)
- [x] chmod +x all 5 scripts (-rwxr-xr-x confirmed)
- [x] Write templates/research-plan.md (Stage 1 output template with {{query}}, sub-questions, stages, tokens, threshold)
- [x] Write templates/source-evaluation.md (credibility rubric: 5 dimensions, 0-100 total)
- [x] Write templates/contradiction-resolution.md (claim comparison + resolution strategy template)
- [x] Write templates/report-template.md (OKF frontmatter + standard report sections)
- [x] Write templates/research-package-manifest.json (JSON schema for .research/manifest.json)
- [x] Verify: python3 -m json.tool → JSON valid
- [x] Verify: ls -la scripts/ → all 5 scripts -rwxr-xr-x confirmed
