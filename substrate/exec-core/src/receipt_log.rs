use std::{
    fs::{self, File, OpenOptions},
    io::{Read as _, Write as _},
    path::{Path, PathBuf},
};

use chrono::Utc;
use prometheus_exec_contracts::{
    ensure_schema, verify_receipt, Digest, ExecutionReceipt, ReceiptLogEntry, ReceiptLogSegment,
    SignatureAlgorithm, VerificationKey, VerificationResult,
};
use thiserror::Error;

const MAX_SEGMENT_FILE_BYTES: u64 = 32 * 1024 * 1024;

#[derive(Clone, Debug)]
pub struct ReceiptLog {
    root: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppendReceiptResult {
    pub sequence: u64,
    pub segment_hash: Digest,
    pub receipt_hash: Digest,
    pub created: bool,
}

#[derive(Debug, Error)]
pub enum ReceiptLogError {
    #[error("receipt log I/O failed at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("receipt log JSON failed at {path}: {source}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("receipt contract failed: {0}")]
    Contract(#[from] prometheus_exec_contracts::ContractError),
    #[error("receipt signature is invalid for run {0}")]
    InvalidReceiptSignature(uuid::Uuid),
    #[error("receipt log chain is invalid: {0}")]
    InvalidChain(String),
    #[error("run id {run_id} already exists with receipt {existing}; candidate is {candidate}")]
    RunIdConflict {
        run_id: uuid::Uuid,
        existing: Digest,
        candidate: Digest,
    },
    #[error("receipt log segment exceeds {MAX_SEGMENT_FILE_BYTES} bytes: {0}")]
    SegmentTooLarge(PathBuf),
}

impl ReceiptLog {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, ReceiptLogError> {
        let log = Self { root: root.into() };
        create_private_dir(&log.segment_root())?;
        let lock = log.open_lock()?;
        fs2::FileExt::lock_exclusive(&lock).map_err(|source| io_error(log.lock_path(), source))?;
        log.load_structural()?;
        Ok(log)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn append(
        &self,
        receipt: ExecutionReceipt,
        verification_key: &VerificationKey,
    ) -> Result<AppendReceiptResult, ReceiptLogError> {
        let verification = verify_receipt(&receipt, verification_key, None, None);
        if !verification.valid {
            return Err(ReceiptLogError::InvalidReceiptSignature(receipt.run_id));
        }
        let candidate_hash = receipt.receipt_hash()?;

        let lock = self.open_lock()?;
        fs2::FileExt::lock_exclusive(&lock).map_err(|source| io_error(self.lock_path(), source))?;
        let segments = self.load_structural()?;
        for segment in &segments {
            for entry in &segment.entries {
                if entry.receipt.run_id == receipt.run_id {
                    if entry.receipt_hash == candidate_hash {
                        return Ok(AppendReceiptResult {
                            sequence: segment.header.sequence,
                            segment_hash: segment.segment_hash.clone().ok_or_else(|| {
                                ReceiptLogError::InvalidChain("unsealed segment".into())
                            })?,
                            receipt_hash: candidate_hash,
                            created: false,
                        });
                    }
                    return Err(ReceiptLogError::RunIdConflict {
                        run_id: receipt.run_id,
                        existing: entry.receipt_hash.clone(),
                        candidate: candidate_hash,
                    });
                }
            }
        }

        let sequence = segments
            .last()
            .map(|segment| segment.header.sequence + 1)
            .unwrap_or(0);
        let previous = segments
            .last()
            .and_then(|segment| segment.segment_hash.clone());
        let entry = ReceiptLogEntry::new(receipt)?;
        let segment = ReceiptLogSegment::seal(sequence, previous, Utc::now(), vec![entry])?;
        let segment_hash = segment
            .segment_hash
            .clone()
            .ok_or_else(|| ReceiptLogError::InvalidChain("new segment was not sealed".into()))?;
        let path = self.segment_path(sequence, &segment_hash);
        let mut bytes =
            serde_json::to_vec_pretty(&segment).map_err(|source| ReceiptLogError::Json {
                path: path.clone(),
                source,
            })?;
        bytes.push(b'\n');
        atomic_create(&path, &bytes)?;
        sync_directory(&self.segment_root())?;
        Ok(AppendReceiptResult {
            sequence,
            segment_hash,
            receipt_hash: candidate_hash,
            created: true,
        })
    }

    pub fn segments(&self) -> Result<Vec<ReceiptLogSegment>, ReceiptLogError> {
        let lock = self.open_lock()?;
        fs2::FileExt::lock_shared(&lock).map_err(|source| io_error(self.lock_path(), source))?;
        self.load_structural()
    }

    pub fn verify<F>(&self, mut resolve_key: F) -> Result<Vec<VerificationResult>, ReceiptLogError>
    where
        F: FnMut(&str, SignatureAlgorithm) -> Option<VerificationKey>,
    {
        let segments = self.segments()?;
        let mut previous = None;
        let mut results = Vec::new();
        for segment in segments {
            let verified = segment
                .verify(previous.as_ref(), |key_id, algorithm| {
                    resolve_key(key_id, algorithm)
                })
                .map_err(|error| ReceiptLogError::InvalidChain(error.to_string()))?;
            previous = segment.segment_hash.clone();
            results.extend(verified);
        }
        Ok(results)
    }

