//! Cedar policy enforcement for Prometheus skill mutations.
//!
//! Implements the Skill Mutation PEP (Policy Enforcement Point) that gates
//! all writes to skill artifacts:
//! - `skill.mutate` — prompt optimization writes back to SKILL.md
//! - `skill.generate` — PMPO creates new skills from gap detection
//! - `skill.promote` — generated skills promoted from staging to active
//! - `trace.capture` — execution data collection in governed contexts
//!
//! Designed to embed in UAR's SkillService as the mutation gateway.
//! Default-deny: if no permit policy matches, mutation is blocked.

use cedar_policy::{Authorizer, Context, Decision, Entities, EntityUid, PolicySet, Request};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Operations governed by the Cedar Skill Mutation PEP.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillOperation {
    /// Prompt optimization writes back to SKILL.md
    Mutate,
    /// PMPO creates a new skill from gap detection
    Generate,
    /// Generated skill promoted from staging to active
    Promote,
    /// Execution trace capture in governed contexts
    TraceCapture,
}

impl SkillOperation {
    pub fn action_id(&self) -> &'static str {
        match self {
            Self::Mutate => "skill.mutate",
            Self::Generate => "skill.generate",
            Self::Promote => "skill.promote",
            Self::TraceCapture => "trace.capture",
        }
    }
}

/// Context passed to Cedar for mutation decisions.
#[derive(Debug, Serialize, Deserialize)]
pub struct MutationContext {
    /// Environment: development, staging, production
    pub environment: String,
    /// Confidence score from the optimizer (0.0-1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence_score: Option<f64>,
    /// Whether automated validation passed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_passed: Option<bool>,
    /// Whether a human explicitly approved this mutation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub human_approved: Option<bool>,
    /// Method that produced the mutation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_method: Option<String>,
    /// Test pass rate for promotion decisions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_pass_rate: Option<f64>,
    /// Whether governance consent was given for trace capture
    #[serde(skip_serializing_if = "Option::is_none")]
    pub governance_consent: Option<bool>,
    /// Trace ID for audit trail
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}

impl Default for MutationContext {
    fn default() -> Self {
        Self {
            environment: "development".to_string(),
            confidence_score: None,
            validation_passed: None,
            human_approved: None,
            generation_method: None,
            test_pass_rate: None,
            governance_consent: None,
            trace_id: None,
        }
    }
}

/// Result of a Cedar policy evaluation.
#[derive(Debug, Clone)]
pub struct PolicyDecision {
    pub allowed: bool,
    pub reasons: Vec<String>,
}

/// The Skill Mutation PEP — embeddable in UAR's SkillService.
pub struct SkillMutationPep {
    authorizer: Authorizer,
    policies: PolicySet,
    entities: Entities,
}

impl SkillMutationPep {
    /// Create from Cedar policy and entity files.
    pub fn from_files(policy_path: &Path, entities_path: &Path) -> anyhow::Result<Self> {
        let policy_text = std::fs::read_to_string(policy_path)
            .map_err(|e| anyhow::anyhow!("Failed to read policy file: {}", e))?;
        let policies: PolicySet = policy_text.parse()
            .map_err(|e| anyhow::anyhow!("Failed to parse Cedar policies: {:?}", e))?;

        let entities_text = std::fs::read_to_string(entities_path)
            .map_err(|e| anyhow::anyhow!("Failed to read entities file: {}", e))?;
        let entities = Entities::from_json_str(&entities_text, None)
            .map_err(|e| anyhow::anyhow!("Failed to parse Cedar entities: {:?}", e))?;

        Ok(Self {
            authorizer: Authorizer::new(),
            policies,
            entities,
        })
    }

    /// Create with inline policies (for testing or embedded defaults).
    pub fn from_policy_text(policy_text: &str) -> anyhow::Result<Self> {
        let policies: PolicySet = policy_text.parse()
            .map_err(|e| anyhow::anyhow!("Failed to parse Cedar policies: {:?}", e))?;

        Ok(Self {
            authorizer: Authorizer::new(),
            policies,
            entities: Entities::empty(),
        })
    }

    /// Create a permissive PEP that allows all operations (for development).
    pub fn permissive() -> Self {
        let policies: PolicySet = r#"permit(principal, action, resource);"#
            .parse()
            .expect("default permit policy should parse");

        Self {
            authorizer: Authorizer::new(),
            policies,
            entities: Entities::empty(),
        }
    }

