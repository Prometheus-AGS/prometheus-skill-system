use std::{
    collections::{BTreeMap, BTreeSet},
    time::Duration,
};

use serde::Serialize;
use wasmtime::{
    component::{Component, HasSelf, Linker, ResourceTable},
    Engine, ResourceLimiter, StoreLimits, StoreLimitsBuilder,
};
use wasmtime_wasi::{
    clocks::{HostMonotonicClock, HostWallClock},
    Deterministic, WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView,
};

use crate::{
    bindings::prometheus::component::{
        clock, input, kv_store, log, output, random,
        types::{self, Error, ErrorKind},
    },
    TierWError,
};

const TYPES_IMPORT: &str = "prometheus:component/types@0.1.0";
const LOG_IMPORT: &str = "prometheus:component/log@0.1.0";
const KV_IMPORT: &str = "prometheus:component/kv-store@0.1.0";
const INPUT_IMPORT: &str = "prometheus:component/input@0.1.0";
const OUTPUT_IMPORT: &str = "prometheus:component/output@0.1.0";
const CLOCK_IMPORT: &str = "prometheus:component/clock@0.1.0";
const RANDOM_IMPORT: &str = "prometheus:component/random@0.1.0";

const SUPPORTED_IMPORTS: [&str; 21] = [
    TYPES_IMPORT,
    LOG_IMPORT,
    KV_IMPORT,
    INPUT_IMPORT,
    OUTPUT_IMPORT,
    CLOCK_IMPORT,
    RANDOM_IMPORT,
    "wasi:io/poll@0.2.9",
    "wasi:clocks/monotonic-clock@0.2.9",
    "wasi:io/error@0.2.9",
    "wasi:io/streams@0.2.9",
    "wasi:cli/stdout@0.2.9",
    "wasi:cli/stderr@0.2.9",
    "wasi:cli/stdin@0.2.9",
    "wasi:cli/environment@0.2.9",
    "wasi:cli/exit@0.2.9",
    "wasi:cli/terminal-input@0.2.9",
    "wasi:cli/terminal-output@0.2.9",
    "wasi:cli/terminal-stdin@0.2.9",
    "wasi:cli/terminal-stdout@0.2.9",
    "wasi:cli/terminal-stderr@0.2.9",
];

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityGrant {
    log: bool,
    readable_keys: BTreeSet<String>,
    readable_prefixes: BTreeSet<String>,
    writable_keys: BTreeSet<String>,
    kv_values: BTreeMap<String, String>,
    readable_inputs: BTreeSet<String>,
    inputs: BTreeMap<String, Vec<u8>>,
    output_prefixes: BTreeSet<String>,
    clock_ms: Option<u64>,
    random_bytes: Option<Vec<u8>>,
}

impl CapabilityGrant {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn allow_log(mut self) -> Self {
        self.log = true;
        self
    }

    pub fn allow_kv_read(mut self, key: impl Into<String>, value: Option<String>) -> Self {
        let key = key.into();
        self.readable_keys.insert(key.clone());
        if let Some(value) = value {
            self.kv_values.insert(key, value);
        }
        self
    }

    /// Grants a portable relative filesystem-like read scope to the typed KV
    /// host. `.` means all relative keys; directory scopes are prefix-bound.
    pub fn allow_kv_scope(mut self, scope: impl Into<String>) -> Result<Self, TierWError> {
        let scope = scope.into();
        if scope == "." {
            self.readable_prefixes.insert(String::new());
        } else if scope.ends_with(['/', '\\']) {
            let normalized = normalize_prefix(scope.trim_end_matches(['/', '\\']))?;
            self.readable_prefixes.insert(normalized);
        } else {
            let normalized = normalize_output_path(&scope)
                .map_err(|message| TierWError::InvalidCapabilityGrant(message.to_string()))?;
            self.readable_keys.insert(normalized);
        }
        Ok(self)
    }

    pub fn allow_kv_write(mut self, key: impl Into<String>) -> Self {
        self.writable_keys.insert(key.into());
        self
    }

    pub fn allow_input(mut self, name: impl Into<String>, value: Option<Vec<u8>>) -> Self {
        let name = name.into();
        self.readable_inputs.insert(name.clone());
        if let Some(value) = value {
            self.inputs.insert(name, value);
        }
        self
    }

    pub fn allow_output_prefix(mut self, prefix: impl Into<String>) -> Result<Self, TierWError> {
        let prefix = normalize_prefix(&prefix.into())?;
        self.output_prefixes.insert(prefix);
        Ok(self)
    }

