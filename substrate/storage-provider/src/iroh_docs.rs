//! Storage adapter backed by iroh-docs 0.101.
//!
//! `iroh-docs` stores document entries in a signed replica log and stores entry
//! bytes in an iroh-blobs store. This adapter owns a local iroh endpoint, blob
//! store, gossip service, docs protocol handler, default author, and one docs
//! namespace. Storage operations are translated directly to `Doc::set_bytes`,
//! `Doc::del`, and `Doc::get_many`.
//!
//! Multi-node sync uses the native iroh-docs ticket flow: call
//! `share_write_ticket` or `share_read_ticket` on the source adapter, construct
//! a second adapter with `memory_from_ticket` or `persistent_from_ticket`, then
//! read with retry/backoff because ticket import joins peers asynchronously.

use crate::traits::{CrdtEngine, Result, StorageError, StorageProvider};
use async_trait::async_trait;
use futures::{pin_mut, StreamExt};
use iroh::{endpoint::presets, protocol::Router, Endpoint, EndpointAddr};
use iroh_blobs::{api::Store as BlobStore, store::mem::MemStore, BlobsProtocol};
use iroh_docs::{
    api::{
        protocol::{AddrInfoOptions, ShareMode},
        Doc,
    },
    protocol::Docs,
    store::Query,
    AuthorId, DocTicket, NamespaceId,
};
use iroh_gossip::net::Gossip;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::OnceCell;

/// iroh-docs storage backend.
///
/// Construction is lazy so existing synchronous call sites can keep using
/// `IrohDocsAdapter::new()`. The first async storage operation initializes the
/// endpoint, protocols, author, and namespace.
#[derive(Clone, Debug)]
pub struct IrohDocsAdapter {
    config: Arc<IrohDocsConfig>,
    state: Arc<OnceCell<IrohDocsState>>,
}

#[derive(Clone, Debug)]
struct IrohDocsConfig {
    storage: IrohDocsStorage,
    ticket: Option<DocTicket>,
}

#[derive(Clone, Debug)]
enum IrohDocsStorage {
    Memory,
    Persistent(PathBuf),
}

#[derive(Debug)]
struct IrohDocsState {
    _router: Router,
    blobs: BlobStore,
    doc: Doc,
    author: AuthorId,
}

impl IrohDocsAdapter {
    /// Create an adapter using an in-memory iroh-docs store.
    pub fn new() -> Self {
        Self::memory()
    }

    /// Create an adapter using in-memory docs and blob stores.
    pub fn memory() -> Self {
        Self {
            config: Arc::new(IrohDocsConfig {
                storage: IrohDocsStorage::Memory,
                ticket: None,
            }),
            state: Arc::new(OnceCell::new()),
        }
    }

    /// Create an in-memory adapter by importing an iroh-docs share ticket.
    ///
    /// The imported document joins the peers embedded in the ticket during
    /// lazy initialization, matching `DocsApi::import` semantics. Initial
    /// reads may need a short retry window while live sync downloads remote
    /// entries and their blobs.
    pub fn memory_from_ticket(ticket: DocTicket) -> Self {
        Self {
            config: Arc::new(IrohDocsConfig {
                storage: IrohDocsStorage::Memory,
                ticket: Some(ticket),
            }),
            state: Arc::new(OnceCell::new()),
        }
    }

    /// Create an in-memory adapter from the string form of an iroh-docs ticket.
    pub fn memory_from_ticket_string(ticket: &str) -> Result<Self> {
        let ticket = DocTicket::from_str(ticket)
            .map_err(|e| StorageError::Crdt(format!("invalid iroh-docs ticket: {e}")))?;
        Ok(Self::memory_from_ticket(ticket))
    }

    /// Create an adapter using persistent docs and blob stores under `path`.
    pub fn persistent(path: impl Into<PathBuf>) -> Self {
        Self {
            config: Arc::new(IrohDocsConfig {
                storage: IrohDocsStorage::Persistent(path.into()),
                ticket: None,
            }),
            state: Arc::new(OnceCell::new()),
        }
    }

    /// Create a persistent adapter by importing an iroh-docs share ticket.
    ///
    /// The ticket import starts peer sync during lazy initialization. Callers
    /// should keep the source adapter alive while the importer catches up.
    pub fn persistent_from_ticket(path: impl Into<PathBuf>, ticket: DocTicket) -> Self {
        Self {
            config: Arc::new(IrohDocsConfig {
                storage: IrohDocsStorage::Persistent(path.into()),
                ticket: Some(ticket),
            }),
            state: Arc::new(OnceCell::new()),
        }
    }