    /// Check whether a skill mutation is allowed.
    pub fn check(
        &self,
        agent_id: &str,
        operation: SkillOperation,
        skill_id: &str,
        context: &MutationContext,
    ) -> PolicyDecision {
        let principal = format!(r#"Agent::"{}""#, agent_id)
            .parse::<EntityUid>();
        let action = format!(r#"Action::"{}""#, operation.action_id())
            .parse::<EntityUid>();
        let resource = format!(r#"Skill::"{}""#, skill_id)
            .parse::<EntityUid>();

        let context_json = serde_json::to_string(context).unwrap_or_default();
        let cedar_context = Context::from_json_str(&context_json, None);

        // If any parsing fails, deny by default
        let (Ok(principal), Ok(action), Ok(resource), Ok(cedar_context)) =
            (principal, action, resource, cedar_context)
        else {
            tracing::warn!(
                agent = agent_id,
                operation = operation.action_id(),
                skill = skill_id,
                "Cedar request parsing failed — denying by default"
            );
            return PolicyDecision {
                allowed: false,
                reasons: vec!["Failed to parse Cedar request".to_string()],
            };
        };

        let request = Request::new(
            principal,
            action,
            resource,
            cedar_context,
            None,
        );

        match request {
            Ok(req) => {
                let response = self.authorizer.is_authorized(&req, &self.policies, &self.entities);
                let allowed = response.decision() == Decision::Allow;

                let reasons: Vec<String> = response.diagnostics().reason()
                    .map(|id| id.to_string())
                    .collect();

                if !allowed {
                    tracing::info!(
                        agent = agent_id,
                        operation = operation.action_id(),
                        skill = skill_id,
                        "Cedar DENIED skill mutation"
                    );
                }

                PolicyDecision { allowed, reasons }
            }
            Err(e) => {
                tracing::warn!("Cedar request construction failed: {:?}", e);
                PolicyDecision {
                    allowed: false,
                    reasons: vec![format!("Request construction failed: {:?}", e)],
                }
            }
        }
    }
}

impl SkillMutationPep {
    /// Load from the default policy directory search path.
    ///
    /// Search order:
    /// 1. `$PROMETHEUS_POLICY_DIR` environment variable
    /// 2. `./policies/` relative to current directory
    /// 3. `~/.prometheus/policies/`
    /// 4. Fallback: use compiled-in DEFAULT_POLICIES
    pub fn from_default_dir() -> anyhow::Result<Self> {
        if let Some(dir) = Self::find_policy_dir() {
            let policy_path = dir.join("skill-mutation.cedar");
            let entities_path = dir.join("entities.json");

            if policy_path.exists() {
                tracing::info!(dir = %dir.display(), "Loading Cedar policies from directory");
                return Self::from_files(&policy_path, &entities_path);
            }
        }

        tracing::info!("Using compiled-in default Cedar policies");
        Self::from_policy_text(DEFAULT_POLICIES)
    }

    fn find_policy_dir() -> Option<std::path::PathBuf> {
        // 1. Environment variable
        if let Ok(dir) = std::env::var("PROMETHEUS_POLICY_DIR") {
            let p = std::path::PathBuf::from(dir);
            if p.exists() { return Some(p); }
        }

        // 2. Local ./policies/
        let local = std::path::PathBuf::from("policies");
        if local.join("skill-mutation.cedar").exists() {
            return Some(local);
        }

        // 3. Home directory
        if let Some(home) = dirs::home_dir() {
            let home_dir = home.join(".prometheus").join("policies");
            if home_dir.join("skill-mutation.cedar").exists() {
                return Some(home_dir);
            }
        }

        None
    }

    /// Display the currently loaded policies as a formatted string.
    pub fn display_policies(&self) -> String {
        format!("{}", self.policies)
    }

    /// Get the number of policies currently loaded.
    pub fn policy_count(&self) -> usize {
        self.policies.policies().count()
    }
}

/// Default Cedar policies for the Prometheus skill pack.
pub const DEFAULT_POLICIES: &str = r#"
// Allow all operations in development
permit(
    principal,
    action,
    resource
) when {
    context.environment == "development"
};

// Staging: require validation for mutations
permit(
    principal,
    action == Action::"skill.mutate",
    resource
) when {
    context.environment == "staging" &&
    context has validation_passed &&
    context.validation_passed == true
};

// Staging: require human approval for promotions
permit(
    principal,
    action == Action::"skill.promote",
    resource
) when {
    context.environment == "staging" &&
    context has human_approved &&
    context.human_approved == true &&
    context has test_pass_rate &&
    context.test_pass_rate >= 0.95
};

// Trace capture: always allow with governance consent
permit(
    principal,
    action == Action::"trace.capture",
    resource
) when {
    context has governance_consent &&
    context.governance_consent == true
};

// Production: deny all mutations by default
forbid(
    principal,
    action == Action::"skill.mutate",
    resource
) when {
    context.environment == "production"
};

forbid(
    principal,
    action == Action::"skill.generate",
    resource
) when {
    context.environment == "production"
};
"#;
