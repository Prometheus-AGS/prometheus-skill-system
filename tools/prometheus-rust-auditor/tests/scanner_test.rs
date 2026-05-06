use prometheus_rust_auditor::config::PartitionDef;
use prometheus_rust_auditor::scanner::{assign_partitions, match_partition_glob, CrateInfo};
use std::path::PathBuf;

fn make_crate(name: &str) -> CrateInfo {
    CrateInfo {
        name: name.to_string(),
        manifest_path: PathBuf::from(format!("{name}/Cargo.toml")),
        root_path: PathBuf::from(name),
    }
}

#[test]
fn glob_wildcard_prefix() {
    assert!(match_partition_glob("my-actor", "*-actor"));
    assert!(!match_partition_glob("my-store", "*-actor"));
}

#[test]
fn glob_exact_match() {
    assert!(match_partition_glob("scheduler", "scheduler"));
    assert!(!match_partition_glob("other", "scheduler"));
}

#[test]
fn assign_partitions_assigns_and_unpartitioned() {
    let crates = vec![make_crate("my-actor"), make_crate("my-store"), make_crate("standalone")];

    let partitions = vec![
        PartitionDef { name: "actor".to_string(), crates: vec!["*-actor".to_string()] },
        PartitionDef {
            name: "persistence".to_string(),
            crates: vec!["*-store".to_string()],
        },
    ];

    let assignments = assign_partitions(&crates, &partitions);

    assert_eq!(assignments.len(), 3);

    let find = |crate_name: &str| {
        assignments
            .iter()
            .find(|(_, c)| c.name == crate_name)
            .map(|(partition, _)| partition.as_str())
    };

    assert_eq!(find("my-actor"), Some("actor"));
    assert_eq!(find("my-store"), Some("persistence"));
    assert_eq!(find("standalone"), Some("_unpartitioned"));
}