    /// Create a persistent adapter from the string form of an iroh-docs ticket.
    pub fn persistent_from_ticket_string(path: impl Into<PathBuf>, ticket: &str) -> Result<Self> {
        let ticket = DocTicket::from_str(ticket)
            .map_err(|e| StorageError::Crdt(format!("invalid iroh-docs ticket: {e}")))?;
        Ok(Self::persistent_from_ticket(path, ticket))
    }

    /// Return the iroh-docs namespace id for this adapter.
    pub async fn namespace_id(&self) -> Result<NamespaceId> {
        Ok(self.state().await?.doc.id())
    }

    /// Return the endpoint address needed by peers to join live sync.
    pub async fn endpoint_addr(&self) -> Result<EndpointAddr> {
        Ok(self.state().await?._router.endpoint().addr())
    }

    /// Start live iroh-docs sync with known peer addresses.
    pub async fn start_sync(&self, peers: Vec<EndpointAddr>) -> Result<()> {
        self.state()
            .await?
            .doc
            .start_sync(peers)
            .await
            .map_err(to_crdt)
    }

    /// Leave live sync for this namespace.
    pub async fn leave_sync(&self) -> Result<()> {
        self.state().await?.doc.leave().await.map_err(to_crdt)
    }

    /// Export a read-only share ticket for this document.
    ///
    /// The ticket includes relay and direct address information so another
    /// adapter can import the same namespace and join live sync.
    pub async fn share_read_ticket(&self) -> Result<DocTicket> {
        self.share_ticket(ShareMode::Read).await
    }

    /// Export a writable share ticket for this document.
    ///
    /// Importers can write with their own local author, and reads use
    /// `single_latest_per_key` so values written by any synced author are
    /// visible through the `StorageProvider` interface.
    pub async fn share_write_ticket(&self) -> Result<DocTicket> {
        self.share_ticket(ShareMode::Write).await
    }

    async fn share_ticket(&self, mode: ShareMode) -> Result<DocTicket> {
        self.state()
            .await?
            .doc
            .share(mode, AddrInfoOptions::RelayAndAddresses)
            .await
            .map_err(to_crdt)
    }

    async fn state(&self) -> Result<&IrohDocsState> {
        self.state
            .get_or_try_init(|| async { self.spawn_state().await })
            .await
    }

    async fn spawn_state(&self) -> Result<IrohDocsState> {
        let endpoint = Endpoint::bind(presets::Minimal)
            .await
            .map_err(to_unavailable)?;

        let (blobs, docs_builder) = match &self.config.storage {
            IrohDocsStorage::Memory => {
                let blobs = MemStore::new();
                ((*blobs).clone(), Docs::memory())
            }
            IrohDocsStorage::Persistent(path) => {
                std::fs::create_dir_all(path).map_err(StorageError::Io)?;
                let blobs = iroh_blobs::store::fs::FsStore::load(path.join("blobs"))
                    .await
                    .map_err(to_unavailable)?;
                ((*blobs).clone(), Docs::persistent(path.join("docs")))
            }
        };

        let gossip = Gossip::builder().spawn(endpoint.clone());
        let docs = docs_builder
            .spawn(endpoint.clone(), blobs.clone(), gossip.clone())
            .await
            .map_err(to_unavailable)?;

        let router = Router::builder(endpoint)
            .accept(iroh_blobs::ALPN, BlobsProtocol::new(&blobs, None))
            .accept(iroh_gossip::ALPN, gossip)
            .accept(iroh_docs::ALPN, docs.clone())
            .spawn();

        let api = docs.api().clone();
        let author = api.author_default().await.map_err(to_crdt)?;
        let doc = match &self.config.ticket {
            Some(ticket) => api.import(ticket.clone()).await.map_err(to_crdt)?,
            None => api.create().await.map_err(to_crdt)?,
        };

        Ok(IrohDocsState {
            _router: router,
            blobs,
            doc,
            author,
        })
    }
}

impl Default for IrohDocsAdapter {
    fn default() -> Self {
        IrohDocsAdapter::new()
    }
}

