use kbd_runtime::{
    registry::{ProjectRegistry, RegistryPruneReceipt},
    ProjectManifest,
};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};
use tempfile::tempdir;
use uuid::Uuid;

fn write_manifest(path: &Path, project_id: &str) {
    fs::create_dir_all(path.join(".prometheus")).expect("create manifest directory");
    let manifest = ProjectManifest {
        schema_version: "1".into(),
        project_id: project_id.into(),
        repository_fingerprint: "sha256:registry-prune-integration".into(),
    };
    let mut bytes = serde_json::to_vec_pretty(&manifest).expect("serialize project manifest");
    bytes.push(b'\n');
    fs::write(path.join(".prometheus/project.json"), bytes).expect("write project manifest");
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn backup_count(registry: &ProjectRegistry) -> usize {
    let root = registry.root().join("registry-maintenance-backups");
    if !root.exists() {
        return 0;
    }
    fs::read_dir(root)
        .expect("read registry maintenance backups")
        .count()
}

#[test]
fn dry_run_is_byte_immutable_and_apply_rechecks_a_reappeared_path() {
    let fixture = tempdir().expect("create fixture");
    let data_root = fixture.path().join("data");
    let checkout = fixture.path().join("checkout");
    let project_id = Uuid::new_v4().to_string();
    write_manifest(&checkout, &project_id);

    let registry = ProjectRegistry::open_at(&data_root);
    let registration = registry
        .register_existing(&checkout)
        .expect("register checkout");
    let registry_before = fs::read(registry.registry_path()).expect("read registry before dry run");
    fs::remove_dir_all(&checkout).expect("remove checkout after registration");

    let dry_run = registry
        .prune_missing(false)
        .expect("inventory missing paths");
    assert!(!dry_run.apply_requested);
    assert!(!dry_run.applied);
    assert_eq!(dry_run.candidates.len(), 1);
    assert_eq!(dry_run.candidates[0].path, registration.path);
    assert_eq!(dry_run.candidates[0].project_id, project_id);
    assert!(dry_run.removed.is_empty());
    assert!(dry_run.backup_path.is_none());
    assert_eq!(
        fs::read(registry.registry_path()).expect("read registry after dry run"),
        registry_before,
        "dry run must leave the registry byte-for-byte unchanged"
    );
    assert_eq!(backup_count(&registry), 0);

    fs::create_dir_all(&checkout).expect("recreate checkout before apply");
    let apply = registry
        .prune_missing(true)
        .expect("apply after candidate path reappears");
    assert!(apply.apply_requested);
    assert!(!apply.applied);
    assert!(apply.candidates.is_empty());
    assert!(apply.removed.is_empty());
    assert!(apply.backup_path.is_none());
    assert_eq!(
        fs::read(registry.registry_path()).expect("read registry after apply recheck"),
        registry_before,
        "locked apply must retain a path that reappeared after dry run"
    );
    assert_eq!(registry.lookup_project(&project_id).unwrap().len(), 1);
    assert_eq!(backup_count(&registry), 0);
}

#[test]
fn apply_preserves_shared_runtime_and_emits_verifiable_idempotent_rollback_evidence() {
    let fixture = tempdir().expect("create fixture");
    let data_root = fixture.path().join("data");
    let stale_checkout = fixture.path().join("stale-checkout");
    let retained_checkout = fixture.path().join("retained-checkout");
    let project_id = Uuid::new_v4().to_string();
    write_manifest(&stale_checkout, &project_id);
    write_manifest(&retained_checkout, &project_id);

    let registry = ProjectRegistry::open_at(&data_root);
    let stale = registry
        .register_existing(&stale_checkout)
        .expect("register stale checkout");
    let retained = registry
        .register_existing(&retained_checkout)
        .expect("register retained checkout");
    let runtime_root = registry.root().join("projects").join(&project_id);
    fs::create_dir_all(&runtime_root).expect("create retained runtime root");
    let retained_journal = runtime_root.join("events.jsonl");
    fs::write(&retained_journal, b"retained-history\n").expect("write retained journal");

    let registry_before = fs::read(registry.registry_path()).expect("read original registry");
    fs::remove_dir_all(&stale_checkout).expect("remove stale checkout");

    let report = registry.prune_missing(true).expect("apply registry prune");
    assert!(report.apply_requested);
    assert!(report.applied);
    assert_eq!(report.candidates, report.removed);
    assert_eq!(report.removed.len(), 1);
    assert_eq!(report.removed[0].path, stale.path);
    assert_eq!(report.removed[0].project_id, project_id);
    assert_eq!(report.retained_registrations, 1);

    let backup_path = report.backup_path.as_ref().expect("backup path");
    let checksum_path = report.checksum_path.as_ref().expect("checksum path");
    let receipt_path = report.receipt_path.as_ref().expect("receipt path");
    let backup_bytes = fs::read(backup_path).expect("read registry backup");
    assert_eq!(backup_bytes, registry_before);
    assert_eq!(
        report.backup_sha256.as_deref(),
        Some(sha256(&backup_bytes).as_str())
    );
    assert_eq!(
        fs::read_to_string(checksum_path).expect("read checksum"),
        format!("{}  registry.json\n", sha256(&backup_bytes))
    );

    let receipt: RegistryPruneReceipt =
        serde_json::from_slice(&fs::read(receipt_path).expect("read prune receipt"))
            .expect("decode prune receipt");
    assert_eq!(receipt.schema_version, "1");
    assert_eq!(receipt.backup_path, *backup_path);
    assert_eq!(receipt.checksum_path, *checksum_path);
    assert_eq!(receipt.backup_sha256, sha256(&registry_before));
    assert_eq!(receipt.removed, report.removed);
    assert_eq!(receipt.retained_registrations, 1);
    assert_eq!(
        receipt.runtime_root,
        registry.root().join("projects").to_str().unwrap()
    );

    let registry_after = fs::read(registry.registry_path()).expect("read pruned registry");
    assert_eq!(receipt.planned_registry_sha256, sha256(&registry_after));
    let rollback_path = Path::new(receipt_path)
        .parent()
        .expect("receipt parent")
        .join("ROLLBACK.md");
    let rollback = fs::read_to_string(rollback_path).expect("read rollback instructions");
    assert!(rollback.contains(&receipt.operation_id));
    assert!(rollback.contains(&receipt.backup_sha256));
    assert!(rollback.contains(&receipt.planned_registry_sha256));
    assert!(rollback.contains("Acquire the exclusive lock"));
    assert!(rollback.contains("never deletes runtime directories"));

    let registrations = registry
        .lookup_project(&project_id)
        .expect("load retained registrations");
    assert_eq!(registrations.len(), 1);
    assert_eq!(registrations[0].0.to_str(), Some(retained.path.as_str()));
    assert_eq!(
        fs::read(&retained_journal).expect("read retained journal"),
        b"retained-history\n",
        "registry maintenance must preserve runtime history"
    );

    let backups_before_repeat = backup_count(&registry);
    let repeat = registry
        .prune_missing(true)
        .expect("repeat idempotent registry prune");
    assert!(repeat.apply_requested);
    assert!(!repeat.applied);
    assert!(repeat.candidates.is_empty());
    assert!(repeat.removed.is_empty());
    assert!(repeat.backup_path.is_none());
    assert_eq!(backup_count(&registry), backups_before_repeat);
    assert_eq!(
        fs::read(registry.registry_path()).expect("read registry after repeat"),
        registry_after,
        "repeat prune must not mutate the registry"
    );
    assert!(retained_journal.is_file());
}
