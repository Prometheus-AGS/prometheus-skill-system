#!/usr/bin/env python3
"""compute-eval-metrics.py — measure learn-grade's accuracy against the eval
dataset's ground truth.

Reads:
  - index.json (item manifest)
  - explanations/*.json (ground truth per item)
  - results/<item_id>.json (grader output per item)

Computes:
  - Precision/recall for misconceptions_absent (binary classification)
  - Pearson + Spearman correlation for completeness/accuracy/clarity
    (continuous, grader score vs. gold score)
  - Per-item absolute error, to identify the worst-scoring items

Writes:
  - metrics-summary.json (machine-readable)
  - METRICS-REPORT.md (human-readable table)

Usage:
  python3 compute-eval-metrics.py
"""
import json
import os
from scipy import stats

BASE = os.path.dirname(os.path.abspath(__file__))
INDEX_PATH = os.path.join(BASE, "index.json")
RESULTS_DIR = os.path.join(BASE, "results")
EXPLANATIONS_DIR = os.path.join(BASE, "explanations")


def load_ground_truth():
    """Return {item_id: {ground_truth, quality_tier, domain}}."""
    gt = {}
    for fname in os.listdir(EXPLANATIONS_DIR):
        if not fname.endswith("-items.json"):
            continue
        data = json.load(open(os.path.join(EXPLANATIONS_DIR, fname)))
        for item in data["items"]:
            gt[item["item_id"]] = {
                "ground_truth": item["ground_truth"],
                "quality_tier": item["quality_tier"],
                "domain": data["domain"],
                "review_status": item["review_status"],
            }
    return gt


def load_results():
    results = {}
    for fname in os.listdir(RESULTS_DIR):
        if not fname.endswith(".json"):
            continue
        item_id = fname[: -len(".json")]
        results[item_id] = json.load(open(os.path.join(RESULTS_DIR, fname)))
    return results


