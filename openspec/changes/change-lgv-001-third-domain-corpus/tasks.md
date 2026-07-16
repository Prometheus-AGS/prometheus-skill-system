# Tasks — change-lgv-001-third-domain-corpus

- [ ] Pick a third domain unrelated to software/KBD (general science or history)
- [ ] Write 10-15 source entries matching content-grounding.sh schema exactly
- [ ] Include 3-5 explicit is_misconception:true entries with clear wrong-belief content_summary
- [ ] Include key_points arrays on non-misconception sources for transfer-problem generation
- [ ] Validate the JSON against the schema learn-grade expects (concept_id, sources[])
- [ ] Store at skills/learn/learn-grade/references/eval-dataset/corpora/<domain>-corpus.json
- [ ] Commit the change