#[async_trait]
impl StorageProvider for IrohDocsAdapter {
    async fn read(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let state = self.state().await?;
        // Query across authors so imported documents can read remote writes.
        let query = Query::single_latest_per_key()
            .key_exact(key.as_bytes())
            .limit(1)
            .build();
        let entries = state.doc.get_many(query).await.map_err(to_crdt)?;
        pin_mut!(entries);
        let Some(entry) = entries.next().await.transpose().map_err(to_crdt)? else {
            return Ok(None);
        };

        let bytes = state
            .blobs
            .get_bytes(entry.content_hash())
            .await
            .map_err(to_unavailable)?;
        Ok(Some(bytes.to_vec()))
    }

    async fn write(&self, key: &str, value: Vec<u8>) -> Result<()> {
        let state = self.state().await?;
        state
            .doc
            .set_bytes(state.author, key.as_bytes().to_vec(), value)
            .await
            .map_err(to_crdt)?;
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let state = self.state().await?;
        state
            .doc
            .del(state.author, key.as_bytes().to_vec())
            .await
            .map_err(to_crdt)?;
        Ok(())
    }

    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>> {
        let state = self.state().await?;
        let query = Query::single_latest_per_key()
            .key_prefix(prefix.as_bytes())
            .build();
        let entries = state.doc.get_many(query).await.map_err(to_crdt)?;
        pin_mut!(entries);
        let mut keys = Vec::new();

        while let Some(entry) = entries.next().await {
            let entry = entry.map_err(to_crdt)?;
            let key = String::from_utf8(entry.key().to_vec()).map_err(|e| {
                StorageError::Crdt(format!("iroh-docs key is not valid UTF-8: {e}"))
            })?;
            keys.push(key);
        }

        Ok(keys)
    }

    fn backend_name(&self) -> &'static str {
        "iroh-docs"
    }
}

/// Synchronous document-delta helper for the crate's `CrdtEngine` trait.
///
/// Live iroh-docs merging is performed by `start_sync` over the async docs
/// protocol. The trait remains useful for local JSON state snapshots and tests,
/// so it keeps last-write-wins JSON-object behavior while storage itself is now
/// backed by real iroh-docs.
impl CrdtEngine for IrohDocsAdapter {
    fn new_doc(&self) -> Vec<u8> {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let mut bytes = [0u8; 16];
        bytes[..8].copy_from_slice(&id.to_le_bytes());
        bytes.to_vec()
    }

    fn merge(&self, local: &[u8], remote_delta: &[u8]) -> Result<Vec<u8>> {
        if local.is_empty() {
            return Ok(remote_delta.to_vec());
        }
        if remote_delta.is_empty() {
            return Ok(local.to_vec());
        }

        let mut local_map: serde_json::Map<String, serde_json::Value> =
            serde_json::from_slice(local).map_err(StorageError::Serde)?;
        let remote_map: serde_json::Map<String, serde_json::Value> =
            serde_json::from_slice(remote_delta).map_err(StorageError::Serde)?;

        for (k, v) in remote_map {
            local_map.insert(k, v);
        }

        serde_json::to_vec(&local_map).map_err(StorageError::Serde)
    }

    fn apply_json(&self, doc: &[u8], patch: serde_json::Value) -> Result<(Vec<u8>, Vec<u8>)> {
        let mut state: serde_json::Map<String, serde_json::Value> = if doc.is_empty() {
            serde_json::Map::new()
        } else {
            serde_json::from_slice(doc).map_err(StorageError::Serde)?
        };

        let patch_obj = patch
            .as_object()
            .ok_or_else(|| StorageError::Crdt("patch must be a JSON object".into()))?;

        for (k, v) in patch_obj {
            state.insert(k.clone(), v.clone());
        }

        let new_doc = serde_json::to_vec(&state).map_err(StorageError::Serde)?;
        let delta = serde_json::to_vec(patch_obj).map_err(StorageError::Serde)?;
        Ok((new_doc, delta))
    }

    fn to_json(&self, doc: &[u8]) -> Result<serde_json::Value> {
        if doc.is_empty() {
            return Ok(serde_json::Value::Object(serde_json::Map::new()));
        }
        serde_json::from_slice(doc).map_err(StorageError::Serde)
    }

    fn engine_name(&self) -> &'static str {
        "iroh-docs"
    }
}

fn to_crdt(error: impl std::fmt::Display) -> StorageError {
    StorageError::Crdt(error.to_string())
}