    pub fn allow_clock(mut self, milliseconds: u64) -> Self {
        self.clock_ms = Some(milliseconds);
        self
    }

    pub fn allow_random(mut self, bytes: Vec<u8>) -> Self {
        self.random_bytes = Some(bytes);
        self
    }

    fn permits_interface(&self, name: &str) -> bool {
        match name {
            TYPES_IMPORT => true,
            LOG_IMPORT => self.log,
            KV_IMPORT => {
                !self.readable_keys.is_empty()
                    || !self.readable_prefixes.is_empty()
                    || !self.writable_keys.is_empty()
            }
            INPUT_IMPORT => !self.readable_inputs.is_empty(),
            OUTPUT_IMPORT => !self.output_prefixes.is_empty(),
            CLOCK_IMPORT => self.clock_ms.is_some(),
            RANDOM_IMPORT => self.random_bytes.is_some(),
            name if name.starts_with("wasi:") => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostLogEntry {
    pub level: String,
    pub message: String,
    pub fields: BTreeMap<String, String>,
}

pub struct CapabilityHost {
    grant: CapabilityGrant,
    outputs: BTreeMap<String, Vec<u8>>,
    logs: Vec<HostLogEntry>,
    random_cursor: usize,
    wasi: WasiCtx,
    wasi_table: ResourceTable,
    store_limits: TrackingStoreLimits,
    max_stream_bytes: usize,
    stream_bytes: usize,
    max_artifact_bytes: usize,
    max_total_artifact_bytes: usize,
    artifact_bytes: usize,
    limit_failure: Option<HostLimitFailure>,
}

impl std::fmt::Debug for CapabilityHost {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CapabilityHost")
            .field("grant", &self.grant)
            .field("outputs", &self.outputs)
            .field("logs", &self.logs)
            .field("random_cursor", &self.random_cursor)
            .field("stream_bytes", &self.stream_bytes)
            .field("artifact_bytes", &self.artifact_bytes)
            .field("limit_failure", &self.limit_failure)
            .finish_non_exhaustive()
    }
}

impl CapabilityHost {
    pub fn new(grant: CapabilityGrant) -> Self {
        Self::with_limits(
            grant,
            StoreLimitsBuilder::new().build(),
            usize::MAX,
            usize::MAX,
            1024 * 1024,
            10 * 1024 * 1024,
            10 * 1024 * 1024,
        )
    }

    pub(crate) fn with_limits(
        grant: CapabilityGrant,
        store_limits: StoreLimits,
        total_memory_bytes: usize,
        total_table_elements: usize,
        max_stream_bytes: usize,
        max_artifact_bytes: usize,
        max_total_artifact_bytes: usize,
    ) -> Self {
        let fixed_time = grant.clock_ms.unwrap_or_default();
        let mut wasi = WasiCtxBuilder::new();
        wasi.wall_clock(FixedWallClock(fixed_time))
            .monotonic_clock(FixedMonotonicClock(fixed_time))
            .secure_random(Deterministic::new(vec![0]))
            .insecure_random(Deterministic::new(vec![0]))
            .insecure_random_seed(0)
            .allow_tcp(false)
            .allow_udp(false);
        Self {
            grant,
            outputs: BTreeMap::new(),
            logs: Vec::new(),
            random_cursor: 0,
            wasi: wasi.build(),
            wasi_table: ResourceTable::new(),
            store_limits: TrackingStoreLimits::new(
                store_limits,
                total_memory_bytes,
                total_table_elements,
            ),
            max_stream_bytes,
            stream_bytes: 0,
            max_artifact_bytes,
            max_total_artifact_bytes,
            artifact_bytes: 0,
            limit_failure: None,
        }
    }

    pub fn outputs(&self) -> &BTreeMap<String, Vec<u8>> {
        &self.outputs
    }

    pub fn logs(&self) -> &[HostLogEntry] {
        &self.logs
    }

    pub fn random_bytes_consumed(&self) -> usize {
        self.random_cursor
    }

    pub(crate) fn store_limits_mut(&mut self) -> &mut dyn ResourceLimiter {
        &mut self.store_limits
    }

    pub(crate) fn resource_limit_failure(&self) -> Option<ResourceLimitFailure> {
        self.store_limits.failure
    }

    pub(crate) fn limit_failure(&self) -> Option<HostLimitFailure> {
        self.limit_failure
    }

    fn active_limit_error(&self) -> Option<Error> {
        self.limit_failure
            .map(|failure| limit_error(format!("{} limit was already exceeded", failure.code())))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HostLimitFailure {
    Stream,
    Artifact,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ResourceLimitFailure {
    Memory,
    Table,
}

struct TrackingStoreLimits {
    inner: StoreLimits,
    failure: Option<ResourceLimitFailure>,
    total_memory_bytes: usize,
    allocated_memory_bytes: usize,
    pending_memory_delta: usize,
    total_table_elements: usize,
    allocated_table_elements: usize,
    pending_table_delta: usize,
}

impl TrackingStoreLimits {
    fn new(inner: StoreLimits, total_memory_bytes: usize, total_table_elements: usize) -> Self {
        Self {
            inner,
            failure: None,
            total_memory_bytes,
            allocated_memory_bytes: 0,
            pending_memory_delta: 0,
            total_table_elements,
            allocated_table_elements: 0,
            pending_table_delta: 0,
        }
    }
}

impl ResourceLimiter for TrackingStoreLimits {
    fn memory_growing(
        &mut self,
        current: usize,
        desired: usize,
        maximum: Option<usize>,
    ) -> wasmtime::Result<bool> {
        let result = self.inner.memory_growing(current, desired, maximum);
        if !matches!(result, Ok(true)) {
            self.failure = Some(ResourceLimitFailure::Memory);
            return result;
        }
        let delta = desired.saturating_sub(current);
        let Some(total) = self.allocated_memory_bytes.checked_add(delta) else {
            self.failure = Some(ResourceLimitFailure::Memory);
            return Ok(false);
        };
        if total > self.total_memory_bytes {
            self.failure = Some(ResourceLimitFailure::Memory);
            return Ok(false);
        }
        self.allocated_memory_bytes = total;
        self.pending_memory_delta = delta;
        Ok(true)
    }

    fn memory_grow_failed(&mut self, error: wasmtime::Error) -> wasmtime::Result<()> {
        self.failure = Some(ResourceLimitFailure::Memory);
        self.allocated_memory_bytes = self
            .allocated_memory_bytes
            .saturating_sub(self.pending_memory_delta);
        self.pending_memory_delta = 0;
        self.inner.memory_grow_failed(error)
    }

    fn table_growing(
        &mut self,
        current: usize,
        desired: usize,
        maximum: Option<usize>,
    ) -> wasmtime::Result<bool> {
        let result = self.inner.table_growing(current, desired, maximum);
        if !matches!(result, Ok(true)) {
            self.failure = Some(ResourceLimitFailure::Table);
            return result;
        }
        let delta = desired.saturating_sub(current);
        let Some(total) = self.allocated_table_elements.checked_add(delta) else {
            self.failure = Some(ResourceLimitFailure::Table);
            return Ok(false);
        };
        if total > self.total_table_elements {
            self.failure = Some(ResourceLimitFailure::Table);
            return Ok(false);
        }
        self.allocated_table_elements = total;
        self.pending_table_delta = delta;
        Ok(true)
    }

    fn table_grow_failed(&mut self, error: wasmtime::Error) -> wasmtime::Result<()> {
        self.failure = Some(ResourceLimitFailure::Table);
        self.allocated_table_elements = self
            .allocated_table_elements
            .saturating_sub(self.pending_table_delta);
        self.pending_table_delta = 0;
        self.inner.table_grow_failed(error)
    }

    fn instances(&self) -> usize {
        self.inner.instances()
    }

    fn tables(&self) -> usize {
        self.inner.tables()
    }

    fn memories(&self) -> usize {
        self.inner.memories()
    }
}

impl HostLimitFailure {
    pub(crate) const fn code(self) -> &'static str {
        match self {
            Self::Stream => "stream",
            Self::Artifact => "artifact",
        }
    }
}

impl WasiView for CapabilityHost {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.wasi_table,
        }
    }
}

struct FixedWallClock(u64);

impl HostWallClock for FixedWallClock {
    fn resolution(&self) -> Duration {
        Duration::from_millis(1)
    }

    fn now(&self) -> Duration {
        Duration::from_millis(self.0)
    }
}

struct FixedMonotonicClock(u64);

impl HostMonotonicClock for FixedMonotonicClock {
    fn resolution(&self) -> u64 {
        1_000_000
    }

    fn now(&self) -> u64 {
        self.0.saturating_mul(1_000_000)
    }
}

impl types::Host for CapabilityHost {}

impl log::Host for CapabilityHost {
    fn emit(&mut self, level: log::Level, message: String, fields: Vec<types::Kv>) {
        if self.limit_failure.is_some() {
            return;
        }
        let field_bytes = fields.iter().fold(0usize, |total, field| {
            total
                .saturating_add(field.key.len())
                .saturating_add(field.value.len())
        });
        let entry_bytes = message.len().saturating_add(field_bytes);
        let Some(total) = self.stream_bytes.checked_add(entry_bytes) else {
            self.limit_failure = Some(HostLimitFailure::Stream);
            return;
        };
        if total > self.max_stream_bytes {
            self.limit_failure = Some(HostLimitFailure::Stream);
            return;
        }
        self.stream_bytes = total;
        let level = match level {
            log::Level::Debug => "debug",
            log::Level::Info => "info",
            log::Level::Warn => "warn",
            log::Level::Error => "error",
        };
        self.logs.push(HostLogEntry {
            level: level.into(),
            message,
            fields: fields
                .into_iter()
                .map(|field| (field.key, field.value))
                .collect(),
        });
    }
}

impl kv_store::Host for CapabilityHost {
    fn get(&mut self, key: String) -> Result<Option<String>, Error> {
        if let Some(error) = self.active_limit_error() {
            return Err(error);
        }
        let permitted_by_scope = self
            .grant
            .readable_prefixes
            .iter()
            .any(|prefix| prefix.is_empty() || key.starts_with(prefix));
        if !self.grant.readable_keys.contains(&key) && !permitted_by_scope {
            return Err(denied(format!("kv read was not granted for {key}")));
        }
        Ok(self.grant.kv_values.get(&key).cloned())
    }

    fn set(&mut self, key: String, value: String) -> Result<(), Error> {
        if let Some(error) = self.active_limit_error() {
            return Err(error);
        }
        if !self.grant.writable_keys.contains(&key) {
            return Err(denied(format!("kv write was not granted for {key}")));
        }
        self.grant.kv_values.insert(key, value);
        Ok(())
    }

    fn delete(&mut self, key: String) -> Result<(), Error> {
        if let Some(error) = self.active_limit_error() {
            return Err(error);
        }
        if !self.grant.writable_keys.contains(&key) {
            return Err(denied(format!("kv delete was not granted for {key}")));
        }
        self.grant.kv_values.remove(&key);
        Ok(())
    }
}

impl input::Host for CapabilityHost {
    fn read(&mut self, name: String) -> Result<Option<Vec<u8>>, Error> {
        if let Some(error) = self.active_limit_error() {
            return Err(error);
        }
        if !self.grant.readable_inputs.contains(&name) {
            return Err(denied(format!("input read was not granted for {name}")));
        }
        Ok(self.grant.inputs.get(&name).cloned())
    }
}

impl output::Host for CapabilityHost {
    fn write(&mut self, path: String, contents: Vec<u8>) -> Result<(), Error> {
        if let Some(error) = self.active_limit_error() {
            return Err(error);
        }
        let normalized = normalize_output_path(&path).map_err(invalid)?;
        if !self
            .grant
            .output_prefixes
            .iter()
            .any(|prefix| normalized.starts_with(prefix))
        {
            return Err(denied(format!("output write was not granted for {path}")));
        }
        if contents.len() > self.max_artifact_bytes {
            self.limit_failure = Some(HostLimitFailure::Artifact);
            return Err(limit_error("artifact exceeds the per-artifact byte limit"));
        }
        let previous = self.outputs.get(&normalized).map_or(0, Vec::len);
        let Some(total) = self
            .artifact_bytes
            .checked_sub(previous)
            .and_then(|bytes| bytes.checked_add(contents.len()))
        else {
            self.limit_failure = Some(HostLimitFailure::Artifact);
            return Err(limit_error("artifact byte accounting overflow"));
        };
        if total > self.max_total_artifact_bytes {
            self.limit_failure = Some(HostLimitFailure::Artifact);
            return Err(limit_error("artifacts exceed the total byte limit"));
        }
        self.artifact_bytes = total;
        self.outputs.insert(normalized, contents);
        Ok(())
    }
}

impl clock::Host for CapabilityHost {
    fn now_ms(&mut self) -> u64 {
        self.grant.clock_ms.unwrap_or_default()
    }
}

impl random::Host for CapabilityHost {
    fn bytes(&mut self, len: u32) -> Result<Vec<u8>, Error> {
        if let Some(error) = self.active_limit_error() {
            return Err(error);
        }
        let requested = usize::try_from(len).map_err(|_| invalid("random length overflow"))?;
        let granted = self.grant.random_bytes.as_deref().unwrap_or_default();
        let end = self
            .random_cursor
            .checked_add(requested)
            .ok_or_else(|| invalid("random length overflow"))?;
        let Some(bytes) = granted.get(self.random_cursor..end) else {
            return Err(denied(format!(
                "requested {requested} random bytes with {} remaining",
                granted.len().saturating_sub(self.random_cursor)
            )));
        };
        self.random_cursor = end;
        Ok(bytes.to_vec())
    }
}

pub(crate) fn validate_supported_imports(
    engine: &Engine,
    component: &Component,
) -> Result<(), TierWError> {
    let mut unsupported = Vec::new();
    let component_type = component.component_type();
    for (name, _) in component_type.imports(engine) {
        if name.starts_with("host:exec") || name.starts_with("host:memory") {
            return Err(TierWError::ForbiddenImport(name.into()));
        }
        if !SUPPORTED_IMPORTS.contains(&name) {
            unsupported.push(name);
        }
    }
    if unsupported.is_empty() {
        Ok(())
    } else {
        Err(TierWError::UnsupportedImport(unsupported.join(", ")))
    }
}

pub fn validate_component_grants(
    engine: &Engine,
    component: &Component,
    grant: &CapabilityGrant,
) -> Result<(), TierWError> {
    validate_supported_imports(engine, component)?;
    for (name, _) in component.component_type().imports(engine) {
        if !grant.permits_interface(name) {
            return Err(TierWError::CapabilityDenied(name.into()));
        }
    }
    Ok(())
}

pub fn capability_linker(
    engine: &Engine,
    grant: &CapabilityGrant,
) -> Result<Linker<CapabilityHost>, TierWError> {
    let mut linker = Linker::new(engine);
    types::add_to_linker::<_, HasSelf<_>>(&mut linker, |host| host)
        .map_err(|error| TierWError::CapabilityLink(error.to_string()))?;
    if grant.log {
        log::add_to_linker::<_, HasSelf<_>>(&mut linker, |host| host)
            .map_err(|error| TierWError::CapabilityLink(error.to_string()))?;
    }
    if !grant.readable_keys.is_empty() || !grant.writable_keys.is_empty() {
        kv_store::add_to_linker::<_, HasSelf<_>>(&mut linker, |host| host)
            .map_err(|error| TierWError::CapabilityLink(error.to_string()))?;
    }
    if !grant.readable_inputs.is_empty() {
        input::add_to_linker::<_, HasSelf<_>>(&mut linker, |host| host)
            .map_err(|error| TierWError::CapabilityLink(error.to_string()))?;
    }
    if !grant.output_prefixes.is_empty() {
        output::add_to_linker::<_, HasSelf<_>>(&mut linker, |host| host)
            .map_err(|error| TierWError::CapabilityLink(error.to_string()))?;
    }
    if grant.clock_ms.is_some() {
        clock::add_to_linker::<_, HasSelf<_>>(&mut linker, |host| host)
            .map_err(|error| TierWError::CapabilityLink(error.to_string()))?;
    }
    if grant.random_bytes.is_some() {
        random::add_to_linker::<_, HasSelf<_>>(&mut linker, |host| host)
            .map_err(|error| TierWError::CapabilityLink(error.to_string()))?;
    }
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker)
        .map_err(|error| TierWError::CapabilityLink(error.to_string()))?;
    Ok(linker)
}

fn normalize_prefix(prefix: &str) -> Result<String, TierWError> {
    let mut normalized = normalize_output_path(prefix)
        .map_err(|message| TierWError::InvalidCapabilityGrant(message.to_string()))?;
    if !normalized.ends_with('/') {
        normalized.push('/');
    }
    Ok(normalized)
}

fn normalize_output_path(path: &str) -> Result<String, &'static str> {
    if path.is_empty() || path.starts_with('/') || path.contains('\\') || path.contains('\0') {
        return Err("output path must be a non-empty portable relative path");
    }
    if path
        .split('/')
        .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err("output path contains an empty or traversal segment");
    }
    Ok(path.to_string())
}

