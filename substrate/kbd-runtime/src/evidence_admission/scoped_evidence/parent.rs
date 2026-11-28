//! Exact, immutable parent obligation construction for the proposed fresh
//! scoped protocol. No adoption writer, activation or runner is connected.
//! A v2 parent has only descriptive supportedEnvironments, never an implicit
//! scenario-by-target denominator. Do not reinterpret existing contracts.

use super::{name_is_valid, ArtifactCheckKind, ArtifactExpectation, ObligationKey, Obligations, RequiredObservation};
use super::super::{refused, sha256_hex};
use crate::phase_closure::{is_digest, ContractBinding};
use crate::Result;
use crate::scope_authority::{self, ScopeAuthorityInput, ScopeAuthorityRecord, SCOPE_AUTHORITY_NAMESPACE};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

// The fresh format requires an explicit parent-bound target matrix. It is
// deliberately not accepted by the current v1/v2 adoption writer.
const SCOPED_CONTRACT_VERSION: &str = "3";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ScopedEvidenceRequirements {
    schema_version: String,
    // A list (not an object map) so repeated scenario IDs cannot be collapsed
    // into a last-wins value by deserialization.
    required_targets: Vec<ScenarioTargets>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ScenarioTargets {
    scenario_id: String,
    // Required for an explicitly signed interpretation of a V2 bound row.
    scenario_digest: Option<String>,
    targets: Vec<TargetSlot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TargetSlot {
    id: String,
    environment: String,
    target_identity_sha256: String,
    #[serde(default)]
    artifact_checks: Vec<ArtifactExpectation>,
    // The existing native product observer, independently qualified and
    // pinned by a signed interpretation, can be reused instead of inventing
    // byte-equality artifacts for dynamic product truth.
    native_observer: Option<NativeObserverBinding>,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct NativeObserverBinding {
    profile_sha256: String,
    verifier_source_sha256: String,
    witness_schema: String,
    // Explicit reviewed complete native prerequisite mapping, not all
    // unrelated parent builds or an inferred command text match.
    prerequisite_command_ids: Vec<String>,
    required_checks: Vec<NativeObserverCheck>,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct NativeObserverCheck {
    kind: ArtifactCheckKind,
    requirement_id: String,
    observer_requirement_sha256: String,
}

/// Separately signed interpretation of the immutable existing V2 parent.
/// It adds explicit target/observer requirements, never edits the parent,
/// owner, denominator or scenario digest, and conveys no execution authority.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct InterpretationManifest {
    schema_version: String,
    kind: String,
    namespace: String,
    action: String,
    project_id: String,
    run_id: String,
    phase_id: String,
    contract_digest: String,
    binding_revision: u32,
    requirements: ScopedEvidenceRequirements,
    independent_mapping_review_sha256: String,
    approver: String,
    timestamp: String,
    nonce: String,
    reason: String,
}
struct VerifiedInterpretation {
    manifest: InterpretationManifest,
    authority: ScopeAuthorityRecord,
}

/// The actual independent mapping review must establish the complete
/// per-scenario real-target applicability AND verify V2 row digest identities
/// using the existing native adoption normalization over the exact snapshot.
/// No caller-deserialized receipt or arbitrary digest constructs this witness.
struct VerifiedInterpretationReview {
    // Review the exact mapping statement before its review receipt is
    // embedded in the final owner-signed envelope.
    statement_sha256: String,
    parent_contract_digest: String,
    binding_revision: u32,
    scenario_digests: BTreeMap<String, String>,
    review_receipt_sha256: String,
}

fn verify_v2_interpretation(
    input: &ScopeAuthorityInput,
    binding: &ContractBinding,
    review: &VerifiedInterpretationReview,
) -> Result<VerifiedInterpretation> {
    let raw = &input.manifest_raw;
    let fail = |why: &str| crate::RuntimeError::ScopeAuthority(format!("scoped-interpretation:{why}"));
    if raw.chars().any(char::is_control) { return Err(fail("nonprintable")); }
    let value: Value = serde_json::from_str(raw).map_err(|_| fail("invalid-json"))?;
    if serde_jcs::to_string(&value).map_err(|_| fail("canonicalization"))? != *raw {
        return Err(fail("noncanonical-manifest"));
    }
    // Domain-separated commitment to the whole mapping/parent statement
    // excluding only the review-receipt digest. This avoids an impossible
    // circular hash between the reviewed statement and signed envelope.
    let mut statement = value.clone();
    statement.as_object_mut().ok_or_else(|| fail("invalid-shape"))?
        .remove("independentMappingReviewSha256")
        .ok_or_else(|| fail("missing-review-reference"))?;
    let statement_bytes = serde_jcs::to_vec(&statement).map_err(|_| fail("statement-canonicalization"))?;
    let statement_sha256 = sha256_hex(&statement_bytes);
    let manifest: InterpretationManifest = serde_json::from_value(value).map_err(|_| fail("invalid-shape"))?;
    if manifest.schema_version != "1" || manifest.kind != "kbd.scoped-parent-interpretation"
        || manifest.namespace != SCOPE_AUTHORITY_NAMESPACE || manifest.action != "interpret-evidence-obligations"
        || !is_digest(&manifest.contract_digest) || manifest.binding_revision == 0
        || manifest.contract_digest != binding.contract_digest
        || manifest.binding_revision != binding.binding_revision
        || review.parent_contract_digest != binding.contract_digest
        || review.binding_revision != binding.binding_revision
        || review.scenario_digests != binding.scenario_digests
        || review.statement_sha256 != statement_sha256
        || manifest.independent_mapping_review_sha256 != review.review_receipt_sha256
        || !is_digest(&review.review_receipt_sha256)
        || manifest.reason.trim().chars().count() < 10
        || chrono::DateTime::parse_from_rfc3339(&manifest.timestamp).is_err()
        || ![manifest.project_id.as_str(), manifest.run_id.as_str(), manifest.phase_id.as_str(),
            manifest.approver.as_str(), manifest.nonce.as_str()].into_iter().all(name_is_valid)
    {
        return Err(fail("invalid-identity"));
    }
    let authority = scope_authority::verify_canonical_payload(input, &manifest.approver)?;
    Ok(VerifiedInterpretation { manifest, authority })
}

/// Internal parent-owned denominator, derived only from the exact bound
/// snapshot. Its binding must be supplied by the future canonical producer,
/// not by a client. Verifying a digest is not authenticating a binding.
pub(super) struct ScopedParent<'a> {
    project_id: String,
    run_id: String,
    phase_id: String,
    binding: &'a ContractBinding,
    interpretation_sha256: String,
    required: Vec<RequiredObservation>,
}

impl<'a> ScopedParent<'a> {
    /// Only exact canonical snapshot bytes may be used. The proposed version
    /// uses JCS for both parent bytes and scenario row digests. No v1/v2
    /// compatibility or migration is implied. Future adoption must retain
    /// all existing full-contract structural/review/owner lint.
    pub(super) fn from_bound_snapshot(
        bytes: &[u8],
        project_id: &str,
        run_id: &str,
        phase_id: &str,
        binding: &'a ContractBinding,
    ) -> Result<Self> {
        Self::derive(bytes, project_id, run_id, phase_id, binding, None)
    }

    /// Exact existing V2 parent bytes remain the authoritative obligations.
    /// The independently signed interpretation is a separate verified input;
    /// it neither modifies the signed parent nor activates an existing phase.
    fn from_bound_v2_snapshot(
        bytes: &[u8], project_id: &str, run_id: &str, phase_id: &str,
        binding: &'a ContractBinding, interpretation: &VerifiedInterpretation,
    ) -> Result<Self> {
        Self::derive(bytes, project_id, run_id, phase_id, binding, Some(interpretation))
    }

    fn derive(
        bytes: &[u8], project_id: &str, run_id: &str, phase_id: &str,
        binding: &'a ContractBinding, interpretation: Option<&VerifiedInterpretation>,
    ) -> Result<Self> {
        let reject = |reason: &str| refused(phase_id, format!("scoped-parent:{reason}"));
        if ![project_id, run_id, phase_id].into_iter().all(name_is_valid)
            || binding.binding_revision == 0
            || !is_digest(&binding.contract_digest)
            || sha256_hex(bytes) != binding.contract_digest
        {
            return Err(reject("binding-identity-or-digest"));
        }
        let snapshot: Value = serde_json::from_slice(bytes).map_err(|_| reject("invalid-json"))?;
        let v2 = interpretation.is_some();
        let version = if v2 { "2" } else { SCOPED_CONTRACT_VERSION };
        // V2 uses the existing exact jq-normalized snapshot and binding row
        // identities. Never silently replace those with JCS row identities.
        if !v2 && serde_jcs::to_vec(&snapshot).map_err(|_| reject("canonicalization"))? != bytes {
            return Err(reject("noncanonical-snapshot"));
        }
        if let Some(verified) = interpretation {
            let m = &verified.manifest;
            scope_authority::verify_record_identity(&verified.authority)?;
            if m.project_id != project_id || m.run_id != run_id || m.phase_id != phase_id
                || m.contract_digest != binding.contract_digest || m.binding_revision != binding.binding_revision
                || m.approver != verified.authority.approver {
                return Err(reject("interpretation-parent-mismatch"));
            }
        }
        if snapshot["kind"].as_str() != Some("kbd.adopted-contract")
            || snapshot["contractVersion"].as_str() != Some(version)
            || snapshot["identity"]["projectId"].as_str() != Some(project_id)
            || snapshot["identity"]["runId"].as_str() != Some(run_id)
            || snapshot["identity"]["phaseId"].as_str() != Some(phase_id)
        {
            return Err(reject("unsupported-version-or-identity"));
        }

        let scenarios = snapshot["scenarioMatrix"].as_array().ok_or_else(|| reject("missing-scenarios"))?;
        let mut by_scenario = BTreeMap::<&str, &Value>::new();
        for row in scenarios {
            let id = text(row, "id").ok_or_else(|| reject("invalid-scenario-id"))?;
            let bound_digest = binding.scenario_digests.get(id)
                .filter(|digest| is_digest(digest)).ok_or_else(|| reject("unbound-scenario"))?;
            if by_scenario.insert(id, row).is_some() {
                return Err(reject("duplicate-or-unbound-scenario"));
            }
            if !v2 {
                let digest = serde_jcs::to_vec(row).map(|value| sha256_hex(&value))
                    .map_err(|_| reject("scenario-canonicalization"))?;
                if bound_digest != &digest { return Err(reject("unbound-scenario-digest")); }
            }
            // These are the immutable requirements, not proof of their
            // execution. Full adoption lint still has to validate semantics.
            if ["ownerTask", "productionEntry", "fixtures", "evidence", "restoration"]
                .into_iter().any(|field| text(row, field).is_none())
                || ["observation", "expectedTruthSource", "comparator"]
                    .into_iter().any(|field| text(&row["oracle"], field).is_none())
                || ["input", "expectedRejection", "commandRef"]
                    .into_iter().any(|field| text(&row["negativeControl"], field).is_none())
            {
                return Err(reject("incomplete-scenario-requirement"));
            }
        }
        if by_scenario.is_empty()
            || !by_scenario.keys().copied().eq(binding.scenario_digests.keys().map(String::as_str))
        {
            return Err(reject("incomplete-parent-scenarios"));
        }

        let commands = snapshot["boundaryPacket"]["commands"].as_array()
            .ok_or_else(|| reject("missing-commands"))?;
        let mut command_index = BTreeMap::<&str, &Value>::new();
        let mut cases = BTreeMap::<&str, Vec<&Value>>::new();
        let mut controls = BTreeMap::<&str, Vec<&Value>>::new();
        for command in commands {
            let id = text(command, "id").ok_or_else(|| reject("invalid-command-id"))?;
            let kind = text(command, "kind").ok_or_else(|| reject("invalid-command-kind"))?;
            if !matches!(kind, "build" | "run" | "scenario" | "negativeControl")
                || command_index.insert(id, command).is_some()
                || command["argv"].as_array().is_none_or(|argv|
                    argv.is_empty() || argv.iter().any(|arg| !arg.is_string()))
            {
                return Err(reject("invalid-or-duplicate-command"));
            }
            let refs = match command.get("scenarioIds") {
                Some(value) => value.as_array().ok_or_else(|| reject("invalid-command-scope"))?.as_slice(),
                None => &[],
            };
            if kind == "negativeControl" && v2 && refs.is_empty() {
                // Existing V2 allows a primary control reference without a
                // redundant scenarioIds list. Preserve the signed reference;
                // never infer any other applicability or relax explicit scope.
                let primary: Vec<_> = by_scenario.iter().filter_map(|(scenario_id,row)|
                    (row["negativeControl"]["commandRef"].as_str() == Some(id)).then_some(*scenario_id)).collect();
                if primary.is_empty() { return Err(reject("unmapped-control-command")); }
                for scenario in primary { controls.entry(scenario).or_default().push(command); }
                continue;
            }
            if matches!(kind, "scenario" | "negativeControl") && refs.is_empty() {
                return Err(reject("unmapped-case-or-control-command"));
            }
            let mut seen = BTreeSet::new();
            for value in refs {
                let scenario = value.as_str().filter(|id| by_scenario.contains_key(*id))
                    .ok_or_else(|| reject("unknown-command-scenario"))?;
                if !seen.insert(scenario) {
                    return Err(reject("duplicate-command-scenario"));
                }
                match kind {
                    "scenario" => cases.entry(scenario).or_default().push(command),
                    "negativeControl" => controls.entry(scenario).or_default().push(command),
                    _ => {}
                }
            }
        }

        let checks = snapshot["requiredSafetyChecks"].as_array().filter(|checks| !checks.is_empty())
            .ok_or_else(|| reject("missing-safety-checks"))?;
        let mut check_ids = BTreeSet::new();
        let mut safety = BTreeMap::<&str, Vec<&Value>>::new();
        for check in checks {
            let id = text(check, "id").ok_or_else(|| reject("invalid-safety-id"))?;
            if !check_ids.insert(id) || text(check, "description").is_none() {
                return Err(reject("invalid-or-duplicate-safety-check"));
            }
            let refs = check["scenarioIds"].as_array().filter(|refs| !refs.is_empty())
                .ok_or_else(|| reject("unmapped-safety-check"))?;
            let mut seen = BTreeSet::new();
            for value in refs {
                let scenario = value.as_str().filter(|id| by_scenario.contains_key(*id))
                    .ok_or_else(|| reject("unknown-safety-scenario"))?;
                if !seen.insert(scenario) {
                    return Err(reject("duplicate-safety-scenario"));
                }
                safety.entry(scenario).or_default().push(check);
            }
        }

        let matrix: ScopedEvidenceRequirements = match interpretation {
            Some(verified) => verified.manifest.requirements.clone(),
            None => serde_json::from_value(snapshot["scopedEvidence"].clone())
                .map_err(|_| reject("missing-or-invalid-target-matrix"))?,
        };
        if matrix.schema_version != "1" { return Err(reject("unsupported-target-matrix-version")); }
        let environments = snapshot["releaseClaim"]["supportedEnvironments"].as_array()
            .ok_or_else(|| reject("missing-release-environments"))?;
        let mut expected_environments = BTreeSet::new();
        for value in environments {
            let environment = value.as_str().filter(|name| name_is_valid(name))
                .ok_or_else(|| reject("invalid-release-environment"))?;
            if !expected_environments.insert(environment) {
                return Err(reject("duplicate-release-environment"));
            }
        }
        if expected_environments.is_empty() { return Err(reject("empty-release-environments")); }

        let mut mapped = BTreeSet::new();
        let mut used_environments = BTreeSet::new();
        let mut stable_targets = BTreeMap::<String, (String, String, Option<String>)>::new();
        let mut required = Vec::new();
        for row in &matrix.required_targets {
            let scenario_id = row.scenario_id.as_str();
            let scenario = by_scenario.get(scenario_id).ok_or_else(|| reject("unknown-target-scenario"))?;
            if !mapped.insert(scenario_id) || row.targets.is_empty()
                || (v2 && row.scenario_digest.as_ref() != binding.scenario_digests.get(scenario_id))
                || row.scenario_digest.as_ref().is_some_and(|digest| Some(digest) != binding.scenario_digests.get(scenario_id))
            {
                return Err(reject("duplicate-empty-or-unbound-scenario-targets"));
            }
            let case_commands = cases.get(scenario_id).filter(|entries| !entries.is_empty())
                .ok_or_else(|| reject("missing-case-command"))?;
            let control_ref = text(&scenario["negativeControl"], "commandRef")
                .ok_or_else(|| reject("missing-control-reference"))?;
            let applicable_controls = controls.get(scenario_id).filter(|entries| !entries.is_empty())
                .ok_or_else(|| reject("missing-control-command"))?;
            if !applicable_controls.iter().any(|command| command["id"].as_str() == Some(control_ref)) {
                return Err(reject("control-reference-not-applicable"));
            }
            // The parent must have required safety checks. A particular row
            // may have none mapped to it; the full parent determines that
            // applicability, not the packet or a caller selection.
            let safety_checks = safety.get(scenario_id).map(Vec::as_slice).unwrap_or(&[]);
            let mut target_ids = BTreeSet::new();
            for target in &row.targets {
                if !name_is_valid(&target.id)
                    || !is_digest(&target.target_identity_sha256)
                    || !expected_environments.contains(target.environment.as_str())
                    || !target_ids.insert(target.id.as_str())
                {
                    return Err(reject("invalid-or-duplicate-target-slot"));
                }
                let target_identity = (target.environment.clone(), target.target_identity_sha256.clone(),
                    target.native_observer.as_ref().map(|observer| observer.profile_sha256.clone()));
                if let Some(previous) = stable_targets.get(&target.id) {
                    if previous != &target_identity { return Err(reject("conflicting-target-slot")); }
                } else { stable_targets.insert(target.id.clone(), target_identity); }
                // Initial machine-comparable proof adapter: every mapped
                // case, control, safety check and restoration has an explicit
                // expected artifact in the immutable independent-truth plan.
                // Unsupported/unmapped proof is never treated as success.
                // This existing V2/native-observer continuation cannot
                // replace the real journey with a transport-only artifact.
                if v2 && target.native_observer.is_none() {
                    return Err(reject("v2-native-observer-required"));
                }
                match &target.native_observer {
                    Some(observer) => {
                        if !target.artifact_checks.is_empty() || !is_digest(&observer.profile_sha256)
                            || !is_digest(&observer.verifier_source_sha256) || !name_is_valid(&observer.witness_schema) {
                            return Err(reject("invalid-native-observer-binding"));
                        }
                        let mut prerequisites = BTreeSet::new();
                        for id in &observer.prerequisite_command_ids {
                            let command = command_index.get(id.as_str()).ok_or_else(|| reject("unknown-prerequisite-command"))?;
                            if !prerequisites.insert(id.as_str())
                                || !matches!(command["kind"].as_str(), Some("build" | "run"))
                                || command.get("scenarioIds").and_then(Value::as_array)
                                    .is_some_and(|ids| !ids.is_empty() && !ids.iter().any(|value| value.as_str() == Some(scenario_id)))
                            {
                                return Err(reject("prerequisite-scope-or-kind"));
                            }
                        }
                        validate_native_observer_checks(observer, case_commands, applicable_controls,
                            safety_checks, scenario_id).map_err(|reason| reject(reason))?;
                    }
                    None => validate_artifact_expectations(&target.artifact_checks, case_commands,
                        applicable_controls, safety_checks, scenario_id).map_err(|reason| reject(reason))?,
                }
                used_environments.insert(target.environment.as_str());
                // Full requirement, exact adopted parent and stable target slot.
                // Parent hash also binds all context outside this row (budget,
                // completeness, ordering and cleanup), without a packet subset.
                let descriptor = json!({
                    "schema": "kbd.scoped-obligation/v1",
                    "parentDigest": binding.contract_digest,
                    "interpretationSha256": interpretation.map(|verified| &verified.authority.manifest_sha256),
                    "scenario": scenario,
                    "caseCommands": case_commands,
                    "controlCommands": applicable_controls,
                    "safetyChecks": safety_checks,
                    "target": target
                });
                let bytes = serde_jcs::to_vec(&descriptor).map_err(|_| reject("obligation-canonicalization"))?;
                required.push(RequiredObservation {
                    key: ObligationKey { scenario_id: row.scenario_id.clone(), target_id: target.id.clone() },
                    scenario_digest: binding.scenario_digests[scenario_id].clone(),
                    target_identity_sha256: target.target_identity_sha256.clone(),
                    requirements_sha256: sha256_hex(&bytes),
                    artifact_checks: target.artifact_checks.clone(),
                    native_observer_profile_sha256: target.native_observer.as_ref().map(|observer| observer.profile_sha256.clone()),
                    case_command_ids: case_commands.iter().filter_map(|row| text(row,"id").map(str::to_owned)).collect(),
                    control_command_ids: applicable_controls.iter().filter_map(|row| text(row,"id").map(str::to_owned)).collect(),
                    safety_check_ids: safety_checks.iter().filter_map(|row| text(row,"id").map(str::to_owned)).collect(),
                    prerequisite_command_ids: target.native_observer.as_ref().map(|observer| observer.prerequisite_command_ids.clone()).unwrap_or_default(),
                });
            }
        }
        if !mapped.iter().copied().eq(by_scenario.keys().copied())
            || used_environments != expected_environments
        {
            return Err(reject("incomplete-scenario-or-environment-denominator"));
        }
        Ok(Self {
            project_id: project_id.into(),
            run_id: run_id.into(),
            phase_id: phase_id.into(),
            binding,
            interpretation_sha256: interpretation.map_or_else(|| binding.contract_digest.clone(),
                |verified| verified.authority.manifest_sha256.clone()),
            required,
        })
    }

    pub(super) fn obligations<'b>(&'b self, current_candidate_sha256: &'b str) -> Obligations<'b> {
        Obligations {
            project_id: &self.project_id,
            run_id: &self.run_id,
            phase_id: &self.phase_id,
            binding: self.binding,
            current_candidate_sha256,
            interpretation_sha256: &self.interpretation_sha256,
            required: &self.required,
        }
    }
}

fn text<'a>(object: &'a Value, key: &str) -> Option<&'a str> {
    object[key].as_str().filter(|value| name_is_valid(value))
}

