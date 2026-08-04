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

fn indexed_descriptor(entry: prometheus_skill_index::SkillIndexEntry) -> SkillDescriptor {
    SkillDescriptor {
        id: entry.id,
        exports: vec!["run".into(), "describe".into()],
        capabilities: Vec::new(),
    }
}

/// Enumerate the exact signed generation index supplied by the host app.
///
/// The bytes are the same `indexes/skills.json` payload used by host search and
/// projected as `mobile/skill-index.json`; mobile does not rescan or reinterpret
/// `SKILL.md` files.
pub fn list_indexed_skills(index_json: String) -> Result<Vec<SkillDescriptor>, SkillError> {
    let index =
        prometheus_skill_index::SkillIndex::from_json(&index_json).map_err(|error| SkillError {
            kind: SkillErrorKind::InvalidInput,
            message: format!("invalid skill index: {error}"),
        })?;
    Ok(index.entries.into_iter().map(indexed_descriptor).collect())
}

/// Search with the same deterministic ranking implementation used by the host.
pub fn search_indexed_skills(
    index_json: String,
    query: String,
    limit: u32,
) -> Result<Vec<SkillDescriptor>, SkillError> {
    let index =
        prometheus_skill_index::SkillIndex::from_json(&index_json).map_err(|error| SkillError {
            kind: SkillErrorKind::InvalidInput,
            message: format!("invalid skill index: {error}"),
        })?;
    Ok(index
        .search(&query, limit as usize)
        .into_iter()
        .map(indexed_descriptor)
        .collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbdMobileCommitPayload {
    pub event_json: String,
    pub state_json: String,
    pub project_updates: Vec<u8>,
}

fn kbd_mobile_error(error: impl std::fmt::Display) -> SkillError {
    SkillError {
        kind: SkillErrorKind::Internal,
        message: format!("KBD mobile peer: {error}"),
    }
}

/// Return the explicit embedded-peer capability boundary. False capabilities
/// are security constraints, not unimplemented promises.
pub fn kbd_mobile_capabilities() -> String {
    serde_json::to_string(&kbd_mobile::MobileCapabilities::default())
        .expect("static KBD capability report must serialize")
}

/// Prepare a mobile event and return the exact bytes for the host secure-key
/// provider to sign. Private key material never crosses this FFI boundary.
pub fn kbd_mobile_prepare_command(
    project_id: String,
    replica_id: String,
    project_updates: Vec<u8>,
    command_json: String,
    signer_key_id: String,
    signer_public_key: String,
) -> Result<String, SkillError> {
    let project = kbd_mobile::MobileProject::from_updates(project_id, replica_id, &project_updates)
        .map_err(kbd_mobile_error)?;
    let command =
        serde_json::from_str::<kbd_runtime::CommandEnvelope>(&command_json).map_err(|e| {
            SkillError {
                kind: SkillErrorKind::InvalidInput,
                message: format!("invalid KBD command envelope: {e}"),
            }
        })?;
    let prepared = project
        .prepare_command(command, &signer_key_id, &signer_public_key)
        .map_err(kbd_mobile_error)?;
    serde_json::to_string(&prepared).map_err(kbd_mobile_error)
}

/// Verify a secure-host signature, commit the prepared event into Loro, and
/// return the updated authority bytes for host-controlled durable storage.
pub fn kbd_mobile_commit_prepared(
    project_id: String,
    replica_id: String,
    project_updates: Vec<u8>,
    prepared_json: String,
    signature_base64: String,
) -> Result<KbdMobileCommitPayload, SkillError> {
    let mut project =
        kbd_mobile::MobileProject::from_updates(project_id, replica_id, &project_updates)
            .map_err(kbd_mobile_error)?;
    let prepared = serde_json::from_str::<kbd_mobile::PreparedMobileEvent>(&prepared_json)
        .map_err(|e| SkillError {
            kind: SkillErrorKind::InvalidInput,
            message: format!("invalid prepared KBD event: {e}"),
        })?;
    let committed = project
        .commit_prepared(prepared, signature_base64)
        .map_err(kbd_mobile_error)?;
    Ok(KbdMobileCommitPayload {
        event_json: serde_json::to_string(&committed.event).map_err(kbd_mobile_error)?,
        state_json: serde_json::to_string(&committed.state).map_err(kbd_mobile_error)?,
        project_updates: committed.project_updates,
    })
}

/// Prepare the exact sovereign-sync-compatible `kbd-control:<project_id>`
/// envelope bytes for host signing before iroh broadcast.
pub fn kbd_mobile_prepare_delta(
    project_id: String,
    replica_id: String,
    project_updates: Vec<u8>,
    signer_key_id: String,
) -> Result<String, SkillError> {
    let project = kbd_mobile::MobileProject::from_updates(project_id, replica_id, &project_updates)
        .map_err(kbd_mobile_error)?;
    let prepared = project
        .prepare_signed_delta(signer_key_id)
        .map_err(kbd_mobile_error)?;
    serde_json::to_string(&prepared).map_err(kbd_mobile_error)
}

/// Attach a signature produced by the host secure-key provider and return the
/// wire-compatible envelope for `MobilePeer::broadcast`.
pub fn kbd_mobile_attach_delta_signature(
    prepared_json: String,
    signer_public_key: String,
    signature_base64: String,
) -> Result<Vec<u8>, SkillError> {
    let mut prepared = serde_json::from_str::<kbd_mobile::PreparedMobileDelta>(&prepared_json)
        .map_err(|e| SkillError {
            kind: SkillErrorKind::InvalidInput,
            message: format!("invalid prepared KBD delta: {e}"),
        })?;
    if prepared.delta.signable_bytes_for_host() != prepared.signing_payload {
        return Err(SkillError {
            kind: SkillErrorKind::InvalidInput,
            message: "prepared KBD delta signing payload was modified".into(),
        });
    }
    prepared
        .delta
        .attach_host_signature(&signer_public_key, signature_base64)
        .map_err(kbd_mobile_error)?;
    prepared.delta.encode().map_err(kbd_mobile_error)
}

/// Import one signed iroh payload into a host-persisted mobile authority.
pub fn kbd_mobile_import_signed_delta(
    project_id: String,
    replica_id: String,
    project_updates: Vec<u8>,
    signed_delta: Vec<u8>,
) -> Result<KbdMobileCommitPayload, SkillError> {
    let mut project =
        kbd_mobile::MobileProject::from_updates(project_id, replica_id, &project_updates)
            .map_err(kbd_mobile_error)?;
    let state = project
        .import_signed_delta(&signed_delta)
        .map_err(kbd_mobile_error)?;
    Ok(KbdMobileCommitPayload {
        event_json: String::new(),
        state_json: serde_json::to_string(&state).map_err(kbd_mobile_error)?,
        project_updates: project.export_updates().map_err(kbd_mobile_error)?,
    })
}
