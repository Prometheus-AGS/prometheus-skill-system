use std::{
    sync::mpsc::{self, RecvTimeoutError},
    thread,
    time::Duration,
};

use prometheus_exec_contracts::{hash_serializable, ComponentAuthorization, Digest};
use serde::Serialize;
use wasmtime::{component::Component, Config, Engine, Strategy};

use crate::authorization::AuthorizedComponent;
use crate::capabilities::validate_supported_imports;
use crate::{BackendProfile, ComponentAuthorizer, COMPONENT_WORLD};

const WASMTIME_VERSION: &str = "46.0.0";
pub(crate) const EPOCH_TICK_MILLIS: u64 = 10;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionTarget {
    Desktop,
    Ios,
    Android,
    PortableReplay,
}

impl ExecutionTarget {
    pub const fn current() -> Self {
        #[cfg(target_os = "ios")]
        {
            return Self::Ios;
        }
        #[cfg(target_os = "android")]
        {
            return Self::Android;
        }
        #[cfg(not(any(target_os = "ios", target_os = "android")))]
        {
            Self::Desktop
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::Desktop => "desktop",
            Self::Ios => "ios",
            Self::Android => "android",
            Self::PortableReplay => "portable-replay",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineProfile {
    pub target: ExecutionTarget,
    pub backend: BackendProfile,
    pub jit_permitted: bool,
}

impl EngineProfile {
    pub const fn desktop() -> Self {
        Self {
            target: ExecutionTarget::Desktop,
            backend: BackendProfile::Cranelift,
            jit_permitted: true,
        }
    }

    pub const fn ios() -> Self {
        Self {
            target: ExecutionTarget::Ios,
            backend: BackendProfile::Pulley,
            jit_permitted: false,
        }
    }

    pub const fn android() -> Self {
        Self {
            target: ExecutionTarget::Android,
            backend: BackendProfile::Pulley,
            jit_permitted: false,
        }
    }

    pub const fn certified_android_cranelift() -> Self {
        Self {
            target: ExecutionTarget::Android,
            backend: BackendProfile::Cranelift,
            jit_permitted: true,
        }
    }

    pub const fn portable_replay() -> Self {
        Self {
            target: ExecutionTarget::PortableReplay,
            backend: BackendProfile::Pulley,
            jit_permitted: false,
        }
    }

    pub const fn for_current_target() -> Self {
        #[cfg(feature = "mobile")]
        {
            match ExecutionTarget::current() {
                ExecutionTarget::Ios => Self::ios(),
                ExecutionTarget::Android => Self::android(),
                ExecutionTarget::Desktop | ExecutionTarget::PortableReplay => {
                    Self::portable_replay()
                }
            }
        }
        #[cfg(not(feature = "mobile"))]
        {
            match ExecutionTarget::current() {
                ExecutionTarget::Ios => Self::ios(),
                ExecutionTarget::Android => Self::android(),
                ExecutionTarget::Desktop | ExecutionTarget::PortableReplay => Self::desktop(),
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackendAvailability {
    pub available: bool,
    pub backend: BackendProfile,
    pub target: ExecutionTarget,
    pub reason: Option<String>,
}

impl BackendAvailability {
    fn available(profile: EngineProfile) -> Self {
        Self {
            available: true,
            backend: profile.backend,
            target: profile.target,
            reason: None,
        }
    }

    fn unavailable(profile: EngineProfile, reason: impl Into<String>) -> Self {
        Self {
            available: false,
            backend: profile.backend,
            target: profile.target,
            reason: Some(reason.into()),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TierWError {
    #[error("Tier W backend {backend} is unavailable for {target}: {reason}")]
    BackendUnavailable {
        backend: &'static str,
        target: &'static str,
        reason: String,
    },
    #[error("failed to configure Tier W engine: {0}")]
    EngineConfiguration(String),
    #[error("component validation failed: {0}")]
    InvalidComponent(String),
    #[error("component authorization failed: {0}")]
    ComponentUnauthorized(String),
    #[error("component import is permanently forbidden in Tier W: {0}")]
    ForbiddenImport(String),
    #[error("component import is unsupported in Tier W: {0}")]
    UnsupportedImport(String),
    #[error("component capability was not granted: {0}")]
    CapabilityDenied(String),
    #[error("invalid Tier W capability grant: {0}")]
    InvalidCapabilityGrant(String),
    #[error("failed to link Tier W capabilities: {0}")]
    CapabilityLink(String),
    #[error("failed to derive deterministic Tier W identity: {0}")]
    Identity(#[from] prometheus_exec_contracts::ContractError),
}

pub struct ValidatedComponent {
    component: Component,
    component_hash: Digest,
    authorization: ComponentAuthorization,
    cache_key: Digest,
}

impl std::fmt::Debug for ValidatedComponent {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ValidatedComponent")
            .field("component_hash", &self.component_hash)
            .field("authorization", &self.authorization)
            .field("cache_key", &self.cache_key)
            .finish_non_exhaustive()
    }
}

impl ValidatedComponent {
    pub fn component(&self) -> &Component {
        &self.component
    }

    pub fn component_hash(&self) -> &Digest {
        &self.component_hash
    }

    pub fn authorization(&self) -> &ComponentAuthorization {
        &self.authorization
    }

    pub fn cache_key(&self) -> &Digest {
        &self.cache_key
    }
}

pub struct TierWEngine {
    engine: Engine,
    profile: EngineProfile,
    profile_hash: Digest,
    _epoch_ticker: EpochTicker,
}

impl TierWEngine {
    /// Reports compile-time and target-policy availability without constructing
    /// an engine or compiling guest bytes.
    pub fn availability(profile: EngineProfile) -> BackendAvailability {
        if profile.target == ExecutionTarget::Ios && profile.backend != BackendProfile::Pulley {
            return BackendAvailability::unavailable(profile, "iOS permits Pulley only");
        }
        if profile.target == ExecutionTarget::PortableReplay
            && profile.backend != BackendProfile::Pulley
        {
            return BackendAvailability::unavailable(
                profile,
                "portable replay permits Pulley only",
            );
        }
        if profile.backend == BackendProfile::Cranelift && !cfg!(feature = "cranelift") {
            return BackendAvailability::unavailable(
                profile,
                "the Cranelift backend was not selected at compile time",
            );
        }
        if profile.backend == BackendProfile::Pulley && !cfg!(feature = "pulley") {
            return BackendAvailability::unavailable(
                profile,
                "the Pulley backend was not selected at compile time",
            );
        }
        if profile.backend == BackendProfile::Cranelift && !profile.jit_permitted {
            return BackendAvailability::unavailable(
                profile,
                "Cranelift requires an explicitly certified executable-memory profile",
            );
        }
        if profile.backend == BackendProfile::Pulley && profile.jit_permitted {
            return BackendAvailability::unavailable(
                profile,
                "Pulley profiles must not claim JIT or executable-memory use",
            );
        }
        BackendAvailability::available(profile)
    }

    /// Constructs the configured engine to prove that the selected backend is
    /// usable on this host. This never validates or compiles a guest component.
    pub fn probe(profile: EngineProfile) -> BackendAvailability {
        match Self::new(profile) {
            Ok(_) => BackendAvailability::available(profile),
            Err(error) => BackendAvailability::unavailable(profile, error.to_string()),
        }
    }

    pub fn new(profile: EngineProfile) -> Result<Self, TierWError> {
        let availability = Self::availability(profile);
        if !availability.available {
            return Err(TierWError::BackendUnavailable {
                backend: profile.backend.name(),
                target: profile.target.name(),
                reason: availability
                    .reason
                    .unwrap_or_else(|| "backend probe failed".into()),
            });
        }

        let profile_hash = hash_serializable(&EngineIdentity::new(profile))?;
        let mut config = Config::new();
        config
            .wasm_component_model(true)
            .consume_fuel(true)
            .epoch_interruption(true);

        match profile.backend {
            BackendProfile::Cranelift => {
                config.strategy(Strategy::Cranelift);
            }
            BackendProfile::Pulley => {
                config
                    .target(pulley_target())
                    .map_err(|error| TierWError::EngineConfiguration(error.to_string()))?;
            }
        }

        let engine = Engine::new(&config)
            .map_err(|error| TierWError::EngineConfiguration(error.to_string()))?;
        if engine.is_pulley() != (profile.backend == BackendProfile::Pulley) {
            return Err(TierWError::EngineConfiguration(format!(
                "configured {} but Wasmtime constructed a different backend",
                profile.backend.name()
            )));
        }
        let epoch_ticker = EpochTicker::start(engine.clone())?;

        Ok(Self {
            engine,
            profile,
            profile_hash,
            _epoch_ticker: epoch_ticker,
        })
    }

    pub fn profile(&self) -> EngineProfile {
        self.profile
    }

    pub fn profile_hash(&self) -> &Digest {
        &self.profile_hash
    }

    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    pub fn cache_key(
        &self,
        component_hash: &Digest,
        authorization: &ComponentAuthorization,
    ) -> Result<Digest, TierWError> {
        Ok(hash_serializable(&ComponentCacheIdentity {
            schema_version: 1,
            profile_hash: &self.profile_hash,
            component_hash,
            authorization,
        })?)
    }

    pub fn load_component(
        &self,
        component_bytes: &[u8],
        authorizer: &ComponentAuthorizer,
    ) -> Result<ValidatedComponent, TierWError> {
        let authorized = authorizer.authorize(component_bytes)?;
        self.validate_authorized_component(authorized)
    }

    fn validate_authorized_component(
        &self,
        authorized: AuthorizedComponent<'_>,
    ) -> Result<ValidatedComponent, TierWError> {
        let component_bytes = authorized.bytes();
        if Engine::detect_precompiled(component_bytes).is_some() {
            return Err(TierWError::InvalidComponent(
                "precompiled engine artifacts are not accepted as component source bytes".into(),
            ));
        }
        let component = Component::from_binary(&self.engine, component_bytes)
            .map_err(|error| TierWError::InvalidComponent(error.to_string()))?;
        validate_supported_imports(&self.engine, &component)?;
        let component_hash = authorized.component_hash().clone();
        let authorization = authorized.authorization().clone();
        let cache_key = self.cache_key(&component_hash, &authorization)?;
        Ok(ValidatedComponent {
            component,
            component_hash,
            authorization,
            cache_key,
        })
    }
}

struct EpochTicker {
    stop: mpsc::Sender<()>,
    thread: Option<thread::JoinHandle<()>>,
}

impl EpochTicker {
    fn start(engine: Engine) -> Result<Self, TierWError> {
        let (stop, receiver) = mpsc::channel();
        let thread = thread::Builder::new()
            .name("prometheus-exec-tier-w-epoch".into())
            .spawn(move || loop {
                match receiver.recv_timeout(Duration::from_millis(EPOCH_TICK_MILLIS)) {
                    Ok(()) | Err(RecvTimeoutError::Disconnected) => break,
                    Err(RecvTimeoutError::Timeout) => engine.increment_epoch(),
                }
            })
            .map_err(|error| TierWError::EngineConfiguration(error.to_string()))?;
        Ok(Self {
            stop,
            thread: Some(thread),
        })
    }
}

impl Drop for EpochTicker {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EngineIdentity {
    schema_version: u32,
    component_world: &'static str,
    wasmtime_version: &'static str,
    profile: EngineProfile,
    pointer_width: u32,
    endianness: &'static str,
    fuel: bool,
    epoch_interruption: bool,
}

impl EngineIdentity {
    const fn new(profile: EngineProfile) -> Self {
        Self {
            schema_version: 1,
            component_world: COMPONENT_WORLD,
            wasmtime_version: WASMTIME_VERSION,
            profile,
            pointer_width: usize::BITS,
            endianness: if cfg!(target_endian = "little") {
                "little"
            } else {
                "big"
            },
            fuel: true,
            epoch_interruption: true,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ComponentCacheIdentity<'a> {
    schema_version: u32,
    profile_hash: &'a Digest,
    component_hash: &'a Digest,
    authorization: &'a ComponentAuthorization,
}

const fn pulley_target() -> &'static str {
    match (usize::BITS, cfg!(target_endian = "little")) {
        (32, true) => "pulley32",
        (32, false) => "pulley32be",
        (64, true) => "pulley64",
        (64, false) => "pulley64be",
        _ => "unsupported-pulley-width",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REFERENCE_COMPONENT: &[u8] = include_bytes!(
        "../../../skills/react/prometheus-entity-skills/entity-graph-optimize/skill.wasm"
    );

    #[test]
    fn target_policy_never_falls_back_to_an_unrequested_backend() {
        let ios_cranelift = EngineProfile {
            target: ExecutionTarget::Ios,
            backend: BackendProfile::Cranelift,
            jit_permitted: true,
        };
        let availability = TierWEngine::availability(ios_cranelift);
        assert!(!availability.available);
        assert_eq!(availability.backend, BackendProfile::Cranelift);
        assert!(availability.reason.unwrap().contains("iOS"));

        let portable_cranelift = EngineProfile {
            target: ExecutionTarget::PortableReplay,
            backend: BackendProfile::Cranelift,
            jit_permitted: true,
        };
        let availability = TierWEngine::availability(portable_cranelift);
        assert!(!availability.available);
        assert_eq!(availability.backend, BackendProfile::Cranelift);
    }

    #[test]
    fn current_profile_is_honestly_available() {
        let profile = EngineProfile::for_current_target();
        let availability = TierWEngine::probe(profile);
        assert!(availability.available, "{:?}", availability.reason);
        assert_eq!(availability.backend, profile.backend);
    }

    #[cfg(feature = "cranelift")]
    #[test]
    fn desktop_engine_validates_the_reference_component() {
        let engine = TierWEngine::new(EngineProfile::desktop()).unwrap();
        let authorizer = crate::test_authorizer(REFERENCE_COMPONENT);
        assert!(!engine.engine().is_pulley());
        let component = engine
            .load_component(REFERENCE_COMPONENT, &authorizer)
            .unwrap();
        assert_eq!(
            component.component_hash(),
            &Digest::from_bytes(REFERENCE_COMPONENT)
        );
        assert_eq!(
            component.cache_key(),
            &engine
                .cache_key(component.component_hash(), component.authorization())
                .unwrap()
        );
    }

    #[cfg(feature = "pulley")]
    #[test]
    fn pulley_engine_validates_the_reference_component_without_jit() {
        let engine = TierWEngine::new(EngineProfile::portable_replay()).unwrap();
        let authorizer = crate::test_authorizer(REFERENCE_COMPONENT);
        assert!(engine.engine().is_pulley());
        let component = engine
            .load_component(REFERENCE_COMPONENT, &authorizer)
            .unwrap();
        assert_eq!(
            component.component_hash(),
            &Digest::from_bytes(REFERENCE_COMPONENT)
        );
    }

    #[cfg(all(feature = "cranelift", feature = "pulley"))]
    #[test]
    fn backend_profiles_have_distinct_cache_namespaces() {
        let cranelift = TierWEngine::new(EngineProfile::desktop()).unwrap();
        let pulley = TierWEngine::new(EngineProfile::portable_replay()).unwrap();
        let authorizer = crate::test_authorizer(REFERENCE_COMPONENT);
        let authorized = authorizer.authorize(REFERENCE_COMPONENT).unwrap();
        assert_ne!(cranelift.profile_hash(), pulley.profile_hash());
        assert_ne!(
            cranelift
                .cache_key(authorized.component_hash(), authorized.authorization())
                .unwrap(),
            pulley
                .cache_key(authorized.component_hash(), authorized.authorization())
                .unwrap()
        );
    }

    #[cfg(feature = "cranelift")]
    #[test]
    fn cache_identity_is_stable_and_component_sensitive() {
        let a = TierWEngine::new(EngineProfile::desktop()).unwrap();
        let b = TierWEngine::new(EngineProfile::desktop()).unwrap();
        let reference_authorizer = crate::test_authorizer(REFERENCE_COMPONENT);
        let reference = reference_authorizer.authorize(REFERENCE_COMPONENT).unwrap();
        let other_authorizer = crate::test_authorizer(b"not the component");
        let other = other_authorizer.authorize(b"not the component").unwrap();
        assert_eq!(a.profile_hash(), b.profile_hash());
        assert_eq!(
            a.cache_key(reference.component_hash(), reference.authorization())
                .unwrap(),
            b.cache_key(reference.component_hash(), reference.authorization())
                .unwrap()
        );
        assert_ne!(
            a.cache_key(reference.component_hash(), reference.authorization())
                .unwrap(),
            a.cache_key(other.component_hash(), other.authorization())
                .unwrap()
        );
    }

    #[cfg(feature = "cranelift")]
    #[test]
    fn invalid_and_precompiled_bytes_are_rejected() {
        let engine = TierWEngine::new(EngineProfile::desktop()).unwrap();
        let authorizer = crate::test_authorizer(b"not a component");
        let error = engine
            .load_component(b"not a component", &authorizer)
            .unwrap_err();
        assert!(matches!(error, TierWError::InvalidComponent(_)));

        let precompiled = engine
            .engine()
            .precompile_component(REFERENCE_COMPONENT)
            .unwrap();
        let authorizer = crate::test_authorizer(&precompiled);
        let error = engine
            .load_component(&precompiled, &authorizer)
            .unwrap_err();
        assert!(error.to_string().contains("precompiled engine artifacts"));
    }
}