fn validate_artifact_expectations(
    artifacts: &[ArtifactExpectation],
    cases: &[&Value],
    controls: &[&Value],
    safety: &[&Value],
    scenario_id: &str,
) -> std::result::Result<(), &'static str> {
    let mut required = BTreeSet::<(ArtifactCheckKind, &str)>::new();
    for (kind, rows) in [
        (ArtifactCheckKind::Case, cases),
        (ArtifactCheckKind::NegativeControl, controls),
        (ArtifactCheckKind::Safety, safety),
    ] {
        for row in rows {
            let id = text(row, "id").ok_or("invalid-artifact-requirement")?;
            required.insert((kind, id));
        }
    }
    required.insert((ArtifactCheckKind::Cleanup, scenario_id));

    let mut mapped = BTreeSet::new();
    let mut unique = BTreeSet::new();
    for artifact in artifacts {
        let key = (artifact.kind, artifact.requirement_id.as_str());
        if !required.contains(&key)
            || !is_digest(&artifact.expected_sha256)
            || !name_is_valid(&artifact.truth_source)
            || prometheus_exec_contracts::validate_artifact_path(&artifact.artifact_path).is_err()
            || !unique.insert((key, artifact.artifact_path.as_str()))
        {
            return Err("invalid-or-duplicate-artifact-expectation");
        }
        mapped.insert(key);
    }
    if mapped != required { return Err("incomplete-artifact-expectations"); }
    Ok(())
}

fn validate_native_observer_checks(
    observer: &NativeObserverBinding, cases: &[&Value], controls: &[&Value],
    safety: &[&Value], scenario_id: &str,
) -> std::result::Result<(), &'static str> {
    let mut expected = BTreeSet::<(ArtifactCheckKind, &str)>::new();
    for (kind, rows) in [(ArtifactCheckKind::Case, cases),
        (ArtifactCheckKind::NegativeControl, controls), (ArtifactCheckKind::Safety, safety)] {
        for row in rows {
            expected.insert((kind, text(row, "id").ok_or("invalid-native-requirement")?));
        }
    }
    expected.insert((ArtifactCheckKind::Cleanup, scenario_id));
    let mut mapped = BTreeSet::new();
    for check in &observer.required_checks {
        let key = (check.kind, check.requirement_id.as_str());
        if !expected.contains(&key) || !is_digest(&check.observer_requirement_sha256)
            || !mapped.insert(key) {
            return Err("invalid-native-observer-check");
        }
    }
    if mapped != expected { return Err("incomplete-native-observer-checks"); }
    Ok(())
}
