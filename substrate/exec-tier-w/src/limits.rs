use std::collections::BTreeMap;

#[cfg(all(test, feature = "cranelift"))]
use std::time::Duration;

use prometheus_exec_contracts::{
    ExecutionFailure, ExecutionFailureKind, ExecutionLimits, RunState,
};
use serde::Serialize;
use wasmtime::{Store, StoreLimits, StoreLimitsBuilder, Trap};

use crate::{
    bindings::{prometheus::component::types::ErrorKind, Skill},
    capabilities::{
        capability_linker, validate_component_grants, CapabilityGrant, CapabilityHost,
        HostLimitFailure, HostLogEntry, ResourceLimitFailure,
    },
    engine::EPOCH_TICK_MILLIS,
    ComponentAuthorizer, TierWEngine, TierWError, ValidatedComponent,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TierWLimits {
    pub memory_bytes: usize,
    pub fuel: u64,
    pub wall_clock_ms: u64,
    pub max_table_elements: usize,
    pub max_instances: usize,
    pub max_tables: usize,
    pub max_memories: usize,
    pub max_stream_bytes: usize,
    pub max_artifact_bytes: usize,
    pub max_total_artifact_bytes: usize,
}

impl TierWLimits {
    pub fn from_contract(limits: &ExecutionLimits) -> Result<Self, TierWError> {
        if limits.fuel == 0 || limits.wall_clock_ms == 0 {
            return Err(TierWError::InvalidCapabilityGrant(
                "fuel and wall-clock limits must be non-zero".into(),
            ));
        }
        let memory_bytes = bytes_from_megabytes(limits.memory_mb, "memory")?;
        let output_bytes = bytes_from_megabytes(limits.output_mb, "output")?;
        Ok(Self {
            memory_bytes,
            fuel: limits.fuel,
            wall_clock_ms: limits.wall_clock_ms,
            max_table_elements: 100_000,
            max_instances: 100,
            max_tables: 4,
            max_memories: 4,
            max_stream_bytes: output_bytes,
            max_artifact_bytes: output_bytes,
            max_total_artifact_bytes: output_bytes,
        })
    }

    fn store_limits(&self) -> StoreLimits {
        StoreLimitsBuilder::new()
            .memory_size(self.memory_bytes)
            .table_elements(self.max_table_elements)
            .instances(self.max_instances)
            .tables(self.max_tables)
            .memories(self.max_memories)
            .trap_on_grow_failure(true)
            .build()
    }

    fn epoch_ticks(&self) -> u64 {
        self.wall_clock_ms
            .saturating_add(EPOCH_TICK_MILLIS - 1)
            .checked_div(EPOCH_TICK_MILLIS)
            .unwrap_or(1)
            .max(1)
    }
}

impl Default for TierWLimits {
    fn default() -> Self {
        Self::from_contract(&ExecutionLimits::default())
            .expect("default execution limits must fit the host")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TierWExecutionSuccess {
    pub state: RunState,
    pub output: String,
    pub artifacts: BTreeMap<String, Vec<u8>>,
    pub logs: Vec<HostLogEntry>,
    pub fuel_consumed: u64,
    pub random_bytes_consumed: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TierWExecutionFailure {
    pub state: RunState,
    pub failure: ExecutionFailure,
    pub fuel_consumed: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TierWExecutionOutcome {
    Succeeded(TierWExecutionSuccess),
    Failed(TierWExecutionFailure),
}

impl TierWExecutionOutcome {
    pub fn failure(&self) -> Option<&ExecutionFailure> {
        match self {
            Self::Succeeded(_) => None,
            Self::Failed(failure) => Some(&failure.failure),
        }
    }
}

impl TierWEngine {
    pub fn execute_component(
        &self,
        component: &ValidatedComponent,
        authorizer: &ComponentAuthorizer,
        grant: CapabilityGrant,
        limits: &TierWLimits,
        input: &str,
    ) -> TierWExecutionOutcome {
        if let Err(error) =
            authorizer.reauthorize(component.component_hash(), component.authorization())
        {
            return failed(
                ExecutionFailureKind::ComponentUnauthorized,
                "tier_w.component_unauthorized",
                error.to_string(),
                0,
            );
        }
        if let Err(error) = validate_component_grants(self.engine(), component.component(), &grant)
        {
            return failed(
                ExecutionFailureKind::CapabilityDenied,
                "tier_w.capability_denied",
                error.to_string(),
                0,
            );
        }
        let linker = match capability_linker(self.engine(), &grant) {
            Ok(linker) => linker,
            Err(error) => {
                return failed(
                    ExecutionFailureKind::Trap,
                    "tier_w.capability_link_failed",
                    error.to_string(),
                    0,
                )
            }
        };
        let host = CapabilityHost::with_limits(
            grant,
            limits.store_limits(),
            limits.memory_bytes,
            limits.max_table_elements,
            limits.max_stream_bytes,
            limits.max_artifact_bytes,
            limits.max_total_artifact_bytes,
        );
        let mut store = Store::new(self.engine(), host);
        store.limiter(|host| host.store_limits_mut());
        if let Err(error) = store.set_fuel(limits.fuel) {
            return failed(
                ExecutionFailureKind::Trap,
                "tier_w.fuel_configuration_failed",
                error.to_string(),
                0,
            );
        }
        store.set_epoch_deadline(limits.epoch_ticks());
        store.epoch_deadline_trap();

        let bindings = match Skill::instantiate(&mut store, component.component(), &linker) {
            Ok(bindings) => bindings,
            Err(error) => {
                let fuel = consumed_fuel(&store, limits.fuel);
                if let Some(fence) = store.data().resource_limit_failure() {
                    return failed_from_resource_limit(fence, fuel);
                }
                return failed_from_wasmtime(&error, fuel);
            }
        };
        let result = bindings.call_run(&mut store, input);
        let fuel = consumed_fuel(&store, limits.fuel);
        if let Some(fence) = store.data().limit_failure() {
            return failed_from_host_limit(fence, fuel);
        }
        if let Some(fence) = store.data().resource_limit_failure() {
            return failed_from_resource_limit(fence, fuel);
        }

        match result {
            Err(error) => failed_from_wasmtime(&error, fuel),
            Ok(Err(error)) => {
                let (kind, code) = match error.kind {
                    ErrorKind::CapabilityDenied => (
                        ExecutionFailureKind::CapabilityDenied,
                        "tier_w.guest_capability_denied",
                    ),
                    ErrorKind::InvalidInput => {
                        (ExecutionFailureKind::Trap, "tier_w.guest_invalid_input")
                    }
                    ErrorKind::Unsupported => {
                        (ExecutionFailureKind::Trap, "tier_w.guest_unsupported")
                    }
                    ErrorKind::Internal => (ExecutionFailureKind::Trap, "tier_w.guest_internal"),
                };
                failed(kind, code, error.message, fuel)
            }
            Ok(Ok(output)) if output.len() > limits.max_stream_bytes => failed(
                ExecutionFailureKind::StreamLimit,
                "tier_w.stream_limit",
                "component result exceeds the configured stream byte limit".into(),
                fuel,
            ),
            Ok(Ok(output)) => TierWExecutionOutcome::Succeeded(TierWExecutionSuccess {
                state: RunState::Succeeded,
                output,
                artifacts: store.data().outputs().clone(),
                logs: store.data().logs().to_vec(),
                fuel_consumed: fuel,
                random_bytes_consumed: store.data().random_bytes_consumed(),
            }),
        }
    }
}

fn consumed_fuel(store: &Store<CapabilityHost>, supplied: u64) -> u64 {
    store
        .get_fuel()
        .map_or(0, |remaining| supplied.saturating_sub(remaining))
}

fn failed_from_host_limit(fence: HostLimitFailure, fuel: u64) -> TierWExecutionOutcome {
    match fence {
        HostLimitFailure::Stream => failed(
            ExecutionFailureKind::StreamLimit,
            "tier_w.stream_limit",
            "component logs exceed the configured stream byte limit".into(),
            fuel,
        ),
        HostLimitFailure::Artifact => failed(
            ExecutionFailureKind::ArtifactLimit,
            "tier_w.artifact_limit",
            "component artifacts exceed the configured artifact byte limit".into(),
            fuel,
        ),
    }
}

fn failed_from_resource_limit(fence: ResourceLimitFailure, fuel: u64) -> TierWExecutionOutcome {
    match fence {
        ResourceLimitFailure::Memory => failed(
            ExecutionFailureKind::MemoryLimit,
            "tier_w.memory_limit",
            "component exceeded its configured memory limit".into(),
            fuel,
        ),
        ResourceLimitFailure::Table => failed(
            ExecutionFailureKind::TableLimit,
            "tier_w.table_limit",
            "component exceeded its configured table limit".into(),
            fuel,
        ),
    }
}

fn failed_from_wasmtime(error: &wasmtime::Error, fuel: u64) -> TierWExecutionOutcome {
    if let Some(trap) = error.downcast_ref::<Trap>() {
        return match trap {
            Trap::OutOfFuel => failed(
                ExecutionFailureKind::FuelExhausted,
                "tier_w.fuel_exhausted",
                "component exhausted its configured fuel".into(),
                fuel,
            ),
            Trap::Interrupt => failed(
                ExecutionFailureKind::EpochDeadline,
                "tier_w.epoch_deadline",
                "component exceeded its wall-clock epoch deadline".into(),
                fuel,
            ),
            Trap::AllocationTooLarge => failed(
                ExecutionFailureKind::MemoryLimit,
                "tier_w.memory_limit",
                "component exceeded its configured memory limit".into(),
                fuel,
            ),
            _ => failed(
                ExecutionFailureKind::Trap,
                "tier_w.trap",
                "component trapped during execution".into(),
                fuel,
            ),
        };
    }

    let message = error.to_string().to_ascii_lowercase();
    if message.contains("memory") && (message.contains("limit") || message.contains("minimum")) {
        return failed(
            ExecutionFailureKind::MemoryLimit,
            "tier_w.memory_limit",
            "component exceeded its configured memory limit".into(),
            fuel,
        );
    }
    if message.contains("table") && (message.contains("limit") || message.contains("minimum")) {
        return failed(
            ExecutionFailureKind::TableLimit,
            "tier_w.table_limit",
            "component exceeded its configured table limit".into(),
            fuel,
        );
    }
    if message.contains("instance") && message.contains("limit") {
        return failed(
            ExecutionFailureKind::InstanceLimit,
            "tier_w.instance_limit",
            "component exceeded its configured instance limit".into(),
            fuel,
        );
    }
    failed(
        ExecutionFailureKind::Trap,
        "tier_w.instantiation_or_call_failed",
        "component instantiation or invocation failed".into(),
        fuel,
    )
}

fn failed(
    kind: ExecutionFailureKind,
    code: &str,
    message: String,
    fuel_consumed: u64,
) -> TierWExecutionOutcome {
    TierWExecutionOutcome::Failed(TierWExecutionFailure {
        state: RunState::Failed,
        failure: ExecutionFailure {
            kind,
            code: code.into(),
            message,
        },
        fuel_consumed,
    })
}

fn bytes_from_megabytes(value: u64, name: &str) -> Result<usize, TierWError> {
    let bytes = value
        .checked_mul(1024 * 1024)
        .ok_or_else(|| TierWError::InvalidCapabilityGrant(format!("{name} limit overflow")))?;
    usize::try_from(bytes)
        .map_err(|_| TierWError::InvalidCapabilityGrant(format!("{name} limit exceeds usize")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bindings::prometheus::component::output;
    #[cfg(feature = "cranelift")]
    use crate::EngineProfile;

    #[cfg(any(feature = "cranelift", feature = "pulley"))]
    const REFERENCE_COMPONENT: &[u8] = include_bytes!(
        "../../../skills/react/prometheus-entity-skills/entity-graph-optimize/skill.wasm"
    );

    #[cfg(any(feature = "cranelift", feature = "pulley"))]
    fn reference_grant() -> CapabilityGrant {
        [
            ".kbd-orchestrator/project.json",
            ".evolver/",
            ".refiner/",
            "openspec/",
        ]
        .into_iter()
        .fold(CapabilityGrant::empty(), |grant, key| {
            grant.allow_kv_read(key, None)
        })
    }

    #[test]
    fn contract_limits_convert_without_unit_ambiguity() {
        let limits = TierWLimits::from_contract(&ExecutionLimits::default()).unwrap();
        assert_eq!(limits.memory_bytes, 256 * 1024 * 1024);
        assert_eq!(limits.max_stream_bytes, 10 * 1024 * 1024);
        assert!(TierWLimits::from_contract(&ExecutionLimits {
            fuel: 0,
            ..ExecutionLimits::default()
        })
        .is_err());
    }

    #[cfg(feature = "cranelift")]
    #[test]
    fn reference_run_and_limit_failures_are_terminal_and_classified() {
        let engine = TierWEngine::new(EngineProfile::desktop()).unwrap();
        let authorizer = crate::test_authorizer(REFERENCE_COMPONENT);
        let component = engine
            .load_component(REFERENCE_COMPONENT, &authorizer)
            .unwrap();

        let success = engine.execute_component(
            &component,
            &authorizer,
            reference_grant(),
            &TierWLimits::default(),
            "{}",
        );
        assert!(matches!(success, TierWExecutionOutcome::Succeeded(_)));

        let fuel = TierWLimits {
            fuel: 1,
            ..TierWLimits::default()
        };
        assert_eq!(
            engine
                .execute_component(&component, &authorizer, reference_grant(), &fuel, "{}")
                .failure()
                .unwrap()
                .kind,
            ExecutionFailureKind::FuelExhausted
        );

        let stream = TierWLimits {
            max_stream_bytes: 1,
            ..TierWLimits::default()
        };
        assert_eq!(
            engine
                .execute_component(&component, &authorizer, reference_grant(), &stream, "{}",)
                .failure()
                .unwrap()
                .kind,
            ExecutionFailureKind::StreamLimit
        );
    }

    #[cfg(feature = "pulley")]
    #[test]
    fn pulley_applies_the_same_fuel_fence() {
        let engine = TierWEngine::new(crate::EngineProfile::portable_replay()).unwrap();
        let authorizer = crate::test_authorizer(REFERENCE_COMPONENT);
        let component = engine
            .load_component(REFERENCE_COMPONENT, &authorizer)
            .unwrap();
        let limits = TierWLimits {
            fuel: 1,
            ..TierWLimits::default()
        };
        assert_eq!(
            engine
                .execute_component(&component, &authorizer, reference_grant(), &limits, "{}",)
                .failure()
                .unwrap()
                .kind,
            ExecutionFailureKind::FuelExhausted
        );
    }

    #[cfg(feature = "cranelift")]
    #[test]
    fn store_memory_table_and_instance_fences_are_classified() {
        let engine = TierWEngine::new(EngineProfile::desktop()).unwrap();
        let authorizer = crate::test_authorizer(REFERENCE_COMPONENT);
        let component = engine
            .load_component(REFERENCE_COMPONENT, &authorizer)
            .unwrap();

        let memory = TierWLimits {
            memory_bytes: 1,
            ..TierWLimits::default()
        };
        assert_eq!(
            engine
                .execute_component(&component, &authorizer, reference_grant(), &memory, "{}",)
                .failure()
                .unwrap()
                .kind,
            ExecutionFailureKind::MemoryLimit
        );

        let table = TierWLimits {
            max_table_elements: 0,
            ..TierWLimits::default()
        };
        assert_eq!(
            engine
                .execute_component(&component, &authorizer, reference_grant(), &table, "{}",)
                .failure()
                .unwrap()
                .kind,
            ExecutionFailureKind::TableLimit
        );

        let instances = TierWLimits {
            max_instances: 0,
            ..TierWLimits::default()
        };
        assert_eq!(
            engine
                .execute_component(&component, &authorizer, reference_grant(), &instances, "{}",)
                .failure()
                .unwrap()
                .kind,
            ExecutionFailureKind::InstanceLimit
        );
    }

    #[cfg(feature = "cranelift")]
    #[test]
    fn epoch_ticker_interrupts_an_expired_store() {
        let engine = TierWEngine::new(EngineProfile::desktop()).unwrap();
        let component = engine
            .load_component(
                REFERENCE_COMPONENT,
                &crate::test_authorizer(REFERENCE_COMPONENT),
            )
            .unwrap();
        let grant = reference_grant();
        let linker = capability_linker(engine.engine(), &grant).unwrap();
        let mut store = Store::new(engine.engine(), CapabilityHost::new(grant));
        store.set_fuel(u64::MAX).unwrap();
        store.set_epoch_deadline(1);
        store.epoch_deadline_trap();
        std::thread::sleep(Duration::from_millis(EPOCH_TICK_MILLIS * 2));
        let error = match Skill::instantiate(&mut store, component.component(), &linker) {
            Ok(bindings) => bindings.call_run(&mut store, "{}").unwrap_err(),
            Err(error) => error,
        };
        assert_eq!(error.downcast_ref::<Trap>(), Some(&Trap::Interrupt));
    }

    #[test]
    fn artifact_host_fence_is_sticky_and_classifiable() {
        let grant = CapabilityGrant::empty()
            .allow_output_prefix("outputs")
            .unwrap();
        let mut host = CapabilityHost::with_limits(
            grant,
            StoreLimitsBuilder::new().build(),
            usize::MAX,
            usize::MAX,
            100,
            1,
            1,
        );
        assert!(output::Host::write(&mut host, "outputs/a".into(), vec![1, 2]).is_err());
        assert_eq!(host.limit_failure(), Some(HostLimitFailure::Artifact));
        assert!(output::Host::write(&mut host, "outputs/b".into(), vec![]).is_err());
    }
}
