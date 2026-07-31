// The iroh-docs backend needs UDP sockets and a multi-thread runtime, so it is
// native-only. Gating the module rather than the crate keeps LocalDirAdapter,
// LoroAdapter, and the traits available on wasm32.
#[cfg(feature = "iroh-docs-backend")]
pub mod iroh_docs;
pub mod local_dir;
pub mod loro_adapter;
pub mod sync_manifest;
pub mod traits;

#[cfg(feature = "iroh-docs-backend")]
pub use iroh_docs::IrohDocsAdapter;
pub use local_dir::LocalDirAdapter;
pub use loro_adapter::LoroAdapter;
pub use sync_manifest::{DomainConfig, PrivacyClass, SyncDomain, SyncManifest};
pub use traits::{CrdtEngine, StorageError, StorageProvider};
