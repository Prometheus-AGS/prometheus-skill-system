use super::*;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn collect_hashes(root: &Path) -> BTreeMap<PathBuf, String> {
    fn visit(root: &Path, path: &Path, hashes: &mut BTreeMap<PathBuf, String>) {
        let mut entries = fs::read_dir(path)
            .unwrap()
            .map(|entry| entry.unwrap())
            .collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            let file_type = entry.file_type().unwrap();
            if file_type.is_dir() {
                visit(root, &path, hashes);
            } else {
                assert!(
                    file_type.is_file(),
                    "runtime proof refuses symlinks: {}",
                    path.display()
                );
                let relative = path.strip_prefix(root).unwrap().to_path_buf();
                hashes.insert(
                    relative,
                    format!("{:x}", Sha256::digest(fs::read(path).unwrap())),
                );
            }
        }
    }

    let mut hashes = BTreeMap::new();
    visit(root, root, &mut hashes);
    hashes
}

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    let mut entries = fs::read_dir(source)
        .unwrap()
        .map(|entry| entry.unwrap())
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let file_type = entry.file_type().unwrap();
        if file_type.is_dir() {
            copy_tree(&source_path, &destination_path);
        } else {
            assert!(
                file_type.is_file(),
                "runtime proof refuses symlinks: {}",
                source_path.display()
            );
            fs::copy(source_path, destination_path).unwrap();
        }
    }
}

fn assert_semantically_equivalent(original: &KbdStateV2, migrated: &KbdStateV2) {
    assert_eq!(migrated.project_id, original.project_id);
    assert_eq!(migrated.run_id, original.run_id);
    assert_eq!(migrated.revision, original.revision);
    assert_eq!(migrated.lifecycle, original.lifecycle);
    assert_eq!(migrated.plan_revision, original.plan_revision);
    assert_eq!(migrated.checkpoint, original.checkpoint);
    assert_eq!(migrated.exact_next_work, original.exact_next_work);
    assert_eq!(migrated.active_path, original.active_path);
    assert_eq!(migrated.phases, original.phases);
    assert_eq!(migrated.completion, original.completion);
    assert_eq!(migrated.decisions, original.decisions);
    assert_eq!(migrated.blockers, original.blockers);
    assert_eq!(migrated.claims, original.claims);
    assert_eq!(migrated.submodule_pins, original.submodule_pins);
    assert_eq!(migrated.conflicts, original.conflicts);
    assert_eq!(migrated.devices, original.devices);
    assert_eq!(migrated.command_revisions, original.command_revisions);
}

fn historical_wire_event(event: &Event, signer: &DeviceSigner) -> serde_json::Value {
    let mut event = event.clone();
    event.replica_id.clear();
    event.lamport = 0;
    event.frontier = CausalFrontier::empty();
    event.actor_id.clear();
    event.migration_provenance = None;
    event.integrity_hash.clear();
    event.signer_key_id = Some(signer.key_id().into());
    event.signer_public_key = Some(signer.public_key().into());
    event.signature = None;
    let mut value = serde_json::to_value(event).unwrap();
    value
        .as_object_mut()
        .unwrap()
        .insert("leaseId".into(), serde_json::Value::Null);
    value
        .as_object_mut()
        .unwrap()
        .insert("fencingToken".into(), serde_json::Value::Null);
    let unsigned = serde_jcs::to_vec(&value).unwrap();
    let object = value.as_object_mut().unwrap();
    object.insert(
        "integrityHash".into(),
        serde_json::Value::String(format!("{:x}", Sha256::digest(&unsigned))),
    );
    object.insert(
        "signature".into(),
        serde_json::Value::String(signer.sign_base64(&unsigned)),
    );
    value
}

#[test]
fn pre_replica_v2_migration_authenticates_the_historical_wire_shape() {
    let fixture = tempdir().unwrap();
    let source = Runtime::open(fixture.path().join("source"));
    source
        .initialize(
            "00000000-0000-4000-8000-000000000099",
            "historical-run",
            Actor::operator("migration-proof", "test"),
        )
        .unwrap();
    let signer = DeviceSigner::from_bytes(&[7; 32]);
    let value = historical_wire_event(&source.events().unwrap()[0], &signer);
    let journal = fixture.path().join("events.jsonl");
    let mut encoded = serde_json::to_vec(&value).unwrap();
    encoded.push(b'\n');
    fs::write(&journal, &encoded).unwrap();
    let decoded = read_event_file(&journal).unwrap();
    validate_pre_replica_v2_journal(&journal, &decoded).unwrap();

    let mut mutated = value.clone();
    mutated["kind"]["payload"]["plan_revision"] = serde_json::json!(2);
    let mut encoded = serde_json::to_vec(&mutated).unwrap();
    encoded.push(b'\n');
    fs::write(&journal, encoded).unwrap();
    assert!(matches!(
        validate_pre_replica_v2_journal(&journal, &read_event_file(&journal).unwrap()),
        Err(RuntimeError::Integrity { revision: 1 })
    ));

    let mut unsupported = value;
    unsupported["unexpectedMigrationField"] = serde_json::json!(true);
    let mut encoded = serde_json::to_vec(&unsupported).unwrap();
    encoded.push(b'\n');
    fs::write(&journal, encoded).unwrap();
    assert!(matches!(
        validate_pre_replica_v2_journal(&journal, &read_event_file(&journal).unwrap()),
        Err(RuntimeError::InvalidState(message)) if message.contains("unsupported key")
    ));
}