    fn load_structural(&self) -> Result<Vec<ReceiptLogSegment>, ReceiptLogError> {
        let mut paths = Vec::new();
        for entry in fs::read_dir(self.segment_root())
            .map_err(|source| io_error(self.segment_root(), source))?
        {
            let entry = entry.map_err(|source| io_error(self.segment_root(), source))?;
            let metadata = fs::symlink_metadata(entry.path())
                .map_err(|source| io_error(entry.path(), source))?;
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(ReceiptLogError::InvalidChain(format!(
                    "unsafe segment entry: {}",
                    entry.path().display()
                )));
            }
            if metadata.len() > MAX_SEGMENT_FILE_BYTES {
                return Err(ReceiptLogError::SegmentTooLarge(entry.path()));
            }
            paths.push(entry.path());
        }
        paths.sort();

        let mut segments = Vec::with_capacity(paths.len());
        let mut previous = None;
        for (expected_sequence, path) in paths.into_iter().enumerate() {
            let bytes = read_segment_bounded(&path)?;
            let segment: ReceiptLogSegment =
                serde_json::from_slice(&bytes).map_err(|source| ReceiptLogError::Json {
                    path: path.clone(),
                    source,
                })?;
            ensure_schema(&segment.header.schema_version)?;
            if segment.header.sequence != expected_sequence as u64 {
                return Err(ReceiptLogError::InvalidChain(format!(
                    "expected sequence {expected_sequence}, found {}",
                    segment.header.sequence
                )));
            }
            if segment.header.previous_segment_hash != previous {
                return Err(ReceiptLogError::InvalidChain(format!(
                    "previous hash mismatch at sequence {expected_sequence}"
                )));
            }
            if segment.header.receipt_count as usize != segment.entries.len() {
                return Err(ReceiptLogError::InvalidChain(format!(
                    "receipt count mismatch at sequence {expected_sequence}"
                )));
            }
            let declared_hash = segment
                .segment_hash
                .clone()
                .ok_or_else(|| ReceiptLogError::InvalidChain("unsealed segment".into()))?;
            let computed_hash = segment.compute_hash()?;
            if declared_hash != computed_hash {
                return Err(ReceiptLogError::InvalidChain(format!(
                    "segment hash mismatch at sequence {expected_sequence}"
                )));
            }
            validate_filename(&path, expected_sequence as u64, &declared_hash)?;
            for entry in &segment.entries {
                entry.receipt.validate()?;
                if entry.receipt.receipt_hash()? != entry.receipt_hash {
                    return Err(ReceiptLogError::InvalidChain(format!(
                        "receipt hash mismatch for run {}",
                        entry.receipt.run_id
                    )));
                }
            }
            previous = Some(declared_hash);
            segments.push(segment);
        }
        Ok(segments)
    }

    fn open_lock(&self) -> Result<File, ReceiptLogError> {
        let path = self.lock_path();
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&path)
            .map_err(|source| io_error(&path, source))?;
        set_private_file(&file, &path)?;
        Ok(file)
    }

    fn segment_root(&self) -> PathBuf {
        self.root.join("segments")
    }

    fn lock_path(&self) -> PathBuf {
        self.root.join("writer.lock")
    }

    fn segment_path(&self, sequence: u64, hash: &Digest) -> PathBuf {
        let hash = hash.as_str().trim_start_matches("sha256:");
        self.segment_root()
            .join(format!("{sequence:020}-{hash}.json"))
    }
}

fn validate_filename(path: &Path, sequence: u64, hash: &Digest) -> Result<(), ReceiptLogError> {
    let expected = format!(
        "{sequence:020}-{}.json",
        hash.as_str().trim_start_matches("sha256:")
    );
    if path.file_name().and_then(|name| name.to_str()) != Some(expected.as_str()) {
        return Err(ReceiptLogError::InvalidChain(format!(
            "segment filename does not match content: {}",
            path.display()
        )));
    }
    Ok(())
}

fn atomic_create(path: &Path, bytes: &[u8]) -> Result<(), ReceiptLogError> {
    let parent = path
        .parent()
        .ok_or_else(|| ReceiptLogError::InvalidChain("segment has no parent".into()))?;
    let mut temporary =
        tempfile::NamedTempFile::new_in(parent).map_err(|source| io_error(parent, source))?;
    set_private_file(temporary.as_file(), temporary.path())?;
    temporary
        .write_all(bytes)
        .map_err(|source| io_error(temporary.path(), source))?;
    temporary
        .as_file()
        .sync_all()
        .map_err(|source| io_error(temporary.path(), source))?;
    temporary
        .persist_noclobber(path)
        .map_err(|error| io_error(path, error.error))?;
    Ok(())
}

fn read_segment_bounded(path: &Path) -> Result<Vec<u8>, ReceiptLogError> {
    let file = File::open(path).map_err(|source| io_error(path, source))?;
    let mut bytes = Vec::new();
    file.take(MAX_SEGMENT_FILE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|source| io_error(path, source))?;
    if bytes.len() as u64 > MAX_SEGMENT_FILE_BYTES {
        return Err(ReceiptLogError::SegmentTooLarge(path.to_path_buf()));
    }
    Ok(bytes)
}

fn create_private_dir(path: &Path) -> Result<(), ReceiptLogError> {
    fs::create_dir_all(path).map_err(|source| io_error(path, source))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .map_err(|source| io_error(path, source))?;
    }
    Ok(())
}

fn set_private_file(file: &File, path: &Path) -> Result<(), ReceiptLogError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        file.set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|source| io_error(path, source))?;
    }
    Ok(())
}

fn sync_directory(path: &Path) -> Result<(), ReceiptLogError> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|source| io_error(path, source))
}

fn io_error(path: impl AsRef<Path>, source: std::io::Error) -> ReceiptLogError {
    ReceiptLogError::Io {
        path: path.as_ref().to_path_buf(),
        source,
    }
}