fn to_unavailable(error: impl std::fmt::Display) -> StorageError {
    StorageError::Unavailable(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration, Instant};

    #[tokio::test]
    async fn read_returns_none_for_missing_key() {
        let adapter = IrohDocsAdapter::new();
        let result = adapter.read("nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn write_then_read_roundtrip() {
        let adapter = IrohDocsAdapter::new();
        adapter.write("key1", b"hello".to_vec()).await.unwrap();
        let value = adapter.read("key1").await.unwrap();
        assert_eq!(value, Some(b"hello".to_vec()));
    }

    #[tokio::test]
    async fn delete_removes_key() {
        let adapter = IrohDocsAdapter::new();
        adapter.write("to-delete", b"data".to_vec()).await.unwrap();
        adapter.delete("to-delete").await.unwrap();
        let value = adapter.read("to-delete").await.unwrap();
        assert!(value.is_none());
    }

    #[tokio::test]
    async fn delete_missing_key_is_noop() {
        let adapter = IrohDocsAdapter::new();
        adapter.delete("ghost").await.unwrap();
    }

    #[tokio::test]
    async fn list_keys_returns_matching_prefix() {
        let adapter = IrohDocsAdapter::new();
        adapter.write("ns/a", b"1".to_vec()).await.unwrap();
        adapter.write("ns/b", b"2".to_vec()).await.unwrap();
        adapter.write("other/c", b"3".to_vec()).await.unwrap();

        let mut keys = adapter.list_keys("ns/").await.unwrap();
        keys.sort();
        assert_eq!(keys, vec!["ns/a", "ns/b"]);
    }

    #[tokio::test]
    async fn list_keys_empty_prefix_returns_all() {
        let adapter = IrohDocsAdapter::new();
        adapter.write("x", b"1".to_vec()).await.unwrap();
        adapter.write("y", b"2".to_vec()).await.unwrap();

        let mut keys = adapter.list_keys("").await.unwrap();
        keys.sort();
        assert_eq!(keys, vec!["x", "y"]);
    }

    #[tokio::test]
    async fn backend_exposes_namespace_and_endpoint_addr() {
        let adapter = IrohDocsAdapter::new();
        adapter.write("boot", b"ok".to_vec()).await.unwrap();
        let _namespace = adapter.namespace_id().await.unwrap();
        let _addr = adapter.endpoint_addr().await.unwrap();
    }

    #[tokio::test]
    async fn imported_write_ticket_syncs_between_two_nodes() {
        let node_a = IrohDocsAdapter::new();
        node_a
            .write("shared/key", b"from-a".to_vec())
            .await
            .unwrap();

        let ticket = node_a.share_write_ticket().await.unwrap();
        let node_b = IrohDocsAdapter::memory_from_ticket(ticket);

        assert_eq!(
            eventually_read(&node_b, "shared/key", Duration::from_secs(10)).await,
            Some(b"from-a".to_vec())
        );

        node_b
            .write("shared/key-from-b", b"from-b".to_vec())
            .await
            .unwrap();

        assert_eq!(
            eventually_read(&node_a, "shared/key-from-b", Duration::from_secs(10)).await,
            Some(b"from-b".to_vec())
        );
    }

    #[tokio::test]
    async fn ticket_string_roundtrip_imports_shared_document() {
        let node_a = IrohDocsAdapter::new();
        node_a
            .write("ticket/string", b"roundtrip".to_vec())
            .await
            .unwrap();

        let ticket = node_a.share_write_ticket().await.unwrap().to_string();
        let node_b = IrohDocsAdapter::memory_from_ticket_string(&ticket).unwrap();

        assert_eq!(
            eventually_read(&node_b, "ticket/string", Duration::from_secs(10)).await,
            Some(b"roundtrip".to_vec())
        );
    }

    async fn eventually_read(
        adapter: &IrohDocsAdapter,
        key: &str,
        timeout: Duration,
    ) -> Option<Vec<u8>> {
        // Ticket import starts sync immediately, but entries and blobs arrive
        // asynchronously. Polling keeps the regression deterministic without
        // relying on internal iroh-docs live-event ordering.
        let deadline = Instant::now() + timeout;
        loop {
            match adapter.read(key).await {
                Ok(Some(value)) => return Some(value),
                Ok(None) if Instant::now() < deadline => sleep(Duration::from_millis(100)).await,
                Ok(None) => return None,
                Err(error) if Instant::now() < deadline => {
                    eprintln!("retrying iroh-docs read after error: {error}");
                    sleep(Duration::from_millis(100)).await;
                }
                Err(error) => panic!("iroh-docs read failed before sync completed: {error}"),
            }
        }
    }

    #[test]
    fn new_doc_returns_16_bytes() {
        let adapter = IrohDocsAdapter::new();
        let doc = adapter.new_doc();
        assert_eq!(doc.len(), 16);
    }

    #[test]
    fn new_doc_ids_are_unique() {
        let adapter = IrohDocsAdapter::new();
        let a = adapter.new_doc();
        let b = adapter.new_doc();
        assert_ne!(a, b);
    }

    #[test]
    fn merge_two_maps_union() {
        let adapter = IrohDocsAdapter::new();
        let local = serde_json::to_vec(&serde_json::json!({"a": 1})).unwrap();
        let remote = serde_json::to_vec(&serde_json::json!({"b": 2})).unwrap();
        let merged = adapter.merge(&local, &remote).unwrap();
        let obj: serde_json::Value = serde_json::from_slice(&merged).unwrap();
        assert_eq!(obj["a"], 1);
        assert_eq!(obj["b"], 2);
    }

    #[test]
    fn merge_remote_wins_on_conflict() {
        let adapter = IrohDocsAdapter::new();
        let local = serde_json::to_vec(&serde_json::json!({"k": "local"})).unwrap();
        let remote = serde_json::to_vec(&serde_json::json!({"k": "remote"})).unwrap();
        let merged = adapter.merge(&local, &remote).unwrap();
        let obj: serde_json::Value = serde_json::from_slice(&merged).unwrap();
        assert_eq!(obj["k"], "remote");
    }

    #[test]
    fn merge_empty_local_returns_remote() {
        let adapter = IrohDocsAdapter::new();
        let remote = serde_json::to_vec(&serde_json::json!({"k": "v"})).unwrap();
        let result = adapter.merge(&[], &remote).unwrap();
        assert_eq!(result, remote);
    }

    #[test]
    fn merge_empty_remote_returns_local() {
        let adapter = IrohDocsAdapter::new();
        let local = serde_json::to_vec(&serde_json::json!({"k": "v"})).unwrap();
        let result = adapter.merge(&local, &[]).unwrap();
        assert_eq!(result, local);
    }

    #[test]
    fn apply_json_inserts_keys() {
        let adapter = IrohDocsAdapter::new();
        let patch = serde_json::json!({"name": "Alice", "score": 42});
        let (new_doc, delta) = adapter.apply_json(&[], patch.clone()).unwrap();

        let doc_obj: serde_json::Value = serde_json::from_slice(&new_doc).unwrap();
        assert_eq!(doc_obj["name"], "Alice");
        assert_eq!(doc_obj["score"], 42);

        let delta_obj: serde_json::Value = serde_json::from_slice(&delta).unwrap();
        assert_eq!(delta_obj["name"], "Alice");
    }

    #[test]
    fn apply_json_updates_existing_key() {
        let adapter = IrohDocsAdapter::new();
        let initial = serde_json::to_vec(&serde_json::json!({"x": 1})).unwrap();
        let patch = serde_json::json!({"x": 99});
        let (new_doc, _) = adapter.apply_json(&initial, patch).unwrap();
        let obj: serde_json::Value = serde_json::from_slice(&new_doc).unwrap();
        assert_eq!(obj["x"], 99);
    }

    #[test]
    fn apply_json_rejects_non_object_patch() {
        let adapter = IrohDocsAdapter::new();
        let result = adapter.apply_json(&[], serde_json::json!([1, 2, 3]));
        assert!(matches!(result, Err(StorageError::Crdt(_))));
    }

    #[test]
    fn to_json_empty_doc_returns_empty_object() {
        let adapter = IrohDocsAdapter::new();
        let val = adapter.to_json(&[]).unwrap();
        assert_eq!(val, serde_json::json!({}));
    }

    #[test]
    fn to_json_roundtrips_apply_json() {
        let adapter = IrohDocsAdapter::new();
        let patch = serde_json::json!({"foo": "bar"});
        let (doc, _) = adapter.apply_json(&[], patch).unwrap();
        let val = adapter.to_json(&doc).unwrap();
        assert_eq!(val["foo"], "bar");
    }

    #[test]
    fn engine_name_is_iroh_docs() {
        let adapter = IrohDocsAdapter::new();
        assert_eq!(adapter.engine_name(), "iroh-docs");
    }

    #[tokio::test]
    async fn adapter_is_clone_and_shares_state() {
        let a = IrohDocsAdapter::new();
        let b = a.clone();
        a.write("shared", b"data".to_vec()).await.unwrap();
        let v = b.read("shared").await.unwrap();
        assert_eq!(v, Some(b"data".to_vec()));
    }
}
