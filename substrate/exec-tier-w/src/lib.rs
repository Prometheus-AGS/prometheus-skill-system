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
    use sha2::{Digest, Sha256};

    const CANONICAL_TYPES: &str = include_str!("../../../wit/prometheus-component/types.wit");
    const CANONICAL_CAPABILITIES: &str =
        include_str!("../../../wit/prometheus-component/capabilities.wit");
    const CANONICAL_SKILL: &str = include_str!("../../../wit/prometheus-component/skill.wit");
    const GUEST_TYPES: &str = include_str!(
        "../../../skills/react/prometheus-entity-skills/entity-graph-optimize/component/wit/types.wit"
    );
    const GUEST_CAPABILITIES: &str = include_str!(
        "../../../skills/react/prometheus-entity-skills/entity-graph-optimize/component/wit/capabilities.wit"
    );
    const GUEST_SKILL: &str = include_str!(
        "../../../skills/react/prometheus-entity-skills/entity-graph-optimize/component/wit/skill.wit"
    );
    const REFERENCE_COMPONENT: &[u8] = include_bytes!(
        "../../../skills/react/prometheus-entity-skills/entity-graph-optimize/skill.wasm"
    );
    const REFERENCE_COMPONENT_SHA256: &str = include_str!("../fixtures/reference-component.sha256");
    const REFERENCE_RUN_FIXTURE: &str = include_str!("../fixtures/reference-run-absent.json");

    #[test]
    fn release_and_component_world_are_pinned() {
        assert_eq!(VERSION, "1.7.0");
        assert_eq!(WASMTIME_MAJOR, 46);
        assert_eq!(COMPONENT_WORLD, "prometheus:component@0.1.0");
    }

    #[test]
    fn reference_guest_uses_the_canonical_world_byte_for_byte() {
        assert_eq!(GUEST_TYPES, CANONICAL_TYPES);
        assert_eq!(GUEST_CAPABILITIES, CANONICAL_CAPABILITIES);
        assert_eq!(GUEST_SKILL, CANONICAL_SKILL);
        assert!(CANONICAL_CAPABILITIES.contains("interface kv-store"));
        assert!(CANONICAL_CAPABILITIES.contains("interface clock"));
    }

    #[test]
    fn checked_in_reference_component_matches_the_release_fixture() {
        let actual = format!("{:x}", Sha256::digest(REFERENCE_COMPONENT));
        assert_eq!(actual, REFERENCE_COMPONENT_SHA256.trim());

        let fixture: serde_json::Value =
            serde_json::from_str(REFERENCE_RUN_FIXTURE).expect("reference fixture must be JSON");
        assert_eq!(fixture["world"], COMPONENT_WORLD);
        assert_eq!(fixture["component_sha256"], actual);
        assert_eq!(fixture["input"], "{}");
        assert_eq!(
            fixture["expected_output"],
            r#"{"kbd":{"available":false},"evolver":{"available":false},"refiner":{"available":false},"openspec":{"available":false}}"#
        );
    }
}
