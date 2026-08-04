use std::collections::BTreeSet;

use prometheus_exec_contracts::SignedExecRequest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyReason {
    NetworkEgress,
    EnvironmentPassthrough,
    ExternalWritePath(String),
    InvalidRequest(String),
    OperatorDenied(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "decision", rename_all = "snake_case")]
pub enum PolicyOutcome {
    AutoApproved,
    GrantRequired { reasons: Vec<PolicyReason> },
    Denied { reasons: Vec<PolicyReason> },
}

pub trait PolicyEvaluator: Send + Sync {
    fn evaluate(&self, request: &SignedExecRequest) -> PolicyOutcome;
}

/// Deterministic hard ceiling evaluated before an operator Cedar bundle.
///
/// The later Cedar adapter may convert an `AutoApproved` outcome to a denial
/// or grant requirement. It cannot convert either restricted outcome back to
/// automatic approval.
#[derive(Clone, Copy, Debug, Default)]
pub struct BaselinePolicy;

impl PolicyEvaluator for BaselinePolicy {
    fn evaluate(&self, request: &SignedExecRequest) -> PolicyOutcome {
        if let Err(error) = request.validate() {
            return PolicyOutcome::Denied {
                reasons: vec![PolicyReason::InvalidRequest(error.to_string())],
            };
        }

        let mut reasons = BTreeSet::new();
        if !request.capabilities.net.egress.is_empty() {
            reasons.insert(PolicyReason::NetworkEgress);
        }
        if !request.capabilities.env.read.is_empty() {
            reasons.insert(PolicyReason::EnvironmentPassthrough);
        }
        for path in &request.capabilities.fs.read_write {
            let normalized = path.trim_end_matches('/');
            if normalized != "outputs" && !normalized.starts_with("outputs/") {
                reasons.insert(PolicyReason::ExternalWritePath(path.clone()));
            }
        }

        if reasons.is_empty() {
            PolicyOutcome::AutoApproved
        } else {
            PolicyOutcome::GrantRequired {
                reasons: reasons.into_iter().collect(),
            }
        }
    }
}
