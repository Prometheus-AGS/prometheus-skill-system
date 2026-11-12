use crate::primitives::{Example, Prediction};

/// Exact match metric.
/// Returns 1.0 if the prediction matches the example output exactly using Field equality.
/// Assumes common field structure or specific field "answer".
/// Usage: dspy::evaluations::metrics::exact_match
pub fn exact_match(_example: &Example, _pred: &Prediction) -> f64 {
    // Determine the field to compare. defaulting to "answer"
    if let Some(truth) = _example.record.get("answer") {
        if let Some(pred_val) = _pred.record.get("answer") {
            if truth == pred_val {
                return 1.0;
            }
        }
    }
    0.0
}

/// Case-insensitive exact match for answer.
pub fn answer_exact_match(example: &Example, pred: &Prediction) -> f64 {
    if let (Some(truth), Some(pred_val)) = (example.record.get("answer"), pred.record.get("answer"))
    {
        // Assume strings or convert to string
        let t_str = truth.as_str().unwrap_or_default().trim().to_lowercase();
        let p_str = pred_val.as_str().unwrap_or_default().trim().to_lowercase();
        if t_str == p_str {
            return 1.0;
        }
    }
    0.0
}

/// Checks if predicted answer contains the ground truth (or vice versa).
/// Useful for long-form generation where the answer keyword must be present.
pub fn answer_passage_match(example: &Example, pred: &Prediction) -> f64 {
    if let (Some(truth), Some(pred_val)) = (example.record.get("answer"), pred.record.get("answer"))
    {
        let t_str = truth.as_str().unwrap_or_default().trim().to_lowercase();
        let p_str = pred_val.as_str().unwrap_or_default().trim().to_lowercase();
        if p_str.contains(&t_str) {
            return 1.0;
        }
    }
    0.0
}
