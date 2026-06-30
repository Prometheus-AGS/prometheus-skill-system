use crate::traits::{Result, StorageError, StorageProvider};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

pub struct LocalDirAdapter {
    base_path: PathBuf,
}

impl LocalDirAdapter {
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        LocalDirAdapter {
            base_path: base_path.into(),
        }
    }

    /// Convert a storage key to an OS filesystem path.
    /// Forward slashes in the key are treated as path separators.
    fn key_to_path(&self, key: &str) -> PathBuf {
        let mut path = self.base_path.clone();
        for component in key.split('/') {
            if !component.is_empty() {
                path.push(component);
            }
        }
        path
    }
}

#[async_trait]
impl StorageProvider for LocalDirAdapter {
    async fn read(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let path = self.key_to_path(key);
        let result = tokio::task::spawn_blocking(move || match std::fs::read(&path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(StorageError::Io(e)),
        })
        .await
        .map_err(|e| StorageError::Io(std::io::Error::other(e)))?;
        result
    }

    async fn write(&self, key: &str, value: Vec<u8>) -> Result<()> {
        let path = self.key_to_path(key);
        tokio::task::spawn_blocking(move || {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&path, &value)?;
            Ok(())
        })
        .await
        .map_err(|e| StorageError::Io(std::io::Error::other(e)))?
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let path = self.key_to_path(key);
        tokio::task::spawn_blocking(move || match std::fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(StorageError::Io(e)),
        })
        .await
        .map_err(|e| StorageError::Io(std::io::Error::other(e)))?
    }

    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>> {
        let base = self.base_path.clone();
        let prefix = prefix.to_string();
        tokio::task::spawn_blocking(move || {
            let mut keys = Vec::new();
            collect_keys(&base, &base, &prefix, &mut keys)?;
            Ok(keys)
        })
        .await
        .map_err(|e| StorageError::Io(std::io::Error::other(e)))?
    }

    fn backend_name(&self) -> &'static str {
        "local-dir"
    }
}

/// Recursively walk `dir`, collecting storage keys relative to `base` that
/// start with `prefix`.
fn collect_keys(
    base: &Path,
    dir: &Path,
    prefix: &str,
    keys: &mut Vec<String>,
) -> std::io::Result<()> {
    let read_dir = match std::fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e),
    };

    for entry in read_dir {
        let entry = entry?;
        let entry_path = entry.path();
        let metadata = entry.metadata()?;

        if metadata.is_dir() {
            collect_keys(base, &entry_path, prefix, keys)?;
        } else {
            // Convert OS path back to forward-slash key
            if let Ok(relative) = entry_path.strip_prefix(base) {
                let key = relative
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy())
                    .collect::<Vec<_>>()
                    .join("/");
                if key.starts_with(prefix) {
                    keys.push(key);
                }
            }
        }
    }
    Ok(())
}