fn denied(message: impl Into<String>) -> Error {
    Error {
        kind: ErrorKind::CapabilityDenied,
        message: message.into(),
    }
}

fn invalid(message: impl Into<String>) -> Error {
    Error {
        kind: ErrorKind::InvalidInput,
        message: message.into(),
    }
}

fn limit_error(message: impl Into<String>) -> Error {
    Error {
        kind: ErrorKind::Internal,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "cranelift")]
    use crate::{EngineProfile, TierWEngine};

    #[cfg(feature = "cranelift")]
    const REFERENCE_COMPONENT: &[u8] = include_bytes!(
        "../../../skills/react/prometheus-entity-skills/entity-graph-optimize/skill.wasm"
    );

    #[test]
    fn typed_hosts_enforce_declared_reads_outputs_clock_and_random() {
        let grant = CapabilityGrant::empty()
            .allow_input("request.json", Some(b"{}".to_vec()))
            .allow_output_prefix("outputs")
            .unwrap()
            .allow_clock(42)
            .allow_random(vec![1, 2, 3]);
        let mut host = CapabilityHost::new(grant);

        assert_eq!(
            input::Host::read(&mut host, "request.json".into()).unwrap(),
            Some(b"{}".to_vec())
        );
        assert!(input::Host::read(&mut host, "secret".into()).is_err());
        output::Host::write(&mut host, "outputs/result.json".into(), b"42".to_vec()).unwrap();
        assert!(output::Host::write(&mut host, "../escape".into(), Vec::new()).is_err());
        assert_eq!(clock::Host::now_ms(&mut host), 42);
        assert_eq!(random::Host::bytes(&mut host, 2).unwrap(), vec![1, 2]);
        assert!(random::Host::bytes(&mut host, 2).is_err());
        assert_eq!(host.random_bytes_consumed(), 2);
    }

    #[test]
    fn portable_read_scopes_cover_only_the_declared_directory() {
        let grant = CapabilityGrant::empty()
            .allow_kv_scope("openspec/")
            .unwrap()
            .allow_kv_read("openspec/change.json", Some("{}".into()));
        let mut host = CapabilityHost::new(grant);

        assert_eq!(
            kv_store::Host::get(&mut host, "openspec/change.json".into()).unwrap(),
            Some("{}".into())
        );
        assert!(kv_store::Host::get(&mut host, "open-spec/secret".into()).is_err());

        let mut all_relative =
            CapabilityHost::new(CapabilityGrant::empty().allow_kv_scope(".").unwrap());
        assert_eq!(
            kv_store::Host::get(&mut all_relative, "any/relative/path".into()).unwrap(),
            None
        );
    }

    #[cfg(feature = "cranelift")]
    #[test]
    fn missing_declared_interface_is_denied_before_instantiation() {
        let engine = TierWEngine::new(EngineProfile::desktop()).unwrap();
        let component = engine
            .load_component(
                REFERENCE_COMPONENT,
                &crate::test_authorizer(REFERENCE_COMPONENT),
            )
            .unwrap();
        let error = validate_component_grants(
            engine.engine(),
            component.component(),
            &CapabilityGrant::empty(),
        )
        .unwrap_err();
        assert!(matches!(error, TierWError::CapabilityDenied(_)));

        let grant = [
            ".kbd-orchestrator/project.json",
            ".evolver/",
            ".refiner/",
            "openspec/",
        ]
        .into_iter()
        .fold(CapabilityGrant::empty(), |grant, key| {
            grant.allow_kv_read(key, None)
        });
        validate_component_grants(engine.engine(), component.component(), &grant).unwrap();
        let linker = capability_linker(engine.engine(), &grant).unwrap();
        linker.instantiate_pre(component.component()).unwrap();
    }

    #[cfg(feature = "cranelift")]
    #[test]
    fn forbidden_and_unknown_imports_fail_during_validation() {
        let engine = TierWEngine::new(EngineProfile::desktop()).unwrap();
        for import in ["host:exec/run@0.1.0", "host:memory/read@0.1.0"] {
            let forbidden = wat::parse_str(format!(
                r#"(component
                    (type $f (func))
                    (import "{import}" (func (type $f))))"#,
            ))
            .unwrap();
            let error = engine
                .load_component(&forbidden, &crate::test_authorizer(&forbidden))
                .unwrap_err();
            assert!(matches!(error, TierWError::ForbiddenImport(_)));
        }

        let unsupported = wat::parse_str(
            r#"(component
                (type $f (func))
                (import "wasi:filesystem/read@0.2.0" (func (type $f))))"#,
        )
        .unwrap();
        let error = engine
            .load_component(&unsupported, &crate::test_authorizer(&unsupported))
            .unwrap_err();
        assert!(matches!(error, TierWError::UnsupportedImport(_)));
    }
}
