use prometheus_exec_contracts::{
    hash_bytes, hash_serializable, ArtifactReference, ComponentAuthorization, Digest,
    ExecutionFailure, RunState, TierWReplayResult,
};
use serde::Serialize;

use crate::{
    engine::WASMTIME_VERSION, CapabilityGrant, TierWError, TierWExecutionOutcome, TierWLimits,
    ValidatedComponent, COMPONENT_WORLD,
};

const PROJECTION_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TierWDeterministicReceiptProjection {
    pub schema_version: u32,
    pub component_hash: Digest,
    pub component_world: String,
    pub authorization: ComponentAuthorization,
    pub engine_version: String,
    pub capability_hash: Digest,
    pub limits_hash: Digest,
    pub input_hash: Digest,
    pub state: RunState,
    pub output_hash: Digest,
    pub artifacts: Vec<ArtifactReference>,
    pub logs_hash: Digest,
    pub random_bytes_consumed: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure: Option<ExecutionFailure>,
}

impl TierWExecutionOutcome {
    pub fn deterministic_receipt_projection(
        &self,
        component: &ValidatedComponent,
        grant: &CapabilityGrant,
        limits: &TierWLimits,
        input: &str,
    ) -> Result<TierWDeterministicReceiptProjection, TierWError> {
        let (state, output_hash, artifacts, logs_hash, random_bytes_consumed, failure) = match self
        {
            Self::Succeeded(success) => {
                let artifacts = success
                    .artifacts
                    .iter()
                    .map(|(path, bytes)| ArtifactReference {
                        path: path.clone(),
                        hash: hash_bytes(bytes),
                        size_bytes: Some(bytes.len() as u64),
                    })
                    .collect();
                (
                    success.state,
                    hash_bytes(success.output.as_bytes()),
                    artifacts,
                    hash_serializable(&success.logs)?,
                    success.random_bytes_consumed,
                    None,
                )
            }
            Self::Failed(failure) => (
                failure.state,
                hash_bytes(&[]),
                Vec::new(),
                hash_serializable(&Vec::<crate::HostLogEntry>::new())?,
                0,
                Some(failure.failure.clone()),
            ),
        };
        Ok(TierWDeterministicReceiptProjection {
            schema_version: PROJECTION_SCHEMA_VERSION,
            component_hash: component.component_hash().clone(),
            component_world: COMPONENT_WORLD.into(),
            authorization: component.authorization().clone(),
            engine_version: WASMTIME_VERSION.into(),
            capability_hash: hash_serializable(grant)?,
            limits_hash: hash_serializable(limits)?,
            input_hash: hash_bytes(input.as_bytes()),
            state,
            output_hash,
            artifacts,
            logs_hash,
            random_bytes_consumed,
            failure,
        })
    }
}

impl TierWDeterministicReceiptProjection {
    pub fn canonical_hash(&self) -> Result<Digest, TierWError> {
        Ok(hash_serializable(self)?)
    }

    pub fn compare(&self, actual: &Self) -> TierWReplayResult {
        let mut checks = Vec::new();
        let mut mismatches = Vec::new();
        compare_field(
            "schemaVersion",
            &self.schema_version,
            &actual.schema_version,
            &mut checks,
            &mut mismatches,
        );
        compare_field(
            "componentHash",
            &self.component_hash,
            &actual.component_hash,
            &mut checks,
            &mut mismatches,
        );
        compare_field(
            "componentWorld",
            &self.component_world,
            &actual.component_world,
            &mut checks,
            &mut mismatches,
        );
        compare_field(
            "authorization",
            &self.authorization,
            &actual.authorization,
            &mut checks,
            &mut mismatches,
        );
        compare_field(
            "engineVersion",
            &self.engine_version,
            &actual.engine_version,
            &mut checks,
            &mut mismatches,
        );
        compare_field(
            "capabilityHash",
            &self.capability_hash,
            &actual.capability_hash,
            &mut checks,
            &mut mismatches,
        );
        compare_field(
            "limitsHash",
            &self.limits_hash,
            &actual.limits_hash,
            &mut checks,
            &mut mismatches,
        );
        compare_field(
            "inputHash",
            &self.input_hash,
            &actual.input_hash,
            &mut checks,
            &mut mismatches,
        );
        compare_field(
            "state",
            &self.state,
            &actual.state,
            &mut checks,
            &mut mismatches,
        );
        compare_field(
            "outputHash",
            &self.output_hash,
            &actual.output_hash,
            &mut checks,
            &mut mismatches,
        );
        compare_field(
            "artifacts",
            &self.artifacts,
            &actual.artifacts,
            &mut checks,
            &mut mismatches,
        );
        compare_field(
            "logsHash",
            &self.logs_hash,
            &actual.logs_hash,
            &mut checks,
            &mut mismatches,
        );
        compare_field(
            "randomBytesConsumed",
            &self.random_bytes_consumed,
            &actual.random_bytes_consumed,
            &mut checks,
            &mut mismatches,
        );
        compare_field(
            "failure",
            &self.failure,
            &actual.failure,
            &mut checks,
            &mut mismatches,
        );
        TierWReplayResult {
            valid: mismatches.is_empty(),
            checks,
            mismatches,
        }
    }
}

