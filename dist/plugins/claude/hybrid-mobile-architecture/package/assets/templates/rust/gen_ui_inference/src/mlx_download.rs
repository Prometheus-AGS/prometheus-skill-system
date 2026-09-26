// TJ-ARCH-MOB-001 compliant
//! Shared multi-file MLX model download/verify, used by both the iOS
//! (Swift-bridge) and macOS (mlx-c) MLX engines. Rust owns the download —
//! resumable, revision-pinned, SHA-256-verified — so a platform binding only
//! ever receives a complete, verified model directory.

use std::io::Read;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use futures::TryStreamExt;
use gen_ui_types::error::{CoreError, CoreResult};
use gen_ui_types::lifecycle::ModelOperationState;
use reqwest::header::RANGE;
use reqwest::StatusCode;
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;
use tokio::sync::watch;

use crate::catalog::{ArtifactSpec, MlxArtifactSet};

const DOWNLOAD_LOG_INTERVAL_BYTES: u64 = 256 * 1024 * 1024;

/// Download + verify every file of `set` into `dir`, emitting aggregate
/// `Downloading` progress. Files are processed in catalog order (configs
/// before shards) so a bad revision fails before any multi-gigabyte transfer.
/// The directory is left fully verified on success.
pub(crate) async fn download_model_set(
    dir: &Path,
    set: &MlxArtifactSet,
    status: &watch::Sender<ModelOperationState>,
    cancelled: &AtomicBool,
) -> CoreResult<()> {
    let _ = status.send(ModelOperationState::Downloading {
        bytes_received: 0,
        total_bytes: set.total_bytes,
        bytes_per_second: 0,
        resumable: true,
    });
    let started = Instant::now();
    let mut completed = 0u64;
    for artifact in set.files {
        ensure_file(
            dir,
            artifact,
            set.total_bytes,
            completed,
            status,
            cancelled,
            started,
        )
        .await?;
        completed = completed.saturating_add(artifact.byte_len);
    }
    Ok(())
}

async fn verify_file(path: &Path, artifact: &ArtifactSpec) -> CoreResult<()> {
    let path = path.to_owned();
    let artifact = *artifact;
    gen_ui_runtime::spawn_blocking(move || {
        let metadata = std::fs::metadata(&path)
            .map_err(|error| CoreError::Terminal(format!("MLX file metadata: {error}")))?;
        if metadata.len() != artifact.byte_len {
            return Err(CoreError::Terminal(format!(
                "MLX file {} byte length: expected {}, got {}",
                artifact.file_name,
                artifact.byte_len,
                metadata.len()
            )));
        }
        let mut file = std::fs::File::open(&path)
            .map_err(|error| CoreError::Terminal(format!("MLX file open: {error}")))?;
        let mut digest = Sha256::new();
        let mut buffer = [0_u8; 1024 * 1024];
        loop {
            let read = file
                .read(&mut buffer)
                .map_err(|error| CoreError::Terminal(format!("MLX file read: {error}")))?;
            if read == 0 {
                break;
            }
            digest.update(&buffer[..read]);
        }
        let actual = format!("{:x}", digest.finalize());
        if actual == artifact.sha256 {
            Ok(())
        } else {
            Err(CoreError::Terminal(format!(
                "MLX file {} checksum: expected {}, got {actual}",
                artifact.file_name, artifact.sha256
            )))
        }
    })
    .await
    .map_err(|error| CoreError::Terminal(format!("MLX file checksum task: {error}")))?
}

/// Download + verify one file into `dir`, resuming a partial `.part` when the
/// server supports ranged requests. `completed_before` is the summed bytes of
/// already-finished files, so progress is aggregate across the whole set.
async fn ensure_file(
    dir: &Path,
    artifact: &ArtifactSpec,
    total_bytes: u64,
    completed_before: u64,
    status: &watch::Sender<ModelOperationState>,
    cancelled: &AtomicBool,
    started: Instant,
) -> CoreResult<()> {
    let path = dir.join(artifact.file_name);
    if path.is_file() && verify_file(&path, artifact).await.is_ok() {
        return Ok(());
    }
    let temporary = path.with_extension(format!(
        "{}.part",
        path.extension().and_then(|e| e.to_str()).unwrap_or("bin")
    ));
    let existing = tokio::fs::metadata(&temporary)
        .await
        .map(|m| m.len())
        .unwrap_or(0);
    let mut received = existing;

    let mut request = reqwest::Client::new().get(artifact.url);
    if existing > 0 && existing < artifact.byte_len {
        request = request.header(RANGE, format!("bytes={existing}-"));
    }
    let response = request
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|error| CoreError::Transient(format!("MLX file download: {error}")))?;
    let append = existing > 0
        && existing < artifact.byte_len
        && response.status() == StatusCode::PARTIAL_CONTENT;
    let mut options = tokio::fs::OpenOptions::new();
    options.write(true).create(true);
    if append {
        options.append(true);
    } else {
        options.truncate(true);
        received = 0;
    }
    let mut file = options
        .open(&temporary)
        .await
        .map_err(|error| CoreError::Terminal(format!("MLX temporary file: {error}")))?;

    let mut stream = response.bytes_stream();
    let mut next_log = received.saturating_add(DOWNLOAD_LOG_INTERVAL_BYTES);
    while let Some(chunk) = stream
        .try_next()
        .await
        .map_err(|error| CoreError::Transient(format!("MLX download stream: {error}")))?
    {
        if cancelled.load(Ordering::Acquire) {
            return Err(CoreError::Cancelled("MLX model operation cancelled".into()));
        }
        file.write_all(&chunk)
            .await
            .map_err(|error| CoreError::Terminal(format!("MLX cache write: {error}")))?;
        received = received.saturating_add(chunk.len() as u64);
        let elapsed = started.elapsed().as_secs().max(1);
        let aggregate = completed_before.saturating_add(received);
        let _ = status.send(ModelOperationState::Downloading {
            bytes_received: aggregate,
            total_bytes,
            bytes_per_second: aggregate / elapsed,
            resumable: true,
        });
        if received >= next_log {
            tracing::info!(
                target: "__APP_NAME___mlx",
                file = artifact.file_name,
                bytes_received = received,
                file_bytes = artifact.byte_len,
                "MLX file download progress"
            );
            next_log = received.saturating_add(DOWNLOAD_LOG_INTERVAL_BYTES);
        }
    }
    file.flush()
        .await
        .map_err(|error| CoreError::Terminal(format!("MLX cache flush: {error}")))?;
    drop(file);

    if let Err(error) = verify_file(&temporary, artifact).await {
        let _ = tokio::fs::remove_file(&temporary).await;
        return Err(error);
    }
    tokio::fs::rename(&temporary, &path)
        .await
        .map_err(|error| CoreError::Terminal(format!("MLX cache publish: {error}")))?;
    Ok(())
}
