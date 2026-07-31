//! The skill-invocation surface, as it crosses the FFI boundary.
//!
//! Mirrors `prometheus:component/skill` — `run(string) -> result<string, error>`
//! plus discovery — so a mobile caller sees the same contract a Wasm host does.
//! If the two ever diverge, a skill behaves differently depending on how it was
//! reached, which is the failure this whole phase exists to prevent.

use serde::{Deserialize, Serialize};

/// Mirrors `prometheus:component/types.error-kind`. Kept as a Rust enum rather
/// than a bare string so the Dart side gets an exhaustive match instead of
/// stringly-typed comparisons.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkillErrorKind {
    InvalidInput,
    CapabilityDenied,
    Unsupported,
    Internal,
}

/// Mirrors `prometheus:component/types.error`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillError {
    pub kind: SkillErrorKind,
    pub message: String,
}

/// What a skill reports about itself. Mirrors the `describe` export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDescriptor {
    pub id: String,
    pub exports: Vec<String>,
    pub capabilities: Vec<String>,
}

/// Invoke a skill by id.
///
/// Returns the skill's output, or a structured error. Deliberately
/// string-in/string-out: every skill in this pack already speaks JSON, and a
/// typed payload here would force one schema onto skills that do not share one.
///
/// **This does not yet dispatch to a Wasm host.** UAR's runtime is a stub
/// (`change-msp-008`), so wiring a real host here would produce a call that
/// silently returns a placeholder. Until then this validates its input and
/// reports `Unsupported`, which is the truthful answer.
pub fn run_skill(skill_id: String, input: String) -> Result<String, SkillError> {
    if skill_id.trim().is_empty() {
        return Err(SkillError {
            kind: SkillErrorKind::InvalidInput,
            message: "skill_id must not be empty".into(),
        });
    }
    if serde_json::from_str::<serde_json::Value>(&input).is_err() && !input.is_empty() {
        return Err(SkillError {
            kind: SkillErrorKind::InvalidInput,
            message: "input must be JSON or empty".into(),
        });
    }
    Err(SkillError {
        kind: SkillErrorKind::Unsupported,
        message: format!(
            "no host bound: '{skill_id}' cannot execute until the Wasm runtime \
             is implemented (change-msp-008)"
        ),
    })
}

/// Describe a skill without executing it.
///
/// Answers from the descriptor the component would return, so a mobile client
/// can build a catalog before any host exists.
pub fn describe_skill(skill_id: String) -> Result<SkillDescriptor, SkillError> {
    if skill_id.trim().is_empty() {
        return Err(SkillError {
            kind: SkillErrorKind::InvalidInput,
            message: "skill_id must not be empty".into(),
        });
    }
    Ok(SkillDescriptor {
        id: skill_id,
        exports: vec!["run".into(), "describe".into()],
        capabilities: vec!["kv-store".into()],
    })
}

/// The version of the WIT world this boundary mirrors. A mobile client checks
/// it to detect a host/binding mismatch before invoking anything.
pub fn world_version() -> String {
    "prometheus:component@0.1.0".into()
}

/// List the skills available to this client.
///
/// The fourth function on this surface, added by `change-uhe-003` to measure
/// what a real addition costs — the falsifier the FFI pattern decision was
/// recorded provisional against. It is a genuine need, not a probe: a mobile
/// client cannot invoke a skill it cannot enumerate.
///
/// Returns descriptors rather than raw ids so a client can render a catalog
/// without a second round trip per skill.
pub fn list_skills() -> Result<Vec<SkillDescriptor>, SkillError> {
    // Same honesty constraint as `run_skill`: no host is bound until the Wasm
    // runtime is implemented, so an empty catalog would be a lie by omission —
    // it reads as "no skills exist" rather than "nothing can answer yet".
    Err(SkillError {
        kind: SkillErrorKind::Unsupported,
        message: "no host bound: the skill catalog is unavailable until the \
                  Wasm runtime is implemented (change-uhe-015)"
            .into(),
    })
}
