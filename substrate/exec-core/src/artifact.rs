use std::{
    fs::{self, File, OpenOptions},
    io::{Read as _, Write as _},
    path::{Path, PathBuf},
    time::SystemTime,
};

use fs2::FileExt as _;
use prometheus_exec_contracts::{
    hash_bytes, validate_artifact_path, ArtifactReference, Digest, ExecutionReceipt,
    SignedExecRequest,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Clone, Debug)]
pub struct ArtifactStore {
    root: PathBuf,
    budget_bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredArtifact {
    pub hash: Digest,
    pub size_bytes: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GcReport {
    pub bytes_before: u64,
    pub bytes_after: u64,
    pub removed: Vec<Digest>,
    pub pinned: Vec<Digest>,
}

#[derive(Debug, Error)]
pub enum CasError {
    #[error("artifact store I/O failed at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("artifact walk failed: {0}")]
    Walk(#[from] walkdir::Error),
    #[error("artifact contract failed: {0}")]
    Contract(#[from] prometheus_exec_contracts::ContractError),
    #[error("CAS corruption for {expected}: observed {observed}")]
    Corrupt { expected: Digest, observed: Digest },
    #[error("unsafe output entry: {0}")]
    UnsafeOutput(PathBuf),
    #[error("output budget exceeded: limit {limit} bytes, observed {observed} bytes")]
    OutputBudgetExceeded { limit: u64, observed: u64 },
    #[error("pin reason must contain 1..=1024 bytes")]
    InvalidPinReason,
    #[error("invalid CAS path: {0}")]
    InvalidCasPath(PathBuf),
    #[error("JSON serialization failed: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PinRecord {
    digest: Digest,
    reason: String,
}

#[derive(Debug)]
struct BlobRecord {
    digest: Digest,
    path: PathBuf,
    size: u64,
    modified: SystemTime,
}

impl ArtifactStore {
    pub fn open(root: impl Into<PathBuf>, budget_bytes: u64) -> Result<Self, CasError> {
        let store = Self {
            root: root.into(),
            budget_bytes,
        };
        create_private_dir(&store.blob_root())?;
        create_private_dir(&store.pin_root())?;
        Ok(store)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn put(&self, bytes: &[u8]) -> Result<StoredArtifact, CasError> {
        let hash = hash_bytes(bytes);
        let destination = self.blob_path(&hash)?;
        let parent = destination
            .parent()
            .ok_or_else(|| CasError::InvalidCasPath(destination.clone()))?;
        create_private_dir(parent)?;

        if destination.exists() {
            self.verify_existing(&hash, &destination)?;
            return Ok(StoredArtifact {
                hash,
                size_bytes: bytes.len() as u64,
            });
        }

        let mut temporary =
            tempfile::NamedTempFile::new_in(parent).map_err(|source| io_error(parent, source))?;
        set_private_file(temporary.as_file())?;
        temporary
            .write_all(bytes)
            .map_err(|source| io_error(temporary.path(), source))?;
        temporary
            .as_file()
            .sync_all()
            .map_err(|source| io_error(temporary.path(), source))?;
        match temporary.persist_noclobber(&destination) {
            Ok(_) => {}
            Err(error) if error.error.kind() == std::io::ErrorKind::AlreadyExists => {
                self.verify_existing(&hash, &destination)?;
            }
            Err(error) => return Err(io_error(&destination, error.error)),
        }
        sync_directory(parent)?;
        Ok(StoredArtifact {
            hash,
            size_bytes: bytes.len() as u64,
        })
    }

    /// Stores content and pins it under one cross-process CAS operation lock.
    pub fn put_pinned(&self, bytes: &[u8], reason: &str) -> Result<StoredArtifact, CasError> {
        validate_reason(reason)?;
        let _lock = self.operation_lock()?;
        let stored = self.put(bytes)?;
        self.pin_unlocked(&stored.hash, reason)?;
        Ok(stored)
    }

    pub fn get(&self, hash: &Digest) -> Result<Vec<u8>, CasError> {
        let path = self.blob_path(hash)?;
        let metadata = fs::symlink_metadata(&path).map_err(|source| io_error(&path, source))?;
        if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
            return Err(CasError::UnsafeOutput(path));
        }
        let bytes = fs::read(&path).map_err(|source| io_error(&path, source))?;
        let observed = hash_bytes(&bytes);
        if &observed != hash {
            return Err(CasError::Corrupt {
                expected: hash.clone(),
                observed,
            });
        }
        Ok(bytes)
    }

    pub fn collect_outputs(
        &self,
        run_root: &Path,
        max_total_bytes: u64,
    ) -> Result<Vec<ArtifactReference>, CasError> {
        let canonical_run_root = run_root
            .canonicalize()
            .map_err(|source| io_error(run_root, source))?;
        let outputs = run_root.join("outputs");
        let output_metadata =
            fs::symlink_metadata(&outputs).map_err(|source| io_error(&outputs, source))?;
        if output_metadata.file_type().is_symlink() || !output_metadata.is_dir() {
            return Err(CasError::UnsafeOutput(outputs));
        }
        let canonical_outputs = outputs
            .canonicalize()
            .map_err(|source| io_error(&outputs, source))?;
        if !canonical_outputs.starts_with(&canonical_run_root) {
            return Err(CasError::UnsafeOutput(canonical_outputs));
        }

        let mut total = 0_u64;
        let mut artifacts = Vec::new();
        for entry in WalkDir::new(&outputs).follow_links(false).min_depth(1) {
            let entry = entry?;
            let file_type = entry.file_type();
            if file_type.is_symlink() {
                return Err(CasError::UnsafeOutput(entry.path().to_path_buf()));
            }
            if file_type.is_dir() {
                continue;
            }
            if !file_type.is_file() {
                return Err(CasError::UnsafeOutput(entry.path().to_path_buf()));
            }

            let canonical_file = entry
                .path()
                .canonicalize()
                .map_err(|source| io_error(entry.path(), source))?;
            if !canonical_file.starts_with(&canonical_outputs) {
                return Err(CasError::UnsafeOutput(entry.path().to_path_buf()));
            }
            let metadata = fs::metadata(&canonical_file)
                .map_err(|source| io_error(&canonical_file, source))?;
            let projected =
                total
                    .checked_add(metadata.len())
                    .ok_or(CasError::OutputBudgetExceeded {
                        limit: max_total_bytes,
                        observed: u64::MAX,
                    })?;
            if projected > max_total_bytes {
                return Err(CasError::OutputBudgetExceeded {
                    limit: max_total_bytes,
                    observed: projected,
                });
            }
            let bytes = read_bounded(&canonical_file, max_total_bytes.saturating_sub(total))?;
            total =
                total
                    .checked_add(bytes.len() as u64)
                    .ok_or(CasError::OutputBudgetExceeded {
                        limit: max_total_bytes,
                        observed: u64::MAX,
                    })?;

            let relative = entry
                .path()
                .strip_prefix(run_root)
                .map_err(|_| CasError::UnsafeOutput(entry.path().to_path_buf()))?;
            let relative = relative
                .to_str()
                .ok_or_else(|| CasError::UnsafeOutput(relative.to_path_buf()))?
                .replace(std::path::MAIN_SEPARATOR, "/");
            validate_artifact_path(&relative)?;
            let stored = self.put(&bytes)?;
            artifacts.push(ArtifactReference {
                path: relative,
                hash: stored.hash,
                size_bytes: Some(stored.size_bytes),
            });
        }
        artifacts.sort_by(|left, right| left.path.cmp(&right.path));
        Ok(artifacts)
    }

    pub fn pin(&self, hash: &Digest, reason: &str) -> Result<(), CasError> {
        validate_reason(reason)?;
        let _lock = self.operation_lock()?;
        self.pin_unlocked(hash, reason)
    }

    fn pin_unlocked(&self, hash: &Digest, reason: &str) -> Result<(), CasError> {
        self.get(hash)?;
        let directory = self.pin_directory(hash)?;
        create_private_dir(&directory)?;
        let marker = directory.join(pin_reason_id(reason));
        let record = PinRecord {
            digest: hash.clone(),
            reason: reason.into(),
        };
        let mut bytes = serde_json::to_vec(&record)?;
        bytes.push(b'\n');
        atomic_create(&marker, &bytes)?;
        Ok(())
    }

    pub fn unpin(&self, hash: &Digest, reason: &str) -> Result<bool, CasError> {
        validate_reason(reason)?;
        let _lock = self.operation_lock()?;
        let marker = self.pin_directory(hash)?.join(pin_reason_id(reason));
        match fs::remove_file(&marker) {
            Ok(()) => Ok(true),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(source) => Err(io_error(&marker, source)),
        }
    }

    pub fn is_pinned(&self, hash: &Digest) -> Result<bool, CasError> {
        let _lock = self.operation_lock()?;
        self.is_pinned_unlocked(hash)
    }

    fn is_pinned_unlocked(&self, hash: &Digest) -> Result<bool, CasError> {
        let directory = self.pin_directory(hash)?;
        match fs::read_dir(&directory) {
            Ok(entries) => {
                for entry in entries {
                    let entry = entry.map_err(|source| io_error(&directory, source))?;
                    let metadata = entry
                        .metadata()
                        .map_err(|source| io_error(entry.path(), source))?;
                    if metadata.is_file() && !metadata.file_type().is_symlink() {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(source) => Err(io_error(&directory, source)),
        }
    }

    /// Retains every currently materialized blob referenced by a durable run.
    ///
    /// Receipt outputs are required to exist. Request code and inputs are also
    /// pinned when present, while a pre-spawn rejection may legitimately refer
    /// to a request blob that was already missing. Pins are deliberately kept
    /// until a future receipt-archival contract explicitly releases them.
    pub fn retain_for_receipt(
        &self,
        request: &SignedExecRequest,
        receipt: &ExecutionReceipt,
    ) -> Result<(), CasError> {
        let reason = format!("receipt:{}", receipt.run_id);
        let _lock = self.operation_lock()?;
        self.pin_unlocked(&receipt.outputs.stdout, &reason)?;
        self.pin_unlocked(&receipt.outputs.stderr, &reason)?;
        for artifact in &receipt.outputs.artifacts {
            self.pin_unlocked(&artifact.hash, &reason)?;
        }

        self.pin_when_present_unlocked(&request.code.hash, &reason)?;
        for input in &request.inputs {
            self.pin_when_present_unlocked(&input.hash, &reason)?;
        }
        Ok(())
    }

    /// Retains available request material while a run is non-terminal.
    pub fn retain_for_request(&self, request: &SignedExecRequest) -> Result<(), CasError> {
        let reason = format!("request:{}", request.request_id);
        let _lock = self.operation_lock()?;
        self.pin_when_present_unlocked(&request.code.hash, &reason)?;
        for input in &request.inputs {
            self.pin_when_present_unlocked(&input.hash, &reason)?;
        }
        Ok(())
    }

    pub fn garbage_collect(&self) -> Result<GcReport, CasError> {
        let _lock = self.operation_lock()?;
        let mut inventory = self.inventory()?;
        let bytes_before = inventory.iter().map(|blob| blob.size).sum();
        let mut report = GcReport {
            bytes_before,
            bytes_after: bytes_before,
            ..GcReport::default()
        };
        if report.bytes_after <= self.budget_bytes {
            return Ok(report);
        }
        inventory.sort_by(|left, right| {
            left.modified
                .cmp(&right.modified)
                .then_with(|| left.digest.as_str().cmp(right.digest.as_str()))
        });
        for blob in inventory {
            if self.is_pinned_unlocked(&blob.digest)? {
                report.pinned.push(blob.digest);
                continue;
            }
            fs::remove_file(&blob.path).map_err(|source| io_error(&blob.path, source))?;
            report.bytes_after = report.bytes_after.saturating_sub(blob.size);
            report.removed.push(blob.digest);
            if report.bytes_after <= self.budget_bytes {
                break;
            }
        }
        Ok(report)
    }

    fn inventory(&self) -> Result<Vec<BlobRecord>, CasError> {
        let mut records = Vec::new();
        for entry in WalkDir::new(self.blob_root())
            .follow_links(false)
            .min_depth(1)
        {
            let entry = entry?;
            if !entry.file_type().is_file() || entry.file_type().is_symlink() {
                continue;
            }
            let relative = entry
                .path()
                .strip_prefix(self.blob_root())
                .map_err(|_| CasError::InvalidCasPath(entry.path().to_path_buf()))?;
            let mut components = relative.components();
            let prefix = components
                .next()
                .and_then(|part| part.as_os_str().to_str())
                .ok_or_else(|| CasError::InvalidCasPath(entry.path().to_path_buf()))?;
            let suffix = components
                .next()
                .and_then(|part| part.as_os_str().to_str())
                .ok_or_else(|| CasError::InvalidCasPath(entry.path().to_path_buf()))?;
            if components.next().is_some() {
                return Err(CasError::InvalidCasPath(entry.path().to_path_buf()));
            }
            let digest = Digest::parse(format!("sha256:{prefix}{suffix}"))?;
            let metadata = entry.metadata()?;
            records.push(BlobRecord {
                digest,
                path: entry.path().to_path_buf(),
                size: metadata.len(),
                modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            });
        }
        Ok(records)
    }

    fn pin_when_present_unlocked(&self, hash: &Digest, reason: &str) -> Result<bool, CasError> {
        match self.pin_unlocked(hash, reason) {
            Ok(()) => Ok(true),
            Err(CasError::Io { source, .. }) if source.kind() == std::io::ErrorKind::NotFound => {
                Ok(false)
            }
            Err(error) => Err(error),
        }
    }

    fn operation_lock(&self) -> Result<File, CasError> {
        let path = self.root.join(".cas-operation.lock");
        let lock = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(&path)
            .map_err(|source| io_error(&path, source))?;
        set_private_file(&lock)?;
        lock.lock_exclusive()
            .map_err(|source| io_error(&path, source))?;
        Ok(lock)
    }

    fn verify_existing(&self, expected: &Digest, path: &Path) -> Result<(), CasError> {
        let metadata = fs::symlink_metadata(path).map_err(|source| io_error(path, source))?;
        if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
            return Err(CasError::UnsafeOutput(path.to_path_buf()));
        }
        let bytes = fs::read(path).map_err(|source| io_error(path, source))?;
        let observed = hash_bytes(&bytes);
        if &observed != expected {
            return Err(CasError::Corrupt {
                expected: expected.clone(),
                observed,
            });
        }
        Ok(())
    }

    fn blob_root(&self) -> PathBuf {
        self.root.join("blobs").join("sha256")
    }

    fn pin_root(&self) -> PathBuf {
        self.root.join("pins").join("sha256")
    }

    fn blob_path(&self, digest: &Digest) -> Result<PathBuf, CasError> {
        digest_path(&self.blob_root(), digest)
    }

    fn pin_directory(&self, digest: &Digest) -> Result<PathBuf, CasError> {
        digest_path(&self.pin_root(), digest)
    }
}

fn digest_path(root: &Path, digest: &Digest) -> Result<PathBuf, CasError> {
    let hex = digest
        .as_str()
        .strip_prefix("sha256:")
        .ok_or_else(|| CasError::InvalidCasPath(root.to_path_buf()))?;
    Ok(root.join(&hex[..2]).join(&hex[2..]))
}

fn pin_reason_id(reason: &str) -> String {
    hash_bytes(reason.as_bytes())
        .as_str()
        .trim_start_matches("sha256:")
        .to_owned()
}

fn validate_reason(reason: &str) -> Result<(), CasError> {
    if reason.is_empty() || reason.len() > 1024 {
        Err(CasError::InvalidPinReason)
    } else {
        Ok(())
    }
}

fn atomic_create(path: &Path, bytes: &[u8]) -> Result<(), CasError> {
    if path.exists() {
        return Ok(());
    }
    let parent = path
        .parent()
        .ok_or_else(|| CasError::InvalidCasPath(path.to_path_buf()))?;
    create_private_dir(parent)?;
    let mut temporary =
        tempfile::NamedTempFile::new_in(parent).map_err(|source| io_error(parent, source))?;
    set_private_file(temporary.as_file())?;
    temporary
        .write_all(bytes)
        .map_err(|source| io_error(temporary.path(), source))?;
    temporary
        .as_file()
        .sync_all()
        .map_err(|source| io_error(temporary.path(), source))?;
    match temporary.persist_noclobber(path) {
        Ok(_) => sync_directory(parent),
        Err(error) if error.error.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
        Err(error) => Err(io_error(path, error.error)),
    }
}

fn read_bounded(path: &Path, limit: u64) -> Result<Vec<u8>, CasError> {
    let file = File::open(path).map_err(|source| io_error(path, source))?;
    let mut bytes = Vec::new();
    file.take(limit.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|source| io_error(path, source))?;
    if bytes.len() as u64 > limit {
        return Err(CasError::OutputBudgetExceeded {
            limit,
            observed: bytes.len() as u64,
        });
    }
    Ok(bytes)
}

fn create_private_dir(path: &Path) -> Result<(), CasError> {
    fs::create_dir_all(path).map_err(|source| io_error(path, source))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .map_err(|source| io_error(path, source))?;
    }
    Ok(())
}

fn set_private_file(file: &File) -> Result<(), CasError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        file.set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|source| io_error(Path::new("<temporary>"), source))?;
    }
    Ok(())
}

fn sync_directory(path: &Path) -> Result<(), CasError> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|source| io_error(path, source))
}

fn io_error(path: impl AsRef<Path>, source: std::io::Error) -> CasError {
    CasError::Io {
        path: path.as_ref().to_path_buf(),
        source,
    }
}