#[test]
#[ignore = "operator proof requiring KBD_RUNTIME_PROOF_ROOT"]
fn all_live_runtime_directories_migrate_and_roll_back_from_copies() {
    let source_root = PathBuf::from(
        std::env::var_os("KBD_RUNTIME_PROOF_ROOT")
            .expect("KBD_RUNTIME_PROOF_ROOT must identify the stopped live runtime root"),
    );
    let mut source_directories = fs::read_dir(&source_root)
        .unwrap()
        .map(|entry| entry.unwrap())
        .filter(|entry| entry.file_type().unwrap().is_dir())
        .collect::<Vec<_>>();
    source_directories.sort_by_key(|entry| entry.file_name());
    assert_eq!(source_directories.len(), 18, "unexpected runtime count");

    let proof = tempdir().unwrap();
    let copied_root = proof.path().join("runtime-copies");
    let checkout_root = proof.path().join("synthetic-checkouts");
    let signer = DeviceSigner::generate();
    let mut journal_count = 0usize;
    let mut migrated_event_count = 0usize;
    let mut raft_count = 0usize;

    for source in source_directories {
        let project_id = source.file_name().to_string_lossy().into_owned();
        Uuid::parse_str(&project_id).expect("runtime directory must be a project UUID");
        let source_path = source.path();
        let copied_path = copied_root.join(&project_id);
        let source_hashes = collect_hashes(&source_path);
        copy_tree(&source_path, &copied_path);
        assert_eq!(collect_hashes(&copied_path), source_hashes);

        if copied_path.join("raft.redb").is_file() {
            raft_count += 1;
        }
        let checkout = checkout_root.join(&project_id);
        fs::create_dir_all(checkout.join(".prometheus")).unwrap();
        atomic_json(
            &checkout.join(".prometheus/project.json"),
            &serde_json::json!({
                "schemaVersion": "1",
                "projectId": project_id,
                "repositoryFingerprint": "sha256:copy-migration-proof"
            }),
        )
        .unwrap();
        let runtime = Runtime {
            root: copied_path.clone(),
            project_root: checkout,
            replica_id: Uuid::new_v4().to_string(),
            key_storage: KeyStorage::PlatformCredentialStore,
            read_only: false,
        };

        let source_journal = copied_path.join("events.jsonl");
        if !source_journal.is_file() {
            assert!(runtime
                .migrate_v1_journal_inner(Some(&signer))
                .unwrap()
                .is_none());
            fs::remove_file(copied_path.join("journal-migration.lock")).unwrap();
            assert_eq!(collect_hashes(&copied_path), source_hashes);
            continue;
        }

        journal_count += 1;
        let original_bytes = fs::read(&source_journal).unwrap();
        let original_events = read_event_file(&source_journal).unwrap();
        validate_journal_for_migration(&source_journal, &original_events).unwrap();
        let semantic_source =
            resign_journal_events(&original_events, &project_id, "copy-proof-source", &signer)
                .unwrap();
        let original_state = replay_events(&semantic_source).unwrap();
        let summary = runtime
            .migrate_v1_journal_inner(Some(&signer))
            .unwrap()
            .unwrap();
        migrated_event_count += summary.migrated_events;
        assert_eq!(summary.original_events, original_events.len());
        assert_eq!(summary.migrated_events, original_events.len());
        assert_eq!(fs::read(&summary.archive_journal).unwrap(), original_bytes);
        assert!(summary.rollback_instructions.is_file());
        assert!(summary.project_document.is_file());
        assert!(summary.active_journal.is_file());
        assert!(
            copied_path.join("raft.redb").is_file()
                == source_hashes.contains_key(Path::new("raft.redb"))
        );

        let migrated_events = read_event_file(&summary.active_journal).unwrap();
        for (migrated, original) in migrated_events.iter().zip(&original_events) {
            let provenance = migrated.migration_provenance.as_ref().unwrap();
            assert_eq!(provenance.source_event_id, original.event_id);
            assert_eq!(provenance.source_integrity_hash, original.integrity_hash);
        }
        assert_semantically_equivalent(&original_state, &runtime.replay().unwrap());

        let repeated = runtime
            .migrate_v1_journal_inner(Some(&signer))
            .unwrap()
            .unwrap();
        assert!(repeated.already_migrated);
        assert_eq!(repeated.archive_sha256, summary.archive_sha256);

        let active_archive = summary.active_journal.with_extension("jsonl.proof-archive");
        let document_archive = summary
            .project_document
            .with_extension("loro.proof-archive");
        fs::rename(&summary.active_journal, active_archive).unwrap();
        fs::rename(&summary.project_document, document_archive).unwrap();
        fs::rename(&summary.archive_journal, &summary.source_journal).unwrap();
        assert_eq!(fs::read(&summary.source_journal).unwrap(), original_bytes);
        let restored_events = read_event_file(&summary.source_journal).unwrap();
        validate_journal_for_migration(&summary.source_journal, &restored_events).unwrap();
        let restored_semantic =
            resign_journal_events(&restored_events, &project_id, "copy-proof-source", &signer)
                .unwrap();
        assert_semantically_equivalent(
            &original_state,
            &replay_events(&restored_semantic).unwrap(),
        );
    }

    assert_eq!(journal_count, 14, "unexpected journal count");
    assert_eq!(raft_count, 5, "unexpected raft.redb count");
    assert!(migrated_event_count > 0);
    eprintln!(
        "copy migration proof passed: 18 runtimes, {journal_count} journals, {migrated_event_count} events, {raft_count} retained raft files"
    );
}

