use std::collections::BTreeSet;

use cedar_policy::{Authorizer, Context, Decision, Entities, EntityUid, PolicySet, Request};
use prometheus_exec_contracts::{hash_bytes, Digest, SignedExecRequest};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const DEFAULT_EXECUTION_POLICY: &str = r#"
permit (
    principal,
    action == Action::"exec.autoApprove",
    resource
) when {
    context.networkFree &&
    context.environmentEmpty &&
    context.outputScoped
};
"#;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyReason {
    NetworkEgress,
    EnvironmentPassthrough,
    ExternalWritePath(String),
    ExternalReadPath(String),
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
            if !is_output_scoped(path) {
                reasons.insert(PolicyReason::ExternalWritePath(path.clone()));
            }
        }
        for path in &request.capabilities.fs.read_only {
            if !is_workspace_relative(path) {
                reasons.insert(PolicyReason::ExternalReadPath(path.clone()));
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

#[derive(Debug, Error)]
pub enum CedarPolicyError {
    #[error("Cedar policy failed to parse: {0}")]
    Parse(String),
}

/// Cedar policy that can only tighten the hard baseline decision.
pub struct CedarTighteningPolicy {
    authorizer: Authorizer,
    policies: PolicySet,
    entities: Entities,
    policy_hash: Digest,
}

impl CedarTighteningPolicy {
    pub fn from_policy_text(policy_text: &str) -> Result<Self, CedarPolicyError> {
        let policies = policy_text
            .parse::<PolicySet>()
            .map_err(|error| CedarPolicyError::Parse(format!("{error:?}")))?;
        Ok(Self {
            authorizer: Authorizer::new(),
            policies,
            entities: Entities::empty(),
            policy_hash: hash_bytes(policy_text.as_bytes()),
        })
    }

    pub fn embedded_default() -> Self {
        Self::from_policy_text(DEFAULT_EXECUTION_POLICY)
            .expect("the compiled-in execution policy must parse")
    }

    pub fn policy_hash(&self) -> &Digest {
        &self.policy_hash
    }

    fn cedar_decision(&self, request: &SignedExecRequest) -> Result<(bool, Vec<String>), String> {
        let principal = r#"Agent::"local""#
            .parse::<EntityUid>()
            .map_err(|error| format!("principal parse failed: {error}"))?;
        let action = r#"Action::"exec.autoApprove""#
            .parse::<EntityUid>()
            .map_err(|error| format!("action parse failed: {error}"))?;
        let resource = format!(r#"Execution::"{}""#, request.request_id)
            .parse::<EntityUid>()
            .map_err(|error| format!("resource parse failed: {error}"))?;
        let output_scoped = request
            .capabilities
            .fs
            .read_write
            .iter()
            .all(|path| is_output_scoped(path));
        let context_json = serde_json::json!({
            "networkFree": request.capabilities.net.egress.is_empty(),
            "environmentEmpty": request.capabilities.env.read.is_empty(),
            "outputScoped": output_scoped,
            "memoryMb": request.limits.memory_mb,
            "wallClockMs": request.limits.wall_clock_ms,
            "runtime": serde_json::to_value(request.code.runtime)
                .map_err(|error| format!("runtime serialization failed: {error}"))?,
            "tier": serde_json::to_value(request.tier)
                .map_err(|error| format!("tier serialization failed: {error}"))?,
        });
        let context = Context::from_json_str(&context_json.to_string(), None)
            .map_err(|error| format!("context parse failed: {error:?}"))?;
        let cedar_request = Request::new(principal, action, resource, context, None)
            .map_err(|error| format!("request construction failed: {error:?}"))?;
        let response =
            self.authorizer
                .is_authorized(&cedar_request, &self.policies, &self.entities);
        let mut reasons: Vec<_> = response
            .diagnostics()
            .reason()
            .map(ToString::to_string)
            .collect();
        reasons.sort();
        Ok((response.decision() == Decision::Allow, reasons))
    }
}

fn is_output_scoped(path: &str) -> bool {
    let normalized = path.replace('\\', "/");
    if normalized.starts_with('/') || has_windows_drive_prefix(&normalized) {
        return false;
    }
    let trimmed = normalized.trim_end_matches('/');
    let mut components = trimmed.split('/');
    matches!(components.next(), Some("outputs"))
        && components.all(|part| !part.is_empty() && part != "." && part != "..")
}

fn is_workspace_relative(path: &str) -> bool {
    let normalized = path.replace('\\', "/");
    let normalized = normalized.trim_end_matches('/');
    if normalized.is_empty() || normalized.starts_with('/') || has_windows_drive_prefix(normalized)
    {
        return false;
    }
    normalized
        .split('/')
        .all(|part| !part.is_empty() && part != "..")
}

fn has_windows_drive_prefix(path: &str) -> bool {
    matches!(path.as_bytes(), [letter, b':', ..] if letter.is_ascii_alphabetic())
}

impl Default for CedarTighteningPolicy {
    fn default() -> Self {
        Self::embedded_default()
    }
}

impl PolicyEvaluator for CedarTighteningPolicy {
    fn evaluate(&self, request: &SignedExecRequest) -> PolicyOutcome {
        let baseline = BaselinePolicy.evaluate(request);
        if baseline != PolicyOutcome::AutoApproved {
            return baseline;
        }

        match self.cedar_decision(request) {
            Ok((true, _)) => PolicyOutcome::AutoApproved,
            Ok((false, policy_ids)) => {
                let reasons = if policy_ids.is_empty() {
                    vec![PolicyReason::OperatorDenied("no_permit_policy".into())]
                } else {
                    policy_ids
                        .into_iter()
                        .map(PolicyReason::OperatorDenied)
                        .collect()
                };
                PolicyOutcome::GrantRequired { reasons }
            }
            Err(error) => PolicyOutcome::Denied {
                reasons: vec![PolicyReason::OperatorDenied(format!(
                    "cedar_evaluation_error:{error}"
                ))],
            },
        }
    }
}
