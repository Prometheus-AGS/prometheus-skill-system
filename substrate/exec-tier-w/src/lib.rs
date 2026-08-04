#![forbid(unsafe_code)]

#[cfg(all(feature = "mobile", feature = "cranelift"))]
compile_error!(
    "mobile builds must disable default features and select the Pulley-only mobile profile"
);

#[cfg(not(any(feature = "cranelift", feature = "pulley")))]
compile_error!("Tier W requires either the cranelift or pulley backend feature");

/// Release-aligned crate version used by doctor and receipt provenance.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Normative component-model world accepted by this adapter.
pub const COMPONENT_WORLD: &str = "prometheus:component@0.1.0";

/// Wasmtime major shared with UAR, KnowMe, and LibreFang cache identities.
pub const WASMTIME_MAJOR: u32 = 46;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackendProfile {
    Cranelift,
    Pulley,
}

/// Returns the compile-time backend profile without initializing Wasmtime.
pub const fn compiled_backend() -> BackendProfile {
    #[cfg(feature = "mobile")]
    {
        BackendProfile::Pulley
    }
    #[cfg(all(not(feature = "mobile"), feature = "cranelift"))]
    {
        BackendProfile::Cranelift
    }
    #[cfg(all(
        not(feature = "mobile"),
        not(feature = "cranelift"),
        feature = "pulley"
    ))]
    {
        BackendProfile::Pulley
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_and_component_world_are_pinned() {
        assert_eq!(VERSION, "1.7.0");
        assert_eq!(WASMTIME_MAJOR, 46);
        assert_eq!(COMPONENT_WORLD, "prometheus:component@0.1.0");
    }
}
