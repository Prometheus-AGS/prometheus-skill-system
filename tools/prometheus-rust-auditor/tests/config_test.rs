use prometheus_rust_auditor::config::{load, AuditorConfig};
use std::io::Write as _;
use std::path::Path;

#[test]
fn default_config_has_six_partitions() {
    let config = AuditorConfig::default();
    assert_eq!(
        config.workspace.partitions.len(),
        6,
        "expected 6 partitions from embedded default-config.toml"
    );
}

#[test]
fn load_from_file_overrides_defaults() {
    let mut tmp = tempfile::NamedTempFile::new().expect("create temp file");
    writeln!(
        tmp,
        r#"[workspace]
path = "my_workspace"
partitions = [{{ name = "custom", crates = ["my-crate"] }}]
"#
    )
    .expect("write temp TOML");

    let config = load(Some(tmp.path())).expect("load should succeed");

    assert_eq!(
        config.workspace.partitions.len(),
        1,
        "expected exactly 1 partition from overriding config file"
    );
    assert_eq!(
        config.workspace.partitions[0].name, "custom",
        "expected partition name to be 'custom'"
    );
}

#[test]
fn load_missing_file_returns_default() {
    let config =
        load(Some(Path::new("nonexistent.toml"))).expect("missing file should fall back to default");

    assert_eq!(
        config.workspace.partitions.len(),
        6,
        "expected 6 partitions when falling back to embedded default"
    );
}