#[test]
#[ignore = "operator verification requiring KBD_MIGRATED_DATA_ROOT"]
fn all_live_migrated_projects_replay_from_journal_and_loro() {
    let data_root = PathBuf::from(
        std::env::var_os("KBD_MIGRATED_DATA_ROOT")
            .expect("KBD_MIGRATED_DATA_ROOT must identify the platform data root"),
    );
    let registry = crate::registry::ProjectRegistry::open_at(&data_root);
    let document = registry.load().unwrap();
    assert_eq!(document.replicas.len(), 28);
    let project_ids = document
        .replicas
        .values()
        .map(|replica| replica.project_id.clone())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(project_ids.len(), 18);
    assert_eq!(
        document
            .replicas
            .values()
            .filter(|replica| replica.kind == crate::registry::ReplicaKind::Recovered)
            .inspect(|replica| assert!(replica.read_only))
            .count(),
        9
    );
    assert_eq!(
        document
            .replicas
            .iter()
            .filter(|(path, replica)| {
                replica.kind == crate::registry::ReplicaKind::Recovered
                    && path.contains("/recovered-projects/")
            })
            .count(),
        8
    );

    let mut journal_count = 0usize;
    let mut event_count = 0usize;
    let mut raft_archive_count = 0usize;
    for project_id in project_ids {
        let project_replicas = document.project_replicas(&project_id);
        if project_replicas
            .iter()
            .any(|(_, replica)| replica.kind == crate::registry::ReplicaKind::Standalone)
        {
            assert_eq!(
                document.authoritative_replica(&project_id).unwrap().1.kind,
                crate::registry::ReplicaKind::Standalone
            );
        }
        let (path, _) = document.authoritative_replica(&project_id).unwrap();
        let runtime =
            Runtime::open_registered_at(Path::new(path), &data_root, &project_id).unwrap();
        assert_eq!(runtime.reconcile_project_document().unwrap(), 0);
        let runtime_root = runtime.runtime_root();
        let archive = runtime_root.join("events.v1.jsonl.archive");
        if archive.is_file() {
            journal_count += 1;
            let source = read_event_file(&archive).unwrap();
            validate_journal_for_migration(&archive, &source).unwrap();
            let active = runtime.replica_events().unwrap();
            assert_eq!(active.len(), source.len());
            for (migrated, original) in active.iter().zip(&source) {
                let provenance = migrated.migration_provenance.as_ref().unwrap();
                assert_eq!(provenance.source_event_id, original.event_id);
                assert_eq!(provenance.source_integrity_hash, original.integrity_hash);
            }
            let authoritative = runtime.events().unwrap();
            assert_eq!(authoritative, active);
            let mut semantic_source = replay_events(
                &resign_journal_events(
                    &source,
                    &project_id,
                    "live-verification-source",
                    &DeviceSigner::from_bytes(&[9; 32]),
                )
                .unwrap(),
            )
            .unwrap();
            let migrated_state = runtime.replay().unwrap();
            semantic_source.devices.clone_from(&migrated_state.devices);
            assert_semantically_equivalent(&semantic_source, &migrated_state);
            event_count += active.len();
            let checksum = fs::read_to_string(archive.with_extension("archive.sha256")).unwrap();
            assert_eq!(
                checksum.split_whitespace().next().unwrap(),
                format!("{:x}", Sha256::digest(fs::read(&archive).unwrap()))
            );
        } else {
            assert!(runtime.events().unwrap().is_empty());
        }
        let raft = runtime_root.join("raft.redb");
        let raft_archive = runtime_root.join("raft.redb.archive");
        assert!(!raft.exists());
        if raft_archive.is_file() {
            raft_archive_count += 1;
            let checksum =
                fs::read_to_string(runtime_root.join("raft.redb.archive.sha256")).unwrap();
            assert_eq!(
                checksum.split_whitespace().next().unwrap(),
                format!("{:x}", Sha256::digest(fs::read(raft_archive).unwrap()))
            );
        }
    }
    assert_eq!(journal_count, 14);
    assert_eq!(event_count, 27);
    assert_eq!(raft_archive_count, 5);
    eprintln!(
        "live migrated verification passed: 18 projects, 28 replicas, {journal_count} journals, {event_count} events, {raft_archive_count} raft archives"
    );
}
