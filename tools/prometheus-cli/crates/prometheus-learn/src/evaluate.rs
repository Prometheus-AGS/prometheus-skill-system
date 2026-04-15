//! Execution trace evaluation and classification.
//!
//! Scores execution traces using automated metrics and optionally LLM-as-judge.
//! When the `sycophancy` feature is enabled, traces with sycophantic outputs
//! are excluded from the optimization dataset to prevent the optimizer from
//! learning sycophantic patterns.

use crate::trace::{ExecutionTrace, TraceClassification};

/// Evaluate an execution trace and assign a score + classification.
///
/// Scoring heuristics (no LLM required):
/// - No errors + output present → 1.0 (success)
/// - Some errors but output present → 0.5-0.8 (partial)
/// - Fatal errors or no output → 0.0-0.3 (failure)
pub fn evaluate_trace(trace: &mut ExecutionTrace) {
    let has_output = !trace.output_summary.trim().is_empty();
    let error_count = trace.errors.len();
    let tool_failures = trace.tool_calls.iter().filter(|t| !t.success).count();
    let total_tools = trace.tool_calls.len();

    let score = if error_count == 0 && has_output {
        if tool_failures == 0 {
            1.0
        } else {
            let success_rate = 1.0 - (tool_failures as f64 / total_tools.max(1) as f64);
            0.7 + (success_rate * 0.3)
        }
    } else if has_output {
        let penalty = (error_count as f64 * 0.15).min(0.5);
        (0.8 - penalty).max(0.3)
    } else {
        let base = 0.2;
        let tool_credit = if total_tools > 0 {
            let success_rate = 1.0 - (tool_failures as f64 / total_tools as f64);
            success_rate * 0.1
        } else {
            0.0
        };
        base + tool_credit
    };

    trace.score = Some(score);
    trace.classification = if score >= 0.8 {
        TraceClassification::Success
    } else if score >= 0.4 {
        TraceClassification::Partial
    } else {
        TraceClassification::Failure
    };

    // Apply sycophancy filter when feature is enabled
    #[cfg(feature = "sycophancy")]
    sycophancy_filter(trace);
}

/// Filter traces with sycophantic outputs from the optimization dataset.
///
/// Traces with sycophancy_score >= 0.5 are downgraded to Failure classification
/// and excluded from dspy-rs optimization. This prevents the optimizer from
/// learning sycophantic patterns (which would compound across iterations).
#[cfg(feature = "sycophancy")]
fn sycophancy_filter(trace: &mut ExecutionTrace) {
    if trace.output_summary.trim().is_empty() {
        return;
    }

    let config = sycophancy_core::config::SkillConfig::default();
    let detector = sycophancy_core::skill::detector::Detector::new(config);
    let detection = detector.detect(
        &trace.output_summary,
        &[],
        &sycophancy_core::Strictness::Standard,
    );

    if detection.sycophancy_score >= 0.5 {
        trace.classification = TraceClassification::Failure;
        let patterns: Vec<&str> = detection
            .classifications
            .iter()
            .map(|c| c.pattern_id.as_str())
            .collect();
        trace.errors.push(format!(
            "Sycophancy score {:.2} (patterns: {}) — excluded from optimization dataset",
            detection.sycophancy_score,
            patterns.join(", ")
        ));
        tracing::info!(
            skill = %trace.skill_name,
            score = detection.sycophancy_score,
            patterns = ?patterns,
            "Trace excluded from optimization: sycophancy detected"
        );
    }
}
