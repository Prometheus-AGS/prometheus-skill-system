# Tasks — change-lgv-005-compute-metrics

- [ ] Write scripts/compute-eval-metrics.py (or shell+jq) reading results/ + index.json ground truth
- [ ] Compute precision/recall for misconceptions_absent detection
- [ ] Compute Pearson + Spearman correlation for completeness/accuracy/clarity
- [ ] Identify the N worst-scoring items (largest gap vs gold) as failure-mode candidates
- [ ] Write metrics-summary.json + human-readable table to references/eval-dataset/
- [ ] Commit the change