fn compare_field<T: PartialEq>(
    name: &str,
    expected: &T,
    actual: &T,
    checks: &mut Vec<String>,
    mismatches: &mut Vec<String>,
) {
    if expected == actual {
        checks.push(name.into());
    } else {
        mismatches.push(name.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[cfg(all(feature = "cranelift", feature = "pulley"))]
    use crate::{EngineProfile, TierWEngine, TierWLimits};

    #[cfg(all(feature = "cranelift", feature = "pulley"))]
    const REFERENCE_COMPONENT: &[u8] = include_bytes!(
        "../../../skills/react/prometheus-entity-skills/entity-graph-optimize/skill.wasm"
    );

    #[cfg(all(feature = "cranelift", feature = "pulley"))]
    const MARKERS: [&str; 4] = [
        ".kbd-orchestrator/project.json",
        ".evolver/",
        ".refiner/",
        "openspec/",
    ];

    #[cfg(all(feature = "cranelift", feature = "pulley"))]
    #[test]
    fn reference_property_corpus_matches_under_cranelift_and_pulley() {
        let cranelift = TierWEngine::new(EngineProfile::desktop()).unwrap();
        let pulley = TierWEngine::new(EngineProfile::portable_replay()).unwrap();
        let authorizer = crate::test_authorizer(REFERENCE_COMPONENT);
        let cranelift_component = cranelift
            .load_component(REFERENCE_COMPONENT, &authorizer)
            .unwrap();
        let pulley_component = pulley
            .load_component(REFERENCE_COMPONENT, &authorizer)
            .unwrap();
        let limits = TierWLimits::default();

        for marker_mask in 0u8..16 {
            let grant = MARKERS.iter().enumerate().fold(
                CapabilityGrant::empty(),
                |grant, (index, marker)| {
                    let value = (marker_mask & (1 << index) != 0).then(|| "present".into());
                    grant.allow_kv_read(*marker, value)
                },
            );
            let input = format!("{{\"propertyCase\":{marker_mask}}}");
            let cranelift_outcome = cranelift.execute_component(
                &cranelift_component,
                &authorizer,
                grant.clone(),
                &limits,
                &input,
            );
            let pulley_outcome = pulley.execute_component(
                &pulley_component,
                &authorizer,
                grant.clone(),
                &limits,
                &input,
            );
            let cranelift_projection = cranelift_outcome
                .deterministic_receipt_projection(&cranelift_component, &grant, &limits, &input)
                .unwrap();
            let pulley_projection = pulley_outcome
                .deterministic_receipt_projection(&pulley_component, &grant, &limits, &input)
                .unwrap();
            assert_eq!(cranelift_projection, pulley_projection);
            assert_eq!(
                cranelift_projection.canonical_hash().unwrap(),
                pulley_projection.canonical_hash().unwrap()
            );
            let comparison = cranelift_projection.compare(&pulley_projection);
            assert!(
                comparison.valid,
                "marker mask {marker_mask}: {comparison:?}"
            );
            assert!(comparison.mismatches.is_empty());
        }

        let grant = MARKERS
            .iter()
            .fold(CapabilityGrant::empty(), |grant, marker| {
                grant.allow_kv_read(*marker, None)
            });
        let exhausted = TierWLimits {
            fuel: 1,
            ..TierWLimits::default()
        };
        let cranelift_failure = cranelift.execute_component(
            &cranelift_component,
            &authorizer,
            grant.clone(),
            &exhausted,
            "{}",
        );
        let pulley_failure = pulley.execute_component(
            &pulley_component,
            &authorizer,
            grant.clone(),
            &exhausted,
            "{}",
        );
        assert_eq!(
            cranelift_failure
                .deterministic_receipt_projection(&cranelift_component, &grant, &exhausted, "{}",)
                .unwrap(),
            pulley_failure
                .deterministic_receipt_projection(&pulley_component, &grant, &exhausted, "{}",)
                .unwrap()
        );
    }

    #[test]
    fn comparison_reports_the_exact_deterministic_field() {
        let baseline = fixture_projection(b"output");
        assert!(baseline.compare(&baseline).valid);
        let mut changed = baseline.clone();
        changed.output_hash = hash_bytes(b"changed");
        let result = baseline.compare(&changed);
        assert!(!result.valid);
        assert_eq!(result.mismatches, ["outputHash"]);
        assert!(!result.checks.contains(&"outputHash".into()));
    }

    proptest! {
        #[test]
        fn any_changed_output_bytes_change_the_projection_hash(
            left in proptest::collection::vec(any::<u8>(), 0..1024),
            suffix in any::<u8>(),
        ) {
            let mut right = left.clone();
            right.push(suffix);
            let left = fixture_projection(&left);
            let right = fixture_projection(&right);
            prop_assert_ne!(&left.output_hash, &right.output_hash);
            prop_assert_ne!(left.canonical_hash().unwrap(), right.canonical_hash().unwrap());
            prop_assert_eq!(left.compare(&right).mismatches, ["outputHash"]);
        }
    }

    fn fixture_projection(output: &[u8]) -> TierWDeterministicReceiptProjection {
        TierWDeterministicReceiptProjection {
            schema_version: PROJECTION_SCHEMA_VERSION,
            component_hash: hash_bytes(b"component"),
            component_world: COMPONENT_WORLD.into(),
            authorization: ComponentAuthorization {
                mode: prometheus_exec_contracts::ComponentAuthorizationMode::HashPin,
                world: COMPONENT_WORLD.into(),
                manifest_hash: None,
                generation_id: None,
            },
            engine_version: WASMTIME_VERSION.into(),
            capability_hash: hash_bytes(b"capability"),
            limits_hash: hash_bytes(b"limits"),
            input_hash: hash_bytes(b"input"),
            state: RunState::Succeeded,
            output_hash: hash_bytes(output),
            artifacts: Vec::new(),
            logs_hash: hash_bytes(b"logs"),
            random_bytes_consumed: 0,
            failure: None,
        }
    }
}