def main():
    gt = load_ground_truth()
    results = load_results()

    missing = set(gt) - set(results)
    if missing:
        print(f"WARNING: {len(missing)} items have no result: {sorted(missing)}")

    common = sorted(set(gt) & set(results))
    n = len(common)

    # --- Misconceptions_absent: binary classification ---
    # "Positive" class = misconception present (i.e. misconceptions_absent == 0.0)
    tp = fp = tn = fn = 0
    misconception_disagreements = []
    for item_id in common:
        gold_absent = gt[item_id]["ground_truth"]["scores"]["misconceptions_absent"]
        pred_absent = results[item_id]["scores"]["misconceptions_absent"]
        gold_present = gold_absent == 0.0
        pred_present = pred_absent == 0.0
        if gold_present and pred_present:
            tp += 1
        elif not gold_present and pred_present:
            fp += 1
        elif not gold_present and not pred_present:
            tn += 1
        elif gold_present and not pred_present:
            fn += 1
        if gold_present != pred_present:
            misconception_disagreements.append(
                {
                    "item_id": item_id,
                    "gold_misconception_present": gold_present,
                    "grader_misconception_present": pred_present,
                    "quality_tier": gt[item_id]["quality_tier"],
                }
            )

    precision = tp / (tp + fp) if (tp + fp) else None
    recall = tp / (tp + fn) if (tp + fn) else None
    f1 = (
        2 * precision * recall / (precision + recall)
        if precision and recall and (precision + recall) > 0
        else None
    )
    accuracy_binary = (tp + tn) / n if n else None

    # --- Continuous dims: completeness, accuracy, clarity ---
    continuous_dims = ["completeness", "accuracy", "clarity"]
    correlations = {}
    per_item_errors = []
    for item_id in common:
        gold = gt[item_id]["ground_truth"]["scores"]
        pred = results[item_id]["scores"]
        errs = {}
        for dim in continuous_dims:
            errs[dim] = abs(gold[dim] - pred[dim])
        errs["misconceptions_absent"] = abs(
            gt[item_id]["ground_truth"]["scores"]["misconceptions_absent"]
            - results[item_id]["scores"]["misconceptions_absent"]
        )
        per_item_errors.append(
            {
                "item_id": item_id,
                "quality_tier": gt[item_id]["quality_tier"],
                "domain": gt[item_id]["domain"],
                "mean_abs_error": sum(errs[d] for d in continuous_dims) / len(continuous_dims),
                "errors": errs,
            }
        )

    for dim in continuous_dims:
        gold_vals = [gt[i]["ground_truth"]["scores"][dim] for i in common]
        pred_vals = [results[i]["scores"][dim] for i in common]
        pearson_r, pearson_p = stats.pearsonr(gold_vals, pred_vals)
        spearman_r, spearman_p = stats.spearmanr(gold_vals, pred_vals)
        mae = sum(abs(g - p) for g, p in zip(gold_vals, pred_vals)) / len(gold_vals)
        correlations[dim] = {
            "pearson_r": round(pearson_r, 4),
            "pearson_p": round(pearson_p, 4),
            "spearman_r": round(spearman_r, 4),
            "spearman_p": round(spearman_p, 4),
            "mean_absolute_error": round(mae, 4),
        }

    per_item_errors.sort(key=lambda x: x["mean_abs_error"], reverse=True)
    worst_5 = per_item_errors[:5]

    # --- Draft ground truth caveat ---
    draft_count = sum(1 for i in common if gt[i]["review_status"] == "draft")

    summary = {
        "generated_at": "2026-07-16T21:00:00Z",
        "total_items": n,
        "items_missing_results": sorted(missing),
        "ground_truth_review_status": {
            "draft": draft_count,
            "reviewed": n - draft_count,
            "caveat": "All ground truth is currently in draft status pending human review per phase-learn-grader-validation assessment.md open question #1. Metrics below should be treated as provisional until ground truth is reviewed.",
        },
        "misconceptions_absent_binary_classification": {
            "confusion_matrix": {"tp": tp, "fp": fp, "tn": tn, "fn": fn},
            "precision": round(precision, 4) if precision is not None else None,
            "recall": round(recall, 4) if recall is not None else None,
            "f1": round(f1, 4) if f1 is not None else None,
            "accuracy": round(accuracy_binary, 4) if accuracy_binary is not None else None,
            "disagreements": misconception_disagreements,
        },
        "continuous_dimension_correlations": correlations,
        "worst_5_items_by_mean_absolute_error": worst_5,
    }

    with open(os.path.join(BASE, "metrics-summary.json"), "w") as f:
        json.dump(summary, f, indent=2)

    # --- Human-readable report ---
    lines = []
    lines.append("# Eval Metrics Report\n")
    lines.append(f"_Generated: {summary['generated_at']}_\n")
    lines.append(
        f"**Caveat**: {draft_count}/{n} ground-truth items are still in `draft` "
        "review status — these metrics are provisional pending human review.\n"
    )
    lines.append("## Misconceptions Detection (binary classification)\n")
    lines.append(f"- Precision: **{summary['misconceptions_absent_binary_classification']['precision']}**")
    lines.append(f"- Recall: **{summary['misconceptions_absent_binary_classification']['recall']}**")
    lines.append(f"- F1: **{summary['misconceptions_absent_binary_classification']['f1']}**")
    lines.append(f"- Accuracy: **{summary['misconceptions_absent_binary_classification']['accuracy']}**")
    lines.append(
        f"- Confusion matrix: TP={tp} FP={fp} TN={tn} FN={fn} (n={n})\n"
    )
    if misconception_disagreements:
        lines.append("### Disagreements\n")
        for d in misconception_disagreements:
            lines.append(
                f"- `{d['item_id']}` ({d['quality_tier']}): gold_present={d['gold_misconception_present']}, "
                f"grader_present={d['grader_misconception_present']}"
            )
        lines.append("")

    lines.append("## Continuous Dimension Correlation (grader score vs. gold score)\n")
    lines.append("| Dimension | Pearson r | Spearman r | MAE |")
    lines.append("|---|---|---|---|")
    for dim in continuous_dims:
        c = correlations[dim]
        lines.append(f"| {dim} | {c['pearson_r']} | {c['spearman_r']} | {c['mean_absolute_error']} |")
    lines.append("")

    lines.append("## Worst 5 Items by Mean Absolute Error\n")
    lines.append("| Item | Domain | Tier | Mean Abs Error |")
    lines.append("|---|---|---|---|")
    for item in worst_5:
        lines.append(
            f"| {item['item_id']} | {item['domain']} | {item['quality_tier']} | {round(item['mean_abs_error'], 4)} |"
        )

    with open(os.path.join(BASE, "METRICS-REPORT.md"), "w") as f:
        f.write("\n".join(lines) + "\n")

    print(f"Computed metrics over {n} items ({len(missing)} missing).")
    print(f"Misconceptions: precision={precision}, recall={recall}, f1={f1}")
    for dim in continuous_dims:
        print(f"{dim}: pearson_r={correlations[dim]['pearson_r']}, spearman_r={correlations[dim]['spearman_r']}, mae={correlations[dim]['mean_absolute_error']}")
    print("Wrote metrics-summary.json and METRICS-REPORT.md")


if __name__ == "__main__":
    main()
