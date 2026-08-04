use std::fs;

use prometheus_exec_contracts::hash_bytes;
use prometheus_exec_core::{ArtifactStore, CasError};
use tempfile::tempdir;

#[test]
fn put_is_atomic_deduplicated_and_corruption_detected() {
    let root = tempdir().unwrap();
    let store = ArtifactStore::open(root.path(), 1024).unwrap();
    let first = store.put(b"same bytes").unwrap();
    let second = store.put(b"same bytes").unwrap();
    assert_eq!(first, second);
    assert_eq!(store.get(&first.hash).unwrap(), b"same bytes");

    let hex = first.hash.as_str().trim_start_matches("sha256:");
    let path = root
        .path()
        .join("blobs/sha256")
        .join(&hex[..2])
        .join(&hex[2..]);
    fs::write(path, b"tampered").unwrap();
    assert!(matches!(
        store.get(&first.hash),
        Err(CasError::Corrupt { .. })
    ));
}

#[test]
fn output_collection_is_sorted_bounded_and_content_addressed() {
    let store_root = tempdir().unwrap();
    let run_root = tempdir().unwrap();
    fs::create_dir_all(run_root.path().join("outputs/nested")).unwrap();
    fs::write(run_root.path().join("outputs/z.txt"), b"z").unwrap();
    fs::write(run_root.path().join("outputs/nested/a.txt"), b"alpha").unwrap();
    let store = ArtifactStore::open(store_root.path(), 1024).unwrap();

    let artifacts = store.collect_outputs(run_root.path(), 6).unwrap();
    assert_eq!(
        artifacts
            .iter()
            .map(|item| item.path.as_str())
            .collect::<Vec<_>>(),
        vec!["outputs/nested/a.txt", "outputs/z.txt"]
    );
    assert_eq!(store.get(&hash_bytes(b"alpha")).unwrap(), b"alpha");
    assert!(matches!(
        store.collect_outputs(run_root.path(), 5),
        Err(CasError::OutputBudgetExceeded { .. })
    ));
}

#[cfg(unix)]
#[test]
fn output_collection_rejects_symlinks_even_when_the_target_is_inside() {
    use std::os::unix::fs::symlink;

    let store_root = tempdir().unwrap();
    let run_root = tempdir().unwrap();
    fs::create_dir(run_root.path().join("outputs")).unwrap();
    fs::write(run_root.path().join("real.txt"), b"secret").unwrap();
    symlink(
        run_root.path().join("real.txt"),
        run_root.path().join("outputs/link.txt"),
    )
    .unwrap();
    let store = ArtifactStore::open(store_root.path(), 1024).unwrap();

    assert!(matches!(
        store.collect_outputs(run_root.path(), 1024),
        Err(CasError::UnsafeOutput(_))
    ));
}

#[test]
fn garbage_collection_never_removes_pinned_content() {
    let root = tempdir().unwrap();
    let store = ArtifactStore::open(root.path(), 4).unwrap();
    let pinned = store.put(b"keep").unwrap();
    let removable = store.put(b"discard").unwrap();
    store.pin(&pinned.hash, "open-certification").unwrap();

    let report = store.garbage_collect().unwrap();
    assert_eq!(store.get(&pinned.hash).unwrap(), b"keep");
    assert!(store.get(&removable.hash).is_err());
    assert!(report.pinned.contains(&pinned.hash));
    assert!(report.removed.contains(&removable.hash));
    assert_eq!(report.bytes_after, 4);

    assert!(store.unpin(&pinned.hash, "open-certification").unwrap());
    assert!(!store.is_pinned(&pinned.hash).unwrap());
}
