//! Round-trip tests across the FFI surface.
//!
//! These call the same functions the generated Dart bindings call and assert on
//! the VALUE returned, not merely that a call completed. A test that only
//! checked "it did not panic" would pass against a stub returning nothing.

use super::api::*;

#[test]
fn run_skill_rejects_an_empty_id() {
    let e = run_skill(String::new(), "{}".into()).unwrap_err();
    assert!(matches!(e.kind, SkillErrorKind::InvalidInput));
    assert!(e.message.contains("skill_id"), "got: {}", e.message);
}

#[test]
fn run_skill_rejects_non_json_input() {
    let e = run_skill("entity-graph-optimize".into(), "not json".into()).unwrap_err();
    assert!(matches!(e.kind, SkillErrorKind::InvalidInput));
}

#[test]
fn run_skill_accepts_empty_input_like_the_shell_version() {
    // detect-orchestrators.sh ignores stdin entirely. Rejecting empty input
    // would be a behaviour change smuggled in under "port".
    let e = run_skill("entity-graph-optimize".into(), String::new()).unwrap_err();
    assert!(
        matches!(e.kind, SkillErrorKind::Unsupported),
        "empty input must reach the host, not be rejected as invalid"
    );
}

#[test]
fn run_skill_reports_no_host_rather_than_faking_success() {
    // THE assertion that matters. Until change-msp-008 de-stubs UAR's runtime,
    // the honest answer is Unsupported with a reason. Returning Ok here would
    // make a mobile caller believe a skill ran when nothing did.
    let e = run_skill("entity-graph-optimize".into(), "{}".into()).unwrap_err();
    assert!(matches!(e.kind, SkillErrorKind::Unsupported));
    assert!(
        e.message.contains("no host bound"),
        "the error must say WHY, got: {}",
        e.message
    );
}

#[test]
fn describe_round_trips_a_real_value() {
    let d = describe_skill("entity-graph-optimize".into()).unwrap();
    assert_eq!(d.id, "entity-graph-optimize");
    assert!(d.exports.contains(&"run".to_string()));
    assert!(d.exports.contains(&"describe".to_string()));
    assert!(d.capabilities.contains(&"kv-store".to_string()));
}

#[test]
fn describe_rejects_an_empty_id() {
    assert!(matches!(
        describe_skill("  ".into()).unwrap_err().kind,
        SkillErrorKind::InvalidInput
    ));
}

#[test]
fn world_version_matches_the_wit_package() {
    // A binding built against a different world than the host expects is a
    // silent mismatch; this is the value a client checks first.
    assert_eq!(world_version(), "prometheus:component@0.1.0");
}

#[test]
fn list_skills_reports_no_host_rather_than_an_empty_catalog() {
    // An empty Ok(vec![]) would read as "no skills exist" when the truth is
    // "nothing can answer yet" — the same lie-by-omission `run_skill` avoids.
    let e = list_skills().unwrap_err();
    assert!(matches!(e.kind, SkillErrorKind::Unsupported));
    assert!(e.message.contains("no host bound"), "got: {}", e.message);
}
